use crate::{
    ExtrusionRole,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon, ThickPolyline},
    project_slice::{
        fill_entities::{FillExtrusionEntity, LayerFillEntities},
        group_fills::{RepresentativeSurface, SurfaceFill, SurfaceFillParams, SurfaceFillPattern},
        perimeters::types::Flow,
        region_slices::RegionSurfaceKind,
    },
};

use super::{append, finalize_polylines, generate_thick_polylines, intersect_no_overlap_domains};

mod source_cases;

#[test]
fn task22o200_concentric_internal_generates_positive_variable_width_loops() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let expolygon = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(scaled(10.0), 0),
            Point::new(scaled(10.0), scaled(10.0)),
            Point::new(0, scaled(10.0)),
        ]),
        Vec::new(),
    );

    let output = generate_thick_polylines(expolygon, scaled(0.4), scaled(0.2), 0.4, scale).unwrap();

    assert!(output.len() > 1);
    assert!(output.iter().all(|line| {
        line.points.len() >= 2
            && line.width.len() == 2 * (line.points.len() - 1)
            && line.width.iter().all(|width| *width > 0.0)
    }));
}

#[test]
fn task22o201_concentric_finalization_rotates_then_clips_closed_loop() {
    let mut polylines = vec![ThickPolyline {
        points: vec![
            Point::new(10, 10),
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
        ],
        width: vec![1.0; 6],
        endpoints: (false, false),
    }];

    finalize_polylines(&mut polylines, 0, 5.0);

    assert_eq!(polylines.len(), 1);
    assert_eq!(polylines[0].points[0], Point::new(0, 0));
    assert_ne!(polylines[0].points.first(), polylines[0].points.last());
}

#[test]
fn task22o202_fill_expolygon_restricts_larger_no_overlap_domain() {
    let rectangle = |minimum, maximum| {
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(minimum, minimum),
                Point::new(maximum, minimum),
                Point::new(maximum, maximum),
                Point::new(minimum, maximum),
            ]),
            Vec::new(),
        )
    };
    let no_overlap = rectangle(0, 1_000);
    let fill = rectangle(400, 600);

    let domains = intersect_no_overlap_domains(&[no_overlap], &fill).unwrap();

    assert_eq!(domains.len(), 1);
    assert_eq!(domains[0].area().abs(), 48_400.0);
}

#[test]
fn task22o203_concentric_arachne_matches_source_narrow_branch() {
    let domain = ExPolygon::new(
        Polygon::new(vec![
            Point::new(4_649_887, -18_239_647),
            Point::new(4_881_733, -18_114_179),
            Point::new(5_418_338, -17_855_755),
            Point::new(6_234_522, -17_595_579),
            Point::new(7_073_916, -17_465_343),
            Point::new(7_358_779, -17_465_343),
            Point::new(4_048_542, -14_155_107),
            Point::new(4_048_542, -15_848_537),
            Point::new(2_009_079, -15_848_537),
            Point::new(4_503_846, -18_343_304),
        ]),
        Vec::new(),
    );
    let mut output =
        generate_thick_polylines(domain, 377_079, 200_000, 0.4, CoordinateScale::Normal).unwrap();
    finalize_polylines(&mut output, 0, 40_000.0);
    assert!(output.iter().any(|line| {
        line.points.windows(3).any(|points| {
            points
                == [
                    Point::new(5_331_706, -16_841_754),
                    Point::new(4_853_308, -17_037_563),
                    Point::new(4_673_461, -17_125_907),
                ]
        })
    }));

    let actual = output
        .iter()
        .find(|line| line.points.contains(&Point::new(6_177_261, -17_415_944)))
        .expect("source target branch is generated");
    assert_eq!(
        actual.points,
        vec![
            Point::new(4_237_081, -14_610_281),
            Point::new(6_924_526, -17_297_725),
            Point::new(6_177_261, -17_415_944),
            Point::new(5_336_532, -17_685_887),
            Point::new(4_783_982, -17_952_958),
            Point::new(4_532_575, -18_105_397),
            Point::new(2_464_252, -16_037_076),
            Point::new(4_048_542, -16_037_076),
            Point::new(4_178_211, -15_985_406),
            Point::new(4_237_081, -15_848_537),
            Point::new(4_237_081, -14_650_281),
        ]
    );
}

#[test]
fn task22o205_concentric_arachne_matches_source_fractional_vertex() {
    let domain = ExPolygon::new(
        Polygon::new(vec![
            Point::new(2_690_706, -20_263_054),
            Point::new(-1_602_493, -15_969_855),
            Point::new(-1_469_177, -15_836_537),
            Point::new(-862_793, -15_909_651),
            Point::new(-831_312, -15_909_411),
            Point::new(-1_519_521, -15_813_200),
            Point::new(-2_495_958, -15_601_912),
            Point::new(-3_040_178, -15_548_538),
            Point::new(-7_889_784, -15_548_538),
            Point::new(-5_249_484, -18_188_838),
            Point::new(-4_583_039, -17_856_194),
            Point::new(-3_765_477, -17_595_579),
            Point::new(-2_926_077, -17_465_343),
            Point::new(-2_073_923, -17_465_343),
            Point::new(-1_234_523, -17_595_579),
            Point::new(-416_961, -17_856_194),
            Point::new(347_353, -18_237_688),
            Point::new(1_041_851, -18_730_630),
            Point::new(1_480_065, -19_137_235),
            Point::new(1_657_323, -19_329_792),
            Point::new(2_027_204, -19_793_598),
            Point::new(2_170_378, -20_012_730),
            Point::new(2_499_256, -20_582_360),
        ]),
        Vec::new(),
    );

    let output =
        generate_thick_polylines(domain, 377_079, 200_000, 0.4, CoordinateScale::Normal).unwrap();
    let target = Point::new(-3_822_737, -17_415_945);
    let (line, index) = output
        .iter()
        .find_map(|line| {
            line.points
                .iter()
                .position(|point| *point == target)
                .map(|index| (line, index))
        })
        .expect("source fractional Voronoi vertex is generated");

    assert_eq!(line.width[2 * index - 1], 377_078.0);
    assert_eq!(line.width[2 * index], 377_078.0);
}

#[test]
fn task22o204_concentric_reorders_each_fill_expolygon_independently() {
    let scale = CoordinateScale::Normal;
    let scaled = |value| scale.checked_scale(value).unwrap();
    let rectangle = |minimum_x: f64, maximum_x: f64| {
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(scaled(minimum_x), 0),
                Point::new(scaled(maximum_x), 0),
                Point::new(scaled(maximum_x), scaled(4.0)),
                Point::new(scaled(minimum_x), scaled(4.0)),
            ]),
            Vec::new(),
        )
    };
    let right = rectangle(20.0, 24.0);
    let left = rectangle(-24.0, -20.0);
    let no_overlap = vec![right.clone(), left.clone()];
    let make_fill = |expolygons| SurfaceFill {
        region_id: 0,
        representative: RepresentativeSurface {
            kind: RegionSurfaceKind::InternalSolid,
            thickness: 0.2,
            thickness_layers: 1,
            bridge_angle: 0.0,
            extra_perimeters: 0,
        },
        expolygons,
        params: SurfaceFillParams {
            extruder: 1,
            pattern: SurfaceFillPattern::ConcentricInternal,
            spacing: 0.4,
            overlap: 0.0,
            angle: 0.0,
            fixed_angle: false,
            bridge: false,
            bridge_angle: 0.0,
            density: 100.0,
            multiline: 0,
            anchor_length: 0.0,
            anchor_length_max: 0.0,
            flow: Flow {
                width: 0.4,
                height: 0.2,
                spacing: 0.357,
                nozzle_diameter: 0.4,
                bridge: false,
                mm3_per_mm: 0.08,
            },
            extrusion_role: ExtrusionRole::SolidInfill,
            idx: 0,
            loop_clipping: scaled(0.04),
            role_speed: 0.0,
            lateral_lattice_angle_1: 0.0,
            lateral_lattice_angle_2: 0.0,
            infill_lock_depth: 0.0,
            skin_infill_depth: 0.0,
            symmetric_infill_y_axis: false,
            infill_overhang_angle: 0.0,
            gyroid_optimized: false,
            filter_out_gap_fill: 0.0,
            gap_fill_target: crate::ProcessGapFillTarget::Nowhere,
        },
        region_id_group: vec![0],
        no_overlap_expolygons: no_overlap.clone(),
    };

    let mut actual = LayerFillEntities::default();
    append(
        &mut actual,
        make_fill(vec![right.clone(), left.clone()]),
        0.4,
        scale,
    )
    .unwrap();

    let mut expected = LayerFillEntities::default();
    append(&mut expected, make_fill(vec![right]), 0.4, scale).unwrap();
    append(&mut expected, make_fill(vec![left]), 0.4, scale).unwrap();

    assert_eq!(actual.collections, expected.collections);
    assert_eq!(actual.collections.len(), 2);
    assert!(actual.collections.iter().all(|collection| {
        collection.no_sort
            && !collection.entities.is_empty()
            && collection
                .entities
                .iter()
                .all(|entity| matches!(entity, FillExtrusionEntity::VariableWidth(_)))
    }));
    assert!(actual.thin_fills.is_empty());
}

#[test]
fn concentric_arachne_matches_source_single_bead_peak_filtering() {
    let domain = ExPolygon::new(
        Polygon::new(vec![
            Point::new(-14_309_044, -27_946_472),
            Point::new(-14_069_596, -27_884_760),
            Point::new(-13_982_065, -27_736_448),
            Point::new(-14_019_779, -27_590_110),
            Point::new(-14_448_804, -27_442_826),
            Point::new(-15_153_489, -27_061_468),
            Point::new(-15_785_790, -26_569_326),
            Point::new(-16_129_292, -26_196_184),
            Point::new(-16_201_104, -26_201_956),
            Point::new(-16_204_118, -26_202_966),
            Point::new(-14_354_065, -28_053_020),
        ]),
        Vec::new(),
    );
    let mut output =
        generate_thick_polylines(domain, 377_079, 200_000, 0.4, CoordinateScale::Normal).unwrap();
    finalize_polylines(&mut output, 0, 40_000.0);

    let actual = output
        .iter()
        .find(|line| line.points.contains(&Point::new(-15_409_326, -26_926_338)))
        .expect("source target branch is generated");
    assert_eq!(
        actual,
        &ThickPolyline {
            points: vec![
                Point::new(-15_409_326, -26_926_338),
                Point::new(-14_537_384, -27_606_503),
                Point::new(-14_392_852, -27_697_968),
            ],
            width: vec![340_000.0, 372_218.0, 372_218.0, 464_814.0],
            endpoints: (false, false),
        },
    );
}
