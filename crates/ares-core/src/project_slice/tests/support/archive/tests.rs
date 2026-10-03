use super::*;

#[test]
fn default_fixture_zip_has_fixed_epoch_and_reproducible_bytes() {
    let archive = KsrArchive::new();
    let first = archive.clone().bytes();
    let second = archive.bytes();
    assert_eq!(first, second);
    let mut zip = ZipArchive::new(Cursor::new(first)).unwrap();
    for index in 0..zip.len() {
        assert_eq!(
            zip.by_index(index).unwrap().last_modified(),
            Some(DateTime::DEFAULT)
        );
    }
}
