use crate::models::{MediaItem, MediaKind, Score};

pub fn score_identification(item: &MediaItem) -> Score {
    let mut confidence: u8 = 0;
    let mut reasons: Vec<String> = Vec::new();

    match item.kind {
        MediaKind::Movie => {
            if item.title.is_some() {
                confidence += 40;
                reasons.push("title found".to_string());
            }
            if item.year.is_some() {
                confidence += 30;
                reasons.push("year found".to_string());
            }
            if !item.nfo.is_empty() {
                confidence += 20;
                reasons.push("NFO data".to_string());
            }
            if item.part.is_some() {
                confidence += 10;
                reasons.push("part found".to_string());
            }
        }
        MediaKind::Episode => {
            if item.title.is_some() {
                confidence += 30;
                reasons.push("title found".to_string());
            }
            if item.season.is_some() && !item.episodes.is_empty() {
                confidence += 40;
                reasons.push("episode info found".to_string());
            }
            if item.date.is_some() {
                confidence += 40;
                reasons.push("date found".to_string());
            }
            if item.year.is_some() {
                confidence += 10;
                reasons.push("year found".to_string());
            }
            if !item.nfo.is_empty() {
                confidence += 20;
                reasons.push("NFO data".to_string());
            }
        }
        _ => {
            if item.title.is_some() {
                confidence += 10;
                reasons.push("title found".to_string());
            }
        }
    }

    confidence = confidence.min(100);

    Score { confidence, reasons }
}
