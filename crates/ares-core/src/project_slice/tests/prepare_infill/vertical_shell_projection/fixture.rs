use crate::project_slice::prepare_infill::{
    fill_surfaces, surface_type_detection, vertical_shell_projection, vertical_shells,
};

pub(in crate::project_slice::tests::prepare_infill) fn prepare(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_projection::PreparedPostVerticalShellProjection {
    vertical_shell_projection::prepare(prepare_o19(bytes)).unwrap()
}

pub(in crate::project_slice::tests::prepare_infill) fn prepare_o19(
    bytes: impl AsRef<[u8]>,
) -> vertical_shells::PreparedPostVerticalShellCache {
    let detected = surface_type_detection::prepare(
        crate::project_slice::perimeters::prepare_post_layer_region_perimeters(bytes.as_ref())
            .unwrap(),
    )
    .unwrap();
    vertical_shells::prepare(fill_surfaces::prepare(detected)).unwrap()
}
