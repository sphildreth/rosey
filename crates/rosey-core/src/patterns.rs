use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static YEAR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\((?P<paren>19\d{2}|20\d{2})\)|(?:^|[._\s-])(?P<bare>19\d{2}|20\d{2})(?=[._\s-]|$)")
        .expect("valid year regex")
});

static EPISODE_SXXEYY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)[Ss](?P<season>\d{1,2})[ ._-]*[Ee](?P<ep1>\d{1,4})(?:[ ._-]*-?[ ._-]*[Ee]?(?P<ep2>\d{1,4}))?")
        .expect("valid SxxEyy regex")
});

static EPISODE_X_FORMAT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?P<season>\d{1,2})x(?P<ep1>\d{1,4})(?:[ ._-]*-[ ._-]*(?P<ep2>\d{1,4}))?")
        .expect("valid 1x02 regex")
});

static DATE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?P<year>\d{4})[-.](?P<month>\d{2})[-.](?P<day>\d{2})")
        .expect("valid date regex")
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeMatch {
    pub season: u16,
    pub episodes: Vec<u16>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateMatch {
    pub date: String,
    pub title: Option<String>,
}

pub fn extract_year(filename: &str) -> Option<u16> {
    for captures in YEAR_PATTERN.captures_iter(filename) {
        let value = captures
            .name("paren")
            .or_else(|| captures.name("bare"))
            .and_then(|m| m.as_str().parse::<u16>().ok())?;

        if (1900..=2040).contains(&value) && !is_part_of_date(filename, value) {
            return Some(value);
        }
    }

    None
}

fn is_part_of_date(filename: &str, year: u16) -> bool {
    let year_text = year.to_string();
    DATE_PATTERN
        .find_iter(filename)
        .any(|m| filename[m.start()..m.end()].starts_with(&year_text))
}

pub fn extract_date(filename: &str) -> Option<DateMatch> {
    let captures = DATE_PATTERN.captures(filename)?;
    Some(DateMatch {
        date: format!(
            "{}-{}-{}",
            &captures["year"], &captures["month"], &captures["day"]
        ),
        title: None,
    })
}

pub fn extract_episode_info(filename: &str) -> Option<EpisodeMatch> {
    extract_episode_with_regex(filename, &EPISODE_SXXEYY)
        .or_else(|| extract_episode_with_regex(filename, &EPISODE_X_FORMAT))
}

fn extract_episode_with_regex(filename: &str, regex: &Regex) -> Option<EpisodeMatch> {
    let captures = regex.captures(filename)?;
    let season = captures.name("season")?.as_str().parse::<u16>().ok()?;
    let ep1 = captures.name("ep1")?.as_str().parse::<u16>().ok()?;
    let mut episodes = vec![ep1];

    if let Some(ep2) = captures.name("ep2").and_then(|m| m.as_str().parse::<u16>().ok()) {
        if ep2 > ep1 {
            episodes = (ep1..=ep2).collect();
        } else if ep2 != ep1 {
            episodes.push(ep2);
        }
    }

    Some(EpisodeMatch {
        season,
        episodes,
        title: None,
    })
}

pub fn clean_title(raw: &str) -> String {
    let mut text = raw.replace(['.', '_'], " ");
    text = text.replace("  ", " ");
    text.trim_matches(|c: char| c.is_whitespace() || c == '-' || c == '.')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_parenthesized_year() {
        assert_eq!(extract_year("The Matrix (1999).mkv"), Some(1999));
    }

    #[test]
    fn ignores_year_when_part_of_daily_date() {
        assert_eq!(extract_year("Daily.Show.2024-05-01.mkv"), None);
    }

    #[test]
    fn extracts_sxxeyy_episode() {
        let parsed = extract_episode_info("Example.Show.S01E02.mkv").unwrap();
        assert_eq!(parsed.season, 1);
        assert_eq!(parsed.episodes, vec![2]);
    }

    #[test]
    fn extracts_episode_range() {
        let parsed = extract_episode_info("Example.Show.S01E02-E03.mkv").unwrap();
        assert_eq!(parsed.season, 1);
        assert_eq!(parsed.episodes, vec![2, 3]);
    }

    #[test]
    fn cleans_basic_title() {
        assert_eq!(clean_title("Example.Show - "), "Example Show");
    }
}
