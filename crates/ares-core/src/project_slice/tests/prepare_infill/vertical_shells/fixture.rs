use crate::project_slice::prepare_infill::{
    fill_surfaces, surface_type_detection, vertical_shells,
};

pub(super) fn prepare(bytes: impl AsRef<[u8]>) -> vertical_shells::PreparedPostVerticalShellCache {
    vertical_shells::prepare(prepare_o18(bytes)).unwrap()
}

pub(super) fn prepare_o18(
    bytes: impl AsRef<[u8]>,
) -> fill_surfaces::PreparedPostFillSurfacePreparation {
    let detected = surface_type_detection::prepare(
        crate::project_slice::perimeters::prepare_post_layer_region_perimeters(bytes).unwrap(),
    )
    .unwrap();
    fill_surfaces::prepare(detected)
}
