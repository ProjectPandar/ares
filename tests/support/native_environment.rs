//! Test-only oracle availability. An omitted comparison is never Native parity evidence.
use std::{ffi::OsString, path::PathBuf};

pub fn required() -> bool {
    std::env::var("ARES_REQUIRE_ORCA").as_deref() == Ok("1")
}

#[track_caller]
pub fn skipped(reason: &str) {
    eprintln!(
        "SKIPPED: {reason}; Native comparison not executed at {}",
        std::panic::Location::caller()
    );
}

#[track_caller]
pub fn value(key: &str) -> Option<OsString> {
    match std::env::var_os(key) {
        Some(value) => {
            assert!(!value.is_empty(), "configured {key} is empty");
            Some(value)
        }
        None if required() => {
            panic!("ARES_REQUIRE_ORCA=1: missing {key}; Native comparison not executed")
        }
        None => {
            skipped(&format!("{key} is unset"));
            None
        }
    }
}

#[track_caller]
pub fn binary(key: &str) -> Option<PathBuf> {
    value(key).map(|value| {
        let path = PathBuf::from(value);
        assert!(
            path.is_file(),
            "configured {key} executable {path:?} is not a file"
        );
        path
    })
}
