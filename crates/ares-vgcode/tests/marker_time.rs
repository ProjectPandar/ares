//! Native LibVGCodeWrapper conversion + source-kernel timing oracle.
//! Failure inventory: exclusive endpoint; f64/reassociated accumulation;
//! phantom/repeated-id vertices skipped; filters or reverse seeks reset time;
//! seam window uses arrow vertex; reload retains prior prefixes.
//! Public seams: ViewerImpl timing queries and SequentialView window selection.

use ares_vgcode::{
    GCodeExtrusionRole, GCodeInputData, MoveType, PathVertex, SequentialView, ViewType, ViewerImpl,
};

struct Record {
    vertex: PathVertex,
    elapsed: [u32; 2],
}

fn records(name: &str) -> Vec<Record> {
    let root = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/parity/marker-time/"
    );
    std::fs::read_to_string(format!("{root}{name}.tsv"))
        .unwrap()
        .lines()
        .enumerate()
        .map(|(id, line)| {
            let values: Vec<_> = line.split('\t').collect();
            let uint = |i: usize| values[i].parse::<u32>().unwrap();
            assert_eq!(uint(0) as usize, id);
            Record {
                vertex: PathVertex {
                    gcode_id: uint(1),
                    move_type: [
                        MoveType::Noop,
                        MoveType::Retract,
                        MoveType::Unretract,
                        MoveType::Seam,
                        MoveType::ToolChange,
                        MoveType::ColorChange,
                        MoveType::PausePrint,
                        MoveType::CustomGCode,
                        MoveType::Travel,
                        MoveType::Wipe,
                        MoveType::Extrude,
                    ][uint(2) as usize],
                    role: GCodeExtrusionRole::ALL[uint(3) as usize],
                    layer_id: uint(4),
                    position: [5, 6, 7].map(|i| values[i].parse().unwrap()),
                    times: [8, 9].map(|i| f32::from_bits(uint(i))),
                    ..Default::default()
                },
                elapsed: [uint(10), uint(11)],
            }
        })
        .collect()
}

fn load(viewer: &mut ViewerImpl, records: &[Record]) {
    viewer.load(GCodeInputData {
        vertices: records.iter().map(|r| r.vertex.clone()).collect(),
        ..Default::default()
    });
}

fn check(viewer: &ViewerImpl, records: &[Record]) {
    for id in (0..records.len()).rev().chain(0..records.len()) {
        let expected = &records[id];
        assert_eq!(
            viewer.get_estimated_time_at(id).to_bits(),
            expected.elapsed[0],
            "prefix {id}"
        );
        let time = viewer.get_marker_time(id);
        assert_eq!(
            time.elapsed_seconds.to_bits(),
            expected.elapsed[0],
            "record {id}"
        );
        assert_eq!(
            time.vertex_seconds.to_bits(),
            expected.vertex.times[0].to_bits(),
            "duration {id}"
        );
    }
}

#[test]
fn native_prefix_bits_survive_visibility_changes_reverse_seeks_and_reload() {
    let native = records("vertices");
    assert!(native.iter().any(|r| r.vertex.move_type == MoveType::Seam));
    assert!(
        native
            .windows(2)
            .any(|r| r[0].vertex.gcode_id == r[1].vertex.gcode_id)
    );
    let mut viewer = ViewerImpl::default();
    load(&mut viewer, &native);
    check(&viewer, &native);
    viewer.set_layers_view_range(1, 1);
    viewer.toggle_extrusion_role_visibility(GCodeExtrusionRole::ExternalPerimeter);
    check(&viewer, &native);
    let synthetic = records("synthetic");
    load(&mut viewer, &synthetic);
    check(&viewer, &synthetic);
    // Native ordered f32 loses unit additions here; f64 final-cast must fail.
    assert_ne!(
        synthetic
            .iter()
            .map(|r| f64::from(r.vertex.times[0]))
            .sum::<f64>() as f32,
        f32::from_bits(synthetic.last().unwrap().elapsed[0])
    );
    load(&mut viewer, &native);
    check(&viewer, &native);
}

#[test]
fn seam_time_describes_the_existing_property_window_vertex() {
    let native = records("vertices");
    let seam = native
        .iter()
        .position(|r| r.vertex.move_type == MoveType::Seam)
        .unwrap();
    let mut viewer = ViewerImpl::default();
    load(&mut viewer, &native);
    let mut sequential = SequentialView::default();
    assert!(sequential.update(&viewer).is_none());
    viewer.set_view_visible_range(0, seam);
    let marker = sequential.update(&viewer).unwrap();
    assert_eq!(marker.window_vertex, seam);
    let feature_time = viewer.get_marker_time(marker.window_vertex);
    assert_eq!(
        feature_time.elapsed_seconds.to_bits(),
        native[seam].elapsed[0]
    );
    viewer.set_view_type(ViewType::Speed);
    let marker = sequential.update(&viewer).unwrap();
    assert_eq!(marker.window_vertex, seam - 1);
    let speed_time = viewer.get_marker_time(marker.window_vertex);
    assert_eq!(
        speed_time.elapsed_seconds.to_bits(),
        native[seam - 1].elapsed[0]
    );
    assert_eq!(
        speed_time.vertex_seconds.to_bits(),
        native[seam - 1].vertex.times[0].to_bits()
    );
}
