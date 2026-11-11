use super::stages::{FailureKind, Stage, StageError};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

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

fn resource_hash(
    root: &Path,
    directory: &Path,
    hash: &mut Sha256,
    follow_links: bool,
) -> std::io::Result<()> {
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
        let kind = entry.file_type()?;
        let kind = if follow_links && kind.is_symlink() {
            fs::metadata(&path)?.file_type()
        } else {
            kind
        };
        if kind.is_dir() {
            hash.update(b"directory");
            resource_hash(root, &path, hash, follow_links)?;
        } else if kind.is_file() {
            hash.update(b"file");
            hash.update(file_hash(&path)?.as_bytes());
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "uncacheable file type",
            ));
        }
    }
    Ok(())
}

pub(super) fn cached_directory_digest(root: &Path) -> std::io::Result<(String, bool)> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, (String, String)>>> = OnceLock::new();
    let signature = directory_signature(root)?;
    let mut cache = CACHE.get_or_init(Mutex::default).lock().unwrap();
    if let Some((cached_signature, digest)) = cache.get(root)
        && cached_signature == &signature
    {
        return Ok((digest.clone(), true));
    }
    let digest = directory_digest(root)?;
    cache.insert(root.to_owned(), (signature, digest.clone()));
    Ok((digest, false))
}

fn directory_signature(root: &Path) -> std::io::Result<String> {
    let mut hash = Sha256::new();
    fn visit(root: &Path, directory: &Path, hash: &mut Sha256) -> std::io::Result<()> {
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .as_os_str()
                .as_encoded_bytes();
            let metadata = fs::symlink_metadata(&path)?;
            hash.update((relative.len() as u64).to_le_bytes());
            hash.update(relative);
            hash.update(metadata.file_type().is_dir().to_string().as_bytes());
            hash.update(metadata.len().to_le_bytes());
            hash.update(
                metadata
                    .modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
                    .to_le_bytes(),
            );
            if metadata.is_dir() {
                visit(root, &path, hash)?;
            }
        }
        Ok(())
    }
    visit(root, root, &mut hash)?;
    Ok(hex(&hash.finalize()))
}

pub(super) fn directory_digest(root: &Path) -> std::io::Result<String> {
    if !fs::symlink_metadata(root)?.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "uncacheable file type",
        ));
    }
    let mut hash = Sha256::new();
    resource_hash(root, root, &mut hash, false)?;
    Ok(hex(&hash.finalize()))
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
    let start = std::time::Instant::now();
    let native_hash = file_hash(&native).map_err(io)?;
    let native_us = start.elapsed().as_micros();
    let start = std::time::Instant::now();
    let launcher_hash = file_hash(bin).map_err(io)?;
    let endpoint = std::env::var_os("ORCA_REPLAY_BIN").map(std::path::PathBuf::from);
    let endpoint_hash = endpoint.as_deref().map(file_hash).transpose().map_err(io)?;
    let launcher_us = start.elapsed().as_micros();
    let start = std::time::Instant::now();
    let mut tree = Sha256::new();
    resource_hash(&resources, &resources, &mut tree, true).map_err(io)?;
    let resources_hash = hex(&tree.finalize());
    let resources_us = start.elapsed().as_micros();
    let policy = ["ORCA_REPLAY_SEED", "ARES_PARITY_SEEDED_REFERENCE", "LC_ALL"]
        .map(|key| (key, std::env::var(key).ok()))
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut components = json!({"schema":"native-cache-v2","native":native_hash,
        "launcher":launcher_hash,"endpoint":endpoint_hash,"resources":resources_hash,"policy":policy,"parallelism":std::thread::available_parallelism().map_err(io)?.get()});
    let start = std::time::Instant::now();
    if let Some(supply) = super::application::project_supply()
        .map_err(|error| StageError::new(Stage::Initialization, FailureKind::Io, error))?
    {
        components["project_preset_catalog"] = json!(supply.digest);
    }
    let catalog_us = start.elapsed().as_micros();
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
        "runtime_environment": runtime_environment,
        "identity_profile":{"native_us":native_us,"launcher_us":launcher_us,"resources_us":resources_us,"catalog_us":catalog_us}}),
    })
}
