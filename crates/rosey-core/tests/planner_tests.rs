use pretty_assertions::assert_eq;
use rosey_core::*;

fn movie_item(title: &str, year: Option<u16>, part: Option<u16>) -> MediaItem {
    let mut item = MediaItem::unknown("/source/file.mkv");
    item.kind = MediaKind::Movie;
    item.title = Some(title.into());
    item.year = year;
    item.part = part;
    item
}

fn episode_item(
    title: &str,
    year: Option<u16>,
    season: u16,
    episodes: Vec<u16>,
    part: Option<u16>,
) -> MediaItem {
    let mut item = MediaItem::unknown("/source/file.mkv");
    item.kind = MediaKind::Episode;
    item.title = Some(title.into());
    item.year = year;
    item.season = Some(season);
    item.episodes = episodes;
    item.part = part;
    item
}

// ─────────────────────────────────────────────────────────────────────────────
// Sanitize
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sanitize_basic() {
    assert_eq!(sanitize_name("Normal Movie Name"), "Normal Movie Name");
}

#[test]
fn sanitize_invalid_chars() {
    let r = sanitize_name("Movie<>:\"/\\|?*Name");
    assert!(!r.contains('<'));
    assert!(!r.contains('>'));
    assert!(!r.contains(':'));
    assert!(!r.contains('"'));
    assert!(!r.contains('/'));
    assert!(!r.contains('\\'));
    assert!(!r.contains('|'));
    assert!(!r.contains('?'));
    assert!(!r.contains('*'));
}

#[test]
fn sanitize_multiple_spaces() {
    assert_eq!(sanitize_name("Movie   Name   Here"), "Movie Name Here");
}

#[test]
fn sanitize_leading_trailing() {
    let r = sanitize_name("  Movie Name.  ");
    assert!(!r.starts_with(' '));
    assert!(!r.ends_with(' '));
    assert!(!r.ends_with('.'));
}

#[test]
fn sanitize_reserved_names() {
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
    for name in &reserved {
        let r = sanitize_name(name);
        assert_ne!(r, *name);
    }
}

#[test]
fn sanitize_empty() {
    assert_eq!(sanitize_name(""), "unknown");
}

// ─────────────────────────────────────────────────────────────────────────────
// Title case
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn title_case_basic() {
    assert_eq!(title_case("the office"), "The Office");
    assert_eq!(title_case("all in the family"), "All in the Family");
}

// ─────────────────────────────────────────────────────────────────────────────
// Movie planning
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn movie_basic() {
    let planner = Planner { movies_root: "/movies".into(), tv_root: "".into() };
    let item = movie_item("The Matrix", Some(1999), None);
    let dest = planner.plan_destination(&item);
    let dest = dest.as_str();
    assert!(dest.starts_with("/movies"));
    assert!(dest.contains("The Matrix"));
    assert!(dest.contains("(1999)"));
    assert!(dest.ends_with(".mkv"));
}

#[test]
fn movie_without_year() {
    let planner = Planner { movies_root: "/movies".into(), tv_root: "".into() };
    let item = movie_item("Inception", None, None);
    let dest = planner.plan_destination(&item);
    let dest = dest.as_str();
    assert!(dest.starts_with("/movies"));
    assert!(dest.contains("Inception"));
    assert!(dest.ends_with(".mkv"));
}

#[test]
fn movie_multipart() {
    let planner = Planner { movies_root: "/movies".into(), tv_root: "".into() };
    let item = movie_item("Kill Bill", Some(2003), Some(1));
    let dest = planner.plan_destination(&item);
    assert!(dest.as_str().contains("Part 1"));
}

#[test]
fn movie_no_root() {
    let planner = Planner { movies_root: "".into(), tv_root: "".into() };
    let item = movie_item("Movie", None, None);
    assert_eq!(planner.plan_destination(&item).as_str(), "/source/file.mkv");
}

#[test]
fn movie_with_tmdb_id() {
    let planner = Planner { movies_root: "/movies".into(), tv_root: "".into() };
    let mut item = movie_item("Movie", Some(2020), None);
    item.nfo.insert("tmdbid".into(), Some("12345".into()));
    let dest = planner.plan_destination(&item);
    assert!(dest.as_str().contains("[tmdbid-12345]"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Episode planning
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn episode_basic() {
    let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };
    let item = episode_item("The Office", None, 2, vec![1], None);
    let dest = planner.plan_destination(&item);
    let dest = dest.as_str();
    assert!(dest.starts_with("/tv"));
    assert!(dest.contains("The Office"));
    assert!(dest.contains("Season 02"));
    assert!(dest.contains("S02E01"));
    assert!(dest.ends_with(".mkv"));
}

#[test]
fn episode_multi_episode() {
    let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };
    let item = episode_item("Show", None, 1, vec![1, 2], None);
    let dest = planner.plan_destination(&item);
    assert!(dest.as_str().contains("S01E01-E02"));
}

#[test]
fn episode_with_title() {
    let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };
    let mut item = episode_item("The Office", None, 2, vec![1], None);
    item.nfo.insert("episode_title".into(), Some("The Dundies".into()));
    let dest = planner.plan_destination(&item);
    assert!(dest.as_str().contains("The Dundies"));
}

#[test]
fn episode_date_based() {
    let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };
    let mut item = episode_item("Daily Show", None, 1, vec![], None);
    item.date = Some("2020-03-15".into());
    let dest = planner.plan_destination(&item);
    assert!(dest.as_str().contains("2020-03-15"));
}

#[test]
fn episode_multipart() {
    let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };
    let item = episode_item("Show", None, 1, vec![1], Some(1));
    let dest = planner.plan_destination(&item);
    assert!(dest.as_str().contains("Part 1"));
}

#[test]
fn episode_specials() {
    let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };
    let item = episode_item("Show", None, 0, vec![1], None);
    let dest = planner.plan_destination(&item);
    assert!(dest.as_str().contains("Season 00"));
    assert!(dest.as_str().contains("S00E01"));
}

#[test]
fn episode_no_root() {
    let planner = Planner { movies_root: "".into(), tv_root: "".into() };
    let item = episode_item("Show", None, 1, vec![1], None);
    assert_eq!(planner.plan_destination(&item).as_str(), "/source/file.mkv");
}

#[test]
fn episode_all_in_the_family_fixture() {
    let planner =
        Planner { movies_root: "".into(), tv_root: "/mnt/fileserver_storage/videos/tv".into() };
    let mut item = MediaItem::unknown(
        "/mnt/fileserver_incoming/complete/All in the family (1971) [tmdbid-1922]/All in the family (Archie Bunker US TV Series) S6EP09 Grandpa blues (moviesbyrizzo).mp4",
    );
    item.kind = MediaKind::Episode;
    item.title = Some("All in the family".into());
    item.year = Some(1971);
    item.season = Some(6);
    item.episodes = vec![9];
    item.nfo.insert("tmdbid".into(), Some("1922".into()));
    item.nfo.insert("episode_title".into(), Some("Grandpa blues".into()));

    let dest = planner.plan_destination(&item);
    assert_eq!(
        dest.as_str(),
        "/mnt/fileserver_storage/videos/tv/All in the Family (1971) [tmdbid-1922]/Season 06/All in the Family - S06E09 - Grandpa blues.mp4"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Unknown handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn unknown_returns_source() {
    let planner = Planner { movies_root: "/movies".into(), tv_root: "/tv".into() };
    let mut item = MediaItem::unknown("/source/mystery.mkv");
    item.kind = MediaKind::Unknown;
    assert_eq!(planner.plan_destination(&item).as_str(), "/source/mystery.mkv");
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience function
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn plan_path_convenience() {
    let item = movie_item("Movie", Some(2020), None);
    let dest = plan_path(&item, "/movies", "");
    assert!(dest.as_str().starts_with("/movies"));
    assert!(dest.as_str().contains("Movie"));
    assert!(dest.as_str().contains("(2020)"));
}
