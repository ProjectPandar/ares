use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    allowlist::{self, OpenField},
    discovery::{PRODUCTION_ROOTS, production_files, production_sources},
    finding::render_findings,
    ratchet,
    visitor::{fingerprints, scan_source},
};

#[test]
fn scope_fingerprint_uses_named_owner_and_local_occurrence_only() {
    let source = "fn parse(value: serde_json::Value) { let _: serde_json::Value = value; }";
    assert_eq!(
        fingerprints("crates/ares-core/src/sample.rs", source).unwrap(),
        BTreeSet::from([
            "crates/ares-core/src/sample.rs#parse@1|type|serde_json::Value".to_owned(),
            "crates/ares-core/src/sample.rs#parse@2|type|serde_json::Value".to_owned(),
        ])
    );
    let reformatted = "fn parse(\nvalue: serde_json::Value\n) { let clean = 1; let _: serde_json::Value=value; drop(clean); }";
    assert_eq!(
        fingerprints("crates/ares-core/src/sample.rs", reformatted).unwrap(),
        fingerprints("crates/ares-core/src/sample.rs", source).unwrap()
    );
    let nested = fingerprints(
        "crates/ares-core/src/sample.rs",
        "mod inner { fn parse(_: serde_json::Value) {} }",
    )
    .unwrap();
    assert_has_fragment(&nested, "#inner::parse@1|type|");
}
#[test]
fn scope_classifier_resolves_imports_reexports_globs_and_alias_fixed_point() {
    let source = r#"
        use serde_json::{Value as Json, Map, value::RawValue};
        use serde_json::*;
        pub use serde_json::value::Value as Exported;
        type First = Json;
        type Second = First;
        type Mapped = Map<String, Second>;
        type Raw = RawValue;
        mod exports { pub use serde_json::Value as Data; }
        use exports::Data as Payload;
        type Third = Payload;
        fn convert(value: Second) {
            let _ = serde_json::from_value::<u8>(value);
            let _ = serde_json::json!({"answer": 42});
            let _ = Json::Null;
        }
    "#;
    let rendered = fingerprints("crates/ares-core/src/imports.rs", source).unwrap();
    for expected in [
        "|use|serde_json::Value as Json",
        "|use|serde_json::Map",
        "|use|serde_json::value::RawValue",
        "|use|serde_json::*",
        "|reexport|serde_json::Value as Exported",
        "#First@1|alias|serde_json::Value",
        "#Second@1|alias|serde_json::Value",
        "#Mapped@1|alias|serde_json::Map<String,serde_json::Value>",
        "#Raw@1|alias|serde_json::value::RawValue",
        "#Third@1|alias|serde_json::Value",
        "#exports@1|reexport|serde_json::Value as Data",
        "|call|serde_json::from_value",
        "|macro|serde_json::json!",
        "|path|serde_json::Value::Null",
    ] {
        assert_has_suffix(&rendered, expected);
    }
}
#[test]
fn scope_classifier_covers_exact_erasure_dom_and_runtime_but_allows_safe_serde() {
    let source = r#"
        enum ConfigValue { Text(String) }
        struct DynamicValue;
        type Erased = Box<dyn std::any::Any>;
        type Serializable = Box<dyn erased_serde::Serialize>;
        type Xml = roxmltree::Document<'static>;
        type Runtime = std::any::TypeId;
        fn inspect(value: &dyn std::any::Any) {
            let _ = value.downcast_ref::<u8>();
            let _ = value.is::<u8>();
            let _ = <dyn core::any::Any>::type_id(value);
        }
        fn safe() {
            let _: serde_json::Number;
            let _ = serde_json::from_slice::<u8>(b"1");
            let _ = serde_json::from_str::<u8>("1");
        }
    "#;
    let rendered = fingerprints("crates/ares-core/src/categories.rs", source).unwrap();
    for expected in [
        "|definition|ConfigValue",
        "|definition|DynamicValue",
        "|alias|Box<dynstd::any::Any>",
        "|alias|Box<dynerased_serde::Serialize>",
        "|alias|roxmltree::Document<'static>",
        "|alias|std::any::TypeId",
        "|runtime_type|downcast_ref",
        "|runtime_type|is",
        "|runtime_type|type_id",
    ] {
        assert_has_suffix(&rendered, expected);
    }
    for unexpected in ["serde_json::Number", "from_slice", "from_str"] {
        assert_lacks_fragment(&rendered, unexpected);
    }
}
#[test]
fn scope_excludes_only_exact_test_items_and_preserves_production_cfg_attr() {
    let source = r#"
        #[cfg(test)] fn cfg_test(_: serde_json::Value) {}
        #[test] fn test_attr(_: serde_json::Value) {}
        #[tokio::test] async fn tokio_test(_: serde_json::Value) {}
        #[cfg(not(test))] fn cfg_not(_: serde_json::Value) {}
        #[cfg_attr(not(test), inline)] fn cfg_attr_not(_: serde_json::Value) {}
        #[cfg(all(test, feature = "x"))] fn compound_cfg(_: serde_json::Value) {}
    "#;
    let rendered = fingerprints("crates/ares-core/src/cfg.rs", source).unwrap();
    for owner in ["cfg_not", "cfg_attr_not", "compound_cfg"] {
        assert_has_fragment(&rendered, &format!("#{owner}@"));
    }
    for owner in ["cfg_test", "test_attr", "tokio_test"] {
        assert_lacks_fragment(&rendered, &format!("#{owner}@"));
    }
}
#[test]
fn scope_discovery_uses_fixed_roots_and_reachable_module_include_graph() {
    let mut sources = roots(
        r#"
        mod ordinary;
        mod inline { mod nested; }
        option_modules!(alpha, pub(crate) beta);
        include!("sub/../included.rs");
        #[path = "redirected.rs"] mod renamed;
        #[cfg(test)] mod hidden;
    "#,
    );
    for path in [
        "crates/ares-core/src/ordinary.rs",
        "crates/ares-core/src/inline/nested.rs",
        "crates/ares-core/src/alpha.rs",
        "crates/ares-core/src/beta.rs",
        "crates/ares-core/src/included.rs",
        "crates/ares-core/src/redirected.rs",
    ] {
        sources.insert(path.to_owned(), String::new());
    }
    assert_eq!(
        production_sources(&sources).unwrap(),
        vec![
            "crates/ares-cli/src/main.rs",
            "crates/ares-core/src/alpha.rs",
            "crates/ares-core/src/beta.rs",
            "crates/ares-core/src/included.rs",
            "crates/ares-core/src/inline/nested.rs",
            "crates/ares-core/src/lib.rs",
            "crates/ares-core/src/ordinary.rs",
            "crates/ares-core/src/redirected.rs",
            "crates/ares-wasm/src/lib.rs",
        ]
    );
}
#[test]
fn scope_discovery_builds_candidate_index_with_walkdir() {
    let repo = TempRepo::new();
    repo.write(PRODUCTION_ROOTS[0], "mod child;");
    repo.write(PRODUCTION_ROOTS[1], "");
    repo.write(PRODUCTION_ROOTS[2], "");
    repo.write("crates/ares-core/src/child.rs", "");
    repo.write(
        "crates/ares-core/src/unreachable.rs",
        "compile_error!(\"not reachable\");",
    );
    let files = production_files(&repo.path)
        .unwrap()
        .into_iter()
        .map(|path| {
            path.strip_prefix(&repo.path)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        files,
        BTreeSet::from([
            PRODUCTION_ROOTS[0].to_owned(),
            PRODUCTION_ROOTS[1].to_owned(),
            PRODUCTION_ROOTS[2].to_owned(),
            "crates/ares-core/src/child.rs".to_owned(),
        ])
    );
}
#[test]
fn scope_discovery_fails_closed_for_graph_uncertainty() {
    let missing = roots("mod absent;");
    assert_err_contains(production_sources(&missing), "missing");
    let mut ambiguous = roots("mod duplicate;");
    ambiguous.insert(
        "crates/ares-core/src/duplicate.rs".to_owned(),
        String::new(),
    );
    ambiguous.insert(
        "crates/ares-core/src/duplicate/mod.rs".to_owned(),
        String::new(),
    );
    assert_err_contains(production_sources(&ambiguous), "ambiguous");
    let nonliteral = roots("include!(concat!(\"generated\", \".rs\"));");
    assert_err_contains(production_sources(&nonliteral), "nonliteral");
    let mut no_wasm = roots("");
    no_wasm.remove(PRODUCTION_ROOTS[2]);
    assert_err_contains(production_sources(&no_wasm), PRODUCTION_ROOTS[2]);
}
#[test]
fn scope_allowlist_is_strict_exact_and_reachable() {
    let missing = r#"[[open_field]]
path="crates/ares-core/src/open.rs"
containing_struct="Open"
field="payload"
upstream_source="Config.cpp:1""#;
    assert!(allowlist::parse(missing).is_err());
    assert!(allowlist::parse("").is_err());
    let unknown = format!("{}\nunknown=true", entry_text());
    assert!(allowlist::parse(&unknown).is_err());
    let parsed = allowlist::parse(&entry_text()).unwrap();
    let path = "crates/ares-core/src/open.rs";
    let source = "struct Open { payload: serde_json::Value }";
    let findings = scan_source(path, source).unwrap();
    let sources = BTreeMap::from([(path.to_owned(), source.to_owned())]);
    let filtered = allowlist::apply(findings, &parsed, &sources).unwrap();
    assert!(render_findings(&filtered).is_empty());
    assert_err_contains(allowlist::apply(Vec::new(), &parsed, &sources), "matched 0");
    assert_err_contains(
        allowlist::apply(
            scan_source(path, source).unwrap(),
            &parsed,
            &BTreeMap::new(),
        ),
        "reachable",
    );
}
#[test]
fn scope_allowlist_rejects_bounded_direct_alias_and_one_hop_dispatch() {
    let source = r#"
        struct Open { payload: serde_json::Value }
        fn choose(value: &serde_json::Value) -> bool { value.is_object() }
        fn run<'a>(open: &'a Open) {
            if open.payload.is_null() {}
            let payload = &open.payload;
            if let Some(_) = payload.as_str() {}
            let choice = choose(payload);
            match choice { true => {}, false => {} }
        }
        impl Open { fn method(&self) { if self.payload.is_array() {} } }
    "#;
    assert_eq!(
        allowlist::dispatch_violations(source, &entry()).unwrap(),
        ["if", "if-let", "match"]
    );
    let bounded = r#"
        struct Open { payload: serde_json::Value }
        struct Other { payload: serde_json::Value }
        fn choose(value: &serde_json::Value) -> bool { value.is_object() }
        fn wrap(value: bool) -> bool { value }
        fn run(open: &Open, other: &Other) {
            if other.payload.is_null() {}
            if wrap(choose(&open.payload)) {}
            let _ = serde_json::to_string(&open.payload);
        }
    "#;
    assert!(
        allowlist::dispatch_violations(bounded, &entry())
            .unwrap()
            .is_empty()
    );
}
#[test]
fn scope_ratchet_is_exact_monotone_on_disk_and_every_parent_edge() {
    let a = fingerprint("a.rs#f@1|type|serde_json::Value");
    let b = fingerprint("b.rs#g@1|type|serde_json::Map");
    let parent = BTreeSet::from([a.clone(), b.clone()]);
    let child = BTreeSet::from([a.clone()]);
    assert!(
        ratchet::edge_errors("child", Some(&child), Some("parent-a"), Some(&parent)).is_empty()
    );
    assert!(
        ratchet::edge_errors("child", Some(&parent), Some("parent-a"), Some(&child))[0]
            .contains("grows")
    );
    assert!(
        ratchet::edge_errors("child", None, Some("parent-b"), Some(&parent))[0].contains("removes")
    );
    assert!(ratchet::disk_errors(&child, &parent).is_empty());
    assert_eq!(ratchet::disk_errors(&parent, &child).len(), 1);
    assert_eq!(ratchet::current_errors(&parent, &child).len(), 1);
    assert!(ratchet::parse_baseline(&format!("{a}\n{a}\n")).is_err());
    assert!(ratchet::parse_baseline("not-a-fingerprint").is_err());
    assert!(ratchet::bootstrap_errors("bootstrap", &child, &child).is_empty());
    assert_eq!(
        ratchet::bootstrap_errors("bootstrap", &parent, &child).len(),
        1
    );
    assert!(
        ratchet::repository_state_error(false, "false")
            .unwrap()
            .contains("missing .git")
    );
    assert!(
        ratchet::repository_state_error(true, "true")
            .unwrap()
            .contains("shallow")
    );
    assert!(ratchet::repository_state_error(true, "false").is_none());
}
fn roots(core: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        (PRODUCTION_ROOTS[0].to_owned(), core.to_owned()),
        (PRODUCTION_ROOTS[1].to_owned(), String::new()),
        (PRODUCTION_ROOTS[2].to_owned(), String::new()),
    ])
}
fn entry_text() -> String {
    r#"[[open_field]]
path="crates/ares-core/src/open.rs"
containing_struct="Open"
field="payload"
upstream_source="Config.cpp:1"
rationale="Upstream preserves opaque metadata.""#
        .to_owned()
}
fn entry() -> OpenField {
    allowlist::parse(&entry_text())
        .unwrap()
        .open_field
        .remove(0)
}
fn fingerprint(text: &str) -> String {
    text.to_owned()
}
fn assert_has_fragment(findings: &BTreeSet<String>, expected: &str) {
    assert!(
        findings.iter().any(|finding| finding.contains(expected)),
        "missing {expected}: {findings:#?}"
    );
}
fn assert_has_suffix(findings: &BTreeSet<String>, expected: &str) {
    assert!(
        findings.iter().any(|finding| finding.ends_with(expected)),
        "missing {expected}: {findings:#?}"
    );
}
fn assert_lacks_fragment(findings: &BTreeSet<String>, unexpected: &str) {
    assert!(
        findings.iter().all(|finding| !finding.contains(unexpected)),
        "unexpected {unexpected}: {findings:#?}"
    );
}
fn assert_err_contains<T>(result: Result<T, String>, expected: &str) {
    let Err(error) = result else {
        panic!("expected error containing {expected}");
    };
    assert!(error.contains(expected), "unexpected error: {error}");
}

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);
struct TempRepo {
    path: PathBuf,
}
impl TempRepo {
    fn new() -> Self {
        let unique = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ares-task1c-{}-{unique}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
    fn write(&self, relative: &str, source: &str) {
        let path = self.path.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
}
impl Drop for TempRepo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap();
    }
}
