use super::ProfileDocument;

/// Preset.cpp:1892–1898; Semver.hpp:64–71; deps_src/semver/semver.c:142–219.
pub(super) fn valid_version(document: &ProfileDocument) -> bool {
    let Some(version) = document
        .fields
        .get("version")
        .and_then(|value| value.token())
    else {
        return false;
    };
    let version = version.split('\0').next().unwrap();
    if version.len() > 255
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".-+".contains(&byte))
    {
        return false;
    }
    let base = version
        .split('+')
        .next()
        .unwrap()
        .split('-')
        .next()
        .unwrap();
    let mut count = 0;
    for component in base.split('.') {
        count += 1;
        if component.len() > 50 || !component.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
    }
    (2..=4).contains(&count)
}
