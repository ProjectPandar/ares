mod config;
mod iterate;
mod types;

pub(in crate::project_slice) use types::{
    ClassicOnionRecord, PostClassicOnionPrintObject, PreparedPostClassicOnion, RawShellDepth,
};

use crate::SliceError;

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
            let records = prepare_object(&predecessor, &configs)?;
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
) -> Result<Vec<Option<ClassicOnionRecord>>, SliceError> {
    predecessor
        .records
        .iter()
        .zip(configs)
        .map(|(record, config)| match (record, config) {
            (Some(record), Some(config)) => prepare_record(record, *config).map(Some),
            (None, None) => Ok(None),
            _ => unreachable!("Task 22O.3 preflight slots must remain aligned"),
        })
        .collect()
}

fn prepare_record(
    record: &ClassicTopSplitRecord,
    config: ValidatedOnionConfig,
) -> Result<ClassicOnionRecord, SliceError> {
    let surfaces = record
        .surfaces
        .iter()
        .map(|surface| {
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
                last: result.last,
                gaps: result.gaps,
            })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(ClassicOnionRecord { surfaces })
}
