use std::{ffi::OsString, path::PathBuf};

pub fn job_appdir(fallback: PathBuf) -> PathBuf {
    select_job_appdir(
        std::env::var_os("ORCA_JOB_APPDIR"),
        std::env::var_os("ORCA_250_JOB_APPDIR"),
        fallback,
    )
}

pub fn select_job_appdir(
    primary: Option<OsString>,
    versioned: Option<OsString>,
    fallback: PathBuf,
) -> PathBuf {
    primary
        .filter(|path| !path.is_empty())
        .or_else(|| versioned.filter(|path| !path.is_empty()))
        .map_or(fallback, PathBuf::from)
}
