#[cfg(test)]
mod tests;

use crate::{
    RegionOptions,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::surface_type_detection::{PreparedPostSurfaceTypeDetection, PreparedSurfaceTypeObject};

pub(in crate::project_slice) struct PreparedPostFillSurfacePreparation {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
}

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static RETAGS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostSurfaceTypeDetection,
) -> PreparedPostFillSurfacePreparation {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

    validate_alignment(&prepared);
    let PreparedPostSurfaceTypeDetection {
        predecessor,
        mut objects,
    } = prepared;
    let spiral_mode = predecessor.resolved.views.full.process.print.spiral_mode.0;
    for (object, traversal) in objects.iter_mut().zip(&predecessor.objects) {
        let input_object = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        for (record, input) in object.records.iter_mut().zip(&input_object.records) {
            if let (Some(record), Some(input)) = (record, input) {
                prepare_record(
                    &mut record.fill_surfaces,
                    input_object.region_options(input),
                    spiral_mode,
                );
            }
        }
    }
    PreparedPostFillSurfacePreparation {
        predecessor,
        objects,
    }
}

pub(in crate::project_slice) fn validate_alignment(prepared: &PreparedPostSurfaceTypeDetection) {
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for (object, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        let input_object = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        assert_eq!(object.records.len(), input_object.records.len());
        let identity = input_object.identity();
        for (record, input) in object.records.iter().zip(&input_object.records) {
            match (record, input) {
                (Some(_), Some(input)) => {
                    assert_eq!((input.source_object_index, input.transform_index), identity);
                    assert_eq!(input.compatible_region_ids, [input.region_id]);
                }
                (None, None) => {}
                _ => panic!("O18 slots remain aligned with O17 and the Classic prelude"),
            }
        }
    }
}

pub(super) fn prepare_record(
    surfaces: &mut [RegionSurface],
    options: &RegionOptions,
    spiral_mode: bool,
) {
    if !spiral_mode && options.top_shell_layers.0 == 0 {
        for surface in &mut *surfaces {
            if surface.as_parts().0 == RegionSurfaceKind::Top {
                retag(surface, RegionSurfaceKind::Internal);
            }
        }
    }
    if options.bottom_shell_layers.0 == 0 {
        for surface in &mut *surfaces {
            if matches!(
                surface.as_parts().0,
                RegionSurfaceKind::Bottom | RegionSurfaceKind::BottomBridge
            ) {
                retag(surface, RegionSurfaceKind::Internal);
            }
        }
    }
    if !spiral_mode && (options.sparse_infill_density.0 - 100.0).abs() < 1e-4 {
        for surface in surfaces {
            if surface.as_parts().0 == RegionSurfaceKind::Internal {
                retag(surface, RegionSurfaceKind::InternalSolid);
            }
        }
    }
}

fn retag(surface: &mut RegionSurface, kind: RegionSurfaceKind) {
    surface.retag(kind);
    #[cfg(test)]
    RETAGS.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
pub(in crate::project_slice) fn retags() -> usize {
    RETAGS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_retags() {
    RETAGS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_invocations() {
    INVOCATIONS.with(|count| count.set(0));
}
