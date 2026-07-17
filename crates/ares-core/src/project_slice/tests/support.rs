use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Write},
};

use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    GenerationMetadata, ObjectOptions, OrcaInt, Point3d, ProjectInstance, ProjectMesh,
    ProjectObject, ProjectSettings, ProjectVolume, ProjectVolumeType, RegionOptions, Transform3d,
    load_project,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
    project::effective_config::types::{
        ResolvedLayerCandidate, ResolvedModelPartCandidate, ResolvedPrintObjectConfig,
        ResolvedProjectObject,
    },
};

const KSR_PROJECT: &[u8] =
    include_bytes!("../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
const FLUSH_MATRIX: &str = concat!(
    "\t\"flush_volumes_matrix\": [\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"0\"\r\n",
    "\t]",
);
const INVALID_FLUSH_MATRIX: &str = "\t\"flush_volumes_matrix\": [\r\n\t\t\"0\"\r\n\t]";

#[derive(Clone)]
pub(super) struct KsrArchive {
    entries: BTreeMap<String, Vec<u8>>,
}

impl KsrArchive {
    pub(super) fn new() -> Self {
        let mut archive = ZipArchive::new(Cursor::new(KSR_PROJECT)).unwrap();
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            if file.is_dir() {
                continue;
            }
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            entries.insert(file.name().to_owned(), bytes);
        }
        Self { entries }
    }

    pub(super) fn insert_text(&mut self, path: &str, text: &str) {
        self.entries
            .insert(path.to_owned(), text.as_bytes().to_vec());
    }

    pub(super) fn replace(&mut self, path: &str, from: &str, to: &str) {
        let text = String::from_utf8(self.entries.remove(path).unwrap()).unwrap();
        assert!(text.contains(from), "{path} does not contain {from:?}");
        self.entries
            .insert(path.to_owned(), text.replace(from, to).into_bytes());
    }

    pub(super) fn invalidate_flush_matrix(&mut self) {
        self.replace(
            "Metadata/project_settings.config",
            FLUSH_MATRIX,
            INVALID_FLUSH_MATRIX,
        );
    }

    pub(super) fn repair_flush_matrix(&mut self) {
        self.replace(
            "Metadata/project_settings.config",
            INVALID_FLUSH_MATRIX,
            FLUSH_MATRIX,
        );
    }

    pub(super) fn bytes(self) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (path, bytes) in self.entries {
            writer.start_file(path, options).unwrap();
            writer.write_all(&bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}

pub(super) fn ksr_project() -> &'static [u8] {
    KSR_PROJECT
}

pub(super) fn metadata() -> GenerationMetadata {
    GenerationMetadata::deterministic(2026, 7, 16, 1, 2, 3)
}

pub(super) fn object_options() -> ObjectOptions {
    ObjectOptions::from_base(&ProjectSettings::default().process.object)
}

pub(super) fn region() -> RegionOptions {
    RegionOptions::from_base(&ProjectSettings::default().process.region)
}

pub(super) fn source(
    object_extruder: Option<i32>,
    volumes: &[(ProjectVolumeType, Option<i32>)],
) -> ProjectObject {
    let object_region = RegionOptionOverrides {
        extruder: object_extruder.map(OrcaInt),
        ..Default::default()
    };
    ProjectObject::new(
        "synthetic.model".to_owned(),
        1,
        (
            "object".to_owned(),
            String::new(),
            ObjectOptionOverrides::default(),
            object_region,
        ),
        volumes
            .iter()
            .enumerate()
            .map(|(index, (volume_type, extruder))| {
                let region = RegionOptionOverrides {
                    extruder: extruder.map(OrcaInt),
                    ..Default::default()
                };
                ProjectVolume::new(
                    "synthetic.model".to_owned(),
                    index as u32 + 1,
                    ProjectMesh::new(vec![Point3d::new(0.0, 0.0, 1.0)], Vec::new()),
                    Transform3d::IDENTITY,
                    (
                        "volume".to_owned(),
                        *volume_type,
                        region,
                        Transform3d::IDENTITY,
                    ),
                )
            })
            .collect(),
        vec![ProjectInstance::new(
            [1, 0, 1_000],
            true,
            false,
            Transform3d::IDENTITY,
        )],
    )
}

pub(super) fn resolved(
    source_object_index: usize,
    object: ObjectOptions,
    regions: Vec<RegionOptions>,
) -> ResolvedProjectObject {
    ResolvedProjectObject {
        source_object_index,
        object,
        print_objects: vec![ResolvedPrintObjectConfig {
            transform: Transform3d::IDENTITY,
        }],
        layer_candidates: vec![ResolvedLayerCandidate {
            min_z: 0.0,
            max_z: 1.0,
            source_range_index: None,
            model_parts: regions
                .into_iter()
                .enumerate()
                .map(|(volume_index, region)| ResolvedModelPartCandidate {
                    volume_index,
                    region,
                })
                .collect(),
        }],
    }
}

pub(super) fn project_with_range(min_z: f64, max_z: f64, extruder: i32) -> crate::Project {
    let mut archive = KsrArchive::new();
    archive.insert_text(
        "Metadata/layer_config_ranges.xml",
        &format!(
            r#"<objects><object id="1"><range min_z="{min_z}" max_z="{max_z}"><option opt_key="extruder">{extruder}</option></range></object></objects>"#
        ),
    );
    load_project(archive.bytes()).unwrap()
}
