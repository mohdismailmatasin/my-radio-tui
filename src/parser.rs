use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Station {
    pub name: String,
    pub url: String,
}

pub fn parse_m3u8(path: &Path) -> Result<Vec<Station>> {
    let content = fs::read_to_string(path)?;
    let mut stations = Vec::new();
    let mut pending_name: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#EXTM3U") {
            continue;
        }
        if line.starts_with("#EXTINF:") {
            if let Some(comma_pos) = line.find(',') {
                pending_name = Some(line[comma_pos + 1..].to_string());
            }
        } else if line.starts_with("http://") || line.starts_with("https://") {
            if let Some(name) = pending_name.take() {
                stations.push(Station {
                    name,
                    url: line.to_string(),
                });
            }
        }
    }

    Ok(stations)
}
