use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use ares_core::{
    ConfigOverrides, OptionValueKind, Placement, ProfileSliceRequest, SliceError, profile_settings,
};
use serde_json::{Value, json};

pub struct Case {
    root: PathBuf,
    input: Vec<u8>,
    catalog: String,
    pub native: Value,
}

impl Case {
    pub fn new(label: &str, settings: Value) -> Option<Self> {
        let binary = super::native_environment::binary("ARES_ORCA_PRESET_SELECT_BIN")?;
        let root = PathBuf::from(super::native_environment::value(
            "ARES_PARITY_ARTIFACT_ROOT",
        )?);
        assert!(root.is_absolute());
        fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap();
        assert!(!root.starts_with(repository));
        let root = tempfile::Builder::new()
            .prefix(label)
            .tempdir_in(root)
            .unwrap()
            .keep();
        eprintln!("artifacts: {}", root.display());
        fs::create_dir_all(root.join("data/system")).unwrap();
        let catalog = json!({"vendors": [], "imports": []}).to_string();
        fs::write(root.join("catalog.json"), &catalog).unwrap();
        fs::write(root.join("project-settings.json"), settings.to_string()).unwrap();
        let input = super::input::project(&settings);
        fs::write(root.join("selection.3mf"), &input).unwrap();
        let options = settings
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(wire(key, value))))
            .collect::<serde_json::Map<_, _>>();
        let request = json!({
            "data_directory": root.join("data"),
            "projects": [{"filename": "selection.3mf", "version": "2.5.0", "options": options}],
            "output": root.join("native.json")
        });
        fs::write(root.join("native-request.json"), request.to_string()).unwrap();
        fs::write(
            root.join("native-command.json"),
            json!({"program": binary, "cwd": root, "request": root.join("native-request.json")})
                .to_string(),
        )
        .unwrap();
        let output = Command::new(&binary)
            .current_dir(&root)
            .env(
                "ARES_PRESET_SELECT_REQUEST",
                root.join("native-request.json"),
            )
            .output()
            .unwrap();
        fs::write(root.join("native.stdout"), &output.stdout).unwrap();
        fs::write(root.join("native.stderr"), &output.stderr).unwrap();
        fs::write(
            root.join("native-exit.json"),
            json!({"code": output.status.code(), "success": output.status.success()}).to_string(),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let native: Value =
            serde_json::from_slice(&fs::read(root.join("native.json")).unwrap()).unwrap();
        assert_eq!(
            native["upstream"],
            "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19"
        );
        Some(Self {
            root,
            input,
            catalog,
            native,
        })
    }

    pub fn write(&self, filename: &str, value: &Value) {
        fs::write(
            self.root.join(filename),
            serde_json::to_vec_pretty(value).unwrap(),
        )
        .unwrap();
    }

    pub async fn api(&self) -> Result<Value, SliceError> {
        let selection = json!({"from_project": true, "load_and_select": "always"}).to_string();
        self.write("selection.json", &serde_json::from_str(&selection).unwrap());
        let result = profile_settings(
            &self.input,
            ProfileSliceRequest {
                filename: "selection.3mf",
                catalog_json: &self.catalog,
                selection_json: &selection,
                plate: None,
                overrides: &ConfigOverrides::default(),
                placement: Placement::None,
            },
        )
        .await
        .map(|settings| serde_json::to_value(settings).unwrap());
        self.write(
            "api-result.json",
            &match &result {
                Ok(value) => json!({"ok": true, "result": value}),
                Err(error) => json!({"ok": false, "error": error.to_string()}),
            },
        );
        result
    }
}

fn scalar(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => if *value { "1" } else { "0" }.to_owned(),
        _ => unreachable!("fixture option is scalar"),
    }
}

fn wire(key: &str, value: &Value) -> String {
    match value {
        Value::Array(values) => {
            if ares_core::option_definition(key).unwrap().kind == OptionValueKind::Strings {
                values
                    .iter()
                    .map(|value| serde_json::to_string(value).unwrap())
                    .collect::<Vec<_>>()
                    .join(";")
            } else {
                values.iter().map(scalar).collect::<Vec<_>>().join(",")
            }
        }
        value => scalar(value),
    }
}
