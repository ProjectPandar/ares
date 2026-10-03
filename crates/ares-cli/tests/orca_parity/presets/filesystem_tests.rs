use super::VendorProfiles;

#[test]
fn partial_vendor_inventory_does_not_ignore_broken_nested_entries() {
    let root = tempfile::tempdir().unwrap();
    for directory in [
        "OrcaFilamentLibrary/filament",
        "Brand/machine",
        "Brand/process",
    ] {
        std::fs::create_dir_all(root.path().join(directory)).unwrap();
    }
    std::fs::write(
        root.path().join("Brand/machine/good.json"),
        br#"{"name":"Good","instantiation":"true"}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(
        root.path().join("unavailable"),
        root.path().join("Brand/machine/broken"),
    )
    .unwrap();
    assert!(VendorProfiles::load(root.path(), "Brand").is_err());
}
