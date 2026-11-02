mod config;
mod iterate;
mod thin_holes;
mod types;

pub(in crate::project_slice) use types::{
    ClassicOnionRecord, PostClassicOnionPrintObject, PreparedPostClassicOnion, RawShellDepth,
};

use crate::{SliceError, geometry::CoordinateScale};

use self::{config::ValidatedOnionConfig, iterate::IterationInput, types::PreparedOnionSurface};
use super::{ClassicTopSplitRecord, PostClassicTopSplitPrintObject, PreparedPostClassicTopSplit};

pub(super) fn finish(
    prepared: PreparedPostClassicTopSplit,
) -> Result<PreparedPostClassicOnion, SliceError> {
    let validated = config::validate_project(&prepared)?;
    let PreparedPostClassicTopSplit {
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
            Ok(PostClassicOnionPrintObject {
                predecessor,
                records,
            })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(PreparedPostClassicOnion {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}

fn prepare_object(
    predecessor: &PostClassicTopSplitPrintObject,
    configs: &[Option<ValidatedOnionConfig>],
    scale: CoordinateScale,
) -> Result<Vec<Option<ClassicOnionRecord>>, SliceError> {
    predecessor
        .records
        .iter()
        .zip(configs)
        .map(|(record, config)| match (record, config) {
            (Some(record), Some(config)) => prepare_record(record, *config, scale).map(Some),
            (None, None) => Ok(None),
            _ => unreachable!("Task 22O.3 preflight slots must remain aligned"),
        })
        .collect()
}

fn prepare_record(
    record: &ClassicTopSplitRecord,
    config: ValidatedOnionConfig,
    scale: CoordinateScale,
) -> Result<ClassicOnionRecord, SliceError> {
    let surfaces = record
        .surfaces
        .iter()
        .map(|surface| {
            let thin_wall_holes = thin_holes::classify(
                &surface.normal_first_offset,
                &surface.smaller_first_offset,
                config.gap_fill_room,
                scale,
            )?;
            let narrow_wall_holes = if let Some(room) = config.inner_wall_room {
                thin_holes::classify(
                    &surface.normal_first_offset,
                    &surface.smaller_first_offset,
                    room,
                    scale,
                )?
            } else {
                vec![false; thin_wall_holes.len()]
            };
            let result = iterate::apply(IterationInput {
                initial_loop_number: surface.initial_loop_number,
                loop_number: surface.effective_loop_number,
                normal_first_offset: &surface.normal_first_offset,
                smaller_first_offset: &surface.smaller_first_offset,
                remaining: &surface.remaining,
                config,
            })?;
            Ok(PreparedOnionSurface {
                source_index: surface.source_index,
                initial_loop_number: surface.initial_loop_number,
                effective_loop_number: result.effective_loop_number,
                shells: result.shells,
                thin_wall_holes,
                narrow_wall_holes,
                last: result.last,
                gaps: result.gaps,
                thin_walls: surface.thin_walls.clone(),
            })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(ClassicOnionRecord { surfaces })
}
