//! `InterlockingGenerator::handleThinAreas/growBorderAreasPerpendicular`.

use std::collections::BTreeSet;

use crate::geometry::{
    ExPolygon, JoinType, closing_ex, difference_ex, intersection_ex, offset_expolygons, opening_ex,
    union_expolygons,
};

use super::{
    Generator, GeometryResult, region_polygons, rotate_all, set_internal, voxel::GridPoint3,
};

impl Generator<'_> {
    pub(super) fn handle_thin_areas(
        &mut self,
        common: &BTreeSet<GridPoint3>,
    ) -> GeometryResult<()> {
        let detect = self.beam_width * self.boundary_avoidance + 5;
        let expand = self.beam_width * (self.boundary_avoidance - 1) + 5;
        let close_gaps = (self.min_line / 4) as f32;
        let layer_count = self.object.plan.layers.len();
        let mut near = vec![Vec::new(); layer_count];
        for &point in common {
            let (_, bottom) = self.voxel.lower_corner(point);
            let polygon = self.voxel.polygon(point);
            let top = (bottom + self.voxel.cell_layers).min(layer_count as i64);
            for layer in bottom..top {
                near[layer as usize].push(polygon.clone());
            }
        }
        for target in &mut near {
            let closed = closing_ex(target, 5.0, JoinType::Miter, 3.0)?;
            let united = union_expolygons(&closed, &[])?;
            *target = offset_expolygons(&united, detect as f32, JoinType::Miter, 3.0)?;
            rotate_all(target, self.rotation);
        }
        for (layer, nearby) in near.iter().enumerate() {
            let a = region_polygons(&self.object.regions[self.regions[0]].layers[layer]);
            let b = region_polygons(&self.object.regions[self.regions[1]].layers[layer]);
            let (border_a, border_b) = self.grow_border_areas(&a, &b, detect)?;
            let large_a = opening_ex(&a, detect as f32, JoinType::Miter, 3.0)?;
            let large_b = opening_ex(&b, detect as f32, JoinType::Miter, 3.0)?;
            let thin_a = thin_expansion(&a, &large_a, [&large_b, nearby, &border_a], expand)?;
            let thin_b = thin_expansion(&b, &large_b, [&large_a, nearby, &border_b], expand)?;
            let out_a = closing_ex(
                &difference_ex(&union_expolygons(&a, &thin_a)?, &thin_b)?,
                close_gaps,
                JoinType::Miter,
                3.0,
            )?;
            let out_b = closing_ex(
                &difference_ex(&union_expolygons(&b, &thin_b)?, &thin_a)?,
                close_gaps,
                JoinType::Miter,
                3.0,
            )?;
            set_internal(
                &mut self.object.regions[self.regions[0]].layers[layer],
                out_a,
            );
            set_internal(
                &mut self.object.regions[self.regions[1]].layers[layer],
                out_b,
            );
        }
        Ok(())
    }

    fn grow_border_areas(
        &self,
        a: &[ExPolygon],
        b: &[ExPolygon],
        detect: i64,
    ) -> GeometryResult<(Vec<ExPolygon>, Vec<ExPolygon>)> {
        let grown_a = offset_expolygons(a, self.min_line as f32, JoinType::Miter, 3.0)?;
        let grown_b = offset_expolygons(b, self.min_line as f32, JoinType::Miter, 3.0)?;
        let united = union_expolygons(&grown_a, &grown_b)?;
        let shrunk = offset_expolygons(&united, (-2 * self.min_line) as f32, JoinType::Miter, 3.0)?;
        let mut from_a = difference_ex(a, &shrunk)?;
        let mut from_b = difference_ex(b, &shrunk)?;
        for _ in 0..detect / self.min_line + 2 {
            let next_a = offset_expolygons(&from_a, self.min_line as f32, JoinType::Miter, 3.0)?;
            let next_b = offset_expolygons(&from_b, self.min_line as f32, JoinType::Miter, 3.0)?;
            from_a = difference_ex(&next_a, &next_b)?;
            from_b = difference_ex(&next_b, &next_a)?;
        }
        Ok((from_a, from_b))
    }
}

fn thin_expansion(
    own: &[ExPolygon],
    large_own: &[ExPolygon],
    clips: [&[ExPolygon]; 3],
    expand: i64,
) -> GeometryResult<Vec<ExPolygon>> {
    let thin = difference_ex(own, large_own)?;
    let thin = offset_expolygons(&thin, expand as f32, JoinType::Miter, 3.0)?;
    let mut clipped = intersection_ex(clips[0], &thin)?;
    for clip in &clips[1..] {
        clipped = intersection_ex(&clipped, clip)?;
    }
    offset_expolygons(&clipped, 5.0, JoinType::Miter, 3.0)
}
