use crate::SliceError;
use crate::geometry::{
    ExPolygon, JoinType, difference_ex, intersection_ex, offset_expolygon, offset_expolygons,
};
use crate::project_slice::region_slices::RegionSurface;

use super::geometry::{
    GeometryStep, expolygons, fresh, geometry_error, observe, paths, subtract_paths,
};

const MITER_LIMIT: f64 = 3.0;

pub(super) fn crack_threshold(external_width: i64) -> f32 {
    ((-external_width) as f64 * 1.5) as f32
}

pub(super) fn resolve(
    top: &mut Vec<RegionSurface>,
    bottom: &mut Vec<RegionSurface>,
    external_width: i64,
    has_lower_layer: bool,
) -> Result<(), SliceError> {
    if top.is_empty() || bottom.is_empty() {
        return Ok(());
    }
    observe(GeometryStep::CrackIntersection)?;
    let cracks = intersection_ex(&expolygons(top), &expolygons(bottom)).map_err(geometry_error)?;
    if cracks.is_empty() {
        return Ok(());
    }

    if has_lower_layer {
        let threshold = crack_threshold(external_width);
        for crack in &cracks {
            observe(GeometryStep::SingletonCrackErosion)?;
            let eroded = offset_expolygon(crack, threshold, JoinType::Miter, MITER_LIMIT)
                .map_err(geometry_error)?;
            if eroded.is_empty() && !belongs_to_large_bottom(crack, bottom, threshold)? {
                remove_from_bottom(crack, bottom, -threshold)?;
            }
        }
    }

    let kind = top[0].as_parts().0;
    let rebuilt = subtract_paths(&paths(top), &paths(bottom))?;
    *top = fresh(kind, rebuilt);
    Ok(())
}

pub(super) fn belongs_to_large_bottom(
    crack: &ExPolygon,
    bottom: &[RegionSurface],
    threshold: f32,
) -> Result<bool, SliceError> {
    for surface in bottom {
        let expolygon = surface.as_parts().1;
        observe(GeometryStep::ContainmentDifference)?;
        let outside = difference_ex(std::slice::from_ref(crack), std::slice::from_ref(expolygon))
            .map_err(geometry_error)?;
        if outside.is_empty() && expolygon.area() > crack.area() * 2.0 {
            observe(GeometryStep::ResidualDifference)?;
            let residual =
                difference_ex(std::slice::from_ref(expolygon), std::slice::from_ref(crack))
                    .map_err(geometry_error)?;
            observe(GeometryStep::CollectionResidualErosion)?;
            if !offset_expolygons(&residual, threshold, JoinType::Miter, MITER_LIMIT)
                .map_err(geometry_error)?
                .is_empty()
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn remove_from_bottom(
    crack: &ExPolygon,
    bottom: &mut Vec<RegionSurface>,
    expansion: f32,
) -> Result<(), SliceError> {
    observe(GeometryStep::SingletonCrackExpansion)?;
    let expanded =
        offset_expolygon(crack, expansion, JoinType::Miter, MITER_LIMIT).map_err(geometry_error)?;
    let mut rebuilt = Vec::new();
    for surface in bottom.iter() {
        let kind = surface.as_parts().0;
        observe(GeometryStep::BottomSubtraction)?;
        let difference = difference_ex(std::slice::from_ref(surface.as_parts().1), &expanded)
            .map_err(geometry_error)?;
        rebuilt.extend(fresh(kind, difference));
    }
    *bottom = rebuilt;
    Ok(())
}
