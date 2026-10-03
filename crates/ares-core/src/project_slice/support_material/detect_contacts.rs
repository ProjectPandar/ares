//! `SupportMaterial.cpp::detect_contacts` and `buildplate_covered`.

use super::layer_input::{LayerAnnotations, LayerInput, to_polygons};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, difference_polygons_paths,
    intersection_polygons_paths, offset_expolygons_paths, offset_paths, offset2_ex,
    union_polygons_paths,
};

pub(super) struct SlicesMarginCache {
    offset: f32,
    pub(super) polygons: Vec<Polygon>,
    pub(super) all_polygons: Vec<Polygon>,
}

impl Default for SlicesMarginCache {
    fn default() -> Self {
        Self {
            offset: -1.0,
            polygons: Vec::new(),
            all_polygons: Vec::new(),
        }
    }
}

impl SlicesMarginCache {
    fn update(
        &mut self,
        offset: f32,
        no_interface_offset: f32,
        lower: &[ExPolygon],
        annotations: &LayerAnnotations<'_>,
    ) -> Result<(), ClipperError> {
        if self.offset == offset {
            return Ok(());
        }
        self.offset = offset;
        self.polygons = if offset == 0.0 {
            to_polygons(lower)
        } else {
            to_polygons(&offset2_ex(
                lower,
                -no_interface_offset * 0.5,
                offset + no_interface_offset * 0.5,
                JoinType::Square,
                0.0,
            )?)
        };
        if let Some(covered) = annotations.buildplate_covered
            && !covered.is_empty()
        {
            if !annotations.enforcers.is_empty() {
                self.all_polygons = self.polygons.clone();
            }
            self.polygons.extend_from_slice(covered);
            self.polygons = union_polygons_paths(&self.polygons)?;
        }
        Ok(())
    }
}

pub(super) struct ContactDetection<'a> {
    pub(super) layer_index: usize,
    pub(super) layer: &'a LayerInput<'a>,
    pub(super) lower_slices: &'a [ExPolygon],
    pub(super) annotations: &'a LayerAnnotations<'a>,
    pub(super) raft_expansion_scaled: f32,
    pub(super) gap_xy_scaled: f32,
}

pub(super) struct DetectedContacts {
    pub(super) polygons: Vec<Polygon>,
    pub(super) enforcer_polygons: Vec<Polygon>,
    pub(super) no_interface_offset: f32,
    pub(super) slices_margin: SlicesMarginCache,
}

pub(super) fn detect_contacts(
    input: &ContactDetection<'_>,
    overhangs: &mut Vec<Polygon>,
) -> Result<DetectedContacts, ClipperError> {
    let mut output = DetectedContacts {
        polygons: Vec::new(),
        enforcer_polygons: Vec::new(),
        no_interface_offset: 0.0,
        slices_margin: SlicesMarginCache::default(),
    };
    if input.layer_index == 0 {
        output.polygons = if input.raft_expansion_scaled > 0.0 {
            offset_paths(overhangs, input.raft_expansion_scaled, JoinType::Miter, 3.0)?
        } else {
            overhangs.clone()
        };
        return Ok(output);
    }
    if input.layer.regions.is_empty() {
        return Ok(output);
    }
    output.no_interface_offset = input
        .layer
        .regions
        .iter()
        .fold(f32::MAX, |minimum, region| {
            minimum.min(region.external_width_scaled)
        });
    // The upstream contact detector leaves lower_layer_offset at zero;
    // its independently computed overhang-angle offset is not used here.
    let margin_offset = 0.0_f32.min(input.gap_xy_scaled);
    for region in input.layer.regions {
        let polygons = intersection_polygons_paths(overhangs, &to_polygons(region.slices))?;
        if polygons.is_empty() {
            continue;
        }
        output.slices_margin.update(
            margin_offset,
            output.no_interface_offset,
            input.lower_slices,
            input.annotations,
        )?;
        output.polygons.extend(difference_polygons_paths(
            &polygons,
            &output.slices_margin.polygons,
        )?);
    }
    if input.annotations.enforcers.is_empty() {
        return Ok(output);
    }
    let polygons = intersection_polygons_paths(
        &to_polygons(input.layer.slices),
        input.annotations.enforcers,
    )?;
    let expanded_lower = offset_paths(
        &to_polygons(input.lower_slices),
        0.05 * output.no_interface_offset,
        JoinType::Square,
        0.0,
    )?;
    output.enforcer_polygons = difference_polygons_paths(&polygons, &expanded_lower)?;
    if !output.enforcer_polygons.is_empty() {
        overhangs.extend_from_slice(&output.enforcer_polygons);
        output.slices_margin.update(
            margin_offset,
            output.no_interface_offset,
            input.lower_slices,
            input.annotations,
        )?;
        let trimming = if output.slices_margin.all_polygons.is_empty() {
            &output.slices_margin.polygons
        } else {
            &output.slices_margin.all_polygons
        };
        output.polygons.extend(difference_polygons_paths(
            &output.enforcer_polygons,
            trimming,
        )?);
    }
    Ok(output)
}

pub(super) fn buildplate_covered(
    layers: &[LayerInput<'_>],
    scale: CoordinateScale,
) -> Result<Vec<Vec<Polygon>>, ClipperError> {
    let mut covered = vec![Vec::new(); layers.len()];
    for layer_index in 1..layers.len() {
        let mut polygons = covered[layer_index - 1].clone();
        polygons.extend(offset_expolygons_paths(
            layers[layer_index - 1].slices,
            (0.01 / scale.factor()) as f32,
            JoinType::Miter,
            3.0,
        )?);
        covered[layer_index] = union_polygons_paths(&polygons)?;
    }
    Ok(covered)
}
