//! `InterlockingGenerator::generateMicrostructure/applyMicrostructureToOutlines`.

use std::collections::BTreeSet;

use crate::geometry::{
    ExPolygon, Point, Polygon, difference_ex, intersection_ex, union_expolygons,
};

use super::{
    Generator, GeometryResult, region_polygons, rotate_all, set_internal, voxel::GridPoint3,
};

impl Generator<'_> {
    pub(super) fn apply_microstructure(
        &mut self,
        cells: &BTreeSet<GridPoint3>,
        layer_regions: &[Vec<ExPolygon>],
    ) -> GeometryResult<()> {
        let structures = self.build_structures(cells)?;
        for (mesh, own_structure) in structures.iter().enumerate() {
            for (layer, outline) in layer_regions
                .iter()
                .take(self.object.plan.layers.len())
                .enumerate()
            {
                let mut outlines = outline.clone();
                rotate_all(&mut outlines, -self.rotation);
                let group = layer / self.beam_layers as usize;
                let here = intersection_ex(&own_structure[group], &outlines)?;
                let other = &structures[1 - mesh][group];
                let index = self.regions[mesh];
                let current = region_polygons(&self.object.regions[index].layers[layer]);
                let replacement = union_expolygons(&difference_ex(&current, other)?, &here)?;
                set_internal(&mut self.object.regions[index].layers[layer], replacement);
            }
        }
        Ok(())
    }

    fn microstructure(&self) -> [[[Point; 4]; 2]; 2] {
        let middle = self.voxel.cell_width / 2;
        let base = std::array::from_fn(|mesh| {
            let (left, right) = if mesh == 0 {
                (0, middle)
            } else {
                (middle, self.voxel.cell_width)
            };
            [
                Point::new(left, 0),
                Point::new(right, 0),
                Point::new(right, self.voxel.cell_width),
                Point::new(left, self.voxel.cell_width),
            ]
        });
        [
            base,
            base.map(|polygon| polygon.map(|point| Point::new(point.y(), point.x()))),
        ]
    }

    fn build_structures(
        &self,
        cells: &BTreeSet<GridPoint3>,
    ) -> GeometryResult<[Vec<Vec<ExPolygon>>; 2]> {
        let layer_count = self.object.plan.layers.len();
        let groups = layer_count.div_ceil(self.beam_layers as usize);
        let mut structures = [vec![Vec::new(); groups], vec![Vec::new(); groups]];
        let templates = self.microstructure();
        for (mesh, target) in structures.iter_mut().enumerate() {
            for &grid in cells {
                self.append_cell(target, grid, mesh, &templates);
            }
        }
        for mesh in &mut structures {
            for group in mesh {
                *group = union_expolygons(group, &[])?;
                rotate_all(group, -self.rotation);
            }
        }
        Ok(structures)
    }

    fn append_cell(
        &self,
        target: &mut [Vec<ExPolygon>],
        grid: GridPoint3,
        mesh: usize,
        templates: &[[[Point; 4]; 2]; 2],
    ) {
        let (lower, bottom) = self.voxel.lower_corner(grid);
        let top = (bottom + self.voxel.cell_layers).min(self.object.plan.layers.len() as i64);
        for layer in (bottom..top).step_by(self.beam_layers as usize) {
            let group = (layer / self.beam_layers) as usize;
            let direction = group % templates.len();
            let polygon = Polygon::new(
                templates[direction][mesh]
                    .iter()
                    .map(|point| Point::new(point.x() + lower.x(), point.y() + lower.y()))
                    .collect(),
            );
            target[group].push(ExPolygon::new(polygon, Vec::new()));
        }
    }
}
