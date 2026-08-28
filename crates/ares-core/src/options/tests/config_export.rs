mod fixture;
mod inventory;
mod nullable;
mod special;
mod value;

use crate::{
    ProjectSettings, SliceError,
    options::{
        config_export::write_config_block,
        project_config_views::{ProjectConfigViews, resolve_project_config_views},
    },
};

fn views(settings: ProjectSettings) -> ProjectConfigViews {
    resolve_project_config_views(settings).unwrap()
}

fn block(settings: ProjectSettings, plate_index: usize) -> Result<Vec<u8>, SliceError> {
    let views = views(settings);
    block_from_views(&views, plate_index)
}

fn block_from_views(
    views: &ProjectConfigViews,
    plate_index: usize,
) -> Result<Vec<u8>, SliceError> {
    let mut output = Vec::new();
    write_config_block(views, &Default::default(), plate_index, &mut output)?;
    Ok(output)
}

fn assignment_lines(bytes: &[u8], key: &str) -> Vec<String> {
    let prefix = format!("; {key} = ");
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .map(str::to_owned)
        .collect()
}
