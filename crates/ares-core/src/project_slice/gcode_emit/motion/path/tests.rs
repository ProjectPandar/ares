use super::{MotionOptions, begin_layer, path};
use crate::geometry::CoordinateScale;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerBegin, LayerGeometry, features::PathProperties, state::AvoidCrossingGeometry,
};

/// `GCode.cpp:6469-6471` builds `_mm3_per_mm = path.mm3_per_mm *
/// print_flow_ratio * filament_flow_ratio`, and the volumetric speed cap
/// (`GCode.cpp:6615-6617`) divides `filament_max_volumetric_speed` by that
/// fully-multiplied value, so both flow ratios cap the speed.
#[test]
fn volumetric_speed_cap_divides_by_print_and_filament_flow_ratios() {
    let mut state = EmitState {
        options: MotionOptions {
            inner_wall_speed: 200.0,
            max_volumetric_speed: 4.0,
            print_flow_ratio: 0.5,
            filament_flow_ratio: 0.98,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    // `advance_for_layer` owns the layer index advance
    // (`GCode.cpp:5690-5696`); the tests set it directly.
    state.layer_index = 1;
    begin_layer(
        &mut Vec::new(),
        &mut state,
        LayerBegin {
            layer_z: 0.4,
            layer_height: 0.2,
            first_layer: false,
            second_layer_reset: false,
        },
    );
    let mut output = Vec::new();

    path::emit(
        &mut output,
        [(0, 0), (1_000_000, 0)].into_iter(),
        PathProperties {
            contour: None,
            mm3_per_mm: 0.08,
            width: 0.45,
            height: 0.2,
            feature: "Inner wall",
            is_perimeter: true,
            end_clip: 0.0,
            fitting: &[],
            slope: None,
        },
        test_geometry(),
        &mut state,
    )
    .unwrap();

    let expected = 4.0 / (0.08 * 0.5 * 0.98) * 60.0;
    assert!(
        (state.extrusion_feedrate - expected).abs() < 1e-9,
        "feedrate {} vs upstream _mm3_per_mm cap {}",
        state.extrusion_feedrate,
        expected
    );
    let output = String::from_utf8(output).unwrap();
    assert!(
        output.contains("G1 F6122.449;_EXTRUDE_SET_SPEED"),
        "speed line must carry the capped feedrate: {output}"
    );
}

/// `GCode.cpp:6615` only applies the volumetric cap while
/// `filament_max_volumetric_speed > 0`; zero leaves the role speed alone.
#[test]
fn zero_max_volumetric_speed_leaves_the_role_speed_uncapped() {
    let mut state = EmitState {
        options: MotionOptions {
            inner_wall_speed: 200.0,
            max_volumetric_speed: 0.0,
            print_flow_ratio: 0.5,
            filament_flow_ratio: 0.98,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    // `advance_for_layer` owns the layer index advance
    // (`GCode.cpp:5690-5696`); the tests set it directly.
    state.layer_index = 1;
    begin_layer(
        &mut Vec::new(),
        &mut state,
        LayerBegin {
            layer_z: 0.4,
            layer_height: 0.2,
            first_layer: false,
            second_layer_reset: false,
        },
    );
    let mut output = Vec::new();

    path::emit(
        &mut output,
        [(0, 0), (1_000_000, 0)].into_iter(),
        PathProperties {
            contour: None,
            mm3_per_mm: 0.08,
            width: 0.45,
            height: 0.2,
            feature: "Inner wall",
            is_perimeter: true,
            end_clip: 0.0,
            fitting: &[],
            slope: None,
        },
        test_geometry(),
        &mut state,
    )
    .unwrap();

    assert!(
        (state.extrusion_feedrate - 200.0 * 60.0).abs() < 1e-9,
        "feedrate {} must keep the uncapped role speed",
        state.extrusion_feedrate
    );
}

fn test_geometry() -> LayerGeometry<'static> {
    LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: &[],
        support_islands: None,
        scale: CoordinateScale::Normal,
        previous_curled: &[],
        previous_layer_boundary: None,
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.42,
            layer_slices: &[],
            perimeter_spacing: 0.0,
            top_surfaces: &[],
            chunk_slices: &[],
            chunk_perimeter_spacing: 0.0,
        },
    }
}

#[test]
fn plate_loops_unscale_after_integer_translation() {
    for feature in ["Skirt", "Brim"] {
        for (extruder_y, expected_y) in [(0.0, "98.188"), (2.0, "96.188")] {
            let mut state = EmitState {
                origin: (110.0, 110.0),
                offset: (110.0, 110.0 - extruder_y),
                extruder_offset: (0.0, extruder_y),
                // The emission path multiplies the brim role flow by the
                // current object ratio (`GCode.cpp:6479-6480`, applied from
                // the object config at `apply_object_config` with the
                // default `brim_flow_ratio: 1.0`); a bare `EmitState`
                // default carries 0.0 and would zero out brim extrusion.
                current_brim_flow_ratio: 1.0,
                scale_factor: 1e-6,
                options: MotionOptions {
                    filament_flow_ratio: 1.0,
                    print_flow_ratio: 1.0,
                    filament_area: 1.0,
                    use_relative_e_distances: true,
                    ..MotionOptions::default()
                },
                ..EmitState::default()
            };
            let mut output = Vec::new();
            path::emit(
                &mut output,
                [(8_000_000, -11_811_500), (8_550_000, -11_811_500)].into_iter(),
                PathProperties {
                    contour: None,
                    mm3_per_mm: 0.08,
                    width: 0.45,
                    height: 0.2,
                    feature,
                    is_perimeter: false,
                    end_clip: 0.0,
                    fitting: &[],
                    slope: None,
                },
                test_geometry(),
                &mut state,
            )
            .unwrap();
            let text = String::from_utf8(output).unwrap();
            assert!(
                text.contains(&format!("G1 X118 Y{expected_y} ")),
                "{feature}: {text}"
            );
            assert!(
                text.contains(&format!("G1 X118.55 Y{expected_y} E")),
                "{feature}: {text}"
            );
            assert_eq!(state.last_scaled_position, (8_550_000, -11_811_500));
            assert!(state.last_position_defined);
            assert_eq!(state.origin, (110.0, 110.0));
            assert_eq!(state.wipe_path.len(), 2);
            for (point, expected_x) in state.wipe_path.iter().zip([8.55, 8.0]) {
                assert!((point.x - expected_x).abs() < 1e-12);
                assert!((point.y + 11.8115).abs() < 1e-12);
            }
        }
    }
}

#[test]
fn wipe_clipping_uses_the_destination_origin_frame() {
    for (source, destination, expected_e) in [
        ("Inner wall", "Brim", "-.70072"),
        ("Brim", "Skirt", "-.70072"),
        ("Skirt", "Inner wall", "-.70071"),
    ] {
        let mut state = EmitState {
            origin: (127.5, 127.5),
            offset: (127.5, 125.5),
            extruder_offset: (0.0, 2.0),
            scale_factor: 1e-6,
            options: MotionOptions {
                filament_flow_ratio: 1.0,
                print_flow_ratio: 1.0,
                filament_area: 1.0,
                use_relative_e_distances: true,
                wipe: true,
                wipe_distance: 1.0,
                wipe_speed: 40.0,
                retraction_length: 0.8,
                retraction_during_wipe_speed: 40.0,
                retraction_during_wipe_distance: 1.0,
                ..MotionOptions::default()
            },
            ..EmitState::default()
        };
        let properties = PathProperties {
            contour: None,
            mm3_per_mm: 0.08,
            width: 0.45,
            height: 0.2,
            feature: source,
            is_perimeter: false,
            end_clip: 0.0,
            fitting: &[],
            slope: None,
        };
        path::emit(
            &mut Vec::new(),
            [
                (-5_489_965, -7_008_781),
                (-5_738_597, -6_933_761),
                (-6_462_748, -6_441_019),
            ]
            .into_iter(),
            properties,
            test_geometry(),
            &mut state,
        )
        .unwrap();
        let mut output = Vec::new();
        path::emit(
            &mut output,
            [(-9_000_000, -9_000_000), (-8_000_000, -9_000_000)].into_iter(),
            PathProperties {
                contour: None,
                feature: destination,
                ..properties
            },
            test_geometry(),
            &mut state,
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(
            text.contains(&format!("G1 X121.761 Y118.566 E{expected_e}\n")),
            "{source} -> {destination}: {text}"
        );
        assert_eq!(state.last_scaled_position, (-8_000_000, -9_000_000));
        assert!(state.last_position_defined);
    }
}
