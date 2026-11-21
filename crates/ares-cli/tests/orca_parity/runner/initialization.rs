use super::{FailureKind, OrcaRunner, PathBuf, Stage, StageError, environment};

impl OrcaRunner {
    /// Path to the executable invoked for reference production — used by
    /// the sweep preflight to verify the wrapper actually runs.
    pub(crate) fn bin(&self) -> &PathBuf {
        &self.bin
    }

    pub(crate) fn work_dir(&self) -> &std::path::Path {
        &self.work
    }

    /// Only absent optional configuration returns None; initialization errors must fail callers.
    #[track_caller]
    pub(crate) fn from_env() -> Result<Option<Self>, StageError> {
        let Some((bin, root)) = environment::available()? else {
            return Ok(None);
        };
        let init_error = |detail| StageError::new(Stage::Initialization, FailureKind::Io, detail);
        let work = tempfile::Builder::new()
            .prefix("orca-runner-")
            .tempdir_in(&root)
            .map_err(|error| init_error(format!("runner work directory under {root:?}: {error}")))?
            .keep();
        // Shared roots store v2 content-addressed entries across runs.
        let cache = match std::env::var_os("ARES_ORCA_CACHE_DIR") {
            Some(cache) => {
                let cache = PathBuf::from(cache);
                std::fs::create_dir_all(&cache).map_err(|error| {
                    init_error(format!("Orca cache directory {cache:?}: {error}"))
                })?;
                cache
            }
            None => work.clone(),
        };
        Ok(Some(Self {
            bin,
            work,
            cache,
            producer: Default::default(),
        }))
    }
}
