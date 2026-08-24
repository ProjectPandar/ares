use std::f64::consts::PI;

use crate::geometry::CoordinateScale;

use super::{
    base::{Beading, BeadingStrategy, BeadingStrategyConfig},
    distributed::DistributedBeadingStrategy,
    factory::{BeadingStrategyFactoryConfig, make_strategy},
    limited::LimitedBeadingStrategy,
    outer_inset::OuterWallInsetBeadingStrategy,
    redistribute::RedistributeBeadingStrategy,
    widening::WideningBeadingStrategy,
};

fn config(optimal_width: i64) -> BeadingStrategyConfig {
    BeadingStrategyConfig {
        optimal_width,
        wall_split_middle_threshold: 0.6,
        wall_add_middle_threshold: 0.4,
        default_transition_length: 1_000,
        transitioning_angle: (PI / 3.0) as f32 as f64,
        coordinate_scale: CoordinateScale::Normal,
    }
}

fn distributed(optimal_width: i64) -> DistributedBeadingStrategy {
    DistributedBeadingStrategy::new(config(optimal_width), 2)
}

#[test]
fn task22o99_distributed_handles_zero_one_two_and_weighted_many_beads() {
    assert_eq!(distributed(100).compute(90, 0), Beading::empty(90));
    assert_eq!(
        distributed(100).compute(91, 1),
        Beading {
            total_thickness: 91,
            bead_widths: vec![91],
            toolpath_locations: vec![45],
            left_over: 0,
        }
    );
    assert_eq!(
        distributed(100).compute(200, 2),
        Beading {
            total_thickness: 200,
            bead_widths: vec![100, 100],
            toolpath_locations: vec![50, 150],
            left_over: 0,
        }
    );
    assert_eq!(
        distributed(100).compute(330, 3),
        Beading {
            total_thickness: 330,
            bead_widths: vec![100, 130, 100],
            toolpath_locations: vec![50, 165, 280],
            left_over: 0,
        }
    );
}

#[test]
fn task22o99_distributed_preserves_f32_weight_and_integer_width_rounding() {
    let strategy = DistributedBeadingStrategy::new(config(100), 3);
    assert_eq!(
        strategy.compute(450, 4),
        Beading {
            total_thickness: 450,
            bead_widths: vec![107, 117, 117, 109],
            toolpath_locations: vec![53, 165, 282, 395],
            left_over: 0,
        }
    );
}

#[test]
fn task22o99_thresholds_transition_lengths_and_anchors_follow_odd_even_source_rules() {
    let strategy = distributed(100);
    for (thickness, expected) in [(139, 1), (160, 2), (239, 2), (240, 3)] {
        assert_eq!(strategy.optimal_bead_count(thickness), expected);
    }

    assert_eq!(strategy.transition_thickness(0), 40);
    assert_eq!(strategy.transition_thickness(1), 160);
    assert_eq!(strategy.transition_thickness(2), 240);
    assert_eq!(strategy.transitioning_length(0), 10_000);
    assert_eq!(strategy.transitioning_length(1), 1_000);
    assert_eq!(strategy.transition_anchor_pos(0).to_bits(), 0x3f19_999a);
    assert_eq!(strategy.transition_anchor_pos(1).to_bits(), 0x3ecc_cccc);
    assert_eq!(
        strategy.transitioning_angle().to_bits(),
        ((PI / 3.0) as f32 as f64).to_bits()
    );
    assert!(strategy.nonlinear_thicknesses(2).is_empty());

    let mut large_bed_config = config(100);
    large_bed_config.coordinate_scale = CoordinateScale::LargeBed;
    assert_eq!(
        DistributedBeadingStrategy::new(large_bed_config, 2).transitioning_length(0),
        999
    );
}

#[test]
fn task22o99_transition_thickness_casts_after_the_complete_f64_sum() {
    let mut negative = config(1);
    negative.wall_add_middle_threshold = 0.4;
    assert_eq!(
        DistributedBeadingStrategy::new(negative, 2).transition_thickness(-1),
        0
    );

    let mut large = config((1_i64 << 53) + 1);
    large.wall_split_middle_threshold = 0.5;
    assert_eq!(
        DistributedBeadingStrategy::new(large, 2).transition_thickness(1),
        13_510_798_882_111_488
    );
}

#[test]
fn task22o99_redistribution_keeps_outer_widths_and_delegates_inner_thickness() {
    let strategy = RedistributeBeadingStrategy::new(100, 0.5, Box::new(distributed(120)));
    assert_eq!(strategy.optimal_thickness(0), 0);
    assert_eq!(strategy.optimal_thickness(1), 100);
    assert_eq!(strategy.optimal_thickness(2), 200);
    assert_eq!(strategy.optimal_thickness(3), 320);
    assert_eq!(strategy.optimal_thickness(4), 440);
    assert_eq!(strategy.transition_thickness(0), 50);
    assert_eq!(strategy.transition_thickness(1), 160);
    assert_eq!(strategy.transition_thickness(2), 248);
    for (thickness, expected) in [(49, 0), (50, 1), (160, 1), (161, 2), (200, 2)] {
        assert_eq!(strategy.optimal_bead_count(thickness), expected);
    }
    assert_eq!(
        strategy.compute(500, 3),
        Beading {
            total_thickness: 500,
            bead_widths: vec![100, 300, 100],
            toolpath_locations: vec![50, 250, 450],
            left_over: 0,
        }
    );

    let fractional = RedistributeBeadingStrategy::new(100, 0.505, Box::new(distributed(120)));
    assert_eq!(fractional.compute(50, 1), Beading::empty(50));
    assert_eq!(fractional.compute(51, 1).bead_widths, vec![51]);
}

#[test]
fn task22o99_widening_only_collapses_at_most_one_requested_bead() {
    let strategy = WideningBeadingStrategy::new(
        Box::new(RedistributeBeadingStrategy::new(
            100,
            0.5,
            Box::new(distributed(100)),
        )),
        30,
        80,
    );
    assert_eq!(strategy.compute(20, 1), Beading::empty(20));
    assert_eq!(
        strategy.compute(50, 1),
        Beading {
            total_thickness: 50,
            bead_widths: vec![80],
            toolpath_locations: vec![25],
            left_over: 0,
        }
    );
    assert_eq!(strategy.compute(159, 2).bead_widths, vec![79, 79]);
    assert_eq!(strategy.optimal_bead_count(20), 0);
    assert_eq!(strategy.optimal_bead_count(30), 1);
    assert_eq!(strategy.transition_thickness(0), 30);
    assert_eq!(strategy.nonlinear_thicknesses(1), vec![80]);
}

#[test]
fn task22o99_limited_inserts_zero_width_boundary_markers_symmetrically() {
    let strategy = LimitedBeadingStrategy::new(4, Box::new(distributed(100)));
    assert_eq!(
        strategy.compute(400, 4),
        Beading {
            total_thickness: 400,
            bead_widths: vec![100, 100, 0, 100, 100],
            toolpath_locations: vec![50, 150, 200, 250, 350],
            left_over: 0,
        }
    );
    assert_eq!(
        strategy.compute(550, 5),
        Beading {
            total_thickness: 550,
            bead_widths: vec![100, 100, 0, 0, 100, 100],
            toolpath_locations: vec![50, 150, 200, 350, 400, 500],
            left_over: 150,
        }
    );
}

#[test]
fn task22o99_limited_caps_counts_at_the_source_ten_micron_threshold() {
    let strategy = LimitedBeadingStrategy::new(4, Box::new(distributed(100_000)));
    assert_eq!(strategy.transition_thickness(4), 490_000);
    assert_eq!(strategy.optimal_bead_count(489_999), 4);
    assert_eq!(strategy.optimal_bead_count(490_000), 5);
}

#[test]
fn task22o99_outer_inset_is_optional_capped_and_skips_single_beads() {
    let positive = OuterWallInsetBeadingStrategy::new(80, Box::new(distributed(100)));
    assert_eq!(positive.compute(200, 2).toolpath_locations, vec![100, 150]);
    assert_eq!(positive.compute(90, 1).toolpath_locations, vec![45]);

    let negative = OuterWallInsetBeadingStrategy::new(-20, Box::new(distributed(100)));
    assert_eq!(negative.compute(200, 2).toolpath_locations, vec![30, 150]);
}

#[test]
fn task22o99_ksr_style_factory_builds_the_exact_ordered_meta_strategy_stack() {
    let spacing = 420_000_f32 * 0.000_001_f32;
    let layer_height = 0.2_f32;
    let extrusion_width = (spacing as f64 + layer_height as f64 * (1.0 - 0.25 * PI)) as f32;
    let min_bead_width_mm = 340_000_f64 * 0.000_001_f64;
    let factory_config = BeadingStrategyFactoryConfig {
        preferred_bead_width_outer: 420_000,
        preferred_bead_width_inner: 420_000,
        preferred_transition_length: 400_000,
        transitioning_angle: (PI / 3.0) as f32,
        print_thin_walls: true,
        min_bead_width: 340_000,
        min_feature_size: 100_000,
        wall_split_middle_threshold: (2.0 * min_bead_width_mm / extrusion_width as f64 - 1.0)
            .clamp(0.01, 0.99),
        wall_add_middle_threshold: (min_bead_width_mm / extrusion_width as f64).clamp(0.01, 0.99),
        max_bead_count: 4,
        outer_wall_offset: 0,
        inward_distributed_center_wall_count: 2,
        minimum_variable_line_ratio: 0.5,
        coordinate_scale: CoordinateScale::Normal,
    };
    let strategy = make_strategy(factory_config);

    assert_eq!(
        strategy.description(),
        "LimitedBeadingStrategy+Widening+RedistributeBeadingStrategy+DistributedBeadingStrategy"
    );
    assert_eq!(strategy.optimal_thickness(4), 1_680_000);
    assert_eq!(strategy.transitioning_length(0), 10_000);
    assert_eq!(strategy.transitioning_length(1), 400_000);
    assert_eq!(strategy.optimal_bead_count(99_999), 0);
    assert_eq!(strategy.optimal_bead_count(100_000), 1);
    assert_eq!(strategy.compute(150_000, 1).bead_widths, vec![340_000]);
    assert_eq!(
        strategy.compute(1_680_000, 4).bead_widths,
        vec![420_000, 420_000, 0, 420_000, 420_000]
    );
    assert!(strategy.nonlinear_thicknesses(1).is_empty());

    let inset_strategy = make_strategy(BeadingStrategyFactoryConfig {
        outer_wall_offset: 10_000,
        ..factory_config
    });
    assert_eq!(
        inset_strategy.description(),
        "LimitedBeadingStrategy+OuterWallOfsetBeadingStrategy+Widening+RedistributeBeadingStrategy+DistributedBeadingStrategy"
    );
    assert!(inset_strategy.nonlinear_thicknesses(1).is_empty());
}
