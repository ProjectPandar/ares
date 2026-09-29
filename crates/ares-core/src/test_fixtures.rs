use std::sync::LazyLock;

/// Reads a repository-relative fixture at test run time.
pub(crate) fn read(path: &str) -> Vec<u8> {
    let full = format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&full).unwrap_or_else(|error| panic!("{full}: {error}"))
}

pub(crate) fn read_str(path: &str) -> String {
    String::from_utf8(read(path)).unwrap()
}

static KSR_PROJECT: LazyLock<Vec<u8>> =
    LazyLock::new(|| read("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"));
static OPTIONS_INVENTORY: LazyLock<String> =
    LazyLock::new(|| read_str("tests/ksr_fdmtest_v4/options-v242.json"));

pub(crate) fn ksr_project() -> &'static [u8] {
    &KSR_PROJECT
}

pub(crate) fn options_inventory() -> &'static str {
    &OPTIONS_INVENTORY
}
