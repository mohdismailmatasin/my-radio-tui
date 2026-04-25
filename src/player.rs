use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
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

struct ManagedChild {
    process: std::process::Child,
}

pub struct Player {
    child: Arc<std::sync::Mutex<Option<ManagedChild>>>,
    status: Arc<std::sync::Mutex<PlaybackStatus>>,
    generation: Arc<AtomicUsize>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            child: Arc::new(std::sync::Mutex::new(None)),
            status: Arc::new(std::sync::Mutex::new(PlaybackStatus::Stopped)),
            generation: Arc::new(AtomicUsize::new(0)),
        }
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
        self.stop_current_process();

        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let url = url.to_string();

        let child = self.child.clone();
        let generation_state = self.generation.clone();
        let status = self.status.clone();

        *status.lock().unwrap() = PlaybackStatus::Loading;

        thread::spawn(move || {
            let stream_url = match Self::get_stream_url(&url) {
                Ok(resolved) => resolved,
                Err(_) => url,
            };

            if generation_state.load(Ordering::SeqCst) != generation {
                return;
            }

            let result = Command::new("mpv")
                .arg("--no-video")
                .arg("--quiet")
                .arg(&stream_url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match result {
                Ok(mut process) => {
                    if generation_state.load(Ordering::SeqCst) != generation {
                        let _ = process.kill();
                        let _ = process.wait();
                        return;
                    }

                    {
                        let mut guard = child.lock().unwrap();
                        if generation_state.load(Ordering::SeqCst) != generation {
                            drop(guard);
                            let _ = process.kill();
                            let _ = process.wait();
                            return;
                        }

                        *guard = Some(ManagedChild { process });
                    }

                    *status.lock().unwrap() = PlaybackStatus::Playing;
                }
                Err(e) => {
                    eprintln!("Failed to start mpv: {}", e);
                    if generation_state.load(Ordering::SeqCst) == generation {
                        *status.lock().unwrap() = PlaybackStatus::Error;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.stop_current_process();
        *self.status.lock().unwrap() = PlaybackStatus::Stopped;
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
                .arg(child.process.id().to_string())
                .spawn();
        }
        *self.status.lock().unwrap() = PlaybackStatus::Paused;
    }

    pub fn resume(&self) {
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = Command::new("kill")
                .arg("-CONT")
                .arg(child.process.id().to_string())
                .spawn();
        }
        *self.status.lock().unwrap() = PlaybackStatus::Playing;
    }

    fn stop_current_process(&self) {
        let mut guard = self.child.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.process.kill();
            let _ = child.process.wait();
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}
