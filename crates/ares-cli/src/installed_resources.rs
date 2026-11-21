//! Installation-relative resources (OrcaSlicer.cpp:8169–8197).

use std::{
    env, io,
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
const FHS_RESOURCES: Option<&str> = {
    let root = option_env!("ARES_FHS_RESOURCES");
    if let Some(root) = root {
        assert!(
            matches!(root.as_bytes(), [b'/', ..]),
            "ARES_FHS_RESOURCES must be an absolute installation path"
        );
    }
    root
};

pub(super) fn resolve(explicit: Option<&Path>) -> io::Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    #[cfg(target_os = "linux")]
    if let Some(path) = FHS_RESOURCES {
        return Ok(PathBuf::from(path));
    }
    executable_resources(&env::current_exe()?)
}

#[cfg(windows)]
fn executable_resources(executable: &Path) -> io::Result<PathBuf> {
    Ok(parent(executable)?.join("resources"))
}

#[cfg(not(windows))]
fn executable_resources(executable: &Path) -> io::Result<PathBuf> {
    let executable = executable.canonicalize()?;
    let prefix = parent(parent(&executable)?)?;
    #[cfg(target_os = "macos")]
    {
        Ok(prefix.join("Resources"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let primary = prefix.join("resources");
        if let Some(outer) = prefix.parent()
            && !primary.try_exists()?
        {
            return Ok(outer.join("resources"));
        }
        Ok(primary)
    }
}

fn parent(path: &Path) -> io::Result<&Path> {
    path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "executable has no installation directory",
        )
    })
}
