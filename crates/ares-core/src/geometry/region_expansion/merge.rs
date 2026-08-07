use super::RegionExpansion;
use super::wave_seeds::sample_in_expolygons;
use crate::geometry::clipper::ordering::fixed_msvc_sort_by;
use crate::geometry::clipper::union_safety_offset_ex;
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon};

pub(crate) fn merge_expansions_into_expolygons(
    src: Vec<ExPolygon>,
    expanded: Vec<RegionExpansion>,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut order = (0..expanded.len()).collect::<Vec<_>>();
    fixed_msvc_sort_by(&mut order, |left, right| {
        expanded[*left].src_id < expanded[*right].src_id
    });
    let mut source = expanded.into_iter().map(Some).collect::<Vec<_>>();
    let mut expanded = order
        .into_iter()
        .map(move |index| source[index].take().expect("sort permutation is unique"))
        .peekable();

    let mut output = Vec::with_capacity(src.len());
    let mut src = src.into_iter();
    let mut src_index = 0_usize;
    let mut polygons = Vec::new();
    while let Some(expansion) = expanded.next() {
        let group_src_id = expansion.src_id;
        let group_src_index = group_src_id as usize;
        while src_index < group_src_index {
            output.push(src.next().expect("source expansion ID is in range"));
            src_index += 1;
        }

        polygons.clear();
        polygons.push(expansion.polygon);
        while expanded
            .peek()
            .is_some_and(|next| next.src_id == group_src_id)
        {
            polygons.push(expanded.next().unwrap().polygon);
        }

        let source = src.next().expect("source expansion ID is in range");
        src_index += 1;
        let (contour, holes) = source.into_parts();
        let sample = contour.points()[0];
        polygons.push(contour);
        polygons.extend(holes);

        let mut merged = union_safety_offset_ex(&polygons)?;
        match merged.len() {
            0 => {}
            1 => output.push(merged.pop().unwrap()),
            _ => {
                let selected = sample_in_expolygons(&merged, sample, scale);
                debug_assert!(selected.is_some());
                if let Some(index) = selected {
                    output.push(merged.swap_remove(index));
                }
            }
        }
    }
    output.extend(src);
    Ok(output)
}
