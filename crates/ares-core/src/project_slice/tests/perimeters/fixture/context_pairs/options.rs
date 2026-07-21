use crate::{
    OrcaBool, OrcaInt, ProcessPerimeterGenerator, load_project,
    project::effective_config::resolve_bounded_project_config,
    project_slice::{task22n_browser_input_oracle, task22n_browser_oracle},
};

use super::super::{
    super::oracle::{NFrame, parse_n},
    archive::{ArchiveBuilder, assert_single_entry_replacement, semantic_identity},
};
use crate::project_slice::tests::region_fixture::checkpoint::sha256;

const PROCESS: &str = "Metadata/project_settings.config";
const ROOT: &str = "3D/3dmodel.model";
const SPIRAL_OFF: &str = r#""spiral_mode": "0""#;
const SPIRAL_ON: &str = r#""spiral_mode": "1""#;
const BOTTOM_THREE: &str = r#""bottom_shell_layers": "3""#;
const BOTTOM_ZERO: &str = r#""bottom_shell_layers": "0""#;
const BOTTOM_ONE: &str = r#""bottom_shell_layers": "1""#;
const THICKNESS_ZERO: &str = r#""bottom_shell_thickness": "0""#;
const THICKNESS_POINT_THREE: &str = r#""bottom_shell_thickness": "0.3""#;
const CLASSIC: &str = r#""wall_generator": "classic""#;
const ARACHNE: &str = r#""wall_generator": "arachne""#;
const ALIGN_OFF: &str = r#""align_infill_direction_to_model": "0""#;
const ALIGN_ON: &str = r#""align_infill_direction_to_model": "1""#;
const IDENTITY: &str = r#"transform="1 0 0 0 1 0 0 0 1 0 0 0""#;
const ROTATED: &str = r#"transform="0 1 0 -1 0 0 0 0 1 0 0 0""#;
const NEGATIVE_ZERO: &str = r#"transform="1 -0 0 0 1 0 0 0 1 0 0 0""#;
const PI_OVER_TWO: u64 = 0x3ff921fb54442d18;
const NEGATIVE_ZERO_BITS: u64 = 0x8000000000000000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MIdentity {
    len: usize,
    sha: &'static str,
}

const M_SPIRAL_OFF_BOTTOM_ONE_POINT_THREE: MIdentity = MIdentity {
    len: 2_262,
    sha: "14d1a4a280c53d9331dfed8e1c4be152f799ab5d1daee7fa6b2f00cb53f7e53b",
};
const M_SPIRAL_ON_BOTTOM_ZERO_ZERO: MIdentity = MIdentity {
    len: 1_539,
    sha: "b6c8dd1a473bcf43ed4fdfcbff3efce196a293aa7408e7389510fe1130ea1f13",
};
const M_SPIRAL_ON_AFTER_BOTTOM_GATE: MIdentity = MIdentity {
    len: 1_780,
    sha: "35d7b6720a3c5252d5f274e40f6b0e96991f737c435afc1a1063c6b2d20b6475",
};
const SPIRAL_M: [MIdentity; 2] = [
    M_SPIRAL_OFF_BOTTOM_ONE_POINT_THREE,
    M_SPIRAL_ON_AFTER_BOTTOM_GATE,
];
const BOTTOM_LAYERS_M: [MIdentity; 2] =
    [M_SPIRAL_ON_BOTTOM_ZERO_ZERO, M_SPIRAL_ON_AFTER_BOTTOM_GATE];
const BOTTOM_THICKNESS_M: [MIdentity; 2] = BOTTOM_LAYERS_M;

#[derive(Clone, Copy)]
enum MRelation {
    Delta([MIdentity; 2]),
    Equal,
}

#[derive(Clone, Copy)]
struct Expected {
    generator: ProcessPerimeterGenerator,
    spiral: bool,
    bottom: i32,
    thickness: u64,
    align: bool,
    column: [u64; 2],
    records: [(bool, u8, u64); 3],
}

#[test]
fn task22n_context_pairs_freeze_spiral_and_bottom_shell_gates() {
    let arachne = ProcessPerimeterGenerator::Arachne;
    for (name, base, from, to, before, after, relation) in [
        (
            "spiral",
            context_archive(arachne, false, 1, 0.3),
            SPIRAL_OFF,
            SPIRAL_ON,
            expected((arachne, false, 1, 0.3, false), [(false, 1, 0); 3]),
            expected(
                (arachne, true, 1, 0.3, false),
                [(false, 1, 0), (true, 0, 0), (true, 0, 0)],
            ),
            MRelation::Delta(SPIRAL_M),
        ),
        (
            "bottom layers",
            context_archive(arachne, true, 0, 0.0),
            BOTTOM_ZERO,
            BOTTOM_ONE,
            expected((arachne, true, 0, 0.0, false), [(true, 0, 0); 3]),
            expected(
                (arachne, true, 1, 0.0, false),
                [(false, 1, 0), (true, 0, 0), (true, 0, 0)],
            ),
            MRelation::Delta(BOTTOM_LAYERS_M),
        ),
        (
            "bottom thickness",
            context_archive(arachne, true, 0, 0.0),
            THICKNESS_ZERO,
            THICKNESS_POINT_THREE,
            expected((arachne, true, 0, 0.0, false), [(true, 0, 0); 3]),
            expected(
                (arachne, true, 0, 0.3, false),
                [(false, 1, 0), (true, 0, 0), (true, 0, 0)],
            ),
            MRelation::Delta(BOTTOM_THICKNESS_M),
        ),
    ] {
        assert_pair(name, base, (PROCESS, from, to), [before, after], relation);
    }
}

#[test]
fn task22n_context_pairs_freeze_alignment_transform_and_generator_dispatch() {
    let classic = ProcessPerimeterGenerator::Classic;
    let arachne = ProcessPerimeterGenerator::Arachne;
    let mut rotated = context_archive(classic, false, 3, 0.0);
    rotated.replace_unique(ROOT, IDENTITY, ROTATED);
    assert_pair(
        "alignment",
        rotated,
        (PROCESS, ALIGN_OFF, ALIGN_ON),
        [
            expected_with_column(classic, false, [0, 1.0_f64.to_bits()], [(false, 0, 0); 3]),
            expected_with_column(
                classic,
                true,
                [0, 1.0_f64.to_bits()],
                [(false, 0, PI_OVER_TWO); 3],
            ),
        ],
        MRelation::Equal,
    );

    let mut signed = context_archive(classic, false, 3, 0.0);
    signed.replace_unique(PROCESS, ALIGN_OFF, ALIGN_ON);
    assert_pair(
        "signed zero",
        signed,
        (ROOT, IDENTITY, NEGATIVE_ZERO),
        [
            expected_with_column(classic, true, [1.0_f64.to_bits(), 0], [(false, 0, 0); 3]),
            expected_with_column(
                classic,
                true,
                [1.0_f64.to_bits(), NEGATIVE_ZERO_BITS],
                [(false, 0, NEGATIVE_ZERO_BITS); 3],
            ),
        ],
        MRelation::Equal,
    );

    assert_pair(
        "generator",
        context_archive(classic, false, 3, 0.0),
        (PROCESS, CLASSIC, ARACHNE),
        [
            expected((classic, false, 3, 0.0, false), [(false, 0, 0); 3]),
            expected((arachne, false, 3, 0.0, false), [(false, 1, 0); 3]),
        ],
        MRelation::Equal,
    );
}

fn expected(
    settings: (ProcessPerimeterGenerator, bool, i32, f64, bool),
    records: [(bool, u8, u64); 3],
) -> Expected {
    let (generator, spiral, bottom, thickness, align) = settings;
    Expected {
        generator,
        spiral,
        bottom,
        thickness: thickness.to_bits(),
        align,
        column: [1.0_f64.to_bits(), 0],
        records,
    }
}

fn expected_with_column(
    generator: ProcessPerimeterGenerator,
    align: bool,
    column: [u64; 2],
    records: [(bool, u8, u64); 3],
) -> Expected {
    Expected {
        generator,
        spiral: false,
        bottom: 3,
        thickness: 0.0_f64.to_bits(),
        align,
        column,
        records,
    }
}

fn context_archive(
    generator: ProcessPerimeterGenerator,
    spiral: bool,
    bottom: i32,
    thickness: f64,
) -> ArchiveBuilder {
    let mut archive = ArchiveBuilder::new().three_layer_two_contour();
    if generator == ProcessPerimeterGenerator::Arachne {
        archive.replace_unique(PROCESS, CLASSIC, ARACHNE);
    }
    if spiral {
        archive.replace_unique(PROCESS, SPIRAL_OFF, SPIRAL_ON);
    }
    if bottom != 3 {
        archive.replace_unique(
            PROCESS,
            BOTTOM_THREE,
            &format!(r#""bottom_shell_layers": "{bottom}""#),
        );
    }
    if thickness != 0.0 {
        archive.replace_unique(
            PROCESS,
            THICKNESS_ZERO,
            &format!(r#""bottom_shell_thickness": "{thickness}""#),
        );
    }
    archive
}

fn assert_pair(
    name: &str,
    base: ArchiveBuilder,
    edit: (&str, &str, &str),
    expected: [Expected; 2],
    relation: MRelation,
) {
    let (path, from, to) = edit;
    let [before, after] = expected;
    let (before_bytes, after_bytes) = replacement_pair(base, path, from, to);
    assert_loaded_context(&before_bytes, before);
    assert_loaded_context(&after_bytes, after);
    let before_m = task22n_browser_input_oracle(&before_bytes).unwrap();
    let after_m = task22n_browser_input_oracle(&after_bytes).unwrap();
    assert_m_relation(name, &before_m, &after_m, relation);
    let before_n = task22n_browser_oracle(&before_bytes).unwrap();
    let after_n = task22n_browser_oracle(&after_bytes).unwrap();
    assert_eq!(&before_n[16..16 + before_m.len()], before_m);
    assert_eq!(&after_n[16..16 + after_m.len()], after_m);
    let before_frame = parse_n(&before_n).unwrap();
    let after_frame = parse_n(&after_n).unwrap();
    assert_contexts(&before_frame, before.records);
    assert_contexts(&after_frame, after.records);
    super::assert_n_geometry_matches_predecessor(&before_frame);
    super::assert_n_geometry_matches_predecessor(&after_frame);
    assert_noncontext_fields(&before_frame, &after_frame);
}

fn assert_m_relation(name: &str, before: &[u8], after: &[u8], relation: MRelation) {
    match relation {
        MRelation::Delta(expected) => {
            for (actual, expected) in [(before, expected[0]), (after, expected[1])] {
                assert_eq!(actual.len(), expected.len, "{name} M length");
                assert_eq!(sha256(actual), expected.sha, "{name} M SHA-256");
            }
            assert_ne!(before, after, "{name} M delta");
        }
        MRelation::Equal => {
            assert_eq!(before, after, "{name} M equality");
        }
    }
}

fn replacement_pair(
    mut base: ArchiveBuilder,
    path: &str,
    from: &str,
    to: &str,
) -> (Vec<u8>, Vec<u8>) {
    let before = base.clone().bytes();
    base.replace_unique(path, from, to);
    let after = base.bytes();
    assert_single_entry_replacement(&before, &after, path, from, to);
    assert_ne!(semantic_identity(&before), semantic_identity(&after));
    (before, after)
}

fn assert_loaded_context(bytes: &[u8], expected: Expected) {
    let project = load_project(bytes).unwrap();
    let raw = project.settings();
    assert_eq!(
        (
            raw.process.object.wall_generator,
            raw.process.print.spiral_mode,
            raw.process.region.bottom_shell_layers,
            raw.process.region.bottom_shell_thickness.0.to_bits(),
            raw.process.region.align_infill_direction_to_model
        ),
        (
            expected.generator,
            OrcaBool(expected.spiral),
            OrcaInt(expected.bottom),
            expected.thickness,
            OrcaBool(expected.align)
        )
    );
    let raw_column = project.objects()[0].instances()[0]
        .transform()
        .first_xy_column();
    assert_eq!(
        [raw_column.0.to_bits(), raw_column.1.to_bits()],
        expected.column
    );
    let resolved = resolve_bounded_project_config(&project).unwrap();
    let object = &resolved.objects[0];
    let region = &object.layer_candidates[0].model_parts[0].region;
    assert_eq!(
        (
            object.object.wall_generator,
            resolved.views.full.process.print.spiral_mode,
            region.bottom_shell_layers,
            region.bottom_shell_thickness.0.to_bits(),
            region.align_infill_direction_to_model
        ),
        (
            expected.generator,
            OrcaBool(expected.spiral),
            OrcaInt(expected.bottom),
            expected.thickness,
            OrcaBool(expected.align)
        )
    );
    let column = object.print_objects[0].transform.first_xy_column();
    assert_eq!([column.0.to_bits(), column.1.to_bits()], expected.column);
}

fn assert_contexts(frame: &NFrame, expected: [(bool, u8, u64); 3]) {
    let [object] = frame.objects.as_slice() else {
        panic!("one context object")
    };
    assert_eq!(object.slots.len(), 3);
    for (slot, expected) in object.slots.iter().zip(expected) {
        let record = slot.as_ref().unwrap();
        assert_eq!((record.spiral, record.dispatch, record.rotation), expected);
    }
}

fn assert_noncontext_fields(before: &NFrame, after: &NFrame) {
    assert_eq!(before.objects.len(), after.objects.len());
    for (before, after) in before.objects.iter().zip(&after.objects) {
        assert_eq!(
            (
                [before.source, before.transform, before.planned],
                before.slots.len()
            ),
            (
                [after.source, after.transform, after.planned],
                after.slots.len()
            )
        );
        for (before, after) in before.slots.iter().zip(&after.slots) {
            match (before, after) {
                (Some(before), Some(after)) => super::assert_noncontext_record(before, after),
                (None, None) => {}
                _ => panic!("matching record presence"),
            }
        }
    }
}
