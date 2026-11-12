use super::{
    super::{cache_identity, stages::Captured},
    request::Request,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, io, path::Path};

#[derive(Serialize, Deserialize)]
pub(super) struct Packet {
    schema: String,
    producer: String,
    request: Value,
    pub(super) code: Option<i32>,
    pub(super) status: String,
    pub(super) rejection: Option<String>,
    stdout: String,
    stderr: String,
    outputs: Vec<Option<String>>,
}

pub(super) fn load(entry: &Path, request: &Request, producer: &str) -> Option<(Packet, Captured)> {
    for name in ["receipt.json", "result.json", "stdout.bin", "stderr.bin"] {
        if !fs::symlink_metadata(entry.join(name)).ok()?.is_file() {
            return None;
        }
    }
    let receipt: Value =
        serde_json::from_slice(&fs::read(entry.join("receipt.json")).ok()?).ok()?;
    let bytes = fs::read(entry.join("result.json")).ok()?;
    if receipt["result_sha256"] != cache_identity::raw_digest(&bytes) {
        return None;
    }
    let packet: Packet = serde_json::from_slice(&bytes).ok()?;
    if packet.schema != "native-command-v1"
        || packet.producer != producer
        || packet.request != request.components
        || packet.outputs.len() != request.outputs.len()
    {
        return None;
    }
    let stdout = fs::read(entry.join("stdout.bin")).ok()?;
    let stderr = fs::read(entry.join("stderr.bin")).ok()?;
    if cache_identity::raw_digest(&stdout) != packet.stdout
        || cache_identity::raw_digest(&stderr) != packet.stderr
    {
        return None;
    }
    for (index, output) in request.outputs.iter().enumerate() {
        if fingerprint(
            &entry.join("outputs").join(index.to_string()),
            output.directory,
        )
        .ok()?
            != packet.outputs[index]
        {
            return None;
        }
    }
    let captured = Captured {
        code: packet.code,
        status: packet.status.clone(),
        stdout,
        stderr,
    };
    Some((packet, captured))
}

fn copy_cwd_outputs(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if super::request::cwd_output(&entry.file_name()) {
            copy(&entry.path(), &destination.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub(super) fn store(
    entry: &Path,
    request: &Request,
    producer: &str,
    captured: &Captured,
    rejection: Option<String>,
) -> io::Result<()> {
    let mut outputs = Vec::new();
    fs::create_dir(entry.join("outputs"))?;
    for (index, output) in request.outputs.iter().enumerate() {
        let destination = entry.join("outputs").join(index.to_string());
        if output.cwd {
            copy_cwd_outputs(&output.path, &destination)?;
        } else if output.path.try_exists()? {
            copy(&output.path, &destination)?;
        }
        outputs.push(fingerprint(&destination, output.directory)?);
    }
    let packet = Packet {
        schema: "native-command-v1".into(),
        producer: producer.into(),
        request: request.components.clone(),
        code: captured.code,
        status: captured.status.clone(),
        rejection,
        stdout: cache_identity::raw_digest(&captured.stdout),
        stderr: cache_identity::raw_digest(&captured.stderr),
        outputs,
    };
    fs::write(entry.join("stdout.bin"), &captured.stdout)?;
    fs::write(entry.join("stderr.bin"), &captured.stderr)?;
    let bytes = serde_json::to_vec_pretty(&packet).unwrap();
    fs::write(entry.join("result.json"), &bytes)?;
    fs::write(entry.join("receipt.json"), serde_json::to_vec_pretty(&json!({"schema":"native-command-v1","result_sha256":cache_identity::raw_digest(&bytes)})).unwrap())
}

pub(super) fn materialize(
    entry: &Path,
    request: &Request,
    packet: &Packet,
    work: &Path,
) -> io::Result<bool> {
    let staging = tempfile::Builder::new()
        .prefix("cache-output-")
        .tempdir_in(work)?;
    for (index, output) in request.outputs.iter().enumerate() {
        let source = entry.join("outputs").join(index.to_string());
        let expected = &packet.outputs[index];
        if expected.is_none() {
            if !matches!(fingerprint(&source, output.directory), Ok(None)) {
                return Ok(false);
            }
            continue;
        }
        let prepared = staging.path().join(index.to_string());
        if copy(&source, &prepared).is_err()
            || fingerprint(&prepared, output.directory).ok().as_ref() != Some(expected)
        {
            return Ok(false);
        }
    }
    for (index, output) in request.outputs.iter().enumerate() {
        if packet.outputs[index].is_some() {
            copy(&staging.path().join(index.to_string()), &output.path)?;
        }
    }
    Ok(true)
}

fn fingerprint(path: &Path, directory: bool) -> io::Result<Option<String>> {
    if !path.try_exists()? {
        return Ok(None);
    }
    if directory {
        cache_identity::directory_digest(path).map(|hash| Some(format!("directory:{hash}")))
    } else if fs::symlink_metadata(path)?.is_file() {
        cache_identity::file_hash(path).map(|hash| Some(format!("file:{hash}")))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "uncacheable output file type",
        ))
    }
}

fn copy(source: &Path, destination: &Path) -> io::Result<()> {
    let kind = fs::symlink_metadata(source)?.file_type();
    if kind.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy(&entry.path(), &destination.join(entry.file_name()))?;
        }
        Ok(())
    } else if kind.is_file() {
        fs::copy(source, destination).map(|_| ())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "uncacheable output file type",
        ))
    }
}
