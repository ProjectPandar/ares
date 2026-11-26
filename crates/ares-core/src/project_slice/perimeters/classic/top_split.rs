mod config;
mod first_outer;
mod split;
mod types;

pub(in crate::project_slice) use types::{
    ClassicTopSplitRecord, PostClassicTopSplitPrintObject, PreparedPostClassicTopSplit,
    PreparedTopSplitSurface,
};

use crate::{SliceError, geometry::ExPolygon};

use self::config::ValidatedTopSplitConfig;
use super::types::{
    ClassicPreludeRecord, PostClassicPreludePrintObject, PreparedClassicSurface,
    PreparedPostClassicPrelude,
};
use crate::project_slice::perimeters::types::PerimeterInputRecord;

pub(super) fn finish(
    prepared: PreparedPostClassicPrelude,
    cancel: &crate::CancelToken,
) -> Result<PreparedPostClassicTopSplit, SliceError> {
    let validated = config::validate_project(&prepared)?;
    let PreparedPostClassicPrelude {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepared;
    let objects = objects
        .into_iter()
        .zip(validated)
        .map(|(predecessor, configs)| {
            let records = prepare_object(&predecessor, &configs, scale, cancel)?;
            Ok(PostClassicTopSplitPrintObject {
                predecessor,
                records,
            })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(PreparedPostClassicTopSplit {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}

fn prepare_object(
    predecessor: &PostClassicPreludePrintObject,
    configs: &[Option<ValidatedTopSplitConfig>],
    scale: crate::geometry::CoordinateScale,
    cancel: &crate::CancelToken,
) -> Result<Vec<Option<ClassicTopSplitRecord>>, SliceError> {
    let inputs = predecessor.object.as_parts().1;
    crate::parallel::par_range_map_try(inputs.len(), cancel, |index| {
        match (&inputs[index], &predecessor.records[index], &configs[index]) {
            (Some(input), Some(prelude), Some(config)) => {
                prepare_record(&predecessor.object, input, prelude, *config, scale).map(Some)
            }
            (None, None, None) => Ok(None),
            _ => unreachable!("Task 22O.2 preflight slots must remain aligned"),
        }
    })
}

fn prepare_record(
    object: &crate::project_slice::perimeters::types::PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    record: &ClassicPreludeRecord,
    config: ValidatedTopSplitConfig,
    scale: crate::geometry::CoordinateScale,
) -> Result<ClassicTopSplitRecord, SliceError> {
    let upper = selected_upper(object, input, config.interface_shells);
    let lower = record.printable_lower_slices.as_deref();
    let surfaces = record
        .surfaces
        .iter()
        .map(|surface| {
            prepare_surface(
                surface,
                SurfaceContext {
                    record,
                    config,
                    scale,
                    upper: upper.as_deref(),
                    lower,
                    has_upper_layer: input.upper_layer_index.is_some(),
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ClassicTopSplitRecord { surfaces })
}

struct SurfaceContext<'a> {
    record: &'a ClassicPreludeRecord,
    config: ValidatedTopSplitConfig,
    scale: crate::geometry::CoordinateScale,
    upper: Option<&'a [ExPolygon]>,
    lower: Option<&'a [ExPolygon]>,
    has_upper_layer: bool,
}

fn prepare_surface(
    surface: &PreparedClassicSurface,
    context: SurfaceContext<'_>,
) -> Result<PreparedTopSplitSurface, SliceError> {
    let SurfaceContext {
        record,
        config,
        scale,
        upper,
        lower,
        has_upper_layer,
    } = context;
    let initial_loop_number = surface.loop_number;
    if initial_loop_number < 0 {
        return Ok(PreparedTopSplitSurface {
            source_index: surface.source_index,
            initial_loop_number,
            effective_loop_number: initial_loop_number,
            normal_first_offset: Vec::new(),
            smaller_first_offset: Vec::new(),
            thin_walls: Vec::new(),
            remaining: surface.polygons.clone(),
            top_fills: Vec::new(),
            fill_clip: Vec::new(),
        });
    }

    let first = first_outer::apply(&surface.polygons, record, scale)?;
    if first.normal.is_empty() && first.smaller.is_empty() {
        return Ok(PreparedTopSplitSurface {
            source_index: surface.source_index,
            initial_loop_number,
            effective_loop_number: -1,
            normal_first_offset: first.normal,
            smaller_first_offset: first.smaller,
            thin_walls: first.thin_walls,
            remaining: Vec::new(),
            top_fills: Vec::new(),
            fill_clip: Vec::new(),
        });
    }

    let remaining = first.normal.clone();
    if config.only_one_wall_top
        && !surface.kind.is_bridge()
        && has_upper_layer
        && initial_loop_number != 0
    {
        let result = split::apply(
            &remaining,
            split::SplitContext {
                upper_slices: upper.expect("upper-layer gate must provide selected upper slices"),
                lower_slices: lower,
                record,
                config,
                scale,
            },
        )?;
        return Ok(PreparedTopSplitSurface {
            source_index: surface.source_index,
            initial_loop_number,
            effective_loop_number: initial_loop_number,
            normal_first_offset: first.normal,
            smaller_first_offset: first.smaller,
            thin_walls: first.thin_walls,
            remaining: result.non_top_polygons,
            top_fills: result.top_fills,
            fill_clip: result.fill_clip,
        });
    }

    Ok(PreparedTopSplitSurface {
        source_index: surface.source_index,
        initial_loop_number,
        effective_loop_number: initial_loop_number,
        normal_first_offset: first.normal,
        smaller_first_offset: first.smaller,
        thin_walls: first.thin_walls,
        remaining,
        top_fills: Vec::new(),
        fill_clip: Vec::new(),
    })
}

fn selected_upper(
    object: &crate::project_slice::perimeters::types::PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    interface_shells: bool,
) -> Option<Vec<ExPolygon>> {
    if interface_shells {
        object.upper_same_region_surfaces(input).map(|surfaces| {
            surfaces
                .iter()
                .map(|surface| surface.as_parts().1.clone())
                .collect()
        })
    } else {
        object.upper_slices(input).map(|slices| slices.to_vec())
    }
}
