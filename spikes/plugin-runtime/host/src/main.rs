#[cfg(feature = "jit")]
mod component;
mod verify;

use anyhow::{Result, bail};
use plugin_spike_host::{Budget, Guest, Hosted, api::Plugin, direct};
use plugin_spike_wire::{GCODE, LAYERS, TRIALS, encode_edits, geometry, set_layer, with_layer};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{env, fs, time::Instant};

#[derive(Serialize)]
struct Sample {
    load_ms: f64,
    warmup_ms: f64,
    hook_total_ms: f64,
    wall_ms: f64,
    reload_ms: Option<f64>,
    layer_sha256: String,
    gcode_sha256: String,
    gcode: String,
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1_000.0
}

fn sample(
    load: impl Fn(&[u8], Option<&[u8]>) -> Result<Box<dyn Guest>>,
    wasm: &[u8],
    replacement: &[u8],
    source: Option<&[u8]>,
    workload: (u32, bool, bool),
) -> Result<Sample> {
    let (layers, subdivide, compiled) = workload;
    let mut input = geometry();
    let t = Instant::now();
    let mut guest = load(wasm, source)?;
    let load_ms = ms(t);
    set_layer(&mut input, 0);
    let t = Instant::now();
    let first = guest.perimeters(&input)?;
    let warmup_ms = ms(t);
    verify::first_layer(&first, subdivide, false);
    let mut sample = direct::sample(subdivide);
    if !compiled {
        let mut adapter = Hosted {
            guest: guest.as_mut(),
            declarations: sample.options(),
        };
        let edits = with_layer(&input, |layer| adapter.perimeters(layer))?;
        verify::first_layer(&encode_edits(&edits), subdivide, false);
    }
    let mut hook_total_ms = 0.0;
    let mut digest = Sha256::new();
    let wall = Instant::now();
    for layer in 0..layers {
        set_layer(&mut input, layer);
        if compiled {
            let (elapsed, output) = with_layer(&input, |layer| {
                let t = Instant::now();
                let edits = sample.perimeters(layer).unwrap();
                (ms(t), edits)
            });
            hook_total_ms += elapsed;
            digest.update(encode_edits(&output));
        } else {
            let t = Instant::now();
            let output = guest.perimeters(&input)?;
            hook_total_ms += ms(t);
            digest.update(output);
        }
    }
    let wall_ms = ms(wall);
    let gcode = guest.gcode(GCODE)?;
    assert!(gcode.starts_with(GCODE));
    let hex = |bytes: &[u8]| bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    let layer_sha256 = hex(&digest.finalize());
    let gcode_sha256 = hex(&Sha256::digest(&gcode));
    let gcode = String::from_utf8(gcode)?;
    set_layer(&mut input, 0);
    let reload_ms = if compiled {
        None
    } else {
        let t = Instant::now();
        if guest.has_source() {
            guest.set_source(replacement)?;
        } else {
            guest = load(replacement, None)?;
        }
        let updated = guest.perimeters(&input)?;
        let elapsed = ms(t);
        verify::first_layer(&updated, subdivide, true);
        Some(elapsed)
    };
    Ok(Sample {
        load_ms,
        warmup_ms,
        hook_total_ms,
        wall_ms,
        reload_ms,
        layer_sha256,
        gcode_sha256,
        gcode,
    })
}

fn run(load: impl Fn(&[u8], Option<&[u8]>) -> Result<Box<dyn Guest>>) -> Result<()> {
    let args = env::args().skip(2).collect::<Vec<_>>();
    let [wasm, replacement, source] = args.as_slice() else {
        bail!("usage: bench-plugin interpreter|jit guest.wasm replacement source.js|-");
    };
    let read = |path: &str| {
        if path == "-" {
            Ok(Vec::new())
        } else {
            fs::read(path)
        }
    };
    let wasm = read(wasm)?;
    let replacement = read(replacement)?;
    let source = (source != "-").then(|| fs::read(source)).transpose()?;
    let subdivide = subdivision();
    let compiled = env::args().nth(1).as_deref() == Some("rust");
    let layers = env::var("PLUGIN_SPIKE_LAYERS").map_or(Ok(LAYERS), |n| n.parse())?;
    let trials = env::var("PLUGIN_SPIKE_TRIALS").map_or(Ok(TRIALS), |n| n.parse())?;
    for _ in 0..trials {
        let result = sample(
            &load,
            &wasm,
            &replacement,
            source.as_deref(),
            (layers, subdivide, compiled),
        )?;
        println!("{}", serde_json::to_string(&result)?);
    }
    Ok(())
}

fn fixture(dir: &std::path::Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    let mut input = geometry();
    set_layer(&mut input, 0);
    fs::write(dir.join("layer.bin"), &input)?;
    for (kind, subdivide) in [("staggered", false), ("subdivision", true)] {
        for (case, reloaded) in [("original", false), ("reloaded", true)] {
            fs::write(
                dir.join(format!("{kind}-{case}.bin")),
                verify::expected(subdivide, reloaded),
            )?;
        }
    }
    fs::write(dir.join("gcode.bin"), GCODE)?;
    fs::write(
        dir.join("layout.mjs"),
        format!(
            "export {}\nexport const LAYERS={LAYERS}, TRIALS={TRIALS};\n",
            plugin_spike_wire::js_layout()
        ),
    )?;
    #[cfg(feature = "jit")]
    component::build(dir)?;
    Ok(())
}

fn subdivision() -> bool {
    env::var("PLUGIN_SPIKE_KIND").as_deref() == Ok("subdivision")
}

fn budget() -> Budget {
    if env::var_os("PLUGIN_SPIKE_METERED").is_some() {
        Budget {
            fuel: Some(1_000_000_000),
            memory: Some(64 << 20),
        }
    } else {
        Budget::default()
    }
}

#[cfg(feature = "files")]
fn files() -> Result<()> {
    let dir = env::args().nth(2).unwrap();
    let out = std::path::Path::new(&dir);
    let prior = b"slice-start bytes";
    for mode in ["custom", "wasi"] {
        let bytes = fs::read(out.join(format!("files-{mode}.wasm")))?;
        let mut samples = Vec::new();
        for _ in 0..TRIALS {
            let start = Instant::now();
            let mut probe =
                plugin_spike_host::files::FileProbe::new(&bytes, mode == "wasi", prior)?;
            let load_ms = ms(start);
            probe.run(1)?;
            let t = Instant::now();
            probe.run(10_000)?;
            let run_ms = ms(t);
            let _ = probe.finish(prior)?;
            samples.push(
                serde_json::json!({ "load_ms": load_ms, "run_ms": run_ms, "iterations": 10_000 }),
            );
        }
        fs::write(
            out.join(format!("native-files-{mode}.json")),
            serde_json::to_vec(&samples)?,
        )?;
    }
    Ok(())
}

fn main() -> Result<()> {
    match env::args().nth(1).as_deref() {
        Some("fixture") => fixture(std::path::Path::new(&env::args().nth(2).unwrap())),
        #[cfg(feature = "files")]
        Some("files") => files(),
        #[cfg(all(feature = "direct-boa", feature = "direct-quickjs"))]
        Some("direct-limits") => {
            println!(
                "{}",
                serde_json::json!({
                    "quickjs": direct::probe_quickjs(),
                    "boa": direct::probe_boa(),
                })
            );
            Ok(())
        }
        Some("rust") => run(|_, _| Ok(Box::new(direct::Rust(direct::sample(subdivision()))))),
        #[cfg(feature = "direct-quickjs")]
        Some("quickjs") => run(|_, src| Ok(Box::new(direct::QuickJs::new(src.unwrap())))),
        #[cfg(feature = "direct-boa")]
        Some("boa") => run(|_, src| Ok(Box::new(direct::Boa::new(src.unwrap())))),
        #[cfg(feature = "interpreter")]
        Some("interpreter") => {
            let host = plugin_spike_host::interpreter::Host::new(budget());
            run(|wasm, src| Ok(Box::new(host.load(wasm, src)?)))
        }
        #[cfg(feature = "jit")]
        Some("jit") => {
            let host = plugin_spike_host::jit::Host::new(budget());
            run(|wasm, src| Ok(Box::new(host.load(wasm, src)?)))
        }
        _ => bail!("choose a built backend (interpreter or jit)"),
    }
}
