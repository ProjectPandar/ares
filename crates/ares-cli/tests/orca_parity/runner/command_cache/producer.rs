use super::super::{cache_identity, stages::StageError};
use std::path::Path;

pub(super) fn identify(bin: &Path) -> Result<cache_identity::Producer, StageError> {
    #[cfg(unix)]
    {
        unix::identify(bin)
    }
    #[cfg(not(unix))]
    {
        cache_identity::identify(bin)
    }
}

#[cfg(unix)]
mod unix {
    use super::super::super::stages::{FailureKind, Stage};
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};
    use std::{collections::BTreeMap, fs, io, os::unix::fs::MetadataExt, path::PathBuf};

    #[derive(Serialize, Deserialize)]
    struct Record {
        key: Value,
        context: Value,
        producer: cache_identity::Producer,
    }

    fn failure(error: io::Error) -> StageError {
        StageError::new(Stage::Initialization, FailureKind::Io, error)
    }

    pub(super) fn identify(bin: &Path) -> Result<cache_identity::Producer, StageError> {
        let Some(root) = std::env::var_os("ARES_ORCA_CACHE_DIR") else {
            return cache_identity::identify(bin);
        };
        let native = std::env::var_os("ARES_ORCA_NATIVE_BIN").ok_or_else(|| {
            failure(io::Error::new(
                io::ErrorKind::NotFound,
                "undeclared Native executable",
            ))
        })?;
        let native = fs::canonicalize(native).map_err(failure)?;
        let metadata = fs::metadata(&native).map_err(failure)?;
        let key = json!({"path":native,"dev":metadata.dev(),"inode":metadata.ino(),
            "size":metadata.len(),"mtime_ns":i128::from(metadata.mtime())*1_000_000_000+i128::from(metadata.mtime_nsec())});
        let environment = [
            "ARES_ORCA_RESOURCES_DIR",
            "ORCA_REPLAY_BIN",
            "ORCA_REPLAY_SEED",
            "ARES_PARITY_SEEDED_REFERENCE",
            "LC_ALL",
            "ARES_PARITY_PROJECT_PRESET_CATALOG",
        ]
        .map(|key| (key, std::env::var_os(key)))
        .into_iter()
        .collect::<BTreeMap<_, _>>();
        let context = json!({"launcher":fs::canonicalize(bin).map_err(failure)?,
            "environment":environment,"parallelism":std::thread::available_parallelism().map_err(failure)?.get()});
        let parent = PathBuf::from(root)
            .join("native-cache-v2")
            .join("producer-identity");
        fs::create_dir_all(&parent).map_err(failure)?;
        let entry = parent.join(cache_identity::digest(&[native
            .as_os_str()
            .as_encoded_bytes()]));
        let lock = fs::File::create(entry.with_extension("lock")).map_err(failure)?;
        lock.lock().map_err(failure)?;
        if let Some(mut producer) = load(&entry, &key, &context) {
            producer.receipt["identity_cache_hit"] = json!(true);
            return Ok(producer);
        }
        let producer = cache_identity::identify(bin)?;
        let record = Record {
            key,
            context,
            producer,
        };
        let bytes = serde_json::to_vec(&record).unwrap();
        let staging = tempfile::Builder::new()
            .prefix("producer-")
            .tempdir_in(&parent)
            .map_err(failure)?;
        fs::write(staging.path().join("record.json"), &bytes).map_err(failure)?;
        fs::write(
            staging.path().join("receipt.json"),
            cache_identity::raw_digest(&bytes),
        )
        .map_err(failure)?;
        if entry.exists() {
            fs::remove_dir_all(&entry).map_err(failure)?;
        }
        fs::rename(staging.path(), &entry).map_err(failure)?;
        let mut producer = record.producer;
        producer.receipt["identity_cache_hit"] = json!(false);
        Ok(producer)
    }

    fn load(entry: &Path, key: &Value, context: &Value) -> Option<cache_identity::Producer> {
        for name in ["record.json", "receipt.json"] {
            if !fs::symlink_metadata(entry.join(name)).ok()?.is_file() {
                return None;
            }
        }
        let bytes = fs::read(entry.join("record.json")).ok()?;
        if fs::read_to_string(entry.join("receipt.json")).ok()?
            != cache_identity::raw_digest(&bytes)
        {
            return None;
        }
        let record: Record = serde_json::from_slice(&bytes).ok()?;
        (record.key == *key && record.context == *context).then_some(record.producer)
    }
}
