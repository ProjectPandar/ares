use super::{consume_appended_collections, consume_expolygon, consume_gap_entity};
use crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject;

#[inline(never)]
pub(in crate::project_slice) fn consume_object(object: PreparedSurfaceTypeObject) {
    for record in object.records.into_iter().flatten() {
        consume_appended_collections(record.perimeters);
        for entity in record.thin_fills {
            consume_gap_entity(entity);
        }
        for surface in record.slices.into_iter().chain(record.fill_surfaces) {
            let (_, expolygon, thickness, thickness_layers, bridge_angle, extra_perimeters) =
                surface.into_parts();
            let _ = (thickness, thickness_layers, bridge_angle, extra_perimeters);
            consume_expolygon(expolygon);
        }
        for expolygon in record.fill_expolygons {
            consume_expolygon(expolygon);
        }
        for expolygon in record.fill_no_overlap_expolygons {
            consume_expolygon(expolygon);
        }
    }
}
