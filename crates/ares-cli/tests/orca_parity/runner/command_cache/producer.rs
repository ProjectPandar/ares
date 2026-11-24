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

pub(super) fn immutable_datadir_digest(directory: &Path) -> Option<std::io::Result<String>> {
    #[cfg(unix)]
    {
        unix::immutable_datadir_digest(directory)
    }
    #[cfg(not(unix))]
    {
        let _ = directory;
        None
    }
}

#[cfg(unix)]
mod unix {
    use super::super::super::{
        application,
        stages::{FailureKind, Stage},
    };
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        collections::BTreeMap, fs, io, os::unix::fs::MetadataExt, path::PathBuf, sync::OnceLock,
    };

    #[derive(Serialize, Deserialize)]
    struct Record {
        key: Value,
        context: Value,
        producer: cache_identity::Producer,
    }

    #[derive(Serialize, Deserialize)]
    struct DatadirRecord {
        key: Value,
        digest: String,
    }

    fn failure(error: io::Error) -> StageError {
        StageError::new(Stage::Initialization, FailureKind::Io, error)
    }

    fn stat_entry(path: &Path) -> io::Result<Value> {
        let metadata = fs::metadata(path)?;
        Ok(json!({
            "path": path,
            "dev": metadata.dev(),
            "inode": metadata.ino(),
            "size": metadata.len(),
            "mtime_ns": i128::from(metadata.mtime()) * 1_000_000_000
                + i128::from(metadata.mtime_nsec()),
            "ctime_ns": i128::from(metadata.ctime()) * 1_000_000_000
                + i128::from(metadata.ctime_nsec()),
        }))
    }

    fn stat_tree(root: &Path, directory: &Path, hash: &mut Sha256) -> io::Result<()> {
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_cached_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let name = path
                .strip_prefix(root)
                .unwrap()
                .as_os_str()
                .as_encoded_bytes();
            hash.update((name.len() as u64).to_le_bytes());
            hash.update(name);
            let metadata = fs::metadata(&path)?;
            let kind = metadata.file_type();
            if kind.is_dir() {
                hash.update(b"directory");
                hash.update(metadata.dev().to_le_bytes());
                hash.update(metadata.ino().to_le_bytes());
                hash.update(metadata.mtime().to_le_bytes());
                hash.update(metadata.mtime_nsec().to_le_bytes());
                hash.update(metadata.ctime().to_le_bytes());
                hash.update(metadata.ctime_nsec().to_le_bytes());
                stat_tree(root, &path, hash)?;
            } else if kind.is_file() {
                hash.update(b"file");
                hash.update(metadata.dev().to_le_bytes());
                hash.update(metadata.ino().to_le_bytes());
                hash.update(metadata.len().to_le_bytes());
                hash.update(metadata.mtime().to_le_bytes());
                hash.update(metadata.mtime_nsec().to_le_bytes());
                hash.update(metadata.ctime().to_le_bytes());
                hash.update(metadata.ctime_nsec().to_le_bytes());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "uncacheable file type",
                ));
            }
        }
        Ok(())
    }

    fn resources_stat(resources: &Path) -> io::Result<Value> {
        let metadata = fs::metadata(resources)?;
        let root = stat_entry(resources)?;
        if metadata.permissions().readonly() {
            return Ok(json!({"mode": "readonly-root", "root": root}));
        }
        let mut hash = Sha256::new();
        stat_tree(resources, resources, &mut hash)?;
        let tree = cache_identity::raw_digest(&hash.finalize());
        Ok(json!({"mode": "stat-tree", "root": root, "tree": tree}))
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
        let resources_dir = std::env::var_os("ARES_ORCA_RESOURCES_DIR").ok_or_else(|| {
            failure(io::Error::new(
                io::ErrorKind::NotFound,
                "undeclared Native resources",
            ))
        })?;
        let native = fs::canonicalize(native).map_err(failure)?;
        let resources = fs::canonicalize(resources_dir).map_err(failure)?;
        let launcher = fs::canonicalize(bin).map_err(failure)?;
        let endpoint = std::env::var_os("ORCA_REPLAY_BIN")
            .map(fs::canonicalize)
            .transpose()
            .map_err(failure)?;
        let key = json!({
            "native": stat_entry(&native).map_err(failure)?,
            "launcher": stat_entry(&launcher).map_err(failure)?,
            "endpoint": endpoint.as_deref().map(stat_entry).transpose().map_err(failure)?,
            "resources": resources_stat(&resources).map_err(failure)?,
        });
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
        let catalog_digest = application::project_supply()
            .map_err(|error| StageError::new(Stage::Initialization, FailureKind::Io, error))?
            .map(|supply| supply.digest.as_str());
        let context = json!({
            "launcher": launcher,
            "environment": environment,
            "catalog_digest": catalog_digest,
            "parallelism": std::thread::available_parallelism().map_err(failure)?.get(),
        });
        let parent = PathBuf::from(root)
            .join("native-cache-v2")
            .join("producer-identity");
        fs::create_dir_all(&parent).map_err(failure)?;
        let entry = parent.join(cache_identity::digest(&[
            native.as_os_str().as_encoded_bytes(),
            &serde_json::to_vec(&context).unwrap(),
        ]));
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
        write_record(&parent, &entry, "producer-", &bytes).map_err(failure)?;
        let mut producer = record.producer;
        producer.receipt["identity_cache_hit"] = json!(false);
        Ok(producer)
    }

    pub(super) fn immutable_datadir_digest(directory: &Path) -> Option<io::Result<String>> {
        let supply = application::project_supply().ok()??;
        let canonical = fs::canonicalize(directory).ok()?;
        let supply_dir = fs::canonicalize(&supply.native_dir).ok()?;
        if canonical != supply_dir {
            return None;
        }
        if !fs::metadata(&canonical).ok()?.permissions().readonly() {
            return None;
        }
        static MEMO: OnceLock<(PathBuf, String, String)> = OnceLock::new();
        if let Some((path, supply_digest, digest)) = MEMO.get()
            && *path == canonical
            && *supply_digest == supply.digest
        {
            return Some(Ok(digest.clone()));
        }
        let digest = match cached_datadir_digest(&canonical, &supply.digest) {
            Ok(digest) => digest,
            Err(error) => return Some(Err(error)),
        };
        let _ = MEMO.set((canonical, supply.digest.clone(), digest.clone()));
        Some(Ok(digest))
    }

    fn cached_datadir_digest(canonical: &Path, supply_digest: &str) -> io::Result<String> {
        let Some(root) = std::env::var_os("ARES_ORCA_CACHE_DIR") else {
            return cache_identity::directory_digest(canonical);
        };
        let key = json!({
            "dir": stat_entry(canonical)?,
            "catalog_digest": supply_digest,
        });
        let parent = PathBuf::from(root)
            .join("native-cache-v2")
            .join("datadir-digest");
        fs::create_dir_all(&parent)?;
        let entry = parent.join(cache_identity::digest(&[
            canonical.as_os_str().as_encoded_bytes(),
            supply_digest.as_bytes(),
        ]));
        let lock = fs::File::create(entry.with_extension("lock"))?;
        lock.lock()?;
        if let Some(digest) = load_datadir(&entry, &key) {
            return Ok(digest);
        }
        let digest = cache_identity::directory_digest(canonical)?;
        let bytes = serde_json::to_vec(&DatadirRecord {
            key,
            digest: digest.clone(),
        })
        .unwrap();
        write_record(&parent, &entry, "datadir-", &bytes)?;
        Ok(digest)
    }

    fn write_record(parent: &Path, entry: &Path, prefix: &str, bytes: &[u8]) -> io::Result<()> {
        let staging = tempfile::Builder::new().prefix(prefix).tempdir_in(parent)?;
        fs::write(staging.path().join("record.json"), bytes)?;
        fs::write(
            staging.path().join("receipt.json"),
            cache_identity::raw_digest(bytes),
        )?;
        if entry.exists() {
            fs::remove_dir_all(entry)?;
        }
        fs::rename(staging.path(), entry)
    }

    fn read_verified_record(entry: &Path) -> Option<Vec<u8>> {
        for name in ["record.json", "receipt.json"] {
            if !fs::symlink_metadata(entry.join(name)).ok()?.is_file() {
                return None;
            }
        }
        let bytes = fs::read(entry.join("record.json")).ok()?;
        (fs::read_to_string(entry.join("receipt.json")).ok()? == cache_identity::raw_digest(&bytes))
            .then_some(bytes)
    }

    fn load(entry: &Path, key: &Value, context: &Value) -> Option<cache_identity::Producer> {
        let bytes = read_verified_record(entry)?;
        let record: Record = serde_json::from_slice(&bytes).ok()?;
        (record.key == *key && record.context == *context).then_some(record.producer)
    }

    fn load_datadir(entry: &Path, key: &Value) -> Option<String> {
        let bytes = read_verified_record(entry)?;
        let record: DatadirRecord = serde_json::from_slice(&bytes).ok()?;
        (record.key == *key).then_some(record.digest)
    }
}
