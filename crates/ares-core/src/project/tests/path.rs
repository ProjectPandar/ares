use crate::project::PackagePath;

#[test]
fn package_path_accepts_canonical_entry() {
    let path = PackagePath::entry(b"3D/Objects/model.model").unwrap();

    assert_eq!(path.as_str(), "3D/Objects/model.model");
}

#[test]
fn package_path_normalizes_percent_encoded_bytes() {
    let path = PackagePath::entry(b"3D/%4fbjects/model%2Emodel").unwrap();

    assert_eq!(path.as_str(), "3D/Objects/model.model");
}

#[test]
fn package_path_rejects_malformed_percent_encoding() {
    for raw in [b"3D/model%".as_slice(), b"3D/model%GG".as_slice()] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_invalid_utf8() {
    for raw in [b"3D/\xff.model".as_slice(), b"3D/%FF.model".as_slice()] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_drive_and_unc_syntax() {
    for raw in [
        b"C:/model.model".as_slice(),
        b"c:model.model".as_slice(),
        b"//server/share/model.model".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_backslash_ambiguity() {
    assert!(PackagePath::entry(b"3D\\model.model").is_err());
}

#[test]
fn package_path_rejects_nul() {
    for raw in [
        b"3D/model\0.model".as_slice(),
        b"3D/model%00.model".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_empty_segments() {
    for raw in [
        b"".as_slice(),
        b"/".as_slice(),
        b"3D//model.model".as_slice(),
        b"3D/model.model/".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_literal_dot_segments() {
    for raw in [
        b"./3D/model.model".as_slice(),
        b"3D/../model.model".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_percent_decoded_dot_segments() {
    for raw in [
        b"%2e/3D/model.model".as_slice(),
        b"3D/%2E./model.model".as_slice(),
        b"3D/.%2e/model.model".as_slice(),
        b"3D/%2E%2E/model.model".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_percent_encoded_separators() {
    for raw in [
        b"3D%2fmodel.model".as_slice(),
        b"3D%5Cmodel.model".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_fragments() {
    for raw in [
        b"3D/model.model#mesh".as_slice(),
        b"3D/model.model%23mesh".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_resolves_root_absolute_relationship_target() {
    let owner = PackagePath::entry(b"3D/3dmodel.model").unwrap();

    let target = owner.resolve("/Metadata/project_settings.config").unwrap();

    assert_eq!(target.as_str(), "Metadata/project_settings.config");
}

#[test]
fn package_path_resolves_owner_relative_relationship_target() {
    let owner = PackagePath::entry(b"3D/3dmodel.model").unwrap();

    let target = owner.resolve("Objects/object.model").unwrap();

    assert_eq!(target.as_str(), "3D/Objects/object.model");
}

#[test]
fn package_path_decodes_owner_relative_target_once() {
    let owner = PackagePath::entry(b"3D/3dmodel.model").unwrap();

    let percent = owner.resolve("Objects/100%25.model").unwrap();
    let encoded_dot = owner.resolve("Objects/%252e.model").unwrap();

    assert_eq!(percent.as_str(), "3D/Objects/100%.model");
    assert_eq!(encoded_dot.as_str(), "3D/Objects/%2e.model");
}

#[test]
fn package_path_canonicalizes_single_root_marker() {
    let path = PackagePath::entry(b"/3D/model.model").unwrap();

    assert_eq!(path.as_str(), "3D/model.model");
}

#[test]
fn package_path_rejects_uri_scheme_or_authority() {
    for raw in [
        b"file:3D/model.model".as_slice(),
        b"https://example.com/model.model".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_queries() {
    for raw in [
        b"3D/model.model?mesh".as_slice(),
        b"3D/model.model%3Fmesh".as_slice(),
    ] {
        assert!(PackagePath::entry(raw).is_err());
    }
}

#[test]
fn package_path_rejects_relationship_authority() {
    let owner = PackagePath::entry(b"3D/3dmodel.model").unwrap();

    assert!(owner.resolve("//server/model.model").is_err());
}
