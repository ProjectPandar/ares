//! Bed-exclusion regions and their arrange blockers, ported from OrcaSlicer's
//! `tests/libslic3r/test_exclusion_volumes.cpp` at v2.5.0-beta `345efec64`, through the public
//! `bed_exclusion` and `arrangement` APIs over a `DynamicPrintConfig` (registry defaults, then the
//! case's keys).

use ares_core::arrangement::{
    BoundingBox, ExPolygon, Point, Polygon, bed_exclusion_arrange_polygon,
};
use ares_core::bed_exclusion::{
    active_bed_exclude_volume_mode, get_bed_excluded_regions, get_bed_excluded_regions_by_extruder,
    has_bed_exclude_volumes, is_valid_bed_exclude_volumes_string,
};
use ares_core::{BedExcludeVolumeMode, DynamicPrintConfig};

/// `Point::new_scale`: `coord_t(scale_(v))`, truncated.
fn scaled(x: f64, y: f64) -> Point {
    Point::new((x / 1e-6) as i64, (y / 1e-6) as i64)
}

/// `rectangle` (`:30-40`), counter-clockwise as listed.
fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Polygon {
    Polygon::new(vec![
        scaled(min_x, min_y),
        scaled(max_x, min_y),
        scaled(max_x, max_y),
        scaled(min_x, max_y),
    ])
}

fn set(config: &mut DynamicPrintConfig, key: &str, value: &str) {
    config.set_deserialize_strict(key, value).unwrap();
}

/// `DynamicPrintConfig::set_num_extruders(2)` (`PrintConfig.cpp:10669-10686`) for the extruder
/// keys the resolution reads (`extruder_option_keys`, `:10229`): two copies of each default.
fn set_two_extruders(config: &mut DynamicPrintConfig) {
    set(config, "nozzle_diameter", "0.4,0.4");
    set(config, "extruder_offset", "0x0,0x0");
    set(config, "extruder_bed_exclude_volumes", "\"\";\"\"");
}

/// `two_extruder_config` (`:49-60`).
fn two_extruder_config() -> DynamicPrintConfig {
    let mut config = DynamicPrintConfig::full_print_config();
    set_two_extruders(&mut config);
    set(&mut config, "nozzle_diameter", "0.4,0.6");
    set(&mut config, "extruder_offset", "0x0,20x5");
    set(&mut config, "extruder_printable_height", "200,200");
    set(&mut config, "master_extruder_id", "1");
    set(&mut config, "printable_height", "200");
    set(
        &mut config,
        "bed_exclude_volumes",
        "10..30;0x0,10x0,10x10,0x10",
    );
    config
}

fn bounds(polygon: &Polygon) -> BoundingBox {
    BoundingBox::from_expolygon(&ExPolygon::new(polygon.clone(), Vec::new())).unwrap()
}

fn unscaled(value: i64) -> f64 {
    value as f64 * 1e-6
}

fn within(value: f64, target: f64, margin: f64) -> bool {
    (value - target).abs() <= margin
}

#[test]
fn bed_area_polygon_also_defines_a_full_height_collision_volume() {
    // `:178-187`
    let definition = "0x0,20x0,20x10,10x10,10x20,0x20";
    let mut config = DynamicPrintConfig::full_print_config();
    set(&mut config, "bed_exclude_area", definition);
    set(&mut config, "bed_exclude_volumes", definition);
    let regions = get_bed_excluded_regions(&config);
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].polygon.points(), regions[1].polygon.points());
}

#[test]
fn exclusion_z_ranges_default_and_clamp_to_printable_height() {
    // `:252-267`
    let mut config = DynamicPrintConfig::full_print_config();
    set(&mut config, "printable_height", "100");
    set(
        &mut config,
        "bed_exclude_volumes",
        "-20..25;0x0,10x0,10x10,0x10|75..150;20x0,30x0,30x10,20x10|..;40x0,50x0,50x10,40x10",
    );
    let regions = get_bed_excluded_regions(&config);
    assert_eq!(regions.len(), 3);
    assert!(within(regions[0].z_min, 0.0, 1e-9));
    assert!(within(regions[0].z_max, 25.0, 1e-9));
    assert!(within(regions[1].z_min, 75.0, 1e-9));
    assert!(within(regions[1].z_max, 100.0, 1e-9));
    assert!(within(regions[2].z_min, 0.0, 1e-9));
    assert!(within(regions[2].z_max, 100.0, 1e-9));
}

#[test]
fn legacy_areas_and_collision_volumes_are_additive() {
    // `:299-317`
    let mut config = DynamicPrintConfig::full_print_config();
    set(&mut config, "printable_height", "100");
    set(&mut config, "bed_exclude_area", "0x0,10x0,10x10,0x10");
    let regions = get_bed_excluded_regions(&config);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].is_collision_volume());
    assert!(!has_bed_exclude_volumes(&config));

    set(
        &mut config,
        "bed_exclude_volumes",
        "20x20,30x20,30x30,20x30",
    );
    let regions = get_bed_excluded_regions(&config);
    assert_eq!(regions.len(), 2);
    assert!(!regions[0].is_collision_volume());
    assert!(regions[0].polygon.contains(&scaled(5.0, 5.0)));
    assert!(regions[1].is_collision_volume());
    assert!(regions[1].polygon.contains(&scaled(25.0, 25.0)));
}

#[test]
fn invalid_collision_volume_does_not_suppress_the_legacy_area() {
    // `:319-330`
    let mut config = DynamicPrintConfig::full_print_config();
    set(&mut config, "bed_exclude_area", "0x0,10x0,10x10,0x10");
    set(&mut config, "bed_exclude_volumes", "not a polygon");
    assert!(has_bed_exclude_volumes(&config));
    let regions = get_bed_excluded_regions(&config);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].is_collision_volume());
    assert!(!is_valid_bed_exclude_volumes_string("not a polygon", 100.0));
}

#[test]
fn empty_collision_volume_configuration_falls_back_to_the_legacy_area() {
    // `:332-348`
    let mut config = DynamicPrintConfig::full_print_config();
    set_two_extruders(&mut config);
    set(&mut config, "bed_exclude_area", "0x0,10x0,10x10,0x10");
    set(&mut config, "bed_exclude_volume_mode", "per_extruder");
    set(&mut config, "extruder_bed_exclude_volumes", "\"\";\"\"");
    assert!(!has_bed_exclude_volumes(&config));
    assert_eq!(
        active_bed_exclude_volume_mode(&config),
        BedExcludeVolumeMode::Shared
    );
    let groups = get_bed_excluded_regions_by_extruder(&config);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 1);
    assert_eq!(groups[1].len(), 1);
    assert!(!groups[0][0].is_collision_volume());
}

#[test]
fn shared_exclusion_volumes_resolve_identically_for_every_extruder() {
    // `:350-365`
    let mut config = two_extruder_config();
    set(&mut config, "bed_exclude_volume_mode", "shared");
    let groups = get_bed_excluded_regions_by_extruder(&config);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 1);
    assert_eq!(groups[1].len(), 1);
    assert_eq!(groups[0][0].polygon.points(), groups[1][0].polygon.points());
    assert!(within(groups[0][0].z_min, 10.0, 1e-9));
    assert!(within(groups[0][0].z_max, 30.0, 1e-9));
    // A shared compatibility view contains one definition, not one duplicate per nozzle.
    assert_eq!(get_bed_excluded_regions(&config).len(), 1);
}

#[test]
fn toolhead_relative_volumes_follow_nozzle_offset_deltas() {
    // `:367-380`
    let mut config = two_extruder_config();
    set(&mut config, "bed_exclude_volume_mode", "toolhead_offset");
    let groups = get_bed_excluded_regions_by_extruder(&config);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 1);
    assert_eq!(groups[1].len(), 1);
    let reference = bounds(&groups[0][0].polygon);
    let shifted = bounds(&groups[1][0].polygon);
    assert!(within(
        unscaled(shifted.min().x() - reference.min().x()),
        20.0,
        1e-6
    ));
    assert!(within(
        unscaled(shifted.min().y() - reference.min().y()),
        5.0,
        1e-6
    ));
}

#[test]
fn toolhead_relative_mode_offsets_volumes_but_not_the_shared_legacy_area() {
    // `:382-401`
    let mut config = two_extruder_config();
    set(&mut config, "bed_exclude_area", "70x70,80x70,80x80,70x80");
    set(&mut config, "bed_exclude_volume_mode", "toolhead_offset");
    let groups = get_bed_excluded_regions_by_extruder(&config);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 2);
    assert_eq!(groups[1].len(), 2);
    assert!(!groups[0][0].is_collision_volume());
    assert!(!groups[1][0].is_collision_volume());
    assert_eq!(groups[0][0].polygon.points(), groups[1][0].polygon.points());
    assert_ne!(groups[0][1].polygon.points(), groups[1][1].polygon.points());
    let flattened = get_bed_excluded_regions(&config);
    assert_eq!(flattened.len(), 3);
    assert_eq!(
        flattened
            .iter()
            .filter(|region| !region.is_collision_volume())
            .count(),
        1
    );
}

#[test]
fn toolhead_relative_exclusions_honour_a_non_default_reference_extruder() {
    // `:403-415`
    let mut config = two_extruder_config();
    set(&mut config, "master_extruder_id", "2");
    set(&mut config, "bed_exclude_volume_mode", "toolhead_offset");
    let groups = get_bed_excluded_regions_by_extruder(&config);
    assert_eq!(groups.len(), 2);
    let first = bounds(&groups[0][0].polygon);
    let reference = bounds(&groups[1][0].polygon);
    assert!(within(
        unscaled(first.min().x() - reference.min().x()),
        -20.0,
        1e-6
    ));
    assert!(within(
        unscaled(first.min().y() - reference.min().y()),
        -5.0,
        1e-6
    ));
}

#[test]
fn individual_exclusion_volumes_remain_authoritative_per_extruder() {
    // `:417-431`
    let mut config = two_extruder_config();
    set(&mut config, "bed_exclude_volume_mode", "per_extruder");
    set(
        &mut config,
        "extruder_bed_exclude_volumes",
        "\"0..25;0x0,8x0,8x8,0x8|40..60;20x20,30x20,30x30,20x30\";\"\"",
    );
    let groups = get_bed_excluded_regions_by_extruder(&config);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 2);
    assert!(groups[1].is_empty());
    assert_eq!(get_bed_excluded_regions(&config).len(), 2);
}

#[test]
fn individual_collision_volumes_retain_the_shared_legacy_area() {
    // `:433-450`
    let mut config = two_extruder_config();
    set(&mut config, "bed_exclude_area", "70x70,80x70,80x80,70x80");
    set(&mut config, "bed_exclude_volume_mode", "per_extruder");
    set(
        &mut config,
        "extruder_bed_exclude_volumes",
        "\"0..25;0x0,8x0,8x8,0x8\";\"0..25;20x0,28x0,28x8,20x8\"",
    );
    let groups = get_bed_excluded_regions_by_extruder(&config);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 2);
    assert_eq!(groups[1].len(), 2);
    assert!(!groups[0][0].is_collision_volume());
    assert_eq!(groups[0][0].polygon.points(), groups[1][0].polygon.points());
    assert!(groups[0][1].is_collision_volume());
    assert!(groups[1][1].is_collision_volume());
}

#[test]
fn plus_signed_coordinates_and_z_bounds_are_not_numbers() {
    // `parse_double_token` (`PrintConfig.cpp:13892-13902`) reads through
    // `string_to_double_decimal_point`, `fast_float::from_chars` (`LocalesUtils.cpp:86-93`), which
    // takes no leading '+'. A '+' coordinate leaves no polygon and a '+' z bound no range.
    assert!(!is_valid_bed_exclude_volumes_string(
        "+5x0,10x0,10x10,0x10",
        100.0
    ));
    assert!(!is_valid_bed_exclude_volumes_string(
        "+1..5;0x0,10x0,10x10,0x10",
        100.0
    ));
}

#[test]
fn arrange_keeps_convex_exclusions_exact_and_bounds_concave_ones() {
    // `:507-523`
    let convex = rectangle(0.0, 0.0, 10.0, 10.0);
    assert_eq!(
        bed_exclusion_arrange_polygon(&convex).points(),
        convex.points()
    );

    let concave = Polygon::new(vec![
        scaled(0.0, 0.0),
        scaled(10.0, 0.0),
        scaled(10.0, 4.0),
        scaled(4.0, 4.0),
        scaled(4.0, 10.0),
        scaled(0.0, 10.0),
    ]);
    assert!(!concave.contains(&scaled(8.0, 8.0)));
    let blocker = bed_exclusion_arrange_polygon(&concave);
    assert_eq!(blocker.points().len(), 4);
    assert!(blocker.contains(&scaled(8.0, 8.0)));
}
