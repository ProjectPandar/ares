use crate::geometry::Point;

pub(crate) struct BrimPlan {
    pub(crate) area: Vec<crate::geometry::ExPolygon>,
    pub(crate) global: bool,
    pub(crate) owner_collection: bool,
    pub(crate) paths: Vec<Vec<Point>>,
    pub(crate) covered_hull: Vec<Point>,
    pub(crate) width: f32,
    pub(crate) spacing: f32,
    pub(crate) height: f32,
    pub(crate) mm3_per_mm: f64,
}
