use camino::Utf8PathBuf;
use std::collections::BTreeMap;

use crate::models::{IdentifiedEpisode, MediaItem, MediaKind, Score, TvSeason, TvShow};
use crate::patterns::{extract_imdb_id_from_path, extract_tmdb_id_from_path};
use crate::planner::Planner;

pub fn build_tv_shows(items: &[(MediaItem, Score)], planner: &Planner) -> Vec<TvShow> {
    let mut episode_groups: BTreeMap<ShowGroupKey, Vec<(MediaItem, Score)>> = BTreeMap::new();

    for (item, score) in items {
        if item.kind == MediaKind::Episode {
            let key =
                ShowGroupKey { title: item.title.clone().unwrap_or_default(), year: item.year };
            episode_groups.entry(key).or_default().push((item.clone(), score.clone()));
        }
    }

    let mut shows = Vec::new();
    for (key, episodes) in episode_groups {
        if key.title.is_empty() {
            continue;
        }
        let show = build_single_show(&key, &episodes, planner);
        shows.push(show);
    }

    shows
}

/// Build TV shows from items, discover assets, and resolve all destinations.
///
/// This encapsulates the full pipeline: `build_tv_shows` → `discover_show_assets` →
/// `resolve_asset_destinations` → `plan_destination` per episode. Both CLI and TUI
/// should call this instead of duplicating the steps.
pub fn build_and_resolve_tv_shows(
    items: &[(MediaItem, Score)],
    planner: &Planner,
    tv_root: &str,
) -> Vec<TvShow> {
    let mut shows = build_tv_shows(items, planner);
    for show in &mut shows {
        let assets = crate::show_assets::discover_show_assets(show);
        show.show_assets = assets;
        show.resolve_asset_destinations(tv_root);
        for season in &mut show.seasons {
            for episode in &mut season.episodes {
                episode.destination = planner.plan_destination(&episode.item);
            }
        }
    }
    shows
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ShowGroupKey {
    title: String,
    year: Option<u16>,
}

fn build_single_show(
    key: &ShowGroupKey,
    episodes: &[(MediaItem, Score)],
    planner: &Planner,
) -> TvShow {
    let mut season_map: BTreeMap<u16, Vec<(MediaItem, Score)>> = BTreeMap::new();
    let mut tmdb_id: Option<String> = None;
    let mut tvdb_id: Option<String> = None;
    let mut imdb_id: Option<String> = None;
    let mut source_root: Option<Utf8PathBuf> = None;

    for (item, _score) in episodes {
        let season = item.season.unwrap_or(0);
        season_map.entry(season).or_default().push((item.clone(), _score.clone()));

        if tmdb_id.is_none() {
            tmdb_id = item.nfo.get("tmdbid").and_then(|v| v.clone());
        }
        if tvdb_id.is_none() {
            tvdb_id = item.nfo.get("tvdbid").and_then(|v| v.clone());
        }
        if imdb_id.is_none() {
            imdb_id = item.nfo.get("imdbid").and_then(|v| v.clone());
        }

        if source_root.is_none() {
            if let Some(parent) = item.source_path.parent() {
                source_root = derive_show_root(parent);
            }
        }
    }

    let source_root = source_root.unwrap_or_else(|| Utf8PathBuf::from("/unknown"));

    if tmdb_id.is_none() {
        tmdb_id = extract_tmdb_id_from_path(source_root.as_str());
        if tmdb_id.is_none() {
            for (item, _score) in episodes {
                if let Some(id) = extract_tmdb_id_from_path(item.source_path.as_str()) {
                    tmdb_id = Some(id);
                    break;
                }
            }
        }
    }
    if imdb_id.is_none() {
        imdb_id = extract_imdb_id_from_path(source_root.as_str());
        if imdb_id.is_none() {
            for (item, _score) in episodes {
                if let Some(id) = extract_imdb_id_from_path(item.source_path.as_str()) {
                    imdb_id = Some(id);
                    break;
                }
            }
        }
    }

    let mut seasons = Vec::new();
    for (season_num, season_episodes) in &season_map {
        let mut identified_eps = Vec::new();
        for (item, score) in season_episodes {
            let destination = planner.plan_destination(item);
            identified_eps.push(IdentifiedEpisode {
                item: item.clone(),
                score: score.clone(),
                destination,
            });
        }
        identified_eps.sort_by(|a, b| {
            a.item.season.cmp(&b.item.season).then_with(|| {
                let a_ep = a.item.episodes.first().copied().unwrap_or(0);
                let b_ep = b.item.episodes.first().copied().unwrap_or(0);
                a_ep.cmp(&b_ep)
            })
        });

        seasons.push(TvSeason {
            season_number: *season_num,
            episodes: identified_eps,
            season_assets: Vec::new(),
        });
    }

    let confidence = compute_show_confidence(episodes);
    let mut reasons = Vec::new();
    reasons.push(format!(
        "TV show with {} season(s) and {} episode(s)",
        seasons.len(),
        episodes.len()
    ));
    if tmdb_id.is_some() {
        reasons.push("TMDB ID available".to_string());
    }
    if tvdb_id.is_some() {
        reasons.push("TVDB ID available".to_string());
    }

    TvShow {
        title: key.title.clone(),
        year: key.year,
        tmdb_id,
        tvdb_id,
        imdb_id,
        source_root,
        seasons,
        show_assets: Vec::new(),
        confidence,
        reasons,
    }
}

fn derive_show_root(episode_parent: &camino::Utf8Path) -> Option<Utf8PathBuf> {
    if episode_parent.is_dir() {
        if let Some(show_dir) = episode_parent.parent() {
            if show_dir.is_dir() {
                return Some(show_dir.to_path_buf());
            }
        }
        return Some(episode_parent.to_path_buf());
    }
    None
}

fn compute_show_confidence(episodes: &[(MediaItem, Score)]) -> u8 {
    if episodes.is_empty() {
        return 0;
    }
    let total: i16 = episodes.iter().map(|(_, s)| s.confidence as i16).sum();
    let avg = total / episodes.len() as i16;

    let bonus: i16 = if episodes.len() >= 3 {
        5
    } else if episodes.len() >= 2 {
        3
    } else {
        0
    };

    (avg + bonus).clamp(0, 100) as u8
}

#[cfg(test)]
mod tests {
    use crate::models::{MediaItem, MediaKind};

    fn make_episode(title: &str, year: Option<u16>, season: u16, episode: u16) -> MediaItem {
        let mut item = MediaItem::unknown(format!(
            "/source/Show/Season {:02}/{} - S{:02}E{:02}.mkv",
            season, title, season, episode
        ));
        item.kind = MediaKind::Episode;
        item.title = Some(title.to_string());
        item.year = year;
        item.season = Some(season);
        item.episodes = vec![episode];
        item
    }

    #[test]
    fn build_tv_show_groups_episodes_by_title() {
        use crate::planner::Planner;
        use crate::scorer::score_identification;

        let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };

        let ep1 = make_episode("Breaking Bad", Some(2008), 1, 1);
        let ep2 = make_episode("Breaking Bad", Some(2008), 1, 2);
        let ep3 = make_episode("Breaking Bad", Some(2008), 2, 1);

        let items: Vec<(MediaItem, crate::models::Score)> = vec![
            (ep1, score_identification(&make_episode("Breaking Bad", Some(2008), 1, 1))),
            (ep2, score_identification(&make_episode("Breaking Bad", Some(2008), 1, 2))),
            (ep3, score_identification(&make_episode("Breaking Bad", Some(2008), 2, 1))),
        ];

        let shows = super::build_tv_shows(&items, &planner);

        assert_eq!(shows.len(), 1);
        let show = &shows[0];
        assert_eq!(show.title, "Breaking Bad");
        assert_eq!(show.year, Some(2008));
        assert_eq!(show.seasons.len(), 2);
        assert_eq!(show.total_episodes(), 3);
    }

    #[test]
    fn build_tv_show_extracts_tmdb_id_from_source_root() {
        use crate::planner::Planner;
        use crate::scorer::score_identification;
        use camino::Utf8PathBuf;

        let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };

        let mut ep = make_episode("Breaking Bad", Some(2008), 1, 1);
        ep.source_path =
            Utf8PathBuf::from("/source/Breaking Bad (2008) [tmdbid-1396]/Season 01/S01E01.mkv");

        let items: Vec<(MediaItem, crate::models::Score)> =
            vec![(ep, score_identification(&make_episode("Breaking Bad", Some(2008), 1, 1)))];

        let shows = super::build_tv_shows(&items, &planner);

        assert_eq!(shows.len(), 1);
        let show = &shows[0];
        assert_eq!(show.tmdb_id.as_deref(), Some("1396"));
    }

    #[test]
    fn build_tv_show_extracts_imdb_id_from_source_root() {
        use crate::planner::Planner;
        use crate::scorer::score_identification;
        use camino::Utf8PathBuf;

        let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };

        let mut ep = make_episode("Breaking Bad", Some(2008), 1, 1);
        ep.source_path = Utf8PathBuf::from(
            "/source/Breaking Bad (2008) [imdbid-tt0903747]/Season 01/S01E01.mkv",
        );

        let items: Vec<(MediaItem, crate::models::Score)> =
            vec![(ep, score_identification(&make_episode("Breaking Bad", Some(2008), 1, 1)))];

        let shows = super::build_tv_shows(&items, &planner);

        assert_eq!(shows.len(), 1);
        let show = &shows[0];
        assert_eq!(show.imdb_id.as_deref(), Some("tt0903747"));
    }

    #[test]
    fn build_tv_show_skips_movies() {
        use crate::planner::Planner;
        use crate::scorer::score_identification;

        let planner = Planner { movies_root: "/movies".into(), tv_root: "/tv".into() };

        let mut movie_item = MediaItem::unknown("/source/Movie (2020)/Movie (2020).mkv");
        movie_item.kind = MediaKind::Movie;
        movie_item.title = Some("Movie".to_string());
        movie_item.year = Some(2020);

        let items: Vec<(MediaItem, crate::models::Score)> =
            vec![(movie_item.clone(), score_identification(&movie_item))];

        let shows = super::build_tv_shows(&items, &planner);
        assert!(shows.is_empty());
    }

    #[test]
    fn tv_show_total_episodes_and_season_numbers() {
        use crate::models::{TvSeason, TvShow};
        use camino::Utf8PathBuf;

        let show = TvShow {
            title: "Test".to_string(),
            year: None,
            tmdb_id: None,
            tvdb_id: None,
            imdb_id: None,
            source_root: Utf8PathBuf::from("/source/Test"),
            seasons: vec![
                TvSeason { season_number: 2, episodes: vec![], season_assets: vec![] },
                TvSeason { season_number: 1, episodes: vec![], season_assets: vec![] },
            ],
            show_assets: vec![],
            confidence: 50,
            reasons: vec![],
        };

        assert_eq!(show.season_numbers(), vec![1, 2]);
    }

    #[test]
    fn build_and_resolve_tv_shows_resolves_episode_destinations() {
        use crate::planner::Planner;
        use crate::scorer::score_identification;

        let planner = Planner { movies_root: "".into(), tv_root: "/tv".into() };

        let ep1 = make_episode("Test Show", Some(2020), 1, 1);
        let ep2 = make_episode("Test Show", Some(2020), 1, 2);

        let items: Vec<(MediaItem, crate::models::Score)> = vec![
            (ep1, score_identification(&make_episode("Test Show", Some(2020), 1, 1))),
            (ep2, score_identification(&make_episode("Test Show", Some(2020), 1, 2))),
        ];

        let shows = super::build_and_resolve_tv_shows(&items, &planner, "/tv");

        assert_eq!(shows.len(), 1);
        let show = &shows[0];
        assert_eq!(show.seasons.len(), 1);
        assert_eq!(show.seasons[0].episodes.len(), 2);
        for ep in &show.seasons[0].episodes {
            assert!(
                ep.destination.starts_with("/tv"),
                "Episode destination should start with /tv, got: {}",
                ep.destination
            );
        }
    }
}
