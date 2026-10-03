use super::vendors;

#[test]
fn broken_machine_inventory_cannot_hide_behind_another_valid_vendor() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("Good/machine")).unwrap();
    std::fs::create_dir(root.path().join("Broken")).unwrap();
    std::fs::write(root.path().join("Broken/machine"), b"not a directory").unwrap();
    assert!(vendors(root.path()).is_err());
}

#[test]
fn unavailable_vendor_root_is_not_an_empty_successful_inventory() {
    let root = tempfile::tempdir().unwrap();
    assert!(vendors(&root.path().join("missing")).is_err());
}

#[test]
fn discovery_sorts_machine_vendors_and_omits_only_nonmachine_collections() {
    let root = tempfile::tempdir().unwrap();
    for directory in ["Zebra/machine", "Alpha/machine", "FilamentOnly/filament"] {
        std::fs::create_dir_all(root.path().join(directory)).unwrap();
    }
    std::fs::write(root.path().join("vendor-index.json"), b"{}").unwrap();
    assert_eq!(vendors(root.path()).unwrap(), ["Alpha", "Zebra"]);
}

#[cfg(unix)]
#[test]
fn dangling_machine_directory_is_not_a_nonmachine_collection() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("Broken")).unwrap();
    std::os::unix::fs::symlink(
        root.path().join("missing"),
        root.path().join("Broken/machine"),
    )
    .unwrap();
    assert!(vendors(root.path()).is_err());
}
