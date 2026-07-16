#[path = "no_unapproved_dynamic_values/allowlist.rs"]
mod allowlist;
#[path = "no_unapproved_dynamic_values/classification.rs"]
mod classification;
#[path = "no_unapproved_dynamic_values/discovery.rs"]
mod discovery;
#[path = "no_unapproved_dynamic_values/finding.rs"]
mod finding;
#[path = "no_unapproved_dynamic_values/imports.rs"]
mod imports;
#[path = "no_unapproved_dynamic_values/profile_shell.rs"]
mod profile_shell;
#[path = "no_unapproved_dynamic_values/ratchet.rs"]
mod ratchet;
#[path = "no_unapproved_dynamic_values/visitor.rs"]
mod visitor;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use finding::{normalize_path, render_findings};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn scan_repository(
    repo: &Path,
) -> Result<(Vec<finding::Finding>, BTreeMap<String, String>), String> {
    let files = discovery::production_files(repo)?;
    let mut sources = BTreeMap::new();
    for absolute in files {
        let path = normalize_path(&absolute.strip_prefix(repo).unwrap().to_string_lossy());
        let source = fs::read_to_string(&absolute)
            .map_err(|error| format!("could not read {}: {error}", absolute.display()))?;
        sources.insert(path, source);
    }
    let findings = visitor::scan_sources(
        sources
            .iter()
            .map(|(path, source)| (path.as_str(), source.as_str())),
    )?;
    Ok((findings, sources))
}

#[test]
fn no_unapproved_dynamic_values() {
    let repo = repo_root();
    let (findings, sources) = scan_repository(&repo).unwrap_or_else(|error| panic!("{error}"));
    let allowlist_text =
        fs::read_to_string(repo.join("scripts/dynamic_value_allowlist.toml")).unwrap();
    let allowlist = allowlist::parse(&allowlist_text).unwrap_or_else(|error| panic!("{error}"));
    let approved =
        allowlist::apply(findings, &allowlist, &sources).unwrap_or_else(|error| panic!("{error}"));
    let current = render_findings(&approved);
    let baseline = fs::read_to_string(repo.join("scripts/dynamic_value_baseline.txt")).unwrap();
    let errors = ratchet::validate(&repo, &current, &baseline);
    assert!(
        errors.is_empty(),
        "dynamic-value audit failed:\n{}",
        errors.join("\n")
    );
}

#[test]
fn profile_modules_use_only_typed_shells() {
    profile_shell::assert_profile_modules_use_only_typed_shells(&repo_root());
}

#[test]
#[ignore = "print-only migration aid; never writes files"]
fn print_current_dynamic_value_baseline() {
    let repo = repo_root();
    let (findings, sources) = scan_repository(&repo).unwrap();
    let allowlist = allowlist::parse(
        &fs::read_to_string(repo.join("scripts/dynamic_value_allowlist.toml")).unwrap(),
    )
    .unwrap();
    for fingerprint in render_findings(&allowlist::apply(findings, &allowlist, &sources).unwrap()) {
        println!("{fingerprint}");
    }
}

#[test]
fn project_resolver_follows_reexport_into_consumer() {
    let sources = BTreeMap::from([
        (
            discovery::PRODUCTION_ROOTS[0].to_owned(),
            "mod exports; mod consumer;".to_owned(),
        ),
        (discovery::PRODUCTION_ROOTS[1].to_owned(), String::new()),
        (discovery::PRODUCTION_ROOTS[2].to_owned(), String::new()),
        (
            "crates/ares-core/src/exports.rs".to_owned(),
            "pub use serde_json::Value as Data;".to_owned(),
        ),
        (
            "crates/ares-core/src/consumer.rs".to_owned(),
            "use crate::exports::Data; fn consume(_: Data) {}".to_owned(),
        ),
    ]);
    let reachable = discovery::production_sources(&sources).unwrap();
    let findings = visitor::scan_sources(
        reachable
            .iter()
            .map(|path| (path.as_str(), sources.get(path).unwrap().as_str())),
    )
    .unwrap();
    assert!(
        render_findings(&findings)
            .contains("crates/ares-core/src/consumer.rs#consume@1|type|serde_json::Value")
    );
}

#[test]
fn grouped_self_rename_resolves_alias() {
    let source = "use serde_json::{self as sj}; type Payload = sj::Value;";
    assert!(
        visitor::fingerprints("crates/ares-core/src/grouped.rs", source)
            .unwrap()
            .contains("crates/ares-core/src/grouped.rs#Payload@1|alias|serde_json::Value")
    );
}

#[test]
fn allowlist_rejects_cross_file_qualified_helper_dispatch() {
    let path = "crates/ares-core/src/open.rs";
    let source = r#"
        struct Open { payload: serde_json::Value }
        fn run(open: &Open) {
            if crate::helpers::choose(&open.payload) {}
        }
    "#;
    let sources = BTreeMap::from([
        (path.to_owned(), source.to_owned()),
        (
            "crates/ares-core/src/helpers.rs".to_owned(),
            "pub fn choose(_: &serde_json::Value) -> bool { true }".to_owned(),
        ),
    ]);
    let allowlist = open_allowlist(path, "Open");

    let error = allowlist::apply(
        visitor::scan_source(path, source).unwrap(),
        &allowlist,
        &sources,
    )
    .unwrap_err();
    assert!(error.contains("controls slicing dispatch: if"));
}

#[test]
fn allowlist_rejects_inline_qualified_owner_dispatch() {
    let path = "crates/ares-core/src/open.rs";
    let source = r#"
        mod model {
            struct Open { payload: serde_json::Value }
            fn run(open: &Open) { if open.payload.is_null() {} }
        }
    "#;
    let findings = visitor::scan_source(path, source).unwrap();
    assert!(
        render_findings(&findings)
            .contains("crates/ares-core/src/open.rs#model::Open.payload@1|type|serde_json::Value")
    );
    let sources = BTreeMap::from([(path.to_owned(), source.to_owned())]);
    let error =
        allowlist::apply(findings, &open_allowlist(path, "model::Open"), &sources).unwrap_err();
    assert!(error.contains("controls slicing dispatch: if"));
}

#[test]
fn allowlist_rejects_self_qualified_signature_dispatch() {
    let path = "crates/ares-core/src/open.rs";
    let source = r#"
        mod model {
            struct Open { payload: serde_json::Value }
            fn run(open: &self::Open) { if open.payload.is_null() {} }
        }
    "#;
    let findings = visitor::scan_source(path, source).unwrap();
    let sources = BTreeMap::from([(path.to_owned(), source.to_owned())]);
    let error =
        allowlist::apply(findings, &open_allowlist(path, "model::Open"), &sources).unwrap_err();
    assert!(error.contains("controls slicing dispatch: if"));
}

#[test]
fn allowlist_rejects_self_qualified_impl_dispatch() {
    let path = "crates/ares-core/src/open.rs";
    let source = r#"
        mod model {
            struct Open { payload: serde_json::Value }
            impl self::Open { fn run(&self) { if self.payload.is_null() {} } }
        }
    "#;
    let findings = visitor::scan_source(path, source).unwrap();
    let sources = BTreeMap::from([(path.to_owned(), source.to_owned())]);
    let error =
        allowlist::apply(findings, &open_allowlist(path, "model::Open"), &sources).unwrap_err();
    assert!(error.contains("controls slicing dispatch: if"));
}

#[test]
fn allowlist_rejects_qualified_associated_helper_dispatch() {
    let path = "crates/ares-core/src/open.rs";
    let source = r#"
        struct Open { payload: serde_json::Value }
        impl Open {
            fn choose(_: &serde_json::Value) -> bool { true }
            fn run(&self) { if Self::choose(&self.payload) {} }
        }
    "#;
    let findings = visitor::scan_source(path, source).unwrap();
    let sources = BTreeMap::from([(path.to_owned(), source.to_owned())]);
    let error = allowlist::apply(findings, &open_allowlist(path, "Open"), &sources).unwrap_err();
    assert!(error.contains("controls slicing dispatch: if"));
}

fn open_allowlist(path: &str, containing_struct: &str) -> allowlist::Allowlist {
    allowlist::Allowlist {
        open_field: vec![allowlist::OpenField {
            path: path.to_owned(),
            containing_struct: containing_struct.to_owned(),
            field: "payload".to_owned(),
            upstream_source: "Config.cpp:1".to_owned(),
            rationale: "Opaque upstream metadata.".to_owned(),
        }],
    }
}

#[cfg(test)]
#[path = "no_unapproved_dynamic_values/tests.rs"]
mod tests;
