use crate::project_slice::{
    perimeters,
    prepare_infill::surface_type_detection::{self, PreparedPostSurfaceTypeDetection},
};

pub(super) fn prepare(bytes: impl AsRef<[u8]>) -> PreparedPostSurfaceTypeDetection {
    surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(bytes.as_ref()).unwrap(),
    )
    .unwrap()
}
