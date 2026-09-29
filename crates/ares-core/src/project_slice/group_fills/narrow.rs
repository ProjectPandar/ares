mod filter;
mod split;
mod trace;

use crate::{
    SliceError, geometry::CoordinateScale, project_slice::region_slices::RegionSurfaceKind,
};

use super::{RepresentativeSurface, SurfaceFill, SurfaceFillPattern, geometry_error};

pub(super) struct Context {
    pub(super) enabled: bool,
    pub(super) layer_id: usize,
    pub(super) scale: CoordinateScale,
}

pub(super) fn apply(fills: &mut Vec<SurfaceFill>, context: Context) -> Result<(), SliceError> {
    if !context.enabled {
        return Ok(());
    }

    let original_count = fills.len();
    for index in 0..original_count {
        if fills[index].representative.kind != RegionSurfaceKind::InternalSolid {
            continue;
        }
        let (normal, narrow) =
            split::split(context.layer_id, &fills[index], context.scale).map_err(geometry_error)?;
        if narrow.is_empty() {
            continue;
        }
        if normal.is_empty() {
            fills[index].params.pattern = SurfaceFillPattern::ConcentricInternal;
            continue;
        }

        let appended = {
            let source = &mut fills[index];
            source.expolygons = normal;
            let mut params = source.params;
            params.pattern = SurfaceFillPattern::ConcentricInternal;
            SurfaceFill {
                region_id: source.region_id,
                representative: RepresentativeSurface {
                    kind: RegionSurfaceKind::InternalSolid,
                    thickness: source.representative.thickness,
                    thickness_layers: 1,
                    bridge_angle: -1.0,
                    extra_perimeters: 0,
                },
                expolygons: narrow,
                params,
                region_id_group: source.region_id_group.clone(),
                no_overlap_expolygons: source.no_overlap_expolygons.clone(),
            }
        };
        fills.push(appended);
    }
    Ok(())
}
