use camino::{Utf8Path, Utf8PathBuf};
use quick_xml::events::Event;
use quick_xml::Reader;

/// Parsed NFO metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NfoData {
    pub title: Option<String>,
    pub year: Option<u16>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub tvdb_id: Option<String>,
    pub episode_title: Option<String>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

/// Parse an NFO file at the given path.
///
/// Returns `None` if the file cannot be read or is not valid XML.
pub fn parse_nfo(path: &Utf8Path) -> Option<NfoData> {
    let xml = std::fs::read_to_string(path).ok()?;
    parse_nfo_str(&xml)
}

fn parse_nfo_str(xml: &str) -> Option<NfoData> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut data = NfoData::default();
    let mut current_text = String::new();
    let mut current_uniqueid_type: Option<String> = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                if tag == "uniqueid" {
                    for attr in e.attributes() {
                        let attr = attr.ok()?;
                        if attr.key.as_ref() == b"type" {
                            current_uniqueid_type =
                                Some(String::from_utf8_lossy(&attr.value).into_owned());
                        }
                    }
                }
                current_text.clear();
            }
            Ok(Event::Text(e)) => {
                current_text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                if tag == "uniqueid" {
                    let mut uid_type = None;
                    for attr in e.attributes() {
                        let attr = attr.ok()?;
                        if attr.key.as_ref() == b"type" {
                            uid_type = Some(String::from_utf8_lossy(&attr.value).into_owned());
                        }
                    }
                    // Empty <uniqueid> has no text, so nothing to record
                    let _ = uid_type;
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let text = current_text.trim();

                if !text.is_empty() {
                    match tag.as_str() {
                        "title" => data.title = Some(text.to_string()),
                        "year" => data.year = text.parse().ok(),
                        "imdbid" | "imdb_id" => data.imdb_id = Some(normalize_imdb_id(text)),
                        "tmdbid" | "tmdb_id" => data.tmdb_id = Some(text.to_string()),
                        "tvdbid" | "tvdb_id" => data.tvdb_id = Some(text.to_string()),
                        "episodetitle" | "episode_title" => {
                            data.episode_title = Some(text.to_string())
                        }
                        "season" => data.season = text.parse().ok(),
                        "episode" => data.episode = text.parse().ok(),
                        "uniqueid" => {
                            if let Some(t) = current_uniqueid_type.take() {
                                match t.as_str() {
                                    "imdb" if data.imdb_id.is_none() => {
                                        data.imdb_id = Some(normalize_imdb_id(text))
                                    }
                                    "tmdb" if data.tmdb_id.is_none() => {
                                        data.tmdb_id = Some(text.to_string())
                                    }
                                    "tvdb" if data.tvdb_id.is_none() => {
                                        data.tvdb_id = Some(text.to_string())
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if tag == "uniqueid" {
                    current_uniqueid_type = None;
                }
                current_text.clear();
            }
            Ok(Event::CData(_))
            | Ok(Event::Comment(_))
            | Ok(Event::Decl(_))
            | Ok(Event::DocType(_))
            | Ok(Event::PI(_)) => {}
            Ok(Event::Eof) => break,
            Err(_) => return None,
        }
        buf.clear();
    }

    Some(data)
}

/// Find an NFO file associated with a video file.
///
/// Looks for:
/// - Same name as video file with `.nfo` extension
/// - `movie.nfo` or `tvshow.nfo` in the same directory
pub fn find_nfo_for_file(video_path: &Utf8Path) -> Option<Utf8PathBuf> {
    let parent = video_path.parent()?;

    // Try same-name NFO
    let nfo_path = video_path.with_extension("nfo");
    if nfo_path.exists() {
        return Some(nfo_path);
    }

    // Try movie.nfo or tvshow.nfo
    for name in ["movie.nfo", "tvshow.nfo"] {
        let nfo_path = parent.join(name);
        if nfo_path.exists() {
            return Some(nfo_path);
        }
    }

    None
}

/// Normalize an IMDB ID to `ttNNNNNNN` format.
pub fn normalize_imdb_id(imdb_id: &str) -> String {
    let mut id = imdb_id.to_string();

    // Remove any URL components
    if id.contains("imdb.com") {
        if let Some(part) = id.split('/').find(|p| p.starts_with("tt")) {
            id = part.to_string();
        }
    }

    // Ensure tt prefix
    if !id.starts_with("tt") {
        id = format!("tt{id}");
    }

    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_imdb_id_basic() {
        assert_eq!(normalize_imdb_id("tt0133093"), "tt0133093");
    }

    #[test]
    fn normalize_imdb_id_adds_prefix() {
        assert_eq!(normalize_imdb_id("0133093"), "tt0133093");
    }

    #[test]
    fn normalize_imdb_id_from_url() {
        assert_eq!(normalize_imdb_id("https://www.imdb.com/title/tt0133093/"), "tt0133093");
    }
}
