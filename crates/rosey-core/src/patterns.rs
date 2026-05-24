use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Regexes ported from Python patterns.py.  Rust's regex crate does not support
// look-ahead/look-behind, so we emulate them by matching more broadly and filtering
// in the surrounding code.
// ─────────────────────────────────────────────────────────────────────────────

/// Stand-alone year: preceded by ^, ., _, space or - and followed by ., _, space
/// or end of string.  The trailing boundary is captured as part of the match so we
/// can inspect it post-match.
static STANDALONE_YEAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|[._\s-])(?P<year>19\d{2}|20\d{2})(?P<trailer>[._\s-]|$)")
        .expect("valid standalone year regex")
});

/// Parenthesized year, e.g. (1999).
static PAREN_YEAR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\((?P<year>\d{4})\)").expect("valid paren year regex"));

/// YYYY-MM-DD or YYYY.MM.DD.
static DATE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?P<year>\d{4})[-.](?P<month>\d{2})[-.](?P<day>\d{2})").expect("valid date regex")
});

/// S01E02, S01 E02, S01E01-E02, S01.E02, etc.
/// Accepts up to 4-digit episodes for long-running shows.
static EPISODE_SXXEYY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)[Ss](?P<season>\d{1,2})[ ._\-]*[Ee](?P<ep1>\d{1,4})(?:[ ._\-]*-?[ ._\-]*[Ee]?(?P<ep2>\d{1,4}))?",
    )
    .expect("valid SxxEyy regex")
});

/// 1x02, 1x02-03, etc.
static EPISODE_X_FORMAT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?P<season>\d{1,2})x(?P<ep1>\d{1,4})(?:[ ._\-]*-[ ._\-]*(?P<ep2>\d{1,4}))?")
        .expect("valid 1x02 regex")
});

/// S01EP02, S01 EP02, Season 1 EP01, etc.
static EPISODE_SEP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)[Ss]eason[ ._\-]*(?P<season>\d{1,2})[ ._\-]*[Ee][Pp](?P<ep1>\d{1,4})(?:[ ._\-]*-?[ ._\-]*[Ee][Pp](?P<ep2>\d{1,4}))?",
    )
    .expect("valid season EP regex")
});

/// S01EP02 shorthand (no "Season" word).
static EPISODE_SXXEPYY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)[Ss](?P<season>\d{1,2})[ ._\-]*[Ee][Pp](?P<ep1>\d{1,4})(?:[ ._\-]*-?[ ._\-]*[Ee][Pp](?P<ep2>\d{1,4}))?",
    )
    .expect("valid SxxEPyy regex")
});

/// Episode XX, Episode 13, etc.
static EPISODE_WORD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)[Ee]pisode[ ._\-]*(?P<ep1>\d{1,4})(?:[ ._\-]*-?[ ._\-]*[Ee]pisode[ ._\-]*(?P<ep2>\d{1,4}))?",
    )
    .expect("valid Episode regex")
});

/// Episode number at start of filename, e.g. "01 Title", "05-Title".
/// Only used when known_season is passed.
static EPISODE_AT_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<ep1>\d{1,3})[ ._\-]").expect("valid episode-at-start regex"));

/// Dash-separated season-episode, e.g. 1-2, 02-15, 1-2-3.
/// Only used when known_season matches.
static EPISODE_DASH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?P<season>\d{1,2})-(?P<ep1>\d{1,4})(?:-(?P<ep2>\d{1,4}))?")
        .expect("valid dash-separated regex")
});

/// Part pattern: Part 1, pt2, Part III, One, Two, etc.
static PART_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b[Pp](?:ar)?t[.\s]*(?P<part>\d+|(?:[IVX]+|One|Two|Three|Four|Five|Six|Seven|Eight|Nine|Ten)\b)",
    )
    .expect("valid part regex")
});

/// Season folder pattern: Season 01, Season 1, S03.
static SEASON_FOLDER_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:[Ss]eason[.\s]*(?P<season>\d{1,2})|(?:^|[.\s_-])[Ss](?P<season2>\d{1,2})(?:[.\s_-]|$))")
        .expect("valid season folder regex")
});

/// TMDB ID pattern: [tmdbid-123] anywhere in a path component.
static TMDB_ID_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\[tmdbid-(\d+)\]").expect("valid tmdbid regex"));

/// Pattern that looks like an episode marker right before a year.
static EPISODE_BEFORE_YEAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)[Ss]\d{1,2}[Ee]\d{1,4}[-_]$|\d{1,2}x\d{1,4}[-_]$")
        .expect("valid episode-before-year regex")
});

static PAREN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(([^)]+)\)").expect("valid paren regex"));
static YEAR_PAREN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\(\d{4}\)").expect("valid year-paren regex"));
static BRACKET_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[.*?\]").expect("valid bracket regex"));
static SEP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[._\-\u{2013}\u{2014}]").expect("valid sep regex"));
static RES_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b\d{3,4}[pi]\b").expect("valid res regex"));
static FORMAT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(3d|imax|\d+mm)\b").expect("valid format regex"));
static H264_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b[Hh]\s*\.?\s*26[45]\b").expect("valid h264 regex"));
static CODEC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(x264|x265|h264|h265|hevc|xvid|divx|mp4)\b").expect("valid codec regex")
});
static BIT_DEPTH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b\d{1,2}\s*bit\b").expect("valid bit-depth regex"));
static AUDIO_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:aac|dd|ddp|ac3|dts|truehd|atmos)\s*\d*\s*\.?\s*\d*\b")
        .expect("valid audio regex")
});
static SOURCE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:webrip|web\s+dl|webdl|web|dl|tvrip|bluray|bdrip|dvdrip|hdrip|hdtv|uhd|4k|amzn|nf|hulu|dv)\b")
        .expect("valid source regex")
});
static RELEASE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(proper|repack|internal|extended\s+edition|unrated|remastered|directors?\s+cut|cut|dubbed)\b")
        .expect("valid release regex")
});
static EDITION_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bedition\b").expect("valid edition regex"));
static SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").expect("valid space regex"));
static SEASON_RANGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bseasons?\s*\d+\s*(?:to|-)+\s*\d+\b").expect("valid season-range regex")
});
static SEASON_WORD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bseason[s]?\s*\d{1,2}\b").expect("valid season-word regex"));
static SEASON_COMPACT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b[Ss]\d{1,2}\b").expect("valid season-compact regex"));
static COLLECTION_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(complete|season pack)\b").expect("valid collection regex"));
static NUM_RANGE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b\d+\s*(?:to|-)+\s*\d+\b").expect("valid numeric-range regex"));
static TO_NUM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bto\s*\d+\b").expect("valid to-num regex"));
static QUOTE_NUM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"'(?P<num>\d+)'").expect("valid quote regex"));
static BRACKET2_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[.*?\]").expect("valid bracket2 regex"));
static RELEASE_GROUP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\b(?:GROUP|KOGI|AVS|GGEZ|BAE|RBB|NTB|RARBG|ION10|MEMENTO|KILLERS|ROVERS|SPARKS|FLUX)\b",
    )
    .expect("valid release groups regex")
});
static MIXED_GROUP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?:KOGi|NTb|RBB)\b").expect("valid mixed release groups regex"));
static STRAY_EP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b[Ee]\d{1,4}\b").expect("valid stray ep regex"));
static STRAY_DIGITS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d\s+\d\b").expect("valid stray digits regex"));
static PAREN2_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\([^)]*\)").expect("valid paren2 regex"));
static VOL_PH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"VOLPLACEHOLDER(\d+)ENDVOL").expect("valid vol placeholder regex"));
static SPIDER_MAN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bSpider Man\b").expect("valid spiderman regex"));
static SPIDER_VERSE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bSpider Verse\b").expect("valid spiderverse regex"));
static X_MEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bX Men\b").expect("valid xmen regex"));
static NAMES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(michael\s+bay|baz\s+luhrmann|david\s+o\s+russell|idris\s+elba|bj\s+novak)\b",
    )
    .expect("valid names regex")
});
static PHRASES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(black\s+and\s+chrome|romantic\s+comedy|unrated)\b")
        .expect("valid phrases regex")
});

// ─────────────────────────────────────────────────────────────────────────────
// Domain types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeMatch {
    pub season: u16,
    pub episodes: Vec<u16>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovieMatch {
    pub title: String,
    pub year: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateMatch {
    pub date: String,
    pub title: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Title extraction
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the portion of a filename that comes *before* the first episode marker.
/// This helps avoid including episode titles in the show title.
pub fn extract_title_before_episode(filename: &str) -> String {
    // Check for date pattern first (to avoid confusion with dash episode patterns)
    if let Some(m) = DATE_PATTERN.find(filename) {
        let mut title_part = &filename[..m.start()];
        title_part = title_part.trim_end_matches(|c: char| {
            c.is_whitespace() || c == '-' || c == '\u{2013}' || c == '\u{2014}'
        });
        return title_part.to_string();
    }

    for regex in ALL_EPISODE_REGEXES.iter() {
        if let Some(m) = regex.find(filename) {
            let mut title_part = &filename[..m.start()];
            title_part = title_part.trim_end_matches(|c: char| {
                c.is_whitespace() || c == '-' || c == '\u{2013}' || c == '\u{2014}'
            });
            return title_part.to_string();
        }
    }

    filename.to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Episode parsing
// ─────────────────────────────────────────────────────────────────────────────

static ALL_EPISODE_REGEXES: &[&Lazy<Regex>] = &[
    &EPISODE_SXXEYY,
    &EPISODE_X_FORMAT,
    &EPISODE_SEP,
    &EPISODE_SXXEPYY,
    &EPISODE_WORD,
    &EPISODE_AT_START,
    &EPISODE_DASH,
];

/// Unique discriminants for each episode regex so we can avoid lifetime issues
/// from comparing `&Regex` pointers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EpisodeRegexId {
    SxxEyy,
    XFormat,
    Sep,
    SxxEpyy,
    Word,
    AtStart,
    Dash,
}

fn regex_id(regex: &Regex) -> EpisodeRegexId {
    if std::ptr::eq(regex.as_str(), EPISODE_SXXEYY.as_str()) {
        EpisodeRegexId::SxxEyy
    } else if std::ptr::eq(regex.as_str(), EPISODE_X_FORMAT.as_str()) {
        EpisodeRegexId::XFormat
    } else if std::ptr::eq(regex.as_str(), EPISODE_SEP.as_str()) {
        EpisodeRegexId::Sep
    } else if std::ptr::eq(regex.as_str(), EPISODE_SXXEPYY.as_str()) {
        EpisodeRegexId::SxxEpyy
    } else if std::ptr::eq(regex.as_str(), EPISODE_WORD.as_str()) {
        EpisodeRegexId::Word
    } else if std::ptr::eq(regex.as_str(), EPISODE_AT_START.as_str()) {
        EpisodeRegexId::AtStart
    } else if std::ptr::eq(regex.as_str(), EPISODE_DASH.as_str()) {
        EpisodeRegexId::Dash
    } else {
        unreachable!()
    }
}

/// Extract episode information from a filename.
///
/// When `known_season` is provided, patterns that only carry an episode number
/// (or dash-separated season-episode) are accepted.
pub fn extract_episode_info(filename: &str, known_season: Option<u16>) -> Option<EpisodeMatch> {
    let mut candidates: Vec<(usize, EpisodeMatch)> = Vec::new();

    for regex in ALL_EPISODE_REGEXES.iter() {
        if let Some(m) = regex.find(filename) {
            if let Some(em) = try_build_episode_match(regex, filename, known_season) {
                candidates.push((m.start(), em));
                // We only take the first match per regex, like Python
                break;
            }
        }
    }

    // Return the match that starts earliest in the filename.
    candidates.sort_by_key(|(start, _)| *start);
    candidates.into_iter().next().map(|(_, em)| em)
}

fn try_build_episode_match(
    regex: &Regex,
    filename: &str,
    known_season: Option<u16>,
) -> Option<EpisodeMatch> {
    let id = regex_id(regex);
    let caps = regex.captures(filename)?;

    let season = if id == EpisodeRegexId::AtStart || id == EpisodeRegexId::Word {
        known_season?
    } else if let Some(s) = caps.name("season") {
        s.as_str().parse::<u16>().ok()?
    } else {
        known_season?
    };

    let ep1 = caps.name("ep1")?.as_str().parse::<u16>().ok()?;
    let mut episodes = vec![ep1];

    if let Some(ep2_raw) = caps.name("ep2") {
        let ep2 = ep2_raw.as_str().parse::<u16>().ok()?;
        if ep2 > ep1 {
            episodes = (ep1..=ep2).collect();
        } else if ep2 != ep1 {
            episodes.push(ep2);
        }
    }

    // Dash-separated season-episode: only use if season matches known_season
    if id == EpisodeRegexId::Dash
        && (known_season.is_none() || season != known_season.unwrap_or(season))
    {
        return None;
    }

    // Episode at start: only use if known_season is available
    if id == EpisodeRegexId::AtStart && known_season.is_none() {
        return None;
    }

    let title = caps.get(0).and_then(|m| extract_episode_title_from_match(filename, m.end()));

    Some(EpisodeMatch { season, episodes, title })
}

fn extract_episode_title_from_match(filename: &str, after_pos: usize) -> Option<String> {
    let remainder = filename[after_pos..].trim_start_matches(|c: char| c.is_whitespace());
    if remainder.is_empty() {
        return None;
    }

    let title: String;
    // Pattern 1: " - Episode Title" or "- Episode Title"
    let dash_re = Regex::new(r"^\s*[-\u{2013}]\s*(.+)").ok()?;
    if let Some(c) = dash_re.captures(remainder) {
        title = c.get(1)?.as_str().trim().to_string();
    } else if let Some(end) = remainder.find(')') {
        // Pattern: " (Episode Title)"
        let inner = remainder[1..end].trim();
        if !inner.is_empty() {
            title = inner.to_string();
        } else {
            return None;
        }
    } else if remainder.chars().next()?.is_alphabetic() {
        title = remainder.to_string();
    } else {
        return None;
    }

    Some(clean_episode_title(&title))
}

fn clean_episode_title(title: &str) -> String {
    let mut t = title.to_string();
    // Remove [tags]
    let tag_re = Regex::new(r"\[.*?\]").expect("valid tag regex");
    t = tag_re.replace_all(&t, "").to_string();
    // Remove quality markers
    let quality_re = Regex::new(r"(?i)\b\d{3,4}p\b").expect("valid quality regex");
    t = quality_re.replace_all(&t, "").to_string();
    // Remove common technical tokens
    let tech_re = Regex::new(r"(?i)\b(?:WEB-?DL|HDTV|BluRay|x264|x265|HEVC|10bit)\b")
        .expect("valid tech regex");
    t = tech_re.replace_all(&t, "").to_string();
    // Remove parentheses
    let paren_re = Regex::new(r"\([^)]*\)").expect("valid paren regex");
    t = paren_re.replace_all(&t, "").to_string();
    // Remove file extensions
    let ext_re = Regex::new(r"(?i)\.(mkv|mp4|avi|mov|wmv|flv|m4v|mpg|mpeg|webm|ts)$")
        .expect("valid ext regex");
    t = ext_re.replace_all(&t, "").to_string();

    t.trim_matches(|c: char| c.is_whitespace() || c == '.' || c == '-' || c == '_')
        .trim()
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// ─────────────────────────────────────────────────────────────────────────────
// Date / Year / Part / Season / TMDB
// ─────────────────────────────────────────────────────────────────────────────

pub fn extract_date(filename: &str) -> Option<DateMatch> {
    let caps = DATE_PATTERN.captures(filename)?;
    let year = caps["year"].parse::<u16>().ok()?;
    let month = caps["month"].parse::<u16>().ok()?;
    let day = caps["day"].parse::<u16>().ok()?;
    // Reject implausible values (e.g. month 00, day 99)
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(DateMatch { date: format!("{:04}-{:02}-{:02}", year, month, day), title: None })
}

pub fn extract_year(filename: &str) -> Option<u16> {
    // First, try parenthesized year (higher priority)
    if let Some(caps) = PAREN_YEAR.captures(filename) {
        let year_match_pos = caps.get(0)?.start();
        let year = caps["year"].parse::<u16>().ok()?;
        if (1900..=2040).contains(&year) {
            // Check if this year is part of a date pattern
            if !is_year_in_date_context(filename, year_match_pos) {
                return Some(year);
            }
        }
    }

    // Then try standalone year with flexible boundaries
    for caps in STANDALONE_YEAR.captures_iter(filename) {
        let year_match_pos = caps.name("year")?.start();
        let year = caps["year"].parse::<u16>().ok()?;
        if !(1900..=2040).contains(&year) {
            continue;
        }

        // Check if this year is part of a date pattern
        if is_year_in_date_context(filename, year_match_pos) {
            continue;
        }

        // Check if this year immediately follows an episode marker (e.g., S05E06-1976)
        let pre_context = &filename[..year_match_pos];
        if EPISODE_BEFORE_YEAR.is_match(pre_context) {
            continue;
        }

        return Some(year);
    }

    None
}

/// Check whether a year at `position` is part of a YYYY-MM-DD or YYYY.MM.DD
/// by inspecting the surrounding characters.
fn is_year_in_date_context(filename: &str, position: usize) -> bool {
    let context_start = position.saturating_sub(2);
    let context_end = (position + 15).min(filename.len());
    let context = &filename[context_start..context_end];
    DATE_PATTERN.is_match(context)
}

pub fn extract_part(filename: &str) -> Option<u16> {
    let caps = PART_PATTERN.captures(filename)?;
    let part_str = caps.name("part")?.as_str();

    // Try integer first
    if let Ok(n) = part_str.parse::<u16>() {
        return Some(n);
    }

    // Try Roman numeral
    let upper = part_str.to_uppercase();
    if upper.chars().all(|c| "IVX".contains(c)) {
        return Some(roman_to_int(&upper));
    }

    // Try spelled-out number
    let word_number = part_str.to_lowercase();
    static WORD_VALUES: &[(&str, u16)] = &[
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
        ("ten", 10),
    ];
    for &(w, v) in WORD_VALUES.iter() {
        if word_number == w {
            return Some(v);
        }
    }

    None
}

fn roman_to_int(s: &str) -> u16 {
    let values: std::collections::HashMap<char, u16> =
        [('I', 1), ('V', 5), ('X', 10)].iter().copied().collect();
    let mut total = 0u16;
    let mut prev = 0u16;
    for ch in s.chars().rev() {
        let value = *values.get(&ch).unwrap_or(&0);
        if value < prev {
            total = total.saturating_sub(value);
        } else {
            total = total.saturating_add(value);
        }
        prev = value;
    }
    total
}

pub fn extract_season_from_folder(folder_name: &str) -> Option<u16> {
    let caps = SEASON_FOLDER_PATTERN.captures(folder_name)?;
    let season_str = caps.name("season").or_else(|| caps.name("season2"))?.as_str();
    season_str.parse::<u16>().ok()
}

/// Extract TMDB ID from any path component (closest match wins).
pub fn extract_tmdb_id_from_path(path: &str) -> Option<String> {
    // Walk path components from right (file) to left (root)
    for component in path.rsplit(['/', '\\']) {
        if let Some(caps) = TMDB_ID_PATTERN.captures(component) {
            return Some(caps.get(1)?.as_str().to_string());
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Title cleanup
// ─────────────────────────────────────────────────────────────────────────────

static COMPOUND_END_PATTERNS: &[&str] = &[
    r"\s+disney\s+plus$",
    r"\s+paramount\s+plus$",
    r"\s+disney\s+animated$",
    r"\s+roku\s+original$",
    r"\s+netflix\s+original$",
    r"\s+hulu\s+original$",
    r"\s+french\s+korean$",
];

static SINGLE_WORD_DESCRIPTORS: &[&str] = &[
    "korean",
    "japanese",
    "chinese",
    "french",
    "spanish",
    "german",
    "italian",
    "russian",
    "polish",
    "persian",
    "irish",
    "belgian",
    "british",
    "telugu",
    "netflix",
    "hulu",
    "original",
    "roku",
    "disney",
    "plus",
    "paramount",
    "animated",
    "criterion",
    "hdr",
    "amzn",
];

/// Clean up a title string by removing patterns and normalizing.
pub fn clean_title(raw: &str) -> String {
    clean_title_with_year(raw, None)
}

/// Clean up a title string, optionally passing the year that was already
/// extracted so we only remove *that* year instead of all year-like numbers.
pub fn clean_title_with_year(raw: &str, extracted_year: Option<u16>) -> String {
    let mut title = raw.to_string();

    // 1. Preserve Vol X / Volume X
    let vol_re = Regex::new(r"(?i)\b(Vol|Volume)\s+(\d+)\b").expect("valid vol regex");
    let mut vol_placeholders: Vec<(String, String)> = Vec::new();
    let title_clone = title.clone();
    for caps in vol_re.captures_iter(&title_clone) {
        let orig = caps.get(0).unwrap().as_str().to_string();
        let num = &caps[2];
        let placeholder = format!("VOLPLACEHOLDER{}ENDVOL", num);
        vol_placeholders.push((orig.clone(), placeholder.clone()));
        title = title.replace(&orig, &placeholder);
    }

    // 2. Preserve parenthetical info that isn't a year
    let mut preserved_parens: Vec<String> = Vec::new();
    for caps in PAREN_RE.captures_iter(&title.clone()) {
        let content = &caps[1];
        if (!content.trim().chars().all(|c| c.is_ascii_digit()) || content.trim().len() != 4)
            && !content.trim().is_empty()
        {
            preserved_parens.push(format!("({})", content));
        }
    }

    // 3. Remove date patterns
    title = DATE_PATTERN.replace_all(&title, "").to_string();

    // 4. Remove year in parentheses
    title = YEAR_PAREN_RE.replace_all(&title, "").to_string();

    // 5. Remove release group tags [xxx]
    title = BRACKET_RE.replace_all(&title, "").to_string();

    // 6. Convert separators to spaces
    title = SEP_RE.replace_all(&title, " ").to_string();

    // 7. Remove episode patterns (skip episode-at-start)
    for regex in ALL_EPISODE_REGEXES.iter() {
        if regex.as_str() == EPISODE_AT_START.as_str() {
            continue;
        }
        title = regex.replace_all(&title, "").to_string();
    }

    // 8. Remove standalone year
    if let Some(ey) = extracted_year {
        let year_pat = Regex::new(&format!(r"(?i)\s+{}[-\s]*", ey)).expect("valid year pat");
        if year_pat.is_match(&title) {
            title = year_pat.replace_all(&title, " ").to_string();
        } else if title.trim() == ey.to_string() {
            // keep year-only title
        } else {
            let start_year_pat =
                Regex::new(&format!(r"^{}[-\s]*", ey)).expect("valid start year pat");
            title = start_year_pat.replace_all(&title, "").to_string();
        }
    } else {
        let year_pat = Regex::new(r"(?i)\s+(19\d{2}|20\d{2})([\s]|$)").expect("valid year pat");
        title = year_pat
            .replace_all(&title, |caps: &regex::Captures| {
                let y = caps[1].parse::<u16>().unwrap_or(0);
                if (1900..=2040).contains(&y) {
                    caps[2].to_string() // keep trailing whitespace or end
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();
    }

    // 9. Remove part indicators
    title = PART_PATTERN.replace_all(&title, "").to_string();

    // 10. Remove quality/format/codec/release descriptors
    title = RES_RE.replace_all(&title, "").to_string();
    title = FORMAT_RE.replace_all(&title, "").to_string();
    title = H264_RE.replace_all(&title, "").to_string();
    title = CODEC_RE.replace_all(&title, "").to_string();
    title = BIT_DEPTH_RE.replace_all(&title, "").to_string();
    title = AUDIO_RE.replace_all(&title, "").to_string();
    title = SOURCE_RE.replace_all(&title, "").to_string();
    title = RELEASE_RE.replace_all(&title, "").to_string();
    title = EDITION_RE.replace_all(&title, "").to_string();

    // 11. Remove descriptor words from end (compound then single-word)
    title = SPACE_RE.replace_all(&title, " ").to_string();

    let compound_end_re: Vec<Regex> = COMPOUND_END_PATTERNS
        .iter()
        .map(|pat| Regex::new(&format!("(?i){}", pat)).expect("valid compound regex"))
        .collect();
    let end_descriptor_re: Vec<Regex> = SINGLE_WORD_DESCRIPTORS
        .iter()
        .map(|word| {
            Regex::new(&format!("(?i)\\s+{}\\s*$", regex::escape(word)))
                .expect("valid descriptor regex")
        })
        .collect();
    let start_descriptor_re: Vec<Regex> = SINGLE_WORD_DESCRIPTORS
        .iter()
        .map(|word| {
            Regex::new(&format!("(?i)^{}\\s+", regex::escape(word)))
                .expect("valid start descriptor regex")
        })
        .collect();

    'outer: loop {
        let before = title.clone();
        for re in &compound_end_re {
            title = re.replace_all(&title, "").to_string();
        }
        for re in &end_descriptor_re {
            title = re.replace_all(&title, "").to_string();
        }
        let new_title = title.trim().to_string();
        if new_title == before.trim() {
            title = new_title;
            break 'outer;
        }
        title = new_title;
    }

    // Remove from start
    'outer2: loop {
        let before = title.clone();
        for re in &start_descriptor_re {
            title = re.replace_all(&title, "").to_string();
        }
        let new_title = title.trim().to_string();
        if new_title == before.trim() {
            title = new_title;
            break 'outer2;
        }
        title = new_title;
    }

    // 12. Remove known director/actor names
    title = NAMES_RE.replace_all(&title, "").to_string();

    // 13. Remove phrases
    title = PHRASES_RE.replace_all(&title, "").to_string();

    // 14. Remove season-range expressions
    title = SEASON_RANGE_RE.replace_all(&title, "").to_string();
    title = SEASON_WORD_RE.replace_all(&title, "").to_string();
    title = SEASON_COMPACT_RE.replace_all(&title, "").to_string();

    // 15. Remove collection words and numeric ranges
    title = COLLECTION_RE.replace_all(&title, "").to_string();
    title = NUM_RANGE_RE.replace_all(&title, "").to_string();
    title = TO_NUM_RE.replace_all(&title, "").to_string();

    // 16. Unquote numeric-only tokens
    title = QUOTE_NUM_RE.replace_all(&title, "$num").to_string();

    // 17. Remove remaining brackets and known release groups
    title = BRACKET2_RE.replace_all(&title, "").to_string();
    title = RELEASE_GROUP_RE.replace_all(&title, "").to_string();
    title = MIXED_GROUP_RE.replace_all(&title, "").to_string();

    // 18. Remove stray episode markers
    title = STRAY_EP_RE.replace_all(&title, "").to_string();

    // 19. Remove leftover stranded digits (only when not at start)
    if !title.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
        title = STRAY_DIGITS_RE.replace_all(&title, "").to_string();
    }

    // 20. Remove remaining parentheses
    title = PAREN2_RE.replace_all(&title, "").to_string();

    // 21. Collapse spaces
    title = title.split_whitespace().collect::<Vec<_>>().join(" ");

    // 22. Restore Vol placeholders
    for (_, placeholder) in vol_placeholders.iter() {
        if let Some(caps) = VOL_PH_RE.captures(placeholder) {
            let num = &caps[1];
            title = title.replace(placeholder, &format!("Vol {}", num));
        }
    }

    // 23. Restore hyphenated compounds
    title = SPIDER_MAN_RE.replace_all(&title, "Spider-Man").to_string();
    title = SPIDER_VERSE_RE.replace_all(&title, "Spider-Verse").to_string();
    title = X_MEN_RE.replace_all(&title, "X-Men").to_string();

    // 24. Restore preserved parentheticals
    if !preserved_parens.is_empty() {
        title = format!("{} {}", title, preserved_parens.join(" "));
    }

    title.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}
