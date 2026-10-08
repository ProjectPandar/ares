use std::path::PathBuf;

use super::{
    repo_root,
    stages::{FailureKind, Stage, StageError},
};

#[track_caller]
pub(crate) fn reference_enabled(flag: &str) -> bool {
    if crate::native_environment::required() {
        available().unwrap_or_else(|error| panic!("{error}"));
    }
    let enabled = std::env::var(flag).as_deref() == Ok("1");
    if !enabled {
        crate::native_environment::skipped(&format!("{flag} is not enabled"));
    }
    enabled
}

#[track_caller]
pub(super) fn available() -> Result<Option<(PathBuf, PathBuf)>, StageError> {
    let required = crate::native_environment::required();
    let init_error = |detail| StageError::new(Stage::Initialization, FailureKind::Io, detail);
    let mut missing = None;
    for (key, directory) in [
        ("ARES_ORCA_NATIVE_BIN", false),
        ("ARES_ORCA_RESOURCES_DIR", true),
    ] {
        let Some(path) = std::env::var_os(key).map(PathBuf::from) else {
            if required {
                return Err(init_error(format!("ARES_REQUIRE_ORCA=1: missing {key}")));
            }
            missing.get_or_insert(key);
            continue;
        };
        if path.as_os_str().is_empty() {
            return Err(init_error(format!("configured {key} is empty")));
        }
        let canonical = path
            .canonicalize()
            .map_err(|error| init_error(format!("configured {key} {path:?}: {error}")))?;
        if if directory {
            !canonical.is_dir()
        } else {
            !canonical.is_file()
        } {
            return Err(init_error(format!(
                "configured {key} has invalid path type: {path:?}"
            )));
        }
    }
    let Some(bin) = std::env::var_os("ARES_ORCA_BIN")
        .map(PathBuf::from)
        .or_else(|| {
            let script = repo_root().join("scripts/orca-parity.sh");
            script.exists().then_some(script)
        })
    else {
        if required {
            return Err(init_error(
                "ARES_REQUIRE_ORCA=1: missing ARES_ORCA_BIN".to_owned(),
            ));
        }
        crate::native_environment::skipped("ARES_ORCA_BIN and repository wrapper are absent");
        return Ok(None);
    };
    let bin = bin.canonicalize().map_err(|error| {
        init_error(format!(
            "configured ARES_ORCA_BIN executable {bin:?}: {error}"
        ))
    })?;
    if !bin.is_file() {
        return Err(init_error(format!(
            "configured ARES_ORCA_BIN executable is not a file: {bin:?}"
        )));
    }
    let root = match crate::artifacts::root_from_env() {
        Ok(root) => root,
        Err(cause) => {
            let configured = std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some();
            let flagged = [
                "ARES_PARITY_SWEEP",
                "ARES_PARITY_CLUSTER",
                "ARES_PARITY_OPTIONS",
            ]
            .iter()
            .any(|flag| std::env::var(flag).as_deref() == Ok("1"));
            if required || configured || flagged {
                return Err(init_error(cause));
            }
            crate::native_environment::skipped(&cause);
            return Ok(None);
        }
    };
    if let Some(key) = missing {
        crate::native_environment::skipped(&format!("{key} is unset"));
        return Ok(None);
    }
    Ok(Some((bin, root)))
}
