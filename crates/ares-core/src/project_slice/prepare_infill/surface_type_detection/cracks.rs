use crate::SliceError;
use crate::geometry::{
    BoundingBox, ExPolygon, JoinType, difference_ex, intersection_ex_refs, offset_expolygon,
    offset_expolygons,
};
use crate::project_slice::region_slices::RegionSurface;

use super::geometry::{expolygon_refs, fresh, geometry_error, subtract_surface_refs};

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
    let (Some(top_bounds), Some(bottom_bounds)) = (surfaces_bounds(top), surfaces_bounds(bottom))
    else {
        return Ok(());
    };
    if !top_bounds.overlap(bottom_bounds) {
        return Ok(());
    }
    let cracks = intersection_ex_refs(&expolygon_refs(top), &expolygon_refs(bottom))
        .map_err(geometry_error)?;
    if cracks.is_empty() {
        return Ok(());
    }

    if has_lower_layer {
        let threshold = crack_threshold(external_width);
        for crack in &cracks {
            let eroded = offset_expolygon(crack, threshold, JoinType::Miter, MITER_LIMIT)
                .map_err(geometry_error)?;
            if eroded.is_empty() && !belongs_to_large_bottom(crack, bottom, threshold)? {
                remove_from_bottom(crack, bottom, -threshold)?;
            }
        }
    }

    let kind = top[0].as_parts().0;
    let rebuilt = subtract_surface_refs(&expolygon_refs(top), &expolygon_refs(bottom))?;
    *top = fresh(kind, rebuilt);
    Ok(())
}

pub(super) fn belongs_to_large_bottom(
    crack: &ExPolygon,
    bottom: &[RegionSurface],
    threshold: f32,
) -> Result<bool, SliceError> {
    let crack_area_double = crack.area() * 2.0;
    let crack_bounds = BoundingBox::from_expolygon(crack);
    for surface in bottom {
        let expolygon = surface.as_parts().1;
        if !matches!(
            expolygon.area().partial_cmp(&crack_area_double),
            Some(std::cmp::Ordering::Greater)
        ) {
            continue;
        }
        if crack_area_double > 0.0
            && let (Some(crack_bb), Some(bottom_bb)) =
                (crack_bounds, BoundingBox::from_expolygon(expolygon))
            && !crack_bb.overlap(bottom_bb)
        {
            continue;
        }
        let outside = difference_ex(std::slice::from_ref(crack), std::slice::from_ref(expolygon))
            .map_err(geometry_error)?;
        if outside.is_empty() {
            let residual =
                difference_ex(std::slice::from_ref(expolygon), std::slice::from_ref(crack))
                    .map_err(geometry_error)?;
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

fn surfaces_bounds(surfaces: &[RegionSurface]) -> Option<BoundingBox> {
    let mut combined: Option<BoundingBox> = None;
    for surface in surfaces {
        let Some(bounds) = BoundingBox::from_expolygon(surface.as_parts().1) else {
            continue;
        };
        combined = Some(match combined {
            Some(mut current) => {
                current.merge_point(bounds.min());
                current.merge_point(bounds.max());
                current
            }
            None => bounds,
        });
    }
    combined
}

fn remove_from_bottom(
    crack: &ExPolygon,
    bottom: &mut Vec<RegionSurface>,
    expansion: f32,
) -> Result<(), SliceError> {
    let expanded =
        offset_expolygon(crack, expansion, JoinType::Miter, MITER_LIMIT).map_err(geometry_error)?;
    let mut rebuilt = Vec::new();
    for surface in bottom.iter() {
        let kind = surface.as_parts().0;
        let difference = difference_ex(std::slice::from_ref(surface.as_parts().1), &expanded)
            .map_err(geometry_error)?;
        rebuilt.extend(fresh(kind, difference));
    }
    *bottom = rebuilt;
    Ok(())
}
