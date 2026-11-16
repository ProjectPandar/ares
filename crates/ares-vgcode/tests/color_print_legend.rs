//! Public neutral legend seam. Failure inventory: layer brackets duplicate or
//! use tag Z instead of lower_bound; Pause consumes a volume; trailing Print
//! becomes another switch; missing volumes are zero-filled; palette/segments
//! disappear on unavailable usage. Expectations follow GCodeViewer.cpp:
//! 3358-3388 and 4172-4239, not browser observations.
use ares_core::{
    ExtrusionRole,
    gcode_processor::{
        CustomGCodeItem, CustomGCodeTime, CustomGCodeType, EMoveType, GCodeProcessorResult,
        MoveVertex,
    },
};
use ares_vgcode::{
    PartialTimeKind, ViewerImpl, color_print_legend, convert::from_processor_result,
};

#[test]
fn partial_records_keep_pause_and_trailing_print_separate_from_layer_brackets() {
    let event = |kind, print_z, color: &str| CustomGCodeItem {
        kind,
        print_z,
        color: color.into(),
        extruder: 1,
        extra: String::new(),
    };
    let mut result = GCodeProcessorResult {
        extruder_colors: vec!["#000000".into()],
        moves: vec![
            MoveVertex::default(),
            MoveVertex {
                move_type: EMoveType::Extrude,
                extrusion_role: ExtrusionRole::Perimeter,
                position: [1.0, 0.0, 0.2],
                layer_id: 0,
                cp_color_id: 0,
                ..Default::default()
            },
            MoveVertex {
                move_type: EMoveType::Extrude,
                extrusion_role: ExtrusionRole::Perimeter,
                position: [2.0, 0.0, 0.4],
                layer_id: 1,
                cp_color_id: 2,
                ..Default::default()
            },
        ]
        .into(),
        custom_gcode_per_print_z: vec![
            event(CustomGCodeType::ColorChange, 0.2, "#FF0000"),
            event(CustomGCodeType::PausePrint, 0.4, ""),
            event(CustomGCodeType::ColorChange, 0.4, "#0000FF"),
        ],
        custom_gcode_times: vec![
            CustomGCodeTime {
                kind: CustomGCodeType::ColorChange,
                duration: 10.0,
                remaining: 100.0,
            },
            CustomGCodeTime {
                kind: CustomGCodeType::PausePrint,
                duration: 20.0,
                remaining: 90.0,
            },
            CustomGCodeTime {
                kind: CustomGCodeType::ColorChange,
                duration: 30.0,
                remaining: 70.0,
            },
            CustomGCodeTime {
                kind: CustomGCodeType::ColorChange,
                duration: 40.0,
                remaining: 40.0,
            },
        ],
        ..Default::default()
    };
    result.print_statistics.volumes_per_color_change = vec![100.0, 200.0, 300.0];
    let mut viewer = ViewerImpl::default();
    viewer.load(from_processor_result(&result));
    let legend = color_print_legend(&viewer, &result);
    let rows = legend.partial_usage.as_ref().unwrap();
    assert_eq!(
        rows.iter().map(|row| row.kind).collect::<Vec<_>>(),
        [
            PartialTimeKind::Print,
            PartialTimeKind::ColorChange,
            PartialTimeKind::Print,
            PartialTimeKind::Pause,
            PartialTimeKind::Print,
            PartialTimeKind::ColorChange,
            PartialTimeKind::Print,
        ]
    );
    assert_eq!(
        rows.iter().map(|row| row.volume).collect::<Vec<_>>(),
        [
            Some(100.0),
            None,
            None,
            None,
            Some(200.0),
            None,
            Some(300.0)
        ]
    );
    assert_eq!(rows[0].color1, [48; 3]);
    assert_eq!(rows[1].color2, [255, 0, 0]);
    assert_eq!(rows[6].color1, [0, 0, 255]);
    assert_eq!((rows[6].duration, rows[6].remaining), (40.0, 40.0));
    assert_eq!(legend.extruders[0].segments.len(), 2);
    assert_eq!(legend.extruders[0].ranges.len(), 2);
    assert_eq!(
        legend.extruders[0].ranges[1].z,
        [f64::from(0.2f32), f64::from(0.4f32)]
    );
    result
        .custom_gcode_per_print_z
        .push(event(CustomGCodeType::ColorChange, 0.4, "#00FF00"));
    assert_eq!(
        color_print_legend(&viewer, &result).extruders[0]
            .ranges
            .len(),
        2
    );
    result.custom_gcode_per_print_z.pop();
    result.print_statistics.volumes_per_color_change.pop();
    let unavailable = color_print_legend(&viewer, &result);
    assert_eq!(unavailable.partial_usage.unwrap_err().missing_index, 2);
    assert_eq!(unavailable.extruders, legend.extruders);
    assert_eq!(
        result.print_statistics.volumes_per_color_change,
        [100.0, 200.0]
    );
    result.custom_gcode_times.clear();
    assert!(
        color_print_legend(&viewer, &result)
            .partial_usage
            .unwrap()
            .is_empty()
    );
}
