//! `init_arrange_params`'s skirt offset (`ArrangeJob.cpp:993-1001` at v2.5.0-beta `345efec64`):
//! `Print::object_skirt_offset()` (`print::object_skirt`) of the current plate's print
//! (`get_current_fff_print`). See `docs/architecture/web-plater-arrange-object-skirt.md`.

use std::collections::BTreeSet;

use crate::plater::PlaterSession;
use crate::plater::transformation;
use crate::print::object_skirt::{
    FirstObject, is_all_objects_are_short, object_skirt_offset, print_object_height,
};
use crate::profiles::VendorType;
use crate::{Point3d, ProjectObject, ProjectVolumeType, SliceError};

/// `raw_bounding_box().size().z()` (`Model.cpp:1761-1775`): the model parts' Z extent through the
/// first instance's matrix without its offset, each mesh vertex a `float` cast to `double`.
fn raw_height(object: &ProjectObject) -> f64 {
    let matrix = transformation::with_offset(object.instances()[0].transform(), [0.; 3]);
    let (mut min, mut max) = (f64::MAX, f64::MIN);
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let volume_matrix = matrix.then(volume.transform());
        for vertex in volume.mesh().vertices() {
            let [x, y, z] = [vertex.x as f32, vertex.y as f32, vertex.z as f32].map(f64::from);
            let z = volume_matrix.transform_point(Point3d::new(x, y, z)).z;
            min = min.min(z);
            max = max.max(z);
        }
    }
    max - min
}

impl PlaterSession {
    /// The current plate's print: the objects with an instance on it, in model order, their own
    /// `line_width` and `support_filament` over the process ones, and `is_BBL_printer` from the
    /// vendor type (`is_bbl_vendor`).
    pub(super) fn current_print_skirt_offset(&self) -> Result<f32, SliceError> {
        let settings = self.project.settings();
        let objects: BTreeSet<usize> = self.plates.plates[self.current_plate]
            .instances
            .iter()
            .map(|&(object, _)| object)
            .collect();
        let first = objects.first().map(|&index| {
            let own = self.project.objects()[index].object_overrides();
            FirstObject {
                line_width: own.line_width.unwrap_or(settings.process.object.line_width),
                support_filament: own
                    .support_filament
                    .map_or(settings.process.object.support_filament.0, |id| id.0),
            }
        });
        let heights = objects
            .iter()
            .map(|&index| print_object_height(raw_height(&self.project.objects()[index])));
        let all_short =
            is_all_objects_are_short(heights, settings.printer.remaining.nozzle_height.0);
        let is_bbl = self.vendor_type == VendorType::MarlinBbl;
        let (offset, _) = object_skirt_offset(settings, first, all_short, is_bbl)?;
        Ok(offset)
    }
}
