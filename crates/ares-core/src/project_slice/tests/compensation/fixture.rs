use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
    panic::catch_unwind,
};

use crate::{FloatOrPercent, OrcaFloat, OrcaInt, Percent};

use super::super::super::{
    prepare_post_conical_overhang, task22m_browser_input_oracle, task22m_browser_oracle,
};
use super::super::{region_fixture::checkpoint as region_checkpoint, support::KsrArchive};

mod checkpoint;
mod options;

use checkpoint::{expolygon, parse_m, surface_geometry};

const PROCESS: &str = "Metadata/project_settings.config";

#[derive(Clone, Copy, Debug)]
enum Variant {
    Enabled,
    Disabled,
    AntiMap([i32; 2]),
}

#[test]
fn task22m_small_archives_drive_option_dependent_compensation() {
    let variants = [
        Variant::Enabled,
        Variant::Disabled,
        Variant::AntiMap([1, 2]),
        Variant::AntiMap([2, 1]),
    ];
    let mut archives = Vec::new();
    let mut checkpoints = Vec::new();
    let mut outputs = Vec::new();
    for variant in variants {
        let archive = small_archive(variant);
        assert_eq!(small_archive(variant), archive);
        assert_loaded_options(&archive, variant);
        let output = task22m_browser_input_oracle(&archive).unwrap();
        assert_eq!(task22m_browser_input_oracle(&archive).unwrap(), output);
        let m = task22m_browser_oracle(&archive).unwrap();
        assert_eq!(task22m_browser_oracle(&archive).unwrap(), m);
        assert_small_semantics(&output, &m, matches!(variant, Variant::Disabled));
        archives.push(archive);
        checkpoints.push(output);
        outputs.push(m);
    }
    assert_only_process_replacement(
        &archives[0],
        &archives[1],
        r#""elefant_foot_compensation": "0.15""#,
        r#""elefant_foot_compensation": "0""#,
    );
    assert_only_process_replacement(&archives[2], &archives[3], FILAMENT_MAP_12, FILAMENT_MAP_21);
    assert!(checkpoints[1..].iter().all(|item| item == &checkpoints[0]));
    assert_ne!(outputs[0], outputs[1]);
    assert!(outputs[2..].iter().all(|item| item == &outputs[0]));

    let underflow = spacing_valid_volume_underflow_archive();
    let output = task22m_browser_oracle(&underflow).unwrap();
    assert_eq!(parse_m(&output).len(), 1);
}

#[test]
fn task22m_m_parser_rejects_wrong_magic_nested_truncation_and_trailing_bytes() {
    let valid = task22m_browser_oracle(small_archive(Variant::Enabled)).unwrap();
    assert_eq!(parse_m(&valid).len(), 1);
    let mut wrong_magic = valid.clone();
    wrong_magic[6] = b'L';
    let mut trailing = valid.clone();
    trailing.push(0);
    for invalid in [
        wrong_magic.as_slice(),
        &valid[..valid.len() / 2],
        &valid[..valid.len() - 1],
        trailing.as_slice(),
    ] {
        assert!(catch_unwind(|| parse_m(invalid)).is_err());
    }
}

fn assert_small_semantics(l: &[u8], m: &[u8], disabled: bool) {
    let l = region_checkpoint::parse_l(l).stream;
    let m = parse_m(m);
    let (l, (m, lslices)) = (&l.objects[0], &m[0]);
    assert_eq!(m.sidecars, l.sidecars);
    assert_eq!(
        (
            m.retained_layers.len(),
            m.retained_layers[0].index,
            m.retained_layers[1].index,
        ),
        (2, 0, 1)
    );
    let raw = expolygon(RAW_CONTOUR);
    assert_eq!(
        surface_geometry(&l.retained_layers[0].regions).as_slice(),
        std::slice::from_ref(&raw)
    );
    assert_eq!(lslices[0].as_slice(), std::slice::from_ref(&raw));
    assert_eq!(
        surface_geometry(&m.retained_layers[0].regions),
        [if disabled {
            raw.clone()
        } else {
            expolygon(ENABLED_CONTOUR)
        }]
    );
    assert_eq!(
        surface_geometry(&m.retained_layers[1].regions).as_slice(),
        std::slice::from_ref(&raw)
    );
    assert_eq!(lslices[1], [raw]);
}

pub(super) fn parse_m_object_count(bytes: &[u8]) -> usize {
    checkpoint::parse_m(bytes).len()
}

fn assert_only_process_replacement(left: &[u8], right: &[u8], from: &str, to: &str) {
    let mut left = archive_entries(left);
    let mut right = archive_entries(right);
    let left_process = String::from_utf8(left.remove(PROCESS).unwrap()).unwrap();
    let right_process = String::from_utf8(right.remove(PROCESS).unwrap()).unwrap();

    assert_eq!(left, right);
    assert_eq!(left_process.match_indices(from).count(), 1);
    assert_eq!(left_process.replacen(from, to, 1), right_process);
}

fn archive_entries(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut entries = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        if file.is_dir() {
            continue;
        }
        let mut body = Vec::new();
        file.read_to_end(&mut body).unwrap();
        entries.insert(file.name().to_owned(), body);
    }
    entries
}

fn assert_loaded_options(archive: &[u8], variant: Variant) {
    let prepared = prepare_post_conical_overhang(archive).unwrap();
    assert_eq!(
        (
            prepared.objects.len(),
            prepared.objects[0].plan.layers.len()
        ),
        (1, 2)
    );
    let object = &prepared.resolved.objects[0].object;
    let region = &prepared.objects[0].regions[0].options;
    let print = &prepared.resolved.views.full;
    let expected_compensation = if matches!(variant, Variant::Disabled) {
        0.0
    } else {
        0.15
    };
    assert_eq!(
        (
            object.elefant_foot_compensation,
            object.elefant_foot_compensation_layers,
            object.raft_layers,
            region.outer_wall_filament_id,
        ),
        (
            OrcaFloat(expected_compensation),
            OrcaInt(1),
            OrcaInt(0),
            OrcaInt(1),
        )
    );
    match variant {
        Variant::AntiMap(expected_map) => {
            let project = &print.project;
            let nozzles = project.print.nozzle_diameter.0.iter().map(|value| value.0);
            let map = project.gcode.filament_map.0.iter().map(|value| value.0);
            assert_eq!(
                print.process.print.initial_layer_line_width,
                FloatOrPercent::Percent(Percent(125.0))
            );
            assert_eq!(nozzles.collect::<Vec<_>>(), [0.4, 0.6]);
            assert_eq!(map.collect::<Vec<_>>(), expected_map);
        }
        Variant::Enabled | Variant::Disabled => assert_eq!(
            print.process.print.initial_layer_line_width,
            FloatOrPercent::Float(0.5)
        ),
    }
}

fn small_archive(variant: Variant) -> Vec<u8> {
    let mut archive = small_archive_source();

    let replace = |archive: &mut KsrArchive, from, to| {
        archive.replace_unique(PROCESS, from, to);
    };
    match variant {
        Variant::Enabled => {}
        Variant::Disabled => replace(
            &mut archive,
            r#""elefant_foot_compensation": "0.15""#,
            r#""elefant_foot_compensation": "0""#,
        ),
        Variant::AntiMap(map) => {
            replace(
                &mut archive,
                r#""initial_layer_line_width": "0.5""#,
                r#""initial_layer_line_width": "125%""#,
            );
            replace(&mut archive, NOZZLES_04_04, NOZZLES_04_06);
            replace(
                &mut archive,
                r#""outer_wall_filament_id": "0""#,
                r#""outer_wall_filament_id": "1""#,
            );
            replace(
                &mut archive,
                FILAMENT_MAP_11,
                if map == [1, 2] {
                    FILAMENT_MAP_12
                } else {
                    FILAMENT_MAP_21
                },
            );
        }
    }
    archive.bytes()
}

fn spacing_valid_volume_underflow_archive() -> Vec<u8> {
    let mut archive = small_archive_source();
    archive.replace("3D/Objects/task22m_box.model", r#"z="0.4""#, r#"z="1e-30""#);
    for (from, to) in [
        (r#""layer_height": "0.2""#, r#""layer_height": "1e-30""#),
        (
            r#""initial_layer_print_height": "0.2""#,
            r#""initial_layer_print_height": "1e-30""#,
        ),
        (
            r#""initial_layer_line_width": "0.5""#,
            r#""initial_layer_line_width": "1e-30""#,
        ),
    ] {
        archive.replace_unique(PROCESS, from, to);
    }
    archive.bytes()
}

fn small_archive_source() -> KsrArchive {
    let mut archive = KsrArchive::new();
    for (path, text) in [
        ("3D/3dmodel.model", ROOT),
        ("3D/_rels/3dmodel.model.rels", RELATIONSHIPS),
        ("3D/Objects/task22m_box.model", LEAF),
        ("Metadata/model_settings.config", SETTINGS),
    ] {
        archive.insert_text(path, text);
    }
    archive
}

const NOZZLES_04_04: &str = "\t\"nozzle_diameter\": [\r\n\t\t\"0.4\",\r\n\t\t\"0.4\"\r\n\t]";
const NOZZLES_04_06: &str = "\t\"nozzle_diameter\": [\r\n\t\t\"0.4\",\r\n\t\t\"0.6\"\r\n\t]";
const FILAMENT_MAP_11: &str = "\t\"filament_map\": [\r\n\t\t\"1\",\r\n\t\t\"1\"\r\n\t]";
const FILAMENT_MAP_12: &str = "\t\"filament_map\": [\r\n\t\t\"1\",\r\n\t\t\"2\"\r\n\t]";
const FILAMENT_MAP_21: &str = "\t\"filament_map\": [\r\n\t\t\"2\",\r\n\t\t\"1\"\r\n\t]";

#[rustfmt::skip]
const RAW_CONTOUR: &[(i64, i64)] = &[(4_000_000,-500_000),(600_000,-500_000),(600_000,4_500_000),(-600_000,4_500_000),(-600_000,-500_000),(-4_000_000,-500_000),(-4_000_000,-4_500_000),(4_000_000,-4_500_000)];
#[rustfmt::skip]
const ENABLED_CONTOUR: &[(i64, i64)] = &[(3_850_000,-650_000),(2_542_857,-650_000),(2_057_142,-649_904),(1_571_428,-648_459),(1_085_714,-640_528),(478_540,-621_460),(478_540,3_500_000),(459_472,4_000_000),(453_904,4_351_013),(200_000,4_350_096),(-200_000,4_350_096),(-453_904,4_351_013),(-459_472,4_000_000),(-478_540,3_500_000),(-478_540,-621_460),(-1_085_714,-640_528),(-1_571_428,-648_459),(-2_057_142,-649_904),(-2_542_857,-650_000),(-3_850_000,-650_000),(-3_850_000,-4_350_000),(3_850_000,-4_350_000)];

#[rustfmt::skip]
const ROOT: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22m_box.model" objectid="1"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>"#;
#[rustfmt::skip]
const RELATIONSHIPS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22m_box.model" Id="box" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#;
#[rustfmt::skip]
const SETTINGS: &str = r#"<config><object id="2"><part id="1" subtype="normal_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#;
#[rustfmt::skip]
const LEAF: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices>
<vertex x="0" y="0" z="0"/><vertex x="8" y="0" z="0"/><vertex x="8" y="4" z="0"/><vertex x="4.6" y="4" z="0"/><vertex x="4.6" y="9" z="0"/><vertex x="3.4" y="9" z="0"/><vertex x="3.4" y="4" z="0"/><vertex x="0" y="4" z="0"/>
<vertex x="0" y="0" z="0.4"/><vertex x="8" y="0" z="0.4"/><vertex x="8" y="4" z="0.4"/><vertex x="4.6" y="4" z="0.4"/><vertex x="4.6" y="9" z="0.4"/><vertex x="3.4" y="9" z="0.4"/><vertex x="3.4" y="4" z="0.4"/><vertex x="0" y="4" z="0.4"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="0" v2="6" v3="3"/><triangle v1="0" v2="7" v3="6"/><triangle v1="3" v2="5" v3="4"/><triangle v1="3" v2="6" v3="5"/>
<triangle v1="8" v2="9" v3="10"/><triangle v1="8" v2="10" v3="11"/><triangle v1="8" v2="11" v3="14"/><triangle v1="8" v2="14" v3="15"/><triangle v1="11" v2="12" v3="13"/><triangle v1="11" v2="13" v3="14"/>
<triangle v1="0" v2="1" v3="9"/><triangle v1="0" v2="9" v3="8"/><triangle v1="1" v2="2" v3="10"/><triangle v1="1" v2="10" v3="9"/><triangle v1="2" v2="3" v3="11"/><triangle v1="2" v2="11" v3="10"/><triangle v1="3" v2="4" v3="12"/><triangle v1="3" v2="12" v3="11"/>
<triangle v1="4" v2="5" v3="13"/><triangle v1="4" v2="13" v3="12"/><triangle v1="5" v2="6" v3="14"/><triangle v1="5" v2="14" v3="13"/><triangle v1="6" v2="7" v3="15"/><triangle v1="6" v2="15" v3="14"/><triangle v1="7" v2="0" v3="8"/><triangle v1="7" v2="8" v3="15"/>
</triangles></mesh></object></resources><build/></model>"#;
