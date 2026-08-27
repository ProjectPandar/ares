use crate::project_slice::prepare_infill::{
    fill_surfaces::{self, PreparedPostFillSurfacePreparation},
    surface_type_detection,
};

pub(super) fn prepare(bytes: impl AsRef<[u8]>) -> PreparedPostFillSurfacePreparation {
    let detected = surface_type_detection::prepare(
        crate::project_slice::perimeters::prepare_post_layer_region_perimeters(bytes.as_ref())
            .unwrap(),
    )
    .unwrap();
    fill_surfaces::prepare(detected)
}
