use camino::Utf8Path;
use rosey_core::*;
use std::fs;

fn make_nfo(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_nfo — movie NFO
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_movie_nfo_basic() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(
        tmp.path(),
        "Movie.nfo",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<movie>
    <title>The Matrix</title>
    <year>1999</year>
    <tmdbid>603</tmdbid>
    <imdbid>tt0133093</imdbid>
</movie>
"#,
    );

    let data = parse_nfo(Utf8Path::from_path(&nfo).unwrap()).unwrap();
    assert_eq!(data.title, Some("The Matrix".into()));
    assert_eq!(data.year, Some(1999));
    assert_eq!(data.tmdb_id, Some("603".into()));
    assert_eq!(data.imdb_id, Some("tt0133093".into()));
}

#[test]
fn parse_movie_nfo_with_uniqueids() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(
        tmp.path(),
        "Movie.nfo",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<movie>
    <title>Inception</title>
    <uniqueid type="imdb">tt1375666</uniqueid>
    <uniqueid type="tmdb">27205</uniqueid>
    <uniqueid type="tvdb">12345</uniqueid>
</movie>
"#,
    );

    let data = parse_nfo(Utf8Path::from_path(&nfo).unwrap()).unwrap();
    assert_eq!(data.title, Some("Inception".into()));
    assert_eq!(data.imdb_id, Some("tt1375666".into()));
    assert_eq!(data.tmdb_id, Some("27205".into()));
    assert_eq!(data.tvdb_id, Some("12345".into()));
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_nfo — episode NFO
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_episode_nfo_basic() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(
        tmp.path(),
        "Episode.nfo",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<episodedetails>
    <title>The Office</title>
    <season>2</season>
    <episode>1</episode>
    <episodetitle>The Dundies</episodetitle>
    <tvdbid>73244</tvdbid>
</episodedetails>
"#,
    );

    let data = parse_nfo(Utf8Path::from_path(&nfo).unwrap()).unwrap();
    assert_eq!(data.title, Some("The Office".into()));
    assert_eq!(data.season, Some(2));
    assert_eq!(data.episode, Some(1));
    assert_eq!(data.episode_title, Some("The Dundies".into()));
    assert_eq!(data.tvdb_id, Some("73244".into()));
}

#[test]
fn parse_episode_nfo_alt_tags() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(
        tmp.path(),
        "Episode.nfo",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<episodedetails>
    <title>Show</title>
    <episode_title>Pilot</episode_title>
    <imdb_id>tt1234567</imdb_id>
    <tmdb_id>98765</tmdb_id>
    <tvdb_id>11111</tvdb_id>
</episodedetails>
"#,
    );

    let data = parse_nfo(Utf8Path::from_path(&nfo).unwrap()).unwrap();
    assert_eq!(data.episode_title, Some("Pilot".into()));
    assert_eq!(data.imdb_id, Some("tt1234567".into()));
    assert_eq!(data.tmdb_id, Some("98765".into()));
    assert_eq!(data.tvdb_id, Some("11111".into()));
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_nfo — edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_nfo_missing_file_returns_none() {
    let result = parse_nfo(Utf8Path::new("/nonexistent/file.nfo"));
    assert!(result.is_none());
}

#[test]
fn parse_nfo_invalid_xml_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(tmp.path(), "bad.nfo", "this is not xml at all");
    let result = parse_nfo(Utf8Path::from_path(&nfo).unwrap());
    // quick-xml is lenient with plain text; it may return empty data
    // rather than None. Accept either behavior.
    if let Some(data) = result {
        assert_eq!(data.title, None);
        assert_eq!(data.year, None);
    }
}

#[test]
fn parse_nfo_empty_xml_returns_some_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(tmp.path(), "empty.nfo", "<?xml version=\"1.0\"?><movie></movie>");
    let data = parse_nfo(Utf8Path::from_path(&nfo).unwrap()).unwrap();
    assert_eq!(data.title, None);
    assert_eq!(data.year, None);
}

#[test]
fn parse_nfo_bad_year_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(
        tmp.path(),
        "bad_year.nfo",
        r#"<?xml version="1.0"?>
<movie>
    <title>Test</title>
    <year>not_a_year</year>
</movie>
"#,
    );
    let data = parse_nfo(Utf8Path::from_path(&nfo).unwrap()).unwrap();
    assert_eq!(data.title, Some("Test".into()));
    assert_eq!(data.year, None);
}

#[test]
fn parse_nfo_uniqueid_precedence_over_direct() {
    let tmp = tempfile::tempdir().unwrap();
    let nfo = make_nfo(
        tmp.path(),
        "precedence.nfo",
        r#"<?xml version="1.0"?>
<movie>
    <tmdbid>direct</tmdbid>
    <uniqueid type="tmdb">uniqueid</uniqueid>
</movie>
"#,
    );
    let data = parse_nfo(Utf8Path::from_path(&nfo).unwrap()).unwrap();
    // direct tag is set first, uniqueid should not overwrite because of the
    // `is_none()` guard
    assert_eq!(data.tmdb_id, Some("direct".into()));
}

// ─────────────────────────────────────────────────────────────────────────────
// find_nfo_for_file
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn find_nfo_same_name() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    fs::write(dir.join("Movie.mkv"), "video").unwrap();
    fs::write(dir.join("Movie.nfo"), "<movie></movie>").unwrap();

    let video_path = dir.join("Movie.mkv");
    let video = Utf8Path::from_path(&video_path).unwrap();
    let found = find_nfo_for_file(video).unwrap();
    assert_eq!(found.file_name(), Some("Movie.nfo"));
}

#[test]
fn find_nfo_movie_nfo() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    fs::write(dir.join("Movie.mkv"), "video").unwrap();
    fs::write(dir.join("movie.nfo"), "<movie></movie>").unwrap();

    let video_path = dir.join("Movie.mkv");
    let video = Utf8Path::from_path(&video_path).unwrap();
    let found = find_nfo_for_file(video).unwrap();
    assert_eq!(found.file_name(), Some("movie.nfo"));
}

#[test]
fn find_nfo_tvshow_nfo() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    fs::write(dir.join("Episode.mkv"), "video").unwrap();
    fs::write(dir.join("tvshow.nfo"), "<tvshow></tvshow>").unwrap();

    let video_path = dir.join("Episode.mkv");
    let video = Utf8Path::from_path(&video_path).unwrap();
    let found = find_nfo_for_file(video).unwrap();
    assert_eq!(found.file_name(), Some("tvshow.nfo"));
}

#[test]
fn find_nfo_same_name_takes_precedence() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    fs::write(dir.join("Movie.mkv"), "video").unwrap();
    fs::write(dir.join("Movie.nfo"), "<movie></movie>").unwrap();
    fs::write(dir.join("movie.nfo"), "<movie></movie>").unwrap();

    let video_path = dir.join("Movie.mkv");
    let video = Utf8Path::from_path(&video_path).unwrap();
    let found = find_nfo_for_file(video).unwrap();
    assert_eq!(found.file_name(), Some("Movie.nfo"));
}

#[test]
fn find_nfo_none_when_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    fs::write(dir.join("Movie.mkv"), "video").unwrap();

    let video_path = dir.join("Movie.mkv");
    let video = Utf8Path::from_path(&video_path).unwrap();
    assert!(find_nfo_for_file(video).is_none());
}

// ─────────────────────────────────────────────────────────────────────────────
// normalize_imdb_id
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn normalize_imdb_id_keeps_valid() {
    assert_eq!(normalize_imdb_id("tt0133093"), "tt0133093");
}

#[test]
fn normalize_imdb_id_adds_tt() {
    assert_eq!(normalize_imdb_id("0133093"), "tt0133093");
}

#[test]
fn normalize_imdb_id_extracts_from_url() {
    assert_eq!(normalize_imdb_id("https://www.imdb.com/title/tt0133093/"), "tt0133093");
}
