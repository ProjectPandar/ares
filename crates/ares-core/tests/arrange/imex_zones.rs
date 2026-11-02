//! The IMEX zone layout, ported from OrcaSlicer's `tests/libslic3r/test_imex_zones.cpp` at
//! v2.5.0-beta `345efec64` through `arrangement::imex_zones`. Library and GUI-consumed only, so the
//! upstream cases are the oracle. Each `SECTION` is its own test.

use ares_core::DynamicPrintConfig;
use ares_core::arrangement::imex::BoundingBoxf;
use ares_core::arrangement::imex_zones::{
    BoundingBoxf3, IMEX_PRIMARY_MODE, ImexZoneLayout, compute_imex_slice_offset,
    compute_imex_zone_layout,
};

#[path = "imex_zones_edges.rs"]
mod edges;

const BED: BoundingBoxf = BoundingBoxf {
    min: [0.0, 0.0],
    max: [200.0, 200.0],
};
const MODE: &str = "parallel";

/// `PrinterCfg` (`:21-31`): one parallel mode named `MODE` after Primary.
#[derive(Clone)]
struct PrinterCfg {
    gantry_count: i32,
    tools_per_gantry: i32,
    /// `imex_tool_layout`, serialized ("front-left", "front-right", "rear-left", "rear-right").
    tool_layout: &'static str,
    active_tools: &'static str,
    clearance_x: f64,
    clearance_y: f64,
    margin: f64,
    is_imex: bool,
}

impl Default for PrinterCfg {
    fn default() -> Self {
        Self {
            gantry_count: 1,
            tools_per_gantry: 2,
            tool_layout: "front-left",
            active_tools: "",
            clearance_x: 0.0,
            clearance_y: 0.0,
            margin: 0.0,
            is_imex: true,
        }
    }
}

fn set(cfg: &mut DynamicPrintConfig, key: &str, value: &str) {
    cfg.set_deserialize_strict(key, value).unwrap();
}

/// `escape_strings_cstyle` of values with no separator: joined by ';', a lone empty one quoted.
fn strings(values: &[&str]) -> String {
    if values == [""] {
        "\"\"".to_owned()
    } else {
        values.join(";")
    }
}

/// `make_cfg` (`:40-59`), over the registry defaults.
fn make_cfg(p: &PrinterCfg) -> DynamicPrintConfig {
    let mut cfg = DynamicPrintConfig::full_print_config();
    set(&mut cfg, "is_imex", if p.is_imex { "1" } else { "0" });
    set(&mut cfg, "imex_gantry_count", &p.gantry_count.to_string());
    set(
        &mut cfg,
        "imex_tools_per_gantry",
        &p.tools_per_gantry.to_string(),
    );
    set(&mut cfg, "imex_tool_layout", p.tool_layout);
    set(
        &mut cfg,
        "imex_mode_names",
        &strings(&[IMEX_PRIMARY_MODE, MODE]),
    );
    set(
        &mut cfg,
        "imex_mode_active_tools",
        &strings(&["", p.active_tools]),
    );
    set(
        &mut cfg,
        "imex_nozzle_clearance_x",
        &p.clearance_x.to_string(),
    );
    set(
        &mut cfg,
        "imex_nozzle_clearance_y",
        &p.clearance_y.to_string(),
    );
    set(&mut cfg, "imex_carriage_margin", &p.margin.to_string());
    cfg
}

fn layout_for(p: &PrinterCfg, plate_mode: &str, process_mode: &str) -> ImexZoneLayout {
    compute_imex_zone_layout(&make_cfg(p), plate_mode, process_mode, BED)
}

fn layout(p: &PrinterCfg) -> ImexZoneLayout {
    layout_for(p, MODE, "")
}

fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-9, "{actual} != {expected}");
}

fn check_rect(b: &BoundingBoxf, [x0, y0, x1, y1]: [f64; 4]) {
    near(b.min[0], x0);
    near(b.min[1], y0);
    near(b.max[0], x1);
    near(b.max[1], y1);
}

fn check_box_xy(b: &BoundingBoxf3, [x0, y0, x1, y1]: [f64; 4]) {
    near(b.min[0], x0);
    near(b.min[1], y0);
    near(b.max[0], x1);
    near(b.max[1], y1);
}

fn check_point(p: [f64; 2], [x, y]: [f64; 2]) {
    near(p[0], x);
    near(p[1], y);
}

fn is_empty(l: &ImexZoneLayout) -> bool {
    l.primary_head == -1
        && l.primary_zone_box.is_none()
        && l.head_zone_centers.is_empty()
        && l.copy_zones.is_empty()
        && l.mirror_zones.is_empty()
        && l.secondary_zone_boxes.is_empty()
        && l.collision_zones.is_empty()
        && l.margin_bands.is_empty()
}

#[test]
fn a_config_missing_the_nozzle_clearance_keys_falls_back_to_the_registered_clearance() {
    // :113-138. Not via make_cfg: the clearance, margin and tool layout keys stay unset, so they
    // read their registered values (30, 0, front-left).
    let mut cfg = DynamicPrintConfig::full_print_config();
    set(&mut cfg, "is_imex", "1");
    set(&mut cfg, "imex_gantry_count", "1");
    set(&mut cfg, "imex_tools_per_gantry", "2");
    set(&mut cfg, "imex_mode_names", &strings(&[MODE]));
    set(&mut cfg, "imex_mode_active_tools", &strings(&["0:P,1:M"]));
    let l = compute_imex_zone_layout(&cfg, MODE, "", BED);
    assert_eq!(l.primary_head, 0);
    check_rect(&l.primary_zone_box.unwrap(), [0.0, 0.0, 100.0, 200.0]);
    // One X boundary, one strip, 30 mm wide against it.
    assert_eq!(l.collision_zones.len(), 1);
    check_box_xy(&l.collision_zones[0], [70.0, 0.0, 100.0, 200.0]);
    // The margin's own fallback is 0, and a zero margin raises no band.
    assert!(l.margin_bands.is_empty());
}

#[test]
fn an_idex_mirror_pair_splits_the_bed_into_two_columns() {
    // :140-179
    let p = PrinterCfg {
        active_tools: "0:P,1:M",
        clearance_x: 5.0,
        margin: 2.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    assert_eq!(l.primary_head, 0);
    check_rect(&l.primary_zone_box.unwrap(), [0.0, 0.0, 100.0, 200.0]);
    assert!(l.copy_zones.is_empty());
    assert_eq!(l.mirror_zones.len(), 1);
    check_rect(&l.mirror_zones[0], [100.0, 0.0, 200.0, 200.0]);
    assert_eq!(l.head_zone_centers.len(), 2);
    check_point(l.head_zone_centers[&0], [50.0, 100.0]);
    check_point(l.head_zone_centers[&1], [150.0, 100.0]);
    // The mirror zone is off limits to objects, over the full Z range.
    assert_eq!(l.secondary_zone_boxes.len(), 1);
    check_box_xy(&l.secondary_zone_boxes[0], [100.0, 0.0, 200.0, 200.0]);
    near(l.secondary_zone_boxes[0].min[2], -1.0);
    assert!(l.secondary_zone_boxes[0].max[2] > 1000.0);
    // The strip sits inside the primary zone, against the boundary, one clearance wide.
    assert_eq!(l.collision_zones.len(), 1);
    check_box_xy(&l.collision_zones[0], [95.0, 0.0, 100.0, 200.0]);
    // The margin band abuts the strip on the primary side, one margin wide.
    assert_eq!(l.margin_bands.len(), 1);
    check_rect(&l.margin_bands[0], [93.0, 0.0, 95.0, 200.0]);
}

#[test]
fn a_copy_secondary_claims_a_zone_but_raises_no_collision_strip() {
    // :181-200
    let p = PrinterCfg {
        active_tools: "0:P,1:C",
        clearance_x: 5.0,
        margin: 2.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    assert_eq!(l.copy_zones.len(), 1);
    check_rect(&l.copy_zones[0], [100.0, 0.0, 200.0, 200.0]);
    assert!(l.mirror_zones.is_empty());
    assert_eq!(l.secondary_zone_boxes.len(), 1);
    assert!(l.collision_zones.is_empty());
    assert!(l.margin_bands.is_empty());
}

#[test]
fn the_tool_layout_decides_which_corner_the_primary_zone_occupies() {
    // :202-234, one DYNAMIC_SECTION per layout.
    for (tool_layout, [x0, y0, x1, y1]) in [
        ("front-left", [0.0, 0.0, 100.0, 100.0]),
        ("front-right", [100.0, 0.0, 200.0, 100.0]),
        ("rear-left", [0.0, 100.0, 100.0, 200.0]),
        ("rear-right", [100.0, 100.0, 200.0, 200.0]),
    ] {
        let p = PrinterCfg {
            gantry_count: 2,
            tool_layout,
            active_tools: "0:P,1:C,2:M,3:M",
            ..PrinterCfg::default()
        };
        let l = layout(&p);
        check_rect(&l.primary_zone_box.unwrap(), [x0, y0, x1, y1]);
        // The primary's zone centre agrees with its zone.
        check_point(l.head_zone_centers[&0], [0.5 * (x0 + x1), 0.5 * (y0 + y1)]);
        // Three secondaries, each with its own quadrant.
        assert_eq!(l.secondary_zone_boxes.len(), 3, "{tool_layout}");
    }
}

#[test]
fn a_2x2_iqex_with_independent_secondaries_gives_one_quadrant_per_tool() {
    // :236-272. Only T2 shares the primary's column, so only it raises a strip.
    let p = PrinterCfg {
        gantry_count: 2,
        active_tools: "0:P,1:C,2:M,3:M",
        clearance_x: 6.0,
        clearance_y: 4.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    check_rect(&l.primary_zone_box.unwrap(), [0.0, 0.0, 100.0, 100.0]);
    assert_eq!(l.copy_zones.len(), 1);
    check_rect(&l.copy_zones[0], [100.0, 0.0, 200.0, 100.0]);
    assert_eq!(l.mirror_zones.len(), 2);
    check_rect(&l.mirror_zones[0], [0.0, 100.0, 100.0, 200.0]);
    check_rect(&l.mirror_zones[1], [100.0, 100.0, 200.0, 200.0]);
    // The copy rects, then the mirror rects.
    assert_eq!(l.secondary_zone_boxes.len(), 3);
    check_box_xy(&l.secondary_zone_boxes[0], [100.0, 0.0, 200.0, 100.0]);
    check_box_xy(&l.secondary_zone_boxes[1], [0.0, 100.0, 100.0, 200.0]);
    check_box_xy(&l.secondary_zone_boxes[2], [100.0, 100.0, 200.0, 200.0]);
    assert_eq!(l.collision_zones.len(), 1);
    check_box_xy(&l.collision_zones[0], [0.0, 96.0, 100.0, 100.0]);
    assert!(l.margin_bands.is_empty());
}

#[test]
fn a_span_partner_collapses_the_far_gantry_into_one_full_width_strip() {
    // :274-309
    let p = PrinterCfg {
        gantry_count: 2,
        active_tools: "0:P,1:S,2:M,3:M",
        clearance_x: 6.0,
        clearance_y: 4.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    assert_eq!(l.primary_head, 0);
    check_rect(&l.primary_zone_box.unwrap(), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(l.mirror_zones.len(), 1);
    check_rect(&l.mirror_zones[0], [0.0, 100.0, 200.0, 200.0]);
    assert!(l.copy_zones.is_empty());
    assert_eq!(l.secondary_zone_boxes.len(), 1);
    // A Span tool prints in the primary's zone and owns no centre.
    assert!(!l.head_zone_centers.contains_key(&1));
    check_point(l.head_zone_centers[&0], [50.0, 50.0]);
    check_point(l.head_zone_centers[&2], [50.0, 150.0]);
    check_point(l.head_zone_centers[&3], [150.0, 150.0]);
    // Only the rear boundary is dangerous, across the widened primary zone.
    assert_eq!(l.collision_zones.len(), 1);
    check_box_xy(&l.collision_zones[0], [0.0, 96.0, 200.0, 100.0]);
}

#[test]
fn inactive_tools_donate_their_bed_share_to_the_active_ones() {
    // :311-338
    let p = PrinterCfg {
        tools_per_gantry: 3,
        active_tools: "0:P,2:M",
        clearance_x: 5.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    let primary = l.primary_zone_box.unwrap();
    check_rect(&primary, [0.0, 0.0, 100.0, 200.0]);
    near(primary.max[0] - primary.min[0], 100.0); // half, not 200/3
    assert_eq!(l.mirror_zones.len(), 1);
    check_rect(&l.mirror_zones[0], [100.0, 0.0, 200.0, 200.0]);
    assert_eq!(l.head_zone_centers.len(), 2);
    check_point(l.head_zone_centers[&0], [50.0, 100.0]);
    check_point(l.head_zone_centers[&2], [150.0, 100.0]);
    assert_eq!(l.collision_zones.len(), 1);
    check_box_xy(&l.collision_zones[0], [95.0, 0.0, 100.0, 200.0]);
}

/// `:340-345` and `:367-372`: the IDEX mirror pair the fallback and empty-layout cases share.
fn mirror_pair() -> PrinterCfg {
    PrinterCfg {
        active_tools: "0:P,1:M",
        ..PrinterCfg::default()
    }
}

#[test]
fn a_plate_still_on_primary_picks_up_the_process_preset_mode() {
    // :347-351
    let l = layout_for(&mirror_pair(), IMEX_PRIMARY_MODE, MODE);
    check_rect(&l.primary_zone_box.unwrap(), [0.0, 0.0, 100.0, 200.0]);
}

#[test]
fn a_plate_with_its_own_mode_ignores_the_process_preset() {
    // :353-359
    let l = layout_for(&mirror_pair(), MODE, "no-such-mode");
    check_rect(&l.primary_zone_box.unwrap(), [0.0, 0.0, 100.0, 200.0]);
}

#[test]
fn primary_on_both_the_plate_and_the_process_preset_yields_no_zones() {
    // :361-364
    assert!(is_empty(&layout_for(&mirror_pair(), IMEX_PRIMARY_MODE, "")));
    assert!(is_empty(&layout_for(
        &mirror_pair(),
        IMEX_PRIMARY_MODE,
        IMEX_PRIMARY_MODE
    )));
}

#[test]
fn a_primary_off_the_grid_yields_no_zones_and_no_slice_offset() {
    // :405-434. A Primary on T3 left over from a 4-tool IQEX after the printer became a 2-tool IDEX.
    let p = PrinterCfg {
        active_tools: "3:P,0:M",
        clearance_x: 5.0,
        margin: 2.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    assert!(is_empty(&l));
    assert_eq!(l.primary_head, -1);
    assert!(l.primary_zone_box.is_none());
    assert!(l.secondary_zone_boxes.is_empty());
    assert!(l.mirror_zones.is_empty());
    assert!(l.head_zone_centers.is_empty());
    // With no primary zone, a firmware-managed printer shifts the slice by nothing.
    check_point(
        compute_imex_slice_offset(true, MODE, l.primary_zone_box),
        [0.0, 0.0],
    );
}

#[test]
fn a_mode_with_no_primary_marker_yields_no_zones() {
    // :436-447
    let p = PrinterCfg {
        active_tools: "0:M,1:M",
        clearance_x: 5.0,
        ..PrinterCfg::default()
    };
    assert!(is_empty(&layout(&p)));
}
