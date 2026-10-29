//! `Feature/Interlocking/InterlockingGenerator.cpp` region-pair generator.

mod micro;
mod thin;
mod voxel;

use std::collections::BTreeSet;

use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloats, RegionOptions, SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, closing_ex, opening_ex, xor_ex,
    },
    project_slice::perimeters::flow::{build_nonbridging_flow, selected_nozzle},
};

use super::{PostRegionPrintObject, RegionLayer, RegionSurface};
use voxel::{DilationKernel, GridPoint3, VoxelUtils};

type GeometryResult<T> = Result<T, ClipperError>;

struct Generator<'a> {
    object: &'a mut PostRegionPrintObject,
    regions: [usize; 2],
    beam_width: i64,
    beam_layers: i64,
    boundary_avoidance: i64,
    rotation: f64,
    voxel: VoxelUtils,
    interface: &'a DilationKernel,
    air: &'a DilationKernel,
    min_line: i64,
}

pub(in crate::project_slice) fn apply(
    object: &mut PostRegionPrintObject,
    options: &ObjectOptions,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    if !options.interlocking_beam.0
        || options.interlocking_beam_layer_count.0 < 1
        || options.interlocking_depth.0 < 1
        || options.interlocking_beam_width.0 < 1e-4
    {
        return Ok(());
    }
    scale
        .checked_scale(2.0 * options.interlocking_beam_width.0)
        .ok_or_else(|| SliceError::InvalidInput("interlocking width out of range".into()))?;
    let beam_width = (options.interlocking_beam_width.0 / scale.factor()) as i64;
    let beam_layers = i64::from(options.interlocking_beam_layer_count.0);
    let boundary_avoidance = i64::from(options.interlocking_boundary_avoidance.0);
    let interface = DilationKernel::prism(i64::from(options.interlocking_depth.0));
    let air = DilationKernel::prism(boundary_avoidance);
    let rotation = f64::from((options.interlocking_orientation.0.to_radians()) as f32);
    for a in 0..object.regions.len() {
        for b in a + 1..object.regions.len() {
            if object.regions[a].options.outer_wall_filament_id
                == object.regions[b].options.outer_wall_filament_id
            {
                continue;
            }
            let min_line = scaled_external_width(
                &object.regions[a].options,
                options,
                nozzle_diameters,
                scale,
            )?
            .min(scaled_external_width(
                &object.regions[b].options,
                options,
                nozzle_diameters,
                scale,
            )?);
            Generator {
                object,
                regions: [a, b],
                beam_width,
                beam_layers,
                boundary_avoidance,
                rotation,
                voxel: VoxelUtils::new(2 * beam_width, 2 * beam_layers, scale),
                interface: &interface,
                air: &air,
                min_line,
            }
            .generate()
            .map_err(|error| {
                SliceError::InvalidInput(format!("interlocking geometry failed: {error:?}"))
            })?;
        }
    }
    Ok(())
}

impl Generator<'_> {
    fn generate(&mut self) -> GeometryResult<()> {
        let shells = self.shell_voxels()?;
        let mut common = shells[0]
            .intersection(&shells[1])
            .copied()
            .collect::<BTreeSet<_>>();
        if common.is_empty() {
            return Ok(());
        }
        let layer_regions = self.unioned_volume_regions()?;
        if self.boundary_avoidance > 0 {
            let mut air_cells = BTreeSet::new();
            self.add_boundary_cells(&layer_regions, self.air, &mut air_cells)?;
            common.retain(|point| !air_cells.contains(point));
            self.handle_thin_areas(&common)?;
        }
        self.apply_microstructure(&common, &layer_regions)
    }

    fn shell_voxels(&self) -> GeometryResult<[BTreeSet<GridPoint3>; 2]> {
        let mut shells = [BTreeSet::new(), BTreeSet::new()];
        for (mesh, cells) in shells.iter_mut().enumerate() {
            let layers = self.object.regions[self.regions[mesh]]
                .layers
                .iter()
                .map(|layer| {
                    let mut polygons = region_polygons(layer);
                    rotate_all(&mut polygons, self.rotation);
                    polygons
                })
                .collect::<Vec<_>>();
            self.add_boundary_cells(&layers, self.interface, cells)?;
        }
        Ok(shells)
    }

    fn add_boundary_cells(
        &self,
        layers: &[Vec<ExPolygon>],
        kernel: &DilationKernel,
        cells: &mut BTreeSet<GridPoint3>,
    ) -> GeometryResult<()> {
        for (layer, polygons) in layers.iter().enumerate() {
            self.voxel
                .walk_dilated_polygons(polygons, layer as i64, kernel, cells);
            let skin = if layer == 0 {
                polygons.clone()
            } else {
                xor_ex(polygons, &layers[layer - 1])?
            };
            let skin = opening_ex(
                &skin,
                self.voxel.cell_width as f32 / 2.0,
                JoinType::Miter,
                3.0,
            )?;
            for polygon in &skin {
                self.voxel
                    .walk_dilated_areas(polygon, layer as i64, kernel, cells)?;
            }
        }
        Ok(())
    }

    fn unioned_volume_regions(&self) -> GeometryResult<Vec<Vec<ExPolygon>>> {
        let mut result = Vec::with_capacity(self.object.plan.layers.len() + 1);
        for layer in 0..self.object.plan.layers.len() {
            let mut combined = region_polygons(&self.object.regions[self.regions[0]].layers[layer]);
            combined.extend(region_polygons(
                &self.object.regions[self.regions[1]].layers[layer],
            ));
            let mut closed = closing_ex(&combined, 100.0, JoinType::Miter, 3.0)?;
            rotate_all(&mut closed, self.rotation);
            result.push(closed);
        }
        result.push(Vec::new());
        Ok(result)
    }
}

fn scaled_external_width(
    region: &RegionOptions,
    object: &ObjectOptions,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<i64, SliceError> {
    let raw = match region.outer_wall_line_width {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => value.0,
    };
    let width = if raw == 0.0 {
        object.line_width
    } else {
        region.outer_wall_line_width
    };
    let nozzle = selected_nozzle(region.outer_wall_filament_id, nozzle_diameters)?;
    let flow = build_nonbridging_flow(width, 0.1, nozzle)?;
    Ok((f64::from(flow.width) / scale.factor()) as i64)
}

fn region_polygons(layer: &RegionLayer) -> Vec<ExPolygon> {
    layer
        .surfaces
        .iter()
        .map(|surface| surface.expolygon.clone())
        .collect()
}

fn set_internal(layer: &mut RegionLayer, polygons: Vec<ExPolygon>) {
    layer.surfaces = polygons.into_iter().map(RegionSurface::internal).collect();
}

fn rotate_all(polygons: &mut [ExPolygon], angle: f64) {
    let (sin, cos) = angle.sin_cos();
    for polygon in polygons {
        let owned = std::mem::replace(
            polygon,
            ExPolygon::new(Polygon::new(Vec::new()), Vec::new()),
        );
        let (mut contour, mut holes) = owned.into_parts();
        contour.rotate(cos, sin);
        for hole in &mut holes {
            hole.rotate(cos, sin);
        }
        *polygon = ExPolygon::new(contour, holes);
    }
}
