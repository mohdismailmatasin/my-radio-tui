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
    parse_m3u8_str(&content)
}

pub fn parse_m3u8_str(content: &str) -> Result<Vec<Station>> {
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

#[cfg(test)]
mod tests {
    use super::parse_m3u8_str;

    #[test]
    fn parses_station_names_and_urls() {
        let content = r#"
#EXTM3U
#EXTINF:-1,Station One
https://example.com/one.m3u8
#EXTINF:-1,Station Two
https://example.com/two.m3u8
"#;

        let stations = parse_m3u8_str(content).expect("playlist should parse");

        assert_eq!(stations.len(), 2);
        assert_eq!(stations[0].name, "Station One");
        assert_eq!(stations[0].url, "https://example.com/one.m3u8");
        assert_eq!(stations[1].name, "Station Two");
        assert_eq!(stations[1].url, "https://example.com/two.m3u8");
    }
}
