use crate::{
    LibraryMovie, LibraryMovieMatchKind, MissingPersonMoviesReport, OwnedPersonMovie,
    PersonIdentity, PersonMovieCredit, PersonReference,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

static TMDB_PERSON_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:https?://)?(?:www\.)?themoviedb\.org/person/(?P<id>\d+)")
        .expect("valid TMDB person URL regex")
});

static IMDB_NAME_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:https?://)?(?:www\.)?imdb\.com/name/(?P<id>nm\d{7,})")
        .expect("valid IMDb name URL regex")
});

static IMDB_NAME_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^nm\d{7,}$").expect("valid IMDb name ID regex"));

static TMDB_PERSON_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d+$").expect("valid TMDB person ID regex"));

pub fn parse_person_reference(input: &str) -> Result<PersonReference, String> {
    let value = input.trim().trim_end_matches('/');
    if value.is_empty() {
        return Err("person reference is empty".to_string());
    }

    if let Some(caps) = TMDB_PERSON_URL.captures(value) {
        return Ok(PersonReference::Tmdb { id: caps["id"].to_string() });
    }

    if let Some(caps) = IMDB_NAME_URL.captures(value) {
        return Ok(PersonReference::Imdb { id: caps["id"].to_ascii_lowercase() });
    }

    if let Some(rest) = value.strip_prefix("tmdb:") {
        let id = rest.trim();
        if TMDB_PERSON_ID.is_match(id) {
            return Ok(PersonReference::Tmdb { id: id.to_string() });
        }
    }

    if IMDB_NAME_ID.is_match(value) {
        return Ok(PersonReference::Imdb { id: value.to_ascii_lowercase() });
    }

    if TMDB_PERSON_ID.is_match(value) {
        return Ok(PersonReference::Tmdb { id: value.to_string() });
    }

    Err(format!(
        "unsupported person reference: {input}. Use a TMDB person URL/ID or IMDb name URL/ID"
    ))
}

pub fn compare_person_movies(
    person: PersonIdentity,
    credits: Vec<PersonMovieCredit>,
    library_movies: Vec<LibraryMovie>,
) -> MissingPersonMoviesReport {
    let library_movies_scanned = library_movies.len();
    let credits = dedupe_credits(credits);
    let credits_scanned = credits.len();

    let by_tmdb = index_by_id(
        library_movies
            .iter()
            .filter_map(|movie| movie.tmdb_id.as_ref().map(|id| (id.as_str(), movie))),
    );
    let by_imdb = index_by_id(
        library_movies
            .iter()
            .filter_map(|movie| movie.imdb_id.as_ref().map(|id| (id.as_str(), movie))),
    );
    let by_title_year = index_by_title_year(&library_movies);

    let mut missing = Vec::new();
    let mut owned = Vec::new();

    for credit in credits {
        if let Some(library_movie) = by_tmdb.get(credit.tmdb_id.as_str()) {
            owned.push(OwnedPersonMovie {
                credit,
                library_movie: (*library_movie).clone(),
                match_kind: LibraryMovieMatchKind::TmdbId,
            });
            continue;
        }

        if let Some(imdb_id) = credit.imdb_id.as_deref() {
            if let Some(library_movie) = by_imdb.get(imdb_id) {
                owned.push(OwnedPersonMovie {
                    credit,
                    library_movie: (*library_movie).clone(),
                    match_kind: LibraryMovieMatchKind::ImdbId,
                });
                continue;
            }
        }

        if let Some(key) = title_year_key(&credit.title, credit.year) {
            if let Some(library_movie) = by_title_year.get(&key) {
                owned.push(OwnedPersonMovie {
                    credit,
                    library_movie: (*library_movie).clone(),
                    match_kind: LibraryMovieMatchKind::TitleYear,
                });
                continue;
            }
        }

        missing.push(credit);
    }

    sort_credits(&mut missing);
    owned.sort_by_key(|owned| credit_sort_key(&owned.credit));

    MissingPersonMoviesReport { person, missing, owned, library_movies_scanned, credits_scanned }
}

fn dedupe_credits(credits: Vec<PersonMovieCredit>) -> Vec<PersonMovieCredit> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for credit in credits {
        if seen.insert(credit.tmdb_id.clone()) {
            deduped.push(credit);
        }
    }

    sort_credits(&mut deduped);
    deduped
}

fn sort_credits(credits: &mut [PersonMovieCredit]) {
    credits.sort_by_key(credit_sort_key);
}

fn credit_sort_key(credit: &PersonMovieCredit) -> (String, u16, String) {
    (
        credit.release_date.clone().unwrap_or_else(|| "9999-99-99".to_string()),
        credit.year.unwrap_or(u16::MAX),
        credit.title.to_ascii_lowercase(),
    )
}

fn index_by_id<'a>(
    values: impl Iterator<Item = (&'a str, &'a LibraryMovie)>,
) -> HashMap<String, &'a LibraryMovie> {
    values
        .filter(|(id, _)| !id.trim().is_empty())
        .map(|(id, movie)| (id.trim().to_ascii_lowercase(), movie))
        .collect()
}

fn index_by_title_year(movies: &[LibraryMovie]) -> HashMap<String, &LibraryMovie> {
    movies
        .iter()
        .filter_map(|movie| {
            let title = movie.title.as_deref()?;
            let key = title_year_key(title, movie.year)?;
            Some((key, movie))
        })
        .collect()
}

fn title_year_key(title: &str, year: Option<u16>) -> Option<String> {
    let normalized = title
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();

    if normalized.is_empty() {
        return None;
    }

    Some(format!("{}:{}", normalized, year?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    #[test]
    fn parses_tmdb_person_references() {
        assert_eq!(
            parse_person_reference("https://www.themoviedb.org/person/368-marlon-brando").unwrap(),
            PersonReference::Tmdb { id: "368".to_string() }
        );
        assert_eq!(
            parse_person_reference("tmdb:368").unwrap(),
            PersonReference::Tmdb { id: "368".to_string() }
        );
        assert_eq!(
            parse_person_reference("368").unwrap(),
            PersonReference::Tmdb { id: "368".to_string() }
        );
    }

    #[test]
    fn parses_imdb_name_references() {
        assert_eq!(
            parse_person_reference("https://www.imdb.com/name/nm0000702/").unwrap(),
            PersonReference::Imdb { id: "nm0000702".to_string() }
        );
        assert_eq!(
            parse_person_reference("NM0000702").unwrap(),
            PersonReference::Imdb { id: "nm0000702".to_string() }
        );
    }

    #[test]
    fn compares_by_tmdb_id_then_title_year() {
        let person = PersonIdentity {
            tmdb_id: "368".to_string(),
            name: Some("Marlon Brando".to_string()),
            imdb_id: Some("nm0000008".to_string()),
        };
        let credits = vec![
            PersonMovieCredit {
                tmdb_id: "238".to_string(),
                title: "The Godfather".to_string(),
                year: Some(1972),
                release_date: Some("1972-03-14".to_string()),
                character: Some("Don Vito Corleone".to_string()),
                order: Some(0),
                imdb_id: None,
            },
            PersonMovieCredit {
                tmdb_id: "3083".to_string(),
                title: "A Streetcar Named Desire".to_string(),
                year: Some(1951),
                release_date: Some("1951-09-19".to_string()),
                character: None,
                order: Some(0),
                imdb_id: None,
            },
            PersonMovieCredit {
                tmdb_id: "999".to_string(),
                title: "Missing Movie".to_string(),
                year: Some(1960),
                release_date: Some("1960-01-01".to_string()),
                character: None,
                order: Some(0),
                imdb_id: None,
            },
        ];
        let library = vec![
            LibraryMovie {
                source_path: Utf8PathBuf::from("/movies/The Godfather.mkv"),
                tmdb_id: Some("238".to_string()),
                imdb_id: None,
                title: Some("Wrong Title".to_string()),
                year: Some(1999),
            },
            LibraryMovie {
                source_path: Utf8PathBuf::from("/movies/Streetcar.mkv"),
                tmdb_id: None,
                imdb_id: None,
                title: Some("A Streetcar Named Desire".to_string()),
                year: Some(1951),
            },
        ];

        let report = compare_person_movies(person, credits, library);

        assert_eq!(report.owned.len(), 2);
        assert_eq!(report.missing.len(), 1);
        assert_eq!(report.owned[0].match_kind, LibraryMovieMatchKind::TitleYear);
        assert_eq!(report.owned[1].match_kind, LibraryMovieMatchKind::TmdbId);
        assert_eq!(report.missing[0].title, "Missing Movie");
    }
}
