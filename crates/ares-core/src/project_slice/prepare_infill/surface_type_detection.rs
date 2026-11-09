mod cracks;
mod extra_bridge;
mod geometry;
mod preflight;
mod stage;
mod top_expansion;

pub(in crate::project_slice) mod types;

pub(in crate::project_slice) use types::{
    PreparedPostSurfaceTypeDetection, PreparedSurfaceTypeObject,
};

use crate::{
    SliceError,
    project_slice::{incomplete_sink, perimeters::layer_region::PreparedPostLayerRegionPerimeters},
};

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostLayerRegionPerimeters,
) -> Result<PreparedPostSurfaceTypeDetection, SliceError> {
    let staged = match stage::project(&prepared) {
        Ok(staged) => staged,
        Err(error) => {
            dispose_predecessor(prepared);
            return Err(error);
        }
    };
    let PreparedPostLayerRegionPerimeters {
        predecessor,
        objects,
    } = prepared;
    assert_eq!(objects.len(), staged.len());
    let objects = objects
        .into_iter()
        .zip(staged)
        .map(|(object, staged)| {
            assert_eq!(object.records.len(), staged.records.len());
            let records = object
                .records
                .into_iter()
                .zip(staged.records)
                .map(|(source, staged)| match (source, staged) {
                    (Some(source), Some(staged)) => Some(types::materialize_record(source, staged)),
                    (None, None) => None,
                    _ => unreachable!("staged O17 slots remain aligned with O16"),
                })
                .collect();
            PreparedSurfaceTypeObject { records }
        })
        .collect();
    Ok(PreparedPostSurfaceTypeDetection {
        predecessor: predecessor.into_classic(),
        objects,
    })
}

fn dispose_predecessor(prepared: PreparedPostLayerRegionPerimeters) {
    let PreparedPostLayerRegionPerimeters {
        predecessor,
        objects,
    } = prepared;
    for object in objects {
        incomplete_sink::consume_layer_region_perimeter_object(object);
    }
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor.into_classic());
}
