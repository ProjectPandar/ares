use std::{collections::BTreeMap, io::Read};

use crate::{
    geometry::CoordinateScale,
    project_slice::{
        compensation::{
            PostCompensationPrintObject, PreparedPostCompensation, apply_project_compensation,
            prepare_post_compensation,
        },
        perimeters::{
            PreparedPostPerimeterInputs, finish_post_perimeter_inputs,
            types::PostPerimeterInputPrintObject,
        },
        region_slices::{
            PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface, RegionSurfaceKind,
        },
        task22m_oracle,
    },
};

use super::super::super::{region_fixture::checkpoint, support::KsrArchive};

const PROCESS: &str = "Metadata/project_settings.config";
const NORMAL_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"256x0\",\r\n",
    "\t\t\"256x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);
const LARGE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"2148x0\",\r\n",
    "\t\t\"2148x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);

const ROOT: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22n_box.model" objectid="1"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>"#;
const RELATIONSHIPS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22n_box.model" Id="box" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#;
const SETTINGS: &str = r#"<config><object id="2"><part id="1" subtype="normal_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#;
const LEAF: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices>
<vertex x="0" y="0" z="0"/><vertex x="8" y="0" z="0"/><vertex x="8" y="4" z="0"/><vertex x="4.6" y="4" z="0"/><vertex x="4.6" y="9" z="0"/><vertex x="3.4" y="9" z="0"/><vertex x="3.4" y="4" z="0"/><vertex x="0" y="4" z="0"/>
<vertex x="0" y="0" z="0.4"/><vertex x="8" y="0" z="0.4"/><vertex x="8" y="4" z="0.4"/><vertex x="4.6" y="4" z="0.4"/><vertex x="4.6" y="9" z="0.4"/><vertex x="3.4" y="9" z="0.4"/><vertex x="3.4" y="4" z="0.4"/><vertex x="0" y="4" z="0.4"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="0" v2="6" v3="3"/><triangle v1="0" v2="7" v3="6"/><triangle v1="3" v2="5" v3="4"/><triangle v1="3" v2="6" v3="5"/>
<triangle v1="8" v2="9" v3="10"/><triangle v1="8" v2="10" v3="11"/><triangle v1="8" v2="11" v3="14"/><triangle v1="8" v2="14" v3="15"/><triangle v1="11" v2="12" v3="13"/><triangle v1="11" v2="13" v3="14"/>
<triangle v1="0" v2="1" v3="9"/><triangle v1="0" v2="9" v3="8"/><triangle v1="1" v2="2" v3="10"/><triangle v1="1" v2="10" v3="9"/><triangle v1="2" v2="3" v3="11"/><triangle v1="2" v2="11" v3="10"/><triangle v1="3" v2="4" v3="12"/><triangle v1="3" v2="12" v3="11"/>
<triangle v1="4" v2="5" v3="13"/><triangle v1="4" v2="13" v3="12"/><triangle v1="5" v2="6" v3="14"/><triangle v1="5" v2="14" v3="13"/><triangle v1="6" v2="7" v3="15"/><triangle v1="6" v2="15" v3="14"/><triangle v1="7" v2="0" v3="8"/><triangle v1="7" v2="8" v3="15"/>
</triangles></mesh></object></resources><build/></model>"#;
const SECOND_VERTICES: &str = r#"<vertex x="12" y="0" z="0"/><vertex x="14" y="0" z="0"/><vertex x="14" y="2" z="0"/><vertex x="12" y="2" z="0"/><vertex x="12" y="0" z="0.6"/><vertex x="14" y="0" z="0.6"/><vertex x="14" y="2" z="0.6"/><vertex x="12" y="2" z="0.6"/>"#;
const SECOND_TRIANGLES: &str = r#"<triangle v1="16" v2="18" v3="17"/><triangle v1="16" v2="19" v3="18"/><triangle v1="20" v2="21" v3="22"/><triangle v1="20" v2="22" v3="23"/><triangle v1="16" v2="17" v3="21"/><triangle v1="16" v2="21" v3="20"/><triangle v1="17" v2="18" v3="22"/><triangle v1="17" v2="22" v3="21"/><triangle v1="18" v2="19" v3="23"/><triangle v1="18" v2="23" v3="22"/><triangle v1="19" v2="16" v3="20"/><triangle v1="19" v2="20" v3="23"/>"#;

#[derive(Clone)]
pub(super) struct ArchiveBuilder {
    archive: KsrArchive,
}

impl ArchiveBuilder {
    pub(super) fn new() -> Self {
        let mut archive = KsrArchive::new();
        for (path, body) in [
            ("3D/3dmodel.model", ROOT),
            ("3D/_rels/3dmodel.model.rels", RELATIONSHIPS),
            ("3D/Objects/task22n_box.model", LEAF),
            ("Metadata/model_settings.config", SETTINGS),
        ] {
            archive.insert_text(path, body);
        }
        archive.replace_unique(
            PROCESS,
            r#""elefant_foot_compensation": "0.15""#,
            r#""elefant_foot_compensation": "0""#,
        );
        Self { archive }
    }

    pub(super) fn replace_unique(&mut self, path: &str, from: &str, to: &str) {
        self.archive.replace_unique(path, from, to);
    }

    pub(super) fn replace_all(&mut self, path: &str, from: &str, to: &str) {
        self.archive.replace(path, from, to);
    }

    pub(super) fn three_layer_two_contour(mut self) -> Self {
        let leaf = LEAF
            .replace(r#"z="0.4""#, r#"z="0.6""#)
            .replace("</vertices>", &[SECOND_VERTICES, "</vertices>"].concat())
            .replace("</triangles>", &[SECOND_TRIANGLES, "</triangles>"].concat());
        self.archive
            .replace_unique("3D/Objects/task22n_box.model", LEAF, &leaf);
        self
    }

    pub(super) fn bytes(self) -> Vec<u8> {
        self.archive.bytes()
    }
}

pub(super) fn semantic_identity(bytes: &[u8]) -> (usize, String) {
    checkpoint::semantic_identity(bytes)
}

pub(super) fn archive_entries(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut entries = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        if !file.is_dir() {
            let mut body = Vec::new();
            file.read_to_end(&mut body).unwrap();
            entries.insert(file.name().to_owned(), body);
        }
    }
    entries
}

pub(super) fn assert_single_entry_replacement(
    before: &[u8],
    after: &[u8],
    path: &str,
    from: &str,
    to: &str,
) {
    let mut before = archive_entries(before);
    let mut after = archive_entries(after);
    let before_entry = String::from_utf8(before.remove(path).unwrap()).unwrap();
    let after_entry = String::from_utf8(after.remove(path).unwrap()).unwrap();
    assert_eq!(before, after);
    assert_eq!(before_entry.match_indices(from).count(), 1);
    assert_eq!(after_entry, before_entry.replacen(from, to, 1));
}

#[derive(Debug, Eq, PartialEq)]
struct OwnedState {
    plans: Vec<Vec<[u64; 4]>>,
    metadata: Vec<Vec<Vec<Vec<SurfaceMetadata>>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SurfaceMetadata {
    kind: RegionSurfaceKind,
    thickness: u64,
    thickness_layers: u16,
    bridge_angle: u64,
    extra_perimeters: u16,
}

fn assert_outer_wrapper_preserved(archive: &[u8], expected_scale: CoordinateScale) {
    let baseline = prepare_post_compensation(archive).unwrap();
    let baseline_m = task22m_oracle::encode(&baseline.objects);
    let baseline_state = owned_state(&baseline.objects);
    assert_eq!(baseline.scale, expected_scale);

    let candidate = prepare_post_compensation(archive).unwrap();
    assert_eq!(candidate.resolved, baseline.resolved);
    assert_eq!(candidate.config_block, baseline.config_block);
    assert_eq!(task22m_oracle::encode(&candidate.objects), baseline_m);
    assert_eq!(owned_state(&candidate.objects), baseline_state);

    let candidate = inject_distinct_surface_metadata(candidate);
    let injected_state = owned_state(&candidate.objects);
    assert_eq!(task22m_oracle::encode(&candidate.objects), baseline_m);
    assert_eq!(injected_state.plans, baseline_state.plans);
    assert_ne!(injected_state.metadata, baseline_state.metadata);
    assert_eq!(injected_count(&injected_state), 1);

    let PreparedPostPerimeterInputs {
        resolved,
        config_block,
        scale,
        objects,
        ..
    } = finish_post_perimeter_inputs(candidate).unwrap();
    assert_eq!(resolved, baseline.resolved);
    assert_eq!(config_block, baseline.config_block);
    assert_eq!(scale, expected_scale);
    assert_eq!(wrapped_state(&objects), injected_state);

    let objects = objects
        .into_iter()
        .map(|object| object.into_parts().0)
        .collect::<Vec<_>>();
    assert_eq!(task22m_oracle::encode(&objects), baseline_m);
}

fn inject_distinct_surface_metadata(
    prepared: PreparedPostCompensation,
) -> PreparedPostCompensation {
    let PreparedPostCompensation {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepared;
    let mut injected = false;
    let objects = objects
        .into_iter()
        .map(|object| inject_object_metadata(object, &mut injected))
        .collect::<Vec<_>>();
    assert!(injected, "archive must produce at least one region surface");

    let initial_layer_width = resolved.views.full.process.print.initial_layer_line_width;
    let nozzle_diameters = &resolved.views.full.project.print.nozzle_diameter;
    let objects = apply_project_compensation(
        objects,
        &resolved.objects,
        initial_layer_width,
        nozzle_diameters,
        scale,
    )
    .unwrap();
    PreparedPostCompensation {
        project,
        resolved,
        config_block,
        scale,
        objects,
    }
}

fn inject_object_metadata(
    object: PostCompensationPrintObject,
    injected: &mut bool,
) -> PostRegionPrintObject {
    let (post_region, _) = object.into_parts();
    let (plan, volume_slices, regions) = post_region.into_parts();
    let regions = regions
        .into_iter()
        .map(|region| inject_region_metadata(region, injected))
        .collect();
    PostRegionPrintObject {
        plan,
        volume_slices,
        regions,
    }
}

fn inject_region_metadata(region: PostRegion, injected: &mut bool) -> PostRegion {
    let (id, options, layers) = region.into_parts();
    let layers = layers
        .into_iter()
        .map(|layer| inject_layer_metadata(layer, injected))
        .collect();
    PostRegion {
        id,
        options,
        layers,
    }
}

fn inject_layer_metadata(layer: RegionLayer, injected: &mut bool) -> RegionLayer {
    let surfaces = layer
        .into_parts()
        .into_iter()
        .map(|surface| inject_surface_metadata(surface, injected))
        .collect();
    RegionLayer { surfaces }
}

fn inject_surface_metadata(surface: RegionSurface, injected: &mut bool) -> RegionSurface {
    if *injected {
        return surface;
    }
    *injected = true;
    let (kind, expolygon, ..) = surface.into_parts();
    assert_eq!(kind, RegionSurfaceKind::Internal);
    RegionSurface::internal_with_metadata(expolygon, 0.3125, 7, -0.625, 11)
}

fn wrapped_state(objects: &[PostPerimeterInputPrintObject]) -> OwnedState {
    owned_state_from(objects.iter().map(|object| object.as_parts().0))
}

fn owned_state(objects: &[PostCompensationPrintObject]) -> OwnedState {
    owned_state_from(objects)
}

fn owned_state_from<'a>(
    objects: impl IntoIterator<Item = &'a PostCompensationPrintObject>,
) -> OwnedState {
    let objects = objects.into_iter().collect::<Vec<_>>();
    OwnedState {
        plans: objects
            .iter()
            .map(|object| {
                object
                    .as_parts()
                    .0
                    .as_parts()
                    .0
                    .layers
                    .iter()
                    .map(|layer| {
                        [
                            layer.id as u64,
                            layer.height.to_bits(),
                            layer.print_z.to_bits(),
                            layer.slice_z.to_bits(),
                        ]
                    })
                    .collect()
            })
            .collect(),
        metadata: objects
            .iter()
            .map(|object| {
                object
                    .as_parts()
                    .0
                    .as_parts()
                    .2
                    .iter()
                    .map(|region| {
                        region
                            .as_parts()
                            .2
                            .iter()
                            .map(|layer| layer.surfaces().iter().map(surface_metadata).collect())
                            .collect()
                    })
                    .collect()
            })
            .collect(),
    }
}

fn surface_metadata(surface: &RegionSurface) -> SurfaceMetadata {
    let (kind, _, thickness, thickness_layers, bridge_angle, extra_perimeters) = surface.as_parts();
    SurfaceMetadata {
        kind,
        thickness: thickness.to_bits(),
        thickness_layers,
        bridge_angle: bridge_angle.to_bits(),
        extra_perimeters,
    }
}

fn injected_count(state: &OwnedState) -> usize {
    let injected = SurfaceMetadata {
        kind: RegionSurfaceKind::Internal,
        thickness: 0.3125_f64.to_bits(),
        thickness_layers: 7,
        bridge_angle: (-0.625_f64).to_bits(),
        extra_perimeters: 11,
    };
    state
        .metadata
        .iter()
        .flatten()
        .flatten()
        .flatten()
        .filter(|&&metadata| metadata == injected)
        .count()
}

#[test]
fn task22n_archive_builder_is_deterministic_and_replaces_one_semantic_entry() {
    let normal = ArchiveBuilder::new().bytes();
    assert_eq!(ArchiveBuilder::new().bytes(), normal);

    let mut large = ArchiveBuilder::new();
    large.replace_unique(PROCESS, NORMAL_AREA, LARGE_AREA);
    let large = large.bytes();
    assert_single_entry_replacement(&normal, &large, PROCESS, NORMAL_AREA, LARGE_AREA);
    assert_ne!(semantic_identity(&normal), semantic_identity(&large));
}

#[test]
fn task22n_outer_wrapper_preserves_scale_resolved_config_and_full_surface_metadata() {
    let normal = ArchiveBuilder::new().bytes();
    let mut large = ArchiveBuilder::new();
    large.replace_unique(PROCESS, NORMAL_AREA, LARGE_AREA);
    let large = large.bytes();

    assert_outer_wrapper_preserved(&normal, crate::geometry::CoordinateScale::Normal);
    assert_outer_wrapper_preserved(&large, crate::geometry::CoordinateScale::LargeBed);
}
