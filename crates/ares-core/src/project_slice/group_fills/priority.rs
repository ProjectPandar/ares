use crate::{
    SliceError,
    geometry::{
        ClipperError, ExPolygon, Polygon, difference_polygons_ex_with_safety_offset,
        union_safety_offset_ex,
    },
};

use super::SurfaceFill;

pub(super) fn apply(fills: &mut [SurfaceFill]) -> Result<(), SliceError> {
    let mut preceding = Vec::new();
    let fill_count = fills.len();
    for (index, fill) in fills.iter_mut().enumerate() {
        if fill.expolygons.is_empty() {
            continue;
        }
        if fill.expolygons.len() > 1 || !preceding.is_empty() {
            let raw = flatten_owned(std::mem::take(&mut fill.expolygons));
            fill.expolygons = if preceding.is_empty() {
                union_safety_offset_ex(&raw).map_err(geometry_error)?
            } else {
                difference_polygons_ex_with_safety_offset(&raw, &preceding)
                    .map_err(geometry_error)?
            };
            preceding.extend(raw);
        } else if index + 1 < fill_count {
            preceding.extend(flatten_borrowed(&fill.expolygons));
        }
    }
    Ok(())
}

fn flatten_owned(expolygons: Vec<ExPolygon>) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    for expolygon in expolygons {
        let (contour, holes) = expolygon.into_parts();
        polygons.push(contour);
        polygons.extend(holes);
    }
    polygons
}

fn flatten_borrowed(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    for expolygon in expolygons {
        polygons.push(expolygon.contour().clone());
        polygons.extend(expolygon.holes().iter().cloned());
    }
    polygons
}

fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "fill-grouping polygon coordinate is outside the supported Clipper range".to_owned(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("fill-grouping operations contain only closed polygon paths")
        }
    }
}
