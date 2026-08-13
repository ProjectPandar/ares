#[cfg(test)]
mod tests;

use crate::project_slice::region_slices::{RegionSurface, RegionSurfaceKind};

pub(in crate::project_slice) fn commit_region_bridge_surfaces(
    mut fill_surfaces: Vec<RegionSurface>,
    new_surfaces: &[RegionSurface],
) -> Vec<RegionSurface> {
    fill_surfaces.retain(|surface| {
        !matches!(
            surface.as_parts().0,
            RegionSurfaceKind::InternalSolid | RegionSurfaceKind::Internal
        )
    });
    fill_surfaces.extend_from_slice(new_surfaces);
    fill_surfaces
}
