use crate::{FloatOrPercent, OrcaFloat, OrcaInt, Percent, SliceError, slice_project};

use super::super::super::super::{prepare_post_conical_overhang, task22m_browser_oracle};
use super::super::super::region_fixture::{checkpoint as region_checkpoint, modifier_projects};
use super::super::super::support::metadata;
use super::{
    ENABLED_CONTOUR, NOZZLES_04_04, NOZZLES_04_06, PROCESS, RAW_CONTOUR, archive_entries,
    checkpoint::{expolygon, parse_m, surface_geometry},
    small_archive_source,
};

type Identity = (usize, &'static str);

const SELECTOR_TWO_ZIP: Identity = (
    181_494,
    "cddacf24d7967f2b722cf981b8b8620d7d4a86d0dcc1bfda22a4e4b6358faa72",
);
const LAYERS_TWO_ZIP: Identity = (
    181_493,
    "b974281992a55a59f9afbdb2ee886de1a8de6e731b759933f326cac8531a2c60",
);
const SELECTOR_TWO_SEMANTIC: Identity = (
    1_020_601,
    "f567c2819de934e82945d01d6529fcc47ffb499f577cc21ff1c307324d9547c8",
);
const LAYERS_TWO_SEMANTIC: Identity = (
    1_020_600,
    "8ccef306de43175f28cf69ea02c2eada4c5fb31e6b7bf70ad9a0fd090617911f",
);
const XY_HOLE_ZIP: Identity = (
    181_497,
    "29a3d0a94a74f2975ba041591012df13469e7bb155fb811661c1d66359b955b8",
);
const XY_CONTOUR_ZIP: Identity = (
    181_497,
    "96d8aeb68cb7fb4ba6b2d248a837d0e40b74f2de1d81d0cf2c8a9b7cdd942a64",
);
const XY_HOLE_SEMANTIC: Identity = (
    1_020_602,
    "ba0e4c29738facb0df0cfa19320eebcc606d2e50fd9dc8740ea0c3a82e5f9b10",
);
const XY_CONTOUR_SEMANTIC: Identity = (
    1_020_603,
    "de3d6410ac067119cbb70ed21715122c621717106e7223661432fb234bf776dd",
);
const MODIFIER_ZIP: Identity = (
    56_046,
    "83ac43d83487ad5f63b7c4b8f8c88ef20bb75b286d09e329fe24c8abc08807ce",
);
const CONTROL_ZIP: Identity = (
    56_027,
    "4e1847cf020e217f9b90bef61cdb06c8fc2a953ca9dce100a161d3bcb99eca69",
);
const MODIFIER_SEMANTIC: Identity = (
    107_932,
    "82a7bdd3571da52daf92ec11a7a243ec279e9f053542804e2dfc1e10365d6fa3",
);
const CONTROL_SEMANTIC: Identity = (
    107_885,
    "e59b8041e64297f880e19ab42b51cbbac9f9394bd3f287ffe845edba595176e5",
);

const SELECTOR_TWO_REPLACEMENTS: [(&str, &str); 3] = [
    (
        r#""initial_layer_line_width": "0.5""#,
        r#""initial_layer_line_width": "125%""#,
    ),
    (NOZZLES_04_04, NOZZLES_04_06),
    (
        r#""outer_wall_filament_id": "0""#,
        r#""outer_wall_filament_id": "2""#,
    ),
];
const LAYERS_TWO_REPLACEMENTS: [(&str, &str); 1] = [(
    r#""elefant_foot_compensation_layers": "1""#,
    r#""elefant_foot_compensation_layers": "2""#,
)];

#[derive(Clone, Copy)]
enum ExactCase {
    SelectorTwo,
    LayersTwo,
}

impl ExactCase {
    fn replacements(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::SelectorTwo => &SELECTOR_TWO_REPLACEMENTS,
            Self::LayersTwo => &LAYERS_TWO_REPLACEMENTS,
        }
    }

    fn identities(self) -> (Identity, Identity) {
        match self {
            Self::SelectorTwo => (SELECTOR_TWO_ZIP, SELECTOR_TWO_SEMANTIC),
            Self::LayersTwo => (LAYERS_TWO_ZIP, LAYERS_TWO_SEMANTIC),
        }
    }
}

#[test]
fn task22m_real_3mf_loads_width_nozzle_and_layer_options() {
    let base = small_archive_source().bytes();
    for case in [ExactCase::SelectorTwo, ExactCase::LayersTwo] {
        let (zip, semantic) = case.identities();
        let archive = exact_archive(case);
        assert_eq!(exact_archive(case), archive);
        assert_frozen(&archive, zip);
        assert_semantic(&archive, semantic);
        assert_only_entry_replacements(&base, &archive, PROCESS, case.replacements());
        assert_loaded_exact_options(&archive, case);

        let m = task22m_browser_oracle(&archive).unwrap();
        assert_exact_m_geometry(&m, case);
    }
}

#[tokio::test]
async fn task22m_real_3mf_xy_and_region_count_reach_public_gates() {
    const HOLE: (&str, &str) = (
        r#""xy_hole_compensation": "0""#,
        r#""xy_hole_compensation": "0.1""#,
    );
    const CONTOUR: (&str, &str) = (
        r#""xy_contour_compensation": "0""#,
        r#""xy_contour_compensation": "-0.1""#,
    );
    const REGION_PATH: &str = "Metadata/model_settings.config";
    const REGION: (&str, &str) = (
        r#"<part id="3" subtype="modifier_part"/>"#,
        r#"<part id="3" subtype="modifier_part"><metadata key="bridge_angle" value="37"/></part>"#,
    );

    let base = small_archive_source().bytes();
    let hole = replaced_archive(PROCESS, &[HOLE]);
    let contour = replaced_archive(PROCESS, &[CONTOUR]);
    assert_eq!(replaced_archive(PROCESS, &[HOLE]), hole);
    assert_eq!(replaced_archive(PROCESS, &[CONTOUR]), contour);
    let (modifier, control) = modifier_projects();
    assert_eq!(modifier_projects(), (modifier.clone(), control.clone()));

    let actual_archives = [&hole, &contour, &modifier, &control].map(|bytes| {
        (
            byte_identity(bytes),
            region_checkpoint::semantic_identity(bytes),
        )
    });
    let expected_archives = [
        (owned(XY_HOLE_ZIP), owned(XY_HOLE_SEMANTIC)),
        (owned(XY_CONTOUR_ZIP), owned(XY_CONTOUR_SEMANTIC)),
        (owned(MODIFIER_ZIP), owned(MODIFIER_SEMANTIC)),
        (owned(CONTROL_ZIP), owned(CONTROL_SEMANTIC)),
    ];
    assert_eq!(actual_archives, expected_archives);
    assert_only_entry_replacements(&base, &hole, PROCESS, &[HOLE]);
    assert_only_entry_replacements(&base, &contour, PROCESS, &[CONTOUR]);
    assert_only_entry_replacements(&control, &modifier, REGION_PATH, &[REGION]);

    for (archive, xy, key) in [
        (
            &hole,
            (OrcaFloat(0.1), OrcaFloat(0.0)),
            "xy_hole_compensation",
        ),
        (
            &contour,
            (OrcaFloat(0.0), OrcaFloat(-0.1)),
            "xy_contour_compensation",
        ),
    ] {
        let prepared = prepare_post_conical_overhang(archive).unwrap();
        let object = &prepared.resolved.objects[0].object;
        assert_eq!(
            (object.xy_hole_compensation, object.xy_contour_compensation),
            xy
        );
        assert_eq!(task22m_browser_oracle(archive), unsupported(key));
        assert_eq!(slice_project(archive, metadata()).await, unsupported(key));
    }

    let region_options = |archive: &[u8]| {
        prepare_post_conical_overhang(archive).unwrap().objects[0]
            .regions
            .iter()
            .map(|region| region.options.bridge_angle)
            .collect::<Vec<_>>()
    };
    assert_eq!(region_options(&control), [OrcaFloat(0.0)]);
    assert_eq!(region_options(&modifier), [OrcaFloat(0.0), OrcaFloat(37.0)]);
    assert_eq!(parse_m(&task22m_browser_oracle(&control).unwrap()).len(), 1);
    assert_eq!(
        task22m_browser_oracle(&modifier),
        unsupported("multi_region_layer_slices")
    );
    assert_eq!(
        slice_project(modifier, metadata()).await,
        unsupported("multi_region_layer_slices")
    );
}

fn replaced_archive(path: &str, replacements: &[(&str, &str)]) -> Vec<u8> {
    let mut archive = small_archive_source();
    for &(from, to) in replacements {
        archive.replace_unique(path, from, to);
    }
    archive.bytes()
}

fn unsupported(feature: &str) -> Result<Vec<u8>, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(feature.to_owned()))
}

fn exact_archive(case: ExactCase) -> Vec<u8> {
    replaced_archive(PROCESS, case.replacements())
}

fn assert_loaded_exact_options(archive: &[u8], case: ExactCase) {
    let prepared = prepare_post_conical_overhang(archive).unwrap();
    let object = &prepared.resolved.objects[0].object;
    let region = &prepared.objects[0].regions[0].options;
    let print = &prepared.resolved.views.full;
    let actual_nozzles = print
        .project
        .print
        .nozzle_diameter
        .0
        .iter()
        .map(|value| value.0)
        .collect::<Vec<_>>();
    let filament_map = print
        .project
        .gcode
        .filament_map
        .0
        .iter()
        .map(|value| value.0)
        .collect::<Vec<_>>();
    let (width, expected_nozzles, selector, layers) = match case {
        ExactCase::SelectorTwo => (
            FloatOrPercent::Percent(Percent(125.0)),
            vec![0.4, 0.6],
            OrcaInt(2),
            OrcaInt(1),
        ),
        ExactCase::LayersTwo => (
            FloatOrPercent::Float(0.5),
            vec![0.4, 0.4],
            OrcaInt(1),
            OrcaInt(2),
        ),
    };
    assert_eq!(
        (
            print.process.print.initial_layer_line_width,
            actual_nozzles,
            filament_map,
            region.outer_wall_filament_id,
            object.elefant_foot_compensation,
            object.elefant_foot_compensation_layers,
            object.xy_hole_compensation,
            object.xy_contour_compensation,
        ),
        (
            width,
            expected_nozzles,
            vec![1, 1],
            selector,
            OrcaFloat(0.15),
            layers,
            OrcaFloat(0.0),
            OrcaFloat(0.0),
        )
    );
}

fn assert_exact_m_geometry(bytes: &[u8], case: ExactCase) {
    let parsed = parse_m(bytes);
    assert_eq!(parsed.len(), 1);
    let (object, lslices) = &parsed[0];
    assert_eq!((object.retained_layers.len(), lslices.len()), (2, 2));
    let raw = expolygon(RAW_CONTOUR);
    let (first, second) = match case {
        ExactCase::SelectorTwo => (expolygon(SELECTOR_TWO_CONTOUR), raw.clone()),
        ExactCase::LayersTwo => (expolygon(ENABLED_CONTOUR), expolygon(SECOND_LAYER_CONTOUR)),
    };
    assert_eq!(
        surface_geometry(&object.retained_layers[0].regions),
        [first]
    );
    assert_eq!(
        surface_geometry(&object.retained_layers[1].regions),
        [second]
    );
    assert_eq!(lslices[0].as_slice(), std::slice::from_ref(&raw));
    assert_eq!(lslices[1], [raw]);
}

fn assert_frozen(bytes: &[u8], expected: Identity) {
    assert_eq!(byte_identity(bytes), owned(expected));
}

fn byte_identity(bytes: &[u8]) -> (usize, String) {
    (bytes.len(), region_checkpoint::sha256(bytes))
}

fn owned(identity: Identity) -> (usize, String) {
    (identity.0, identity.1.to_owned())
}

fn assert_semantic(bytes: &[u8], expected: Identity) {
    assert_eq!(
        region_checkpoint::semantic_identity(bytes),
        (expected.0, expected.1.to_owned())
    );
}

fn assert_only_entry_replacements(
    left: &[u8],
    right: &[u8],
    path: &str,
    replacements: &[(&str, &str)],
) {
    let mut left = archive_entries(left);
    let mut right = archive_entries(right);
    let mut expected = String::from_utf8(left.remove(path).unwrap()).unwrap();
    let actual = String::from_utf8(right.remove(path).unwrap()).unwrap();
    assert_eq!(left, right);
    for &(from, to) in replacements {
        assert_eq!(expected.match_indices(from).count(), 1);
        expected = expected.replacen(from, to, 1);
    }
    assert_eq!(expected, actual);
}

#[rustfmt::skip]
const SELECTOR_TWO_CONTOUR: &[(i64, i64)] = &[(3_850_000,-650_000),(2_542_857,-650_000),(2_057_142,-649_494),(1_571_428,-641_900),(1_085_714,-600_219),(600_000,-500_000),(600_000,3_500_000),(499_781,4_000_000),(470_135,4_355_635),(200_000,4_350_506),(-200_000,4_350_506),(-470_135,4_355_635),(-499_781,4_000_000),(-600_000,3_500_000),(-600_000,-500_000),(-1_085_714,-600_219),(-1_571_428,-641_900),(-2_057_142,-649_494),(-2_542_857,-650_000),(-3_850_000,-650_000),(-3_850_000,-4_350_000),(3_850_000,-4_350_000)];
#[rustfmt::skip]
const SECOND_LAYER_CONTOUR: &[(i64, i64)] = &[(3_925_000,-575_000),(525_000,-575_000),(525_000,4_425_000),(-525_000,4_425_000),(-525_000,-575_000),(-3_925_000,-575_000),(-3_925_000,-4_425_000),(3_925_000,-4_425_000)];
