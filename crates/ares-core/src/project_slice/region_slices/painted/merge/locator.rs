//! MultiMaterialSegmentation.cpp:1938-1997 IslandLocator.
use crate::geometry::{
    BoundingBox, ClipperError, ExPolygon, Point, clip_clipper_expolygons_with_subject_bbox,
    intersection_polygons_polygons_ex,
};

pub(super) struct IslandLocator<'a> {
    islands: &'a [ExPolygon],
    bounds: Vec<BoundingBox>,
    extent: Option<BoundingBox>,
    cell_width: i64,
    cell_height: i64,
    cells: Vec<Vec<usize>>,
}
impl<'a> IslandLocator<'a> {
    pub(super) fn new(islands: &'a [ExPolygon]) -> Self {
        let bounds = islands
            .iter()
            .map(|p| BoundingBox::from_expolygon(p).unwrap())
            .collect::<Vec<_>>();
        let extent = BoundingBox::from_expolygons(islands);
        let mut out = Self {
            islands,
            bounds,
            extent,
            cell_width: 1,
            cell_height: 1,
            cells: vec![Vec::new(); 64 * 64],
        };
        let Some(extent) = extent else { return out };
        out.cell_width = ((extent.max().x() - extent.min().x()) / 64 + 1).max(1);
        out.cell_height = ((extent.max().y() - extent.min().y()) / 64 + 1).max(1);
        for index in 0..out.bounds.len() {
            for cell in out.cells_for(out.bounds[index]) {
                out.cells[cell].push(index);
            }
        }
        out
    }
    fn cells_for(&self, bbox: BoundingBox) -> Vec<usize> {
        let origin = self.extent.unwrap().min();
        let x0 = ((bbox.min().x() - origin.x()) / self.cell_width).clamp(0, 63);
        let x1 = ((bbox.max().x() - origin.x()) / self.cell_width).clamp(0, 63);
        let y0 = ((bbox.min().y() - origin.y()) / self.cell_height).clamp(0, 63);
        let y1 = ((bbox.max().y() - origin.y()) / self.cell_height).clamp(0, 63);
        (y0..=y1)
            .flat_map(|y| (x0..=x1).map(move |x| (y * 64 + x) as usize))
            .collect()
    }
    pub(super) fn find(&self, piece: &ExPolygon, epsilon: i64) -> Result<Vec<usize>, ClipperError> {
        let bbox = BoundingBox::from_expolygon(piece).unwrap();
        let Some(extent) = self.extent else {
            return Ok(Vec::new());
        };
        if !extent.overlap(bbox) {
            return Ok(Vec::new());
        }
        let mut found = self
            .cells_for(bbox)
            .into_iter()
            .flat_map(|cell| self.cells[cell].iter().copied())
            .filter(|&i| self.bounds[i].overlap(bbox))
            .collect::<Vec<_>>();
        found.sort_unstable();
        found.dedup();
        if found.len() <= 1 {
            return Ok(found);
        }
        let mut trimmed = Vec::new();
        for i in found {
            let b = self.bounds[i];
            let common = BoundingBox::new(
                Point::new(
                    b.min().x().max(bbox.min().x()),
                    b.min().y().max(bbox.min().y()),
                ),
                Point::new(
                    b.max().x().min(bbox.max().x()),
                    b.max().y().min(bbox.max().y()),
                ),
            )
            .inflated(epsilon);
            let subject =
                clip_clipper_expolygons_with_subject_bbox(std::slice::from_ref(piece), common);
            let clip = clip_clipper_expolygons_with_subject_bbox(
                std::slice::from_ref(&self.islands[i]),
                common,
            );
            if !intersection_polygons_polygons_ex(&subject, &clip)?.is_empty() {
                trimmed.push(i);
            }
        }
        found = trimmed;
        Ok(found)
    }
}
