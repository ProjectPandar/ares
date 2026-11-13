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
        Self::with_catalog(label, settings, json!({"vendors": [], "imports": []}))
    }

    pub fn with_catalog(label: &str, settings: Value, catalog: Value) -> Option<Self> {
        Self::with_native_controls(label, settings, catalog, &[])
    }

    pub fn with_version(
        label: &str,
        settings: Value,
        catalog: Value,
        version: &str,
    ) -> Option<Self> {
        Self::with_version_controls(label, settings, catalog, &[], version)
    }

    pub fn with_native_controls(
        label: &str,
        settings: Value,
        catalog: Value,
        controls: &[Value],
    ) -> Option<Self> {
        Self::with_version_controls(label, settings, catalog, controls, "2.5.0")
    }

    fn with_version_controls(
        label: &str,
        settings: Value,
        catalog: Value,
        controls: &[Value],
        version: &str,
    ) -> Option<Self> {
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
        let system = root.join("data/system");
        fs::create_dir_all(&system).unwrap();
        for vendor in catalog["vendors"].as_array().unwrap() {
            let id = vendor["id"].as_str().unwrap();
            fs::write(
                system.join(format!("{id}.json")),
                vendor["index"].to_string(),
            )
            .unwrap();
            for (filename, document) in vendor["files"].as_object().unwrap() {
                let path = system.join(id).join(filename);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, document.to_string()).unwrap();
            }
        }
        for import in catalog["imports"].as_array().unwrap() {
            for profile in import["profiles"].as_array().unwrap() {
                let kind = profile["type"].as_str().unwrap();
                let name = profile["name"].as_str().unwrap();
                let directory = root.join("data/user/default").join(kind);
                fs::create_dir_all(&directory).unwrap();
                fs::write(directory.join(format!("{name}.json")), profile.to_string()).unwrap();
            }
        }
        let catalog = catalog.to_string();
        fs::write(root.join("catalog.json"), &catalog).unwrap();
        fs::write(root.join("project-settings.json"), settings.to_string()).unwrap();
        let input = super::input::project_version(&settings, version);
        fs::write(root.join("selection.3mf"), &input).unwrap();
        let projects = std::iter::once(&settings)
            .chain(controls.iter())
            .map(|settings| native_project(settings, version))
            .collect::<Vec<_>>();
        let request = json!({
            "data_directory": root.join("data"),
            "projects": projects,
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
        self.view(
            json!({"from_project": true, "load_and_select": "always"}),
            "api",
            (&self.input, &ConfigOverrides::default()),
        )
        .await
    }

    pub async fn cli(&self) -> Result<Value, SliceError> {
        self.view(
            json!({"from_project": true, "uptodate": false}),
            "cli",
            (&self.input, &ConfigOverrides::default()),
        )
        .await
    }

    pub async fn api_with_overrides(
        &self,
        overrides: &ConfigOverrides,
    ) -> Result<Value, SliceError> {
        self.view(
            json!({"from_project": true, "load_and_select": "always"}),
            "overrides",
            (&self.input, overrides),
        )
        .await
    }

    pub async fn boundary(&self, settings: &Value, always: bool) -> Result<Value, SliceError> {
        let input = super::input::project(settings);
        let label = if always {
            "boundary-api"
        } else {
            "boundary-cli"
        };
        fs::write(self.root.join(format!("{label}.3mf")), &input).unwrap();
        let selection = if always {
            json!({"from_project": true, "load_and_select": "always"})
        } else {
            json!({"from_project": true, "uptodate": false})
        };
        self.view(selection, label, (&input, &ConfigOverrides::default()))
            .await
    }

    pub async fn placed(&self, always: bool) -> Result<ares_core::Project, SliceError> {
        let selection = if always {
            json!({"from_project": true, "load_and_select": "always"})
        } else {
            json!({"from_project": true, "uptodate": false})
        }
        .to_string();
        let result = ares_core::profile_placed_project(
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
        .await;
        let label = if always { "api-placed" } else { "cli-placed" };
        self.write(&format!("{label}.json"), &match &result {
            Ok(project) => json!({"ok": true, "models": format!("{:?}", project.models()), "objects": format!("{:?}", project.objects()), "plates": format!("{:?}", project.plates())}),
            Err(error) => json!({"ok": false, "error": error.to_string()}),
        });
        result
    }

    async fn view(
        &self,
        selection: Value,
        label: &str,
        (input, overrides): (&[u8], &ConfigOverrides),
    ) -> Result<Value, SliceError> {
        self.write(&format!("{label}-selection.json"), &selection);
        let selection = selection.to_string();
        let result = profile_settings(
            input,
            ProfileSliceRequest {
                filename: "selection.3mf",
                catalog_json: &self.catalog,
                selection_json: &selection,
                plate: None,
                overrides,
                placement: Placement::None,
            },
        )
        .await
        .map(|settings| serde_json::to_value(settings).unwrap());
        self.write(
            &format!("{label}-result.json"),
            &match &result {
                Ok(value) => json!({"ok": true, "result": value}),
                Err(error) => json!({"ok": false, "error": error.to_string()}),
            },
        );
        result
    }
}

pub fn native_strings(report: &Value) -> Vec<String> {
    assert_eq!(report["type"], 16_387);
    serde_json::from_str(
        &ares_core::decode_config_value(
            OptionValueKind::Strings,
            report["serialized"].as_str().unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn native_project(settings: &Value, version: &str) -> Value {
    let options = settings
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| (key.clone(), Value::String(wire(key, value))))
        .collect::<serde_json::Map<_, _>>();
    json!({"filename": "selection.3mf", "version": version, "options": options})
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
            if ares_core::option_definition(key)
                .is_some_and(|definition| definition.kind == OptionValueKind::Strings)
            {
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
