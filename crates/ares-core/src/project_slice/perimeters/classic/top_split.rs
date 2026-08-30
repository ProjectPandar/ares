mod config;
mod first_outer;
mod split;
mod types;

pub(in crate::project_slice) use types::{
    ClassicTopSplitRecord, PostClassicTopSplitPrintObject, PreparedPostClassicTopSplit,
    PreparedTopSplitSurface, TopSplitOutcome, TopSplitUpperSource,
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
            let records = prepare_object(&predecessor, &configs, scale)?;
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
) -> Result<Vec<Option<ClassicTopSplitRecord>>, SliceError> {
    let inputs = predecessor.object.as_parts().1;
    inputs
        .iter()
        .zip(&predecessor.records)
        .zip(configs)
        .map(
            |((input, prelude), config)| match (input, prelude, config) {
                (Some(input), Some(prelude), Some(config)) => {
                    prepare_record(&predecessor.object, input, prelude, *config, scale).map(Some)
                }
                (None, None, None) => Ok(None),
                _ => unreachable!("Task 22O.2 preflight slots must remain aligned"),
            },
        )
        .collect()
}

fn prepare_record(
    object: &crate::project_slice::perimeters::types::PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    record: &ClassicPreludeRecord,
    config: ValidatedTopSplitConfig,
    scale: crate::geometry::CoordinateScale,
) -> Result<ClassicTopSplitRecord, SliceError> {
    let upper_source = if config.interface_shells {
        TopSplitUpperSource::SameRegion
    } else {
        TopSplitUpperSource::WholeLayer
    };
    let upper = selected_upper(object, input, upper_source);
    let lower = object.lower_slices(input);
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
                    upper_source,
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
    upper_source: TopSplitUpperSource,
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
        upper_source,
    } = context;
    let initial_loop_number = surface.loop_number;
    if initial_loop_number < 0 {
        let mut output = empty_surface(
            surface,
            initial_loop_number,
            TopSplitOutcome::NoLoops,
            upper_source,
        );
        output.remaining = surface.polygons.clone();
        return Ok(output);
    }

    let first = first_outer::apply(&surface.polygons, record, scale)?;
    if first.normal.is_empty() && first.smaller.is_empty() {
        return Ok(PreparedTopSplitSurface {
            source_index: surface.source_index,
            initial_loop_number,
            effective_loop_number: -1,
            normal_first_offset: first.normal,
            smaller_first_offset: first.smaller,
            remaining: Vec::new(),
            top_fills: Vec::new(),
            fill_clip: Vec::new(),
            outcome: TopSplitOutcome::Collapsed,
            upper_source,
        });
    }

    let mut remaining = first.normal.clone();
    let outcome = if !config.only_one_wall_top {
        TopSplitOutcome::Disabled
    } else if surface.kind.is_bridge() {
        TopSplitOutcome::Bridge
    } else if !has_upper_layer {
        TopSplitOutcome::NoUpperLayer
    } else if initial_loop_number == 0 {
        TopSplitOutcome::OneLoop
    } else {
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
        remaining = result.non_top_polygons;
        return Ok(PreparedTopSplitSurface {
            source_index: surface.source_index,
            initial_loop_number,
            effective_loop_number: initial_loop_number,
            normal_first_offset: first.normal,
            smaller_first_offset: first.smaller,
            remaining,
            top_fills: result.top_fills,
            fill_clip: result.fill_clip,
            outcome: TopSplitOutcome::Applied,
            upper_source,
        });
    };

    Ok(PreparedTopSplitSurface {
        source_index: surface.source_index,
        initial_loop_number,
        effective_loop_number: initial_loop_number,
        normal_first_offset: first.normal,
        smaller_first_offset: first.smaller,
        remaining,
        top_fills: Vec::new(),
        fill_clip: Vec::new(),
        outcome,
        upper_source,
    })
}

fn empty_surface(
    surface: &PreparedClassicSurface,
    loop_number: i32,
    outcome: TopSplitOutcome,
    upper_source: TopSplitUpperSource,
) -> PreparedTopSplitSurface {
    PreparedTopSplitSurface {
        source_index: surface.source_index,
        initial_loop_number: loop_number,
        effective_loop_number: loop_number,
        normal_first_offset: Vec::new(),
        smaller_first_offset: Vec::new(),
        remaining: Vec::new(),
        top_fills: Vec::new(),
        fill_clip: Vec::new(),
        outcome,
        upper_source,
    }
}

fn selected_upper(
    object: &crate::project_slice::perimeters::types::PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    source: TopSplitUpperSource,
) -> Option<Vec<ExPolygon>> {
    match source {
        TopSplitUpperSource::WholeLayer => object.upper_slices(input).map(|slices| slices.to_vec()),
        TopSplitUpperSource::SameRegion => {
            object.upper_same_region_surfaces(input).map(|surfaces| {
                surfaces
                    .iter()
                    .map(|surface| surface.as_parts().1.clone())
                    .collect()
            })
        }
    }
}
