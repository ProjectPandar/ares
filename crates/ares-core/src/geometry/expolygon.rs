use super::Polygon;

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

    pub(crate) fn into_parts(self) -> (Polygon, Vec<Polygon>) {
        (self.contour, self.holes)
    }
}
