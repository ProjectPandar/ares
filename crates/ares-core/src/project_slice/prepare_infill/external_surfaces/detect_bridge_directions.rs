use super::{Bridge, ExpansionZone};
use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Line, Polygon, WaveSeed, detect_bridging_direction,
    difference_open_polylines, offset_paths,
};

fn append_anchor_area(
    anchor_id: i32,
    expansion_zones: &[ExpansionZone],
    anchor_areas: &mut Vec<Polygon>,
) {
    let mut start_index = 0_u32;
    let mut end_index = 0_u32;
    for zone in expansion_zones {
        end_index = end_index.wrapping_add(zone.expolygons.len() as u32);
        if i64::from(anchor_id) < i64::from(end_index) {
            let local_index = (anchor_id as u32).wrapping_sub(start_index) as usize;
            let expolygon = &zone.expolygons[local_index];
            anchor_areas.push(expolygon.contour().clone());
            anchor_areas.extend(expolygon.holes().iter().cloned());
            break;
        }
        start_index = start_index.wrapping_add(zone.expolygons.len() as u32);
    }
}

pub(in crate::project_slice) fn detect_bridge_directions(
    bridge_anchors: &[WaveSeed],
    bridges: &mut [Bridge],
    expansion_zones: &[ExpansionZone],
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    assert!(
        !expansion_zones.is_empty(),
        "At least one expansion zone must exist!"
    );

    let mut anchor_cursor = bridge_anchors.iter().peekable();
    for bridge_id in 0..bridges.len() as u32 {
        let mut anchor_areas = Vec::new();
        let mut last_anchor_id = -1_i32;
        while anchor_cursor
            .peek()
            .is_some_and(|anchor| anchor.src == bridge_id)
        {
            let anchor = anchor_cursor.next().expect("peeked anchor exists");
            let boundary = anchor.boundary as i32;
            if boundary == last_anchor_id {
                continue;
            }
            last_anchor_id = boundary;
            append_anchor_area(last_anchor_id, expansion_zones, &mut anchor_areas);
        }

        let bridge = &mut bridges[bridge_id as usize];
        let overhang_area = std::iter::once(bridge.expolygon.contour())
            .chain(bridge.expolygon.holes())
            .cloned()
            .collect::<Vec<_>>();
        let paths = overhang_area
            .iter()
            .map(Polygon::split_at_first_point)
            .collect::<Vec<_>>();
        let scaled_epsilon = (1e-4_f64 / scale.factor()) as f32;
        assert!(scaled_epsilon > 0.0);
        let expanded_anchors = offset_paths(&anchor_areas, scaled_epsilon, JoinType::Miter, 3.0)?;
        let floating_paths = difference_open_polylines(&paths, &expanded_anchors)?;
        let floating_edges = floating_paths
            .iter()
            .flat_map(|path| {
                path.points()
                    .windows(2)
                    .map(|points| Line::new(points[0], points[1]))
            })
            .collect::<Vec<_>>();
        let (direction, _) = detect_bridging_direction(&floating_edges, &overhang_area, scale);
        bridge.angle = Some(std::f64::consts::PI + direction.1.atan2(direction.0));
    }
    Ok(())
}
