use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackStatus {
    Stopped,
    Loading,
    Playing,
    Paused,
    Error,
}

pub struct Player {
    child: Arc<std::sync::Mutex<Option<std::process::Child>>>,
    status: Arc<std::sync::Mutex<PlaybackStatus>>,
}

impl Player {
    pub fn new() -> Result<Self> {
        Ok(Self {
            child: Arc::new(std::sync::Mutex::new(None)),
            status: Arc::new(std::sync::Mutex::new(PlaybackStatus::Stopped)),
        })
    }

    fn get_stream_url(playlist_url: &str) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let response = client.get(playlist_url).send()?;
        let body = response.text()?;

        for line in body.lines() {
            let line = line.trim();
            if line.starts_with("http://") || line.starts_with("https://") {
                return Ok(line.to_string());
            }
        }

        Err(anyhow!("No stream URL found"))
    }

    pub fn play(&self, url: &str) -> Result<()> {
        self.stop();

        let stream_url = match Self::get_stream_url(url) {
            Ok(u) => u,
            Err(_) => url.to_string(),
        };

        let status = self.status.clone();
        let child = self.child.clone();

        *status.lock().unwrap() = PlaybackStatus::Loading;

        thread::spawn(move || {
            let result = Command::new("mpv")
                .arg("--no-video")
                .arg("--quiet")
                .arg(&stream_url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match result {
                Ok(c) => {
                    {
                        let mut guard = child.lock().unwrap();
                        *guard = Some(c);
                    }
                    *status.lock().unwrap() = PlaybackStatus::Playing;
                }
                Err(e) => {
                    eprintln!("Failed to start mpv: {}", e);
                    *status.lock().unwrap() = PlaybackStatus::Error;
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        *self.status.lock().unwrap() = PlaybackStatus::Stopped;
        let mut guard = self.child.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }
    }

    pub fn is_playing(&self) -> bool {
        *self.status.lock().unwrap() == PlaybackStatus::Playing
    }

    pub fn get_status(&self) -> PlaybackStatus {
        *self.status.lock().unwrap()
    }

    pub fn pause(&self) {
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = Command::new("kill")
                .arg("-STOP")
                .arg(child.id().to_string())
                .spawn();
        }
        *self.status.lock().unwrap() = PlaybackStatus::Paused;
    }

    pub fn resume(&self) {
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = Command::new("kill")
                .arg("-CONT")
                .arg(child.id().to_string())
                .spawn();
        }
        *self.status.lock().unwrap() = PlaybackStatus::Playing;
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}
