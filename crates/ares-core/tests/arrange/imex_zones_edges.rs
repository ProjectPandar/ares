//! `test_imex_zones.cpp`'s layouts with nothing to divide, the grid's last Primary, the strips'
//! zero widths and the aggregated gantry (`:367-403, 449-579` at v2.5.0-beta `345efec64`).

use super::{
    PrinterCfg, check_box_xy, check_point, check_rect, is_empty, layout, layout_for, mirror_pair,
};

#[test]
fn a_non_imex_printer_has_no_zones() {
    // :374-378
    let p = PrinterCfg {
        is_imex: false,
        ..mirror_pair()
    };
    assert!(is_empty(&layout(&p)));
}

#[test]
fn an_empty_plate_mode_has_no_zones() {
    // :380-382
    assert!(is_empty(&layout_for(&mirror_pair(), "", "")));
}

#[test]
fn a_single_tool_grid_has_no_second_zone_to_dim() {
    // :384-390
    let p = PrinterCfg {
        tools_per_gantry: 1,
        active_tools: "0:P",
        ..mirror_pair()
    };
    assert!(is_empty(&layout(&p)));
}

#[test]
fn a_mode_the_printer_does_not_define_has_no_zones() {
    // :392-396: a stale mode carried in from another printer's process preset.
    assert!(is_empty(&layout_for(
        &mirror_pair(),
        "carried-over-mode",
        ""
    )));
}

#[test]
fn a_mode_with_an_empty_tool_roster_has_no_zones() {
    // :398-402
    let p = PrinterCfg {
        active_tools: "",
        ..mirror_pair()
    };
    assert!(is_empty(&layout(&p)));
}

#[test]
fn a_primary_on_the_last_column_of_a_1x2_idex_lays_out_normally() {
    // :453-480: the mirror sits to the primary's left, so the strip hugs the primary zone's left
    // boundary and the band lies further inside it.
    let p = PrinterCfg {
        active_tools: "1:P,0:M",
        clearance_x: 5.0,
        margin: 2.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    assert_eq!(l.primary_head, 1);
    check_rect(&l.primary_zone_box.unwrap(), [100.0, 0.0, 200.0, 200.0]);
    assert_eq!(l.mirror_zones.len(), 1);
    check_rect(&l.mirror_zones[0], [0.0, 0.0, 100.0, 200.0]);
    assert_eq!(l.head_zone_centers.len(), 2);
    check_point(l.head_zone_centers[&0], [50.0, 100.0]);
    check_point(l.head_zone_centers[&1], [150.0, 100.0]);
    assert_eq!(l.collision_zones.len(), 1);
    check_box_xy(&l.collision_zones[0], [100.0, 0.0, 105.0, 200.0]);
    assert_eq!(l.margin_bands.len(), 1);
    check_rect(&l.margin_bands[0], [105.0, 0.0, 107.0, 200.0]);
}

#[test]
fn a_primary_on_the_last_tool_of_a_2x2_iqex_lays_out_normally() {
    // :482-502: T0 is diagonally opposite, so no boundary of the primary zone is at risk.
    let p = PrinterCfg {
        gantry_count: 2,
        active_tools: "3:P,0:M",
        clearance_x: 5.0,
        clearance_y: 5.0,
        ..PrinterCfg::default()
    };
    let l = layout(&p);
    assert_eq!(l.primary_head, 3);
    check_rect(&l.primary_zone_box.unwrap(), [100.0, 100.0, 200.0, 200.0]);
    assert_eq!(l.mirror_zones.len(), 1);
    check_rect(&l.mirror_zones[0], [0.0, 0.0, 100.0, 100.0]);
    assert!(l.collision_zones.is_empty());
}

#[test]
fn no_nozzle_clearance_raises_no_collision_strip() {
    // :514-518
    let l = layout(&mirror_pair());
    assert!(l.collision_zones.is_empty());
    assert!(l.margin_bands.is_empty());
}

#[test]
fn a_clearance_without_a_margin_gives_the_strip_but_no_advisory_band() {
    // :520-525
    let p = PrinterCfg {
        clearance_x: 5.0,
        ..mirror_pair()
    };
    let l = layout(&p);
    assert_eq!(l.collision_zones.len(), 1);
    assert!(l.margin_bands.is_empty());
}

#[test]
fn an_aggregated_gantry_raises_its_collision_strip_from_the_zone_it_paints() {
    // :528-578: both rosters are one primary gantry with one mirrored gantry behind it.
    for (tools_per_gantry, active_tools) in [(2, "0:P,1:S,2:M,3:M"), (3, "0:P,1:S,4:M,5:M")] {
        let p = PrinterCfg {
            gantry_count: 2,
            tools_per_gantry,
            active_tools,
            clearance_x: 6.0,
            clearance_y: 4.0,
            margin: 2.0,
            ..PrinterCfg::default()
        };
        let l = layout(&p);
        assert_eq!(l.primary_head, 0, "{active_tools}");
        check_rect(&l.primary_zone_box.unwrap(), [0.0, 0.0, 200.0, 100.0]);
        assert!(l.copy_zones.is_empty());
        assert_eq!(l.mirror_zones.len(), 1, "{active_tools}");
        check_rect(&l.mirror_zones[0], [0.0, 100.0, 200.0, 200.0]);
        assert_eq!(l.secondary_zone_boxes.len(), 1);
        check_box_xy(&l.secondary_zone_boxes[0], [0.0, 100.0, 200.0, 200.0]);
        assert_eq!(l.collision_zones.len(), 1, "{active_tools}");
        check_box_xy(&l.collision_zones[0], [0.0, 96.0, 200.0, 100.0]);
        assert_eq!(l.margin_bands.len(), 1);
        check_rect(&l.margin_bands[0], [0.0, 94.0, 200.0, 96.0]);
    }
}
