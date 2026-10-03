//! `SupportMaterial.cpp::trim_top_contacts_by_bottom_contacts`.

use super::{
    bottom_contacts::BottomContactLayer, contact_layer::TopContactLayer,
    layer_index::idx_lower_or_equal,
};
use crate::geometry::{ClipperError, difference_polygons_paths};

pub(super) fn trim_top_contacts_by_bottom_contacts(
    bottoms: &[BottomContactLayer],
    tops: &mut [TopContactLayer],
) -> Result<(), ClipperError> {
    let mut first_bottom = -2;
    for top in tops.iter_mut().rev() {
        let index = idx_lower_or_equal(bottoms, &mut first_bottom, |bottom| {
            bottom.print_z - bottom.height - 1e-4 <= top.bottom_z
        });
        for index in (0..=index).rev() {
            let bottom = &bottoms[index as usize];
            if top.print_z >= bottom.print_z + 1e-4 {
                break;
            }
            top.polygons = difference_polygons_paths(&top.polygons, &bottom.polygons)?;
        }
    }
    Ok(())
}
