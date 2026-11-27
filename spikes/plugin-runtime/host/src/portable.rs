use anyhow::{Context, Result, ensure};
use plugin_spike_host::{jit::Host, metering::instrument};
use sha2::{Digest, Sha256};
use std::{env, fs};

pub fn probe() -> Result<()> {
    let bytes = fs::read(env::args().nth(2).context("expected counter fixture")?)?;
    let limit = Host::portable(2, 1_000).portable_probe(&bytes)?;
    ensure!(limit == [1, 71, 1, -1], "unexpected limits: {limit:?}");
    let module = instrument(&bytes, 2)?;
    println!(
        "{}",
        serde_json::json!({"limit": limit, "sha256": digest(&module.bytes)})
    );
    Ok(())
}

pub fn sizes() -> Result<()> {
    for path in env::args().skip(2) {
        let wasm = fs::read(&path)?;
        let module = instrument(&wasm, 1024)?;
        println!(
            "{}",
            serde_json::json!({
                "path": path, "segments": module.segments,
                "code_bytes": [module.code_size.0, module.code_size.1],
                "module_bytes": [wasm.len(), module.bytes.len()], "sha256": digest(&module.bytes),
            })
        );
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    super::hex(&Sha256::digest(bytes))
}
