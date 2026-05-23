use pretty_assertions::assert_eq;
use rosey_core::*;

// ─────────────────────────────────────────────────────────────────────────────
// Episode parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn episode_sxxeyy_basic() {
    let m = extract_episode_info("Show.S01E05.mkv", None).unwrap();
    assert_eq!(m.season, 1);
    assert_eq!(m.episodes, vec![5]);
}

#[test]
fn episode_sxxeyy_range() {
    let m = extract_episode_info("Show.S02E10-E11.mkv", None).unwrap();
    assert_eq!(m.season, 2);
    assert_eq!(m.episodes, vec![10, 11]);
}

#[test]
fn episode_xformat() {
    let m = extract_episode_info("Show.1x05.mkv", None).unwrap();
    assert_eq!(m.season, 1);
    assert_eq!(m.episodes, vec![5]);
}

#[test]
fn episode_xformat_range() {
    let m = extract_episode_info("Program.2x05-07.mp4", None).unwrap();
    assert_eq!(m.season, 2);
    assert_eq!(m.episodes, vec![5, 6, 7]);
}

#[test]
fn episode_sep_format() {
    let m = extract_episode_info("Three's Company Season 1 EP01.mp4", None).unwrap();
    assert_eq!(m.season, 1);
    assert_eq!(m.episodes, vec![1]);
}

#[test]
fn episode_sxxepyy_format() {
    let m = extract_episode_info("All.in.the.family.S6EP09.mp4", None).unwrap();
    assert_eq!(m.season, 6);
    assert_eq!(m.episodes, vec![9]);
}

#[test]
fn episode_word_format() {
    let m = extract_episode_info("Happy Days - Episode 13.avi", Some(5)).unwrap();
    assert_eq!(m.season, 5);
    assert_eq!(m.episodes, vec![13]);
}

#[test]
fn episode_at_start_with_known_season() {
    let m = extract_episode_info("05 Title Here.mkv", Some(1)).unwrap();
    assert_eq!(m.season, 1);
    assert_eq!(m.episodes, vec![5]);
}

#[test]
fn episode_at_start_without_known_season_ignored() {
    assert_eq!(extract_episode_info("05 Title Here.mkv", None), None);
}

#[test]
fn episode_dash_with_matching_known_season() {
    // Use a filename where AtStart does NOT steal the match
    let m = extract_episode_info("Happy Days 11-07 Title.mkv", Some(11)).unwrap();
    assert_eq!(m.season, 11);
    assert_eq!(m.episodes, vec![7]);
}

#[test]
fn episode_dash_without_known_season_ignored() {
    assert_eq!(extract_episode_info("Happy Days 11-07 Title.mkv", None), None);
}

#[test]
fn episode_dash_mismatched_season_ignored() {
    assert_eq!(extract_episode_info("Happy Days 11-07 Title.mkv", Some(1)), None);
}

#[test]
fn episode_at_start_overrides_dash_when_at_start() {
    // When episode number is at the very start, AtStart pattern wins because
    // it is checked before Dash in the Python pattern list.
    let m = extract_episode_info("02-15 Title.mkv", Some(2)).unwrap();
    assert_eq!(m.season, 2);
    assert_eq!(m.episodes, vec![2]);
}

#[test]
fn episode_multi_ep_sxxeyy_range() {
    let m = extract_episode_info("Show.S04E01-E04.mkv", None).unwrap();
    assert_eq!(m.season, 4);
    assert_eq!(m.episodes, vec![1, 2, 3, 4]);
}

#[test]
fn episode_case_insensitive() {
    let low = extract_episode_info("show.s01e05.mkv", None).unwrap();
    let up = extract_episode_info("SHOW.S01E05.MKV", None).unwrap();
    assert_eq!(low.season, up.season);
    assert_eq!(low.episodes, up.episodes);
}

#[test]
fn episode_no_match_for_movie() {
    assert_eq!(extract_episode_info("Just a Movie (2020).mkv", None), None);
}

#[test]
fn episode_multi_digit() {
    let m = extract_episode_info("Show.S01E123.mkv", None).unwrap();
    assert_eq!(m.season, 1);
    assert_eq!(m.episodes, vec![123]);
}

#[test]
fn episode_title_extracted_after_dash() {
    let m = extract_episode_info("Show.S01E02 - Episode Title.mkv", None).unwrap();
    assert_eq!(m.season, 1);
    assert_eq!(m.episodes, vec![2]);
    assert_eq!(m.title, Some("Episode Title".into()));
}

#[test]
fn episode_title_extracted_in_parens() {
    let m = extract_episode_info("Show S01E09 (Brenda's Last Date).mp4", None).unwrap();
    assert_eq!(m.season, 1);
    assert_eq!(m.episodes, vec![9]);
    assert_eq!(m.title, Some("Brenda's Last Date".into()));
}

#[test]
fn episode_title_none_when_not_present() {
    let m = extract_episode_info("Show.S01E02.mkv", None).unwrap();
    assert_eq!(m.title, None);
}

#[test]
fn extract_title_before_episode_basic() {
    // In Python, extract_title_before_episode trims trailing whitespace/dashes
    // but leaves dots (they are cleaned later by clean_title).
    assert_eq!(extract_title_before_episode("Show.S01E02.mkv"), "Show.");
}

#[test]
fn extract_title_before_episode_date_priority() {
    assert_eq!(extract_title_before_episode("Series Name - 2018-03-10.mkv"), "Series Name");
}

// ─────────────────────────────────────────────────────────────────────────────
// Date parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn date_with_dashes() {
    let d = extract_date("Show.2020-03-15.mkv").unwrap();
    assert_eq!(d.date, "2020-03-15");
}

#[test]
fn date_with_dots() {
    let d = extract_date("Show.2021.12.25.mkv").unwrap();
    assert_eq!(d.date, "2021-12-25");
}

#[test]
fn date_no_match() {
    assert_eq!(extract_date("Movie (2020).mkv"), None);
}

#[test]
fn date_rejects_invalid_month_day() {
    // Month 00 or day 99 should be rejected
    assert_eq!(extract_date("Show.2020-00-15.mkv"), None);
    assert_eq!(extract_date("Show.2020-12-99.mkv"), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// Year parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn year_in_parentheses() {
    assert_eq!(extract_year("The Matrix (1999).mkv"), Some(1999));
}

#[test]
fn year_standalone() {
    assert_eq!(extract_year("Movie 2020.mkv"), Some(2020));
}

#[test]
fn year_no_year() {
    assert_eq!(extract_year("Movie Title.mkv"), None);
}

#[test]
fn year_ignores_date_context() {
    assert_eq!(extract_year("Daily.Show.2024-05-01.mkv"), None);
}

#[test]
fn year_ignores_year_after_episode_marker() {
    assert_eq!(extract_year("Columbo S05E06-1976.mp4"), None);
}

#[test]
fn year_prefers_first_parenthesized() {
    assert_eq!(extract_year("Movie (1999) vs (2020).mkv"), Some(1999));
}

// ─────────────────────────────────────────────────────────────────────────────
// Part parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn part_with_space() {
    assert_eq!(extract_part("Movie Part 1.mkv"), Some(1));
}

#[test]
fn part_abbreviated() {
    assert_eq!(extract_part("Movie pt2.mkv"), Some(2));
}

#[test]
fn part_with_dot() {
    assert_eq!(extract_part("Movie.Part.3.mkv"), Some(3));
}

#[test]
fn part_no_match() {
    assert_eq!(extract_part("Movie.mkv"), None);
}

#[test]
fn part_roman_numeral() {
    assert_eq!(extract_part("Movie Part III.mkv"), Some(3));
}

#[test]
fn part_spelled_out() {
    assert_eq!(extract_part("Movie Part One.mkv"), Some(1));
    assert_eq!(extract_part("Movie Part Two.mkv"), Some(2));
}

// ─────────────────────────────────────────────────────────────────────────────
// Season folder parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn season_standard() {
    assert_eq!(extract_season_from_folder("Season 01"), Some(1));
}

#[test]
fn season_no_zero_pad() {
    assert_eq!(extract_season_from_folder("Season 1"), Some(1));
}

#[test]
fn season_case_insensitive() {
    assert_eq!(extract_season_from_folder("season 02"), Some(2));
}

#[test]
fn season_compact() {
    assert_eq!(extract_season_from_folder("S03"), Some(3));
}

#[test]
fn season_no_match() {
    assert_eq!(extract_season_from_folder("Random Folder"), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// TMDB ID extraction
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tmdb_id_from_directory() {
    assert_eq!(
        extract_tmdb_id_from_path("/movies/The Matrix (1999) [tmdbid-603]/movie.mkv"),
        Some("603".into())
    );
}

#[test]
fn tmdb_id_from_nested_path() {
    assert_eq!(
        extract_tmdb_id_from_path("/tv/Breaking Bad (2008) [tmdbid-1396]/Season 01/S01E01.mkv"),
        Some("1396".into())
    );
}

#[test]
fn tmdb_id_no_match() {
    assert_eq!(extract_tmdb_id_from_path("/movies/Random Movie/movie.mkv"), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// Title cleanup
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn clean_removes_separators() {
    let r = clean_title("Movie_Name-With.Separators");
    assert!(!r.contains('_'));
    assert!(!r.contains('-'));
    assert!(!r.contains('.'));
    assert!(r.contains(' '));
}

#[test]
fn clean_collapses_multiple_spaces() {
    assert_eq!(clean_title("Movie   Name   Here"), "Movie Name Here");
}

#[test]
fn clean_removes_episode_patterns() {
    let r = clean_title("Show.S01E05.Episode.Title");
    assert!(!r.contains("S01E05"));
    assert!(r.contains("Show"));
}

#[test]
fn clean_removes_quality_and_codec() {
    let r = clean_title_with_year("The.Matrix.1999.1080p.x264", Some(1999));
    assert!(r.contains("Matrix"));
    assert!(!r.contains("1999"));
    assert!(!r.contains("1080p"));
    assert!(!r.contains("x264"));
}

#[test]
fn clean_preserves_vol() {
    let r = clean_title("Guardians.of.the.Galaxy.Vol.3.2023.1080p.mkv");
    assert!(r.contains("Vol 3"));
}

#[test]
fn clean_restores_spider_man() {
    let r = clean_title("Spider.Man.2022.mkv");
    assert!(r.contains("Spider-Man"));
}

#[test]
fn clean_title_basic() {
    assert_eq!(clean_title("Example.Show - "), "Example Show");
}

#[test]
fn clean_title_underscores() {
    assert_eq!(clean_title("Show_Name_S01E01"), "Show Name");
}
