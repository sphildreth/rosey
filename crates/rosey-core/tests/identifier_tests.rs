use camino::Utf8PathBuf;
use rosey_core::{identify_file, MediaKind};

#[test]
fn identify_file_uses_file_stem_for_movie_titles() {
    let item = identify_file(&Utf8PathBuf::from("/media/Some.Movie.2024.mkv"));

    assert_eq!(item.kind, MediaKind::Movie);
    assert_eq!(item.year, Some(2024));
    assert_eq!(item.title.as_deref(), Some("Some Movie"));
    assert!(!item.title.as_deref().unwrap().contains("mkv"));
}
