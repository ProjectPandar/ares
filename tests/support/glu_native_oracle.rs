//! Test-only original glu-libtess supplier; no Native outputs are stored in Git.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fmt::Write as _,
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::native_environment;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize, Serialize)]
pub struct NativeCase {
    pub name: String,
    pub contours: Vec<Vec<[i64; 2]>>,
    pub z: f64,
    pub flip: bool,
}

pub struct NativeOracle {
    root: PathBuf,
    executable: PathBuf,
    sequence: usize,
}

impl NativeOracle {
    pub fn from_env() -> Result<Option<Self>> {
        let Some(_) = native_environment::binary("ARES_ORCA_NATIVE_BIN") else {
            return Ok(None);
        };
        let Some(resources) =
            native_environment::value("ARES_ORCA_RESOURCES_DIR").map(PathBuf::from)
        else {
            return Ok(None);
        };
        if !resources.is_dir() {
            return Err(
                format!("configured Native resources are not a directory: {resources:?}").into(),
            );
        }
        let Some(artifacts) =
            native_environment::value("ARES_PARITY_ARTIFACT_ROOT").map(PathBuf::from)
        else {
            return Ok(None);
        };
        let root = artifacts.join(format!(
            "glu-tessellation-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir_all(&root)?;
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let supplier = repo.join("OrcaSlicer/deps_src/glu-libtess");
        let mut objects = Vec::new();
        let mut hashes = String::new();
        for unit in [
            "dict",
            "geom",
            "memalloc",
            "mesh",
            "normal",
            "priorityq",
            "render",
            "sweep",
            "tess",
            "tessmono",
        ] {
            let source = supplier.join(format!("src/{unit}.c"));
            writeln!(
                hashes,
                "{} {}",
                sha256(&fs::read(&source)?),
                source.display()
            )?;
            let object = root.join(format!("{unit}.o"));
            let mut compiler = Command::new("gcc");
            compiler
                .args(["-std=c99", "-O2", "-DNDEBUG", "-I"])
                .arg(supplier.join("include"))
                .arg("-I")
                .arg(supplier.join("src"))
                .arg("-c")
                .arg(source)
                .arg("-o")
                .arg(&object);
            checked(&mut compiler, &root)?;
            objects.push(object);
        }
        // priorityq.c includes priorityq-heap.c; headers affect the compiled supplier too.
        for directory in ["src", "include"] {
            let mut files = fs::read_dir(supplier.join(directory))?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            files.sort();
            for source in files.into_iter().filter(|path| {
                path.extension().is_some_and(|extension| extension == "h")
                    || path
                        .file_name()
                        .is_some_and(|name| name == "priorityq-heap.c")
            }) {
                writeln!(
                    hashes,
                    "{} {}",
                    sha256(&fs::read(&source)?),
                    source.display()
                )?;
            }
        }
        let wrapper = repo.join("tests/parity/glu-tessellation/native_oracle.cpp");
        writeln!(
            hashes,
            "{} {}",
            sha256(&fs::read(&wrapper)?),
            wrapper.display()
        )?;
        fs::write(root.join("source-sha256.txt"), hashes)?;
        let executable = root.join("native-oracle");
        let mut compiler = Command::new("g++");
        compiler
            .args(["-std=c++17", "-O2", "-DNDEBUG", "-I"])
            .arg(supplier.join("include"))
            .arg(wrapper)
            .args(objects)
            .arg("-lm")
            .arg("-o")
            .arg(&executable);
        checked(&mut compiler, &root)?;
        Ok(Some(Self {
            root,
            executable,
            sequence: 0,
        }))
    }

    pub fn run(&mut self, case: &NativeCase) -> Result<Vec<[u64; 3]>> {
        self.sequence += 1;
        let prefix = format!("{}-{}", self.sequence, case.name);
        fs::write(
            self.root.join(format!("{prefix}.input.json")),
            serde_json::to_vec(case)?,
        )?;
        let mut request = format!(
            "{:.17} {} {}\n",
            case.z,
            u8::from(case.flip),
            case.contours.len()
        );
        for contour in &case.contours {
            writeln!(request, "{}", contour.len())?;
            for [x, y] in contour {
                writeln!(request, "{x} {y}")?;
            }
        }
        fs::write(self.root.join(format!("{prefix}.stdin")), &request)?;
        let mut child = Command::new(&self.executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child.stdin.take().unwrap().write_all(request.as_bytes())?;
        let output = child.wait_with_output()?;
        fs::write(
            self.root.join(format!("{prefix}.native.json")),
            &output.stdout,
        )?;
        fs::write(self.root.join(format!("{prefix}.stderr")), &output.stderr)?;
        fs::write(
            self.root.join(format!("{prefix}.status")),
            output.status.to_string(),
        )?;
        assert!(
            output.status.success(),
            "Native oracle failed: {:?}",
            self.root
        );
        let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        assert!(
            result["errors"].as_array().unwrap().is_empty(),
            "{}",
            case.name
        );
        Ok(serde_json::from_value(result["triangle_bits"].clone())?)
    }

    pub fn record_bits(&self, case: &NativeCase, producer: &str, bits: &[[u64; 3]]) -> Result<()> {
        fs::write(
            self.root.join(format!(
                "{}-{}.{producer}-bits.json",
                self.sequence, case.name
            )),
            serde_json::to_vec(bits)?,
        )?;
        Ok(())
    }
}

fn checked(command: &mut Command, root: &Path) -> Result<()> {
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("build.log"))?;
    writeln!(log, "{command:?}")?;
    let output = command.output()?;
    writeln!(log, "status: {}", output.status)?;
    log.write_all(&output.stdout)?;
    log.write_all(&output.stderr)?;
    if !output.status.success() {
        return Err(format!("Native compile failed; see {:?}", root.join("build.log")).into());
    }
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
