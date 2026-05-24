use crate::models::{IdentificationResult, MediaItem, MediaKind, Score};

pub fn score_identification(item: &MediaItem) -> Score {
    let result =
        IdentificationResult { item: item.clone(), reasons: Vec::new(), errors: Vec::new() };
    score_identification_result(&result)
}

pub fn score_identification_result(result: &IdentificationResult) -> Score {
    let mut confidence: i16 = 0;
    let mut reasons: Vec<String> = Vec::new();
    let item = &result.item;

    if item.kind == MediaKind::Unknown {
        reasons.push("Unknown media type".to_string());
        return Score { confidence: 0, reasons };
    }

    if !item.nfo.is_empty() {
        let source = item.nfo.get("_source").and_then(|v| v.as_deref()).unwrap_or("file");
        let label = if source == "file" { "NFO" } else { "identification" };

        if item.nfo.get("imdbid").and_then(|v| v.as_ref()).is_some() {
            confidence += 50;
            reasons.push(format!("IMDB ID from {label}"));
        } else if item.nfo.get("tmdbid").and_then(|v| v.as_ref()).is_some() {
            confidence += 45;
            reasons.push(format!("TMDB ID from {label}"));
        } else if item.nfo.get("tvdbid").and_then(|v| v.as_ref()).is_some() {
            confidence += 40;
            reasons.push(format!("TVDB ID from {label}"));
        }
    }

    if item.title.is_some() {
        if item.nfo.get("title").and_then(|v| v.as_ref()).is_some() {
            let source = item.nfo.get("_source").and_then(|v| v.as_deref()).unwrap_or("file");
            let label = if source == "file" { "NFO" } else { "identification" };
            confidence += 20;
            reasons.push(format!("Title from {label}"));
        } else {
            confidence += 10;
            reasons.push("Title from filename".to_string());
        }
    } else {
        confidence -= 20;
        reasons.push("No title identified".to_string());
    }

    if matches!(item.kind, MediaKind::Movie | MediaKind::Show) {
        if let Some(year) = item.year {
            confidence += 15;
            reasons.push(format!("Year identified: {year}"));
        } else {
            confidence -= 10;
            reasons.push("No year found".to_string());
        }
    }

    if item.kind == MediaKind::Episode {
        if let (Some(season), Some(episode)) = (item.season, item.episodes.first()) {
            confidence += 20;
            reasons.push(format!("Season/episode identified: S{season:02}E{episode:02}"));
        } else if let Some(date) = &item.date {
            confidence += 15;
            reasons.push(format!("Date episode identified: {date}"));
        } else {
            confidence -= 15;
            reasons.push("No season/episode information".to_string());
        }

        if item.nfo.get("episode_title").and_then(|v| v.as_ref()).is_some() {
            confidence += 10;
            reasons.push("Episode title from NFO".to_string());
        }
    }

    if let Some(part) = item.part {
        confidence += 5;
        reasons.push(format!("Part {part} identified"));
    }

    if !result.errors.is_empty() {
        confidence -= 5 * result.errors.len() as i16;
        reasons.push(format!("{} error(s) during identification", result.errors.len()));
    }

    Score { confidence: confidence.clamp(0, 100) as u8, reasons }
}
