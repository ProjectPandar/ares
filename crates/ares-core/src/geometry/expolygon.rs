use super::{Line, Point, Polygon};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExPolygon {
    contour: Polygon,
    holes: Vec<Polygon>,
}

impl ExPolygon {
    pub(crate) fn new(contour: Polygon, holes: Vec<Polygon>) -> Self {
        Self { contour, holes }
    }

    pub(crate) fn contour(&self) -> &Polygon {
        &self.contour
    }

    pub(crate) fn holes(&self) -> &[Polygon] {
        &self.holes
    }
    pub(crate) fn holes_mut(&mut self) -> &mut [Polygon] {
        &mut self.holes
    }

    pub(crate) fn into_parts(self) -> (Polygon, Vec<Polygon>) {
        (self.contour, self.holes)
    }

    pub(crate) fn douglas_peucker(&mut self, tolerance: f64) {
        self.contour.douglas_peucker(tolerance);
        for hole in &mut self.holes {
            hole.douglas_peucker(tolerance);
        }
    }

    pub(crate) fn lines(&self) -> Vec<Line> {
        let mut lines = self.contour.lines();
        for hole in &self.holes {
            lines.extend(hole.lines());
        }
        lines
    }

    pub(crate) fn on_boundary(&self, point: Point, epsilon: f64) -> bool {
        self.contour.on_boundary(point, epsilon)
            || self
                .holes
                .iter()
                .any(|hole| hole.on_boundary(point, epsilon))
    }

    pub(crate) fn area(&self) -> f64 {
        self.holes
            .iter()
            .fold(self.contour.area(), |area, hole| area + hole.area())
    }
}

pub(crate) fn keep_largest_contour_only(expolygons: &mut Vec<ExPolygon>) {
    if expolygons.len() <= 1 {
        return;
    }

    let mut largest_area = 0.0;
    let mut largest_index = None;
    for (index, expolygon) in expolygons.iter().enumerate() {
        let area = expolygon.contour.area();
        if area > largest_area {
            largest_area = area;
            largest_index = Some(index);
        }
    }

    let largest = expolygons.swap_remove(
        largest_index.expect("multiple ExPolygons must include a positive signed contour"),
    );
    expolygons.clear();
    expolygons.push(largest);
}
