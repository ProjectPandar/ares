use super::stages::{FailureKind, Stage, StageError};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

pub(super) struct Producer {
    pub(super) digest: String,
    pub(super) receipt: Value,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn raw_digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

pub(super) fn digest(parts: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    hex(&hash.finalize())
}

pub(super) fn file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        hash.update(&buffer[..length]);
    }
    Ok(hex(&hash.finalize()))
}

fn resource_hash(root: &Path, directory: &Path, hash: &mut Sha256) -> std::io::Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = path
            .strip_prefix(root)
            .unwrap()
            .as_os_str()
            .as_encoded_bytes();
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name);
        if fs::metadata(&path)?.is_dir() {
            hash.update(b"directory");
            resource_hash(root, &path, hash)?;
        } else {
            hash.update(b"file");
            hash.update(file_hash(&path)?.as_bytes());
        }
    }
    Ok(())
}

pub(super) fn directory_digest(root: &Path) -> std::io::Result<String> {
    regular_tree(root)?;
    let mut hash = Sha256::new();
    resource_hash(root, root, &mut hash)?;
    Ok(hex(&hash.finalize()))
}

fn regular_tree(path: &Path) -> std::io::Result<()> {
    let kind = fs::symlink_metadata(path)?.file_type();
    if kind.is_dir() {
        for entry in fs::read_dir(path)? {
            regular_tree(&entry?.path())?;
        }
        Ok(())
    } else if kind.is_file() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "uncacheable file type",
        ))
    }
}

pub(super) fn identify(bin: &Path) -> Result<Producer, StageError> {
    let io = |error: std::io::Error| StageError::new(Stage::Initialization, FailureKind::Io, error);
    let declared = |key| {
        std::env::var_os(key)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| {
                StageError::new(
                    Stage::Initialization,
                    FailureKind::Io,
                    format!("{key} must declare the cached Native producer"),
                )
            })
    };
    let native = declared("ARES_ORCA_NATIVE_BIN")?
        .canonicalize()
        .map_err(io)?;
    let resources = declared("ARES_ORCA_RESOURCES_DIR")?
        .canonicalize()
        .map_err(io)?;
    let native_hash = file_hash(&native).map_err(io)?;
    let launcher_hash = file_hash(bin).map_err(io)?;
    let endpoint = std::env::var_os("ORCA_REPLAY_BIN").map(std::path::PathBuf::from);
    let endpoint_hash = endpoint.as_deref().map(file_hash).transpose().map_err(io)?;
    let mut tree = Sha256::new();
    resource_hash(&resources, &resources, &mut tree).map_err(io)?;
    let resources_hash = hex(&tree.finalize());
    let policy = ["ORCA_REPLAY_SEED", "ARES_PARITY_SEEDED_REFERENCE", "LC_ALL"]
        .map(|key| (key, std::env::var(key).ok()))
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut components = json!({"schema":"native-cache-v2","native":native_hash,
        "launcher":launcher_hash,"endpoint":endpoint_hash,"resources":resources_hash,"policy":policy,"parallelism":std::thread::available_parallelism().map_err(io)?.get()});
    if let Some(supply) = super::application::project_supply()
        .map_err(|error| StageError::new(Stage::Initialization, FailureKind::Io, error))?
    {
        components["project_preset_catalog"] = json!(supply.digest);
    }
    let key = digest(&[b"producer-v2", &serde_json::to_vec(&components).unwrap()]);
    let runtime_environment = [
        "ORCA_REPLAY_CPU",
        "ORCA_APPDIR",
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "LD_LIBRARY_PATH",
        "ORCA_LIB_CACHE",
    ]
    .map(|key| (key, std::env::var(key).ok()))
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    Ok(Producer {
        digest: key,
        receipt: json!({"components":components,
        "native_path":native,"resources_path":resources,"launcher_path":bin,"endpoint_path":endpoint,
        "runtime_environment": runtime_environment}),
    })
}
