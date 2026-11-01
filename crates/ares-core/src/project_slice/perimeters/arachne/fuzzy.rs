// `FuzzySkin.cpp:415-473/502-558/685-814`: actual compatible-region
// grouping, eligible effect merging, exclusive masks, and ordered LineSplit.
use crate::{
    SliceError, SlicerContext,
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::{
        CoordinateScale, ExPolygon, JoinType, SAFETY_OFFSET, difference_ex, intersection_ex,
        offset_expolygons, split_line, union_expolygons,
    },
    project_slice::perimeters::types::{PerimeterInputRecord, PostPerimeterInputPrintObject},
};

mod config;
mod junctions;
mod ripple;
use config::Config;

#[derive(Clone, Debug)]
pub(in crate::project_slice::perimeters) struct Region {
    pub(in crate::project_slice::perimeters) config: Config,
    pub(in crate::project_slice::perimeters) mask: Vec<ExPolygon>,
}

#[derive(Clone, Debug)]
pub(in crate::project_slice) struct Regions {
    pub(in crate::project_slice::perimeters) entries: Vec<Region>,
    pub(in crate::project_slice::perimeters) has_contour: bool,
    pub(in crate::project_slice::perimeters) has_hole: bool,
    supported: Option<Vec<ExPolygon>>,
}

impl Regions {
    pub(in crate::project_slice::perimeters) fn prepare(
        object: &PostPerimeterInputPrintObject,
        input: &PerimeterInputRecord,
        scale: CoordinateScale,
        lower_slices: Option<&[ExPolygon]>,
    ) -> Result<Self, SliceError> {
        let mut grouped: Vec<(Config, Vec<ExPolygon>)> = Vec::new();
        let (mut has_contour, mut has_hole) = (false, false);
        for (options, surfaces) in object.compatible_fuzzy_regions(input) {
            let config = Config::new(options, scale);
            has_contour |= config.base.should_fuzzify(input.layer_id, 0, true);
            has_hole |= config.base.should_fuzzify(input.layer_id, 0, false);
            let index = grouped
                .iter()
                .position(|(cfg, _)| *cfg == config)
                .unwrap_or_else(|| {
                    grouped.push((config, Vec::new()));
                    grouped.len() - 1
                });
            grouped[index]
                .1
                .extend(surfaces.iter().map(|surface| surface.as_parts().1.clone()));
        }
        // Feature/FuzzySkin.cpp:480-488: the offset distance is source-scaled f32.
        let supported = if has_contour || has_hole {
            let max_thickness = grouped
                .iter()
                .filter(|(config, _)| {
                    config.base.should_fuzzify(input.layer_id, 0, true)
                        || config.base.should_fuzzify(input.layer_id, 0, false)
                })
                .map(|(config, _)| config.thickness)
                .max()
                .unwrap_or(0);
            lower_slices
                .map(|lower| {
                    offset_expolygons(
                        lower,
                        (input.ext_perimeter_flow.scaled_width(scale) + max_thickness) as f32,
                        JoinType::Miter,
                        3.0,
                    )
                    .map_err(geometry_error)
                })
                .transpose()?
        } else {
            None
        };
        let whole = grouped.len() == 1;
        let entries = grouped
            .into_iter()
            .map(|(config, surfaces)| {
                let mask = if whole {
                    Vec::new()
                } else {
                    offset_expolygons(&surfaces, SAFETY_OFFSET, JoinType::Miter, 3.0)
                        .map_err(geometry_error)?
                };
                Ok(Region { config, mask })
            })
            .collect::<Result<_, SliceError>>()?;
        Ok(Self {
            entries,
            has_contour,
            has_hole,
            supported,
        })
    }

    pub(in crate::project_slice::perimeters) fn merged(
        &self,
        layer_id: usize,
        inset: usize,
        contour: bool,
    ) -> Result<Vec<Region>, SliceError> {
        let mut merged: Vec<Region> = Vec::new();
        for region in &self.entries {
            if !region.config.base.should_fuzzify(layer_id, inset, contour) {
                continue;
            }
            if let Some(existing) = merged
                .iter_mut()
                .find(|r| r.config.same_effect(region.config))
            {
                merge_mask(existing, region);
            } else {
                merged.push(Region {
                    config: region.config,
                    mask: region.mask.clone(),
                });
            }
        }
        for region in &mut merged {
            if !region.mask.is_empty() {
                region.mask = union_expolygons(&region.mask, &[]).map_err(geometry_error)?;
            }
        }
        Ok(merged)
    }

    pub(super) fn apply(
        &self,
        extrusion: &mut ExtrusionLine,
        contour: bool,
        layer: Layer,
        context: &mut SlicerContext,
    ) -> Result<(), SliceError> {
        if self.entries.len() == 1 && self.supported.is_none() {
            let config = self.entries[0].config;
            if config
                .base
                .should_fuzzify(layer.id, extrusion.inset_index, contour)
            {
                junctions::fuzzify(&mut extrusion.junctions, config, layer, context, true);
            }
            return Ok(());
        }
        let mut merged = self.merged(layer.id, extrusion.inset_index, contour)?;
        if merged.len() == 1 && merged[0].mask.is_empty() && self.supported.is_none() {
            junctions::fuzzify(
                &mut extrusion.junctions,
                merged[0].config,
                layer,
                context,
                true,
            );
            return Ok(());
        }
        make_exclusive(&mut merged)?;
        restrict_to_supported(&mut merged, self.supported.as_deref())?;
        for region in merged {
            if self.supported.is_some() && region.mask.is_empty() {
                continue;
            }
            apply_mask(extrusion, &region, layer, context)?;
        }
        Ok(())
    }
}

// Feature/FuzzySkin.cpp:587-593: restricted empty masks mean no fuzzy coverage.
fn restrict_to_supported(
    regions: &mut [Region],
    supported: Option<&[ExPolygon]>,
) -> Result<(), SliceError> {
    let Some(supported) = supported else {
        return Ok(());
    };
    for region in regions {
        region.mask = if region.mask.is_empty() {
            supported.to_vec()
        } else {
            intersection_ex(&region.mask, supported).map_err(geometry_error)?
        };
    }
    Ok(())
}

fn merge_mask(existing: &mut Region, region: &Region) {
    if existing.mask.is_empty() {
        return;
    }
    if region.mask.is_empty() {
        existing.mask.clear();
    } else {
        existing.mask.extend_from_slice(&region.mask);
    }
}

pub(in crate::project_slice::perimeters) fn make_exclusive(
    merged: &mut [Region],
) -> Result<(), SliceError> {
    for index in 0..merged.len() {
        let (earlier, later) = merged.split_at_mut(index + 1);
        let region = &mut earlier[index];
        for next in later {
            if !region.mask.is_empty() && !next.mask.is_empty() {
                region.mask = difference_ex(&region.mask, &next.mask).map_err(geometry_error)?;
            }
        }
    }
    Ok(())
}

fn apply_mask(
    extrusion: &mut ExtrusionLine,
    region: &Region,
    layer: Layer,
    context: &mut SlicerContext,
) -> Result<(), SliceError> {
    let points: Vec<_> = extrusion.junctions.iter().map(|j| j.point).collect();
    let split = split_line(&points, &region.mask, false, layer.scale).map_err(geometry_error)?;
    if split.is_empty() {
        return Ok(());
    }
    if split.iter().all(|point| point.clipped) {
        junctions::fuzzify(
            &mut extrusion.junctions,
            region.config,
            layer,
            context,
            true,
        );
    } else {
        rebuild(extrusion, split, region.config, layer, context);
    }
    Ok(())
}

fn rebuild(
    extrusion: &mut ExtrusionLine,
    split: Vec<crate::geometry::SplitJunction>,
    config: Config,
    layer: Layer,
    context: &mut SlicerContext,
) {
    let current = std::mem::take(&mut extrusion.junctions);
    let mut segment = Vec::with_capacity(current.len());
    for point in split {
        let mut junction = current[point.source_index()];
        if !point.is_source() {
            junction.point = point.point;
        }
        if point.clipped {
            segment.push(junction);
        } else if segment.is_empty() {
            extrusion.junctions.push(junction);
        } else {
            segment.push(junction);
            append_segment(
                &mut extrusion.junctions,
                &mut segment,
                config,
                layer,
                context,
            );
        }
    }
    if !segment.is_empty() {
        append_segment(
            &mut extrusion.junctions,
            &mut segment,
            config,
            layer,
            context,
        );
    }
    // This assignment is upstream's multi-region behaviour (:811-812),
    // distinct from the whole-line Ripple kernel, which stays open.
    if let Some(first) = extrusion.junctions.first().copied() {
        let last = extrusion
            .junctions
            .last_mut()
            .expect("a nonempty split has a tail");
        if first.point != last.point {
            last.point = first.point;
            last.width = first.width;
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Layer {
    pub(super) id: usize,
    pub(super) slice_z: f64,
    pub(super) scale: CoordinateScale,
}

fn append_segment(
    output: &mut Vec<ExtrusionJunction>,
    segment: &mut Vec<ExtrusionJunction>,
    config: Config,
    layer: Layer,
    context: &mut SlicerContext,
) {
    let front = segment[0];
    let back = *segment.last().expect("a clipped segment has endpoints");
    junctions::fuzzify(segment, config, layer, context, false);
    if !output.is_empty() && output[0].point != front.point {
        output.push(front);
    }
    output.append(segment);
    if output.last().is_some_and(|last| last.point != front.point) {
        output.push(back);
    }
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("fuzzy region clipping exceeds the coordinate range".to_owned())
}
