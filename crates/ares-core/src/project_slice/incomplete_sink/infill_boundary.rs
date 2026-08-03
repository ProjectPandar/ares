use super::{consume_expolygon, consume_gap_extrusion_surface};
use crate::project_slice::perimeters::classic::infill_boundary::PreparedInfillBoundaryObject;

#[inline(never)]
pub(in crate::project_slice) fn consume_infill_boundary_object(
    object: PreparedInfillBoundaryObject,
) {
    for record in object.records.into_iter().flatten() {
        for surface in record.surfaces {
            consume_gap_extrusion_surface(surface);
        }
        for surface in record.fill_surfaces {
            let (_, expolygon, thickness, thickness_layers, bridge_angle, extra_perimeters) =
                surface.into_parts();
            let _ = (thickness, thickness_layers, bridge_angle, extra_perimeters);
            consume_expolygon(expolygon);
        }
        for expolygon in record.fill_no_overlap {
            consume_expolygon(expolygon);
        }
        for overlap in record.overlap {
            let _ = (
                overlap.source_index,
                overlap.inset,
                overlap.infill_peri_overlap,
                overlap.top_infill_peri_overlap,
                overlap.min_perimeter_infill_spacing,
                overlap.scaled_resolution,
            );
        }
    }
}
