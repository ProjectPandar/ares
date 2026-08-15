use crate::geometry::{Line, Point, Polygon};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PolygonSegmentIndex {
    pub(super) polygon_index: usize,
    pub(super) point_index: usize,
}

impl PolygonSegmentIndex {
    pub(super) fn from(self, polygons: &[Polygon]) -> Point {
        polygons[self.polygon_index].points()[self.point_index]
    }

    pub(super) fn to(self, polygons: &[Polygon]) -> Point {
        let points = polygons[self.polygon_index].points();
        points[(self.point_index + 1) % points.len()]
    }

    pub(super) fn line(self, polygons: &[Polygon]) -> Line {
        Line::new(self.from(polygons), self.to(polygons))
    }

    pub(super) fn source_point(
        self,
        polygons: &[Polygon],
        category: boostvoronoi::prelude::SourceCategory,
    ) -> Option<Point> {
        match category {
            boostvoronoi::prelude::SourceCategory::SegmentStart => Some(self.from(polygons)),
            boostvoronoi::prelude::SourceCategory::SegmentEnd => Some(self.to(polygons)),
            boostvoronoi::prelude::SourceCategory::Segment
            | boostvoronoi::prelude::SourceCategory::SinglePoint => None,
        }
    }

    pub(super) fn source_point_index(
        self,
        polygons: &[Polygon],
        category: boostvoronoi::prelude::SourceCategory,
    ) -> Option<Self> {
        match category {
            boostvoronoi::prelude::SourceCategory::SegmentStart => Some(self),
            boostvoronoi::prelude::SourceCategory::SegmentEnd => Some(Self {
                point_index: (self.point_index + 1) % polygons[self.polygon_index].points().len(),
                ..self
            }),
            boostvoronoi::prelude::SourceCategory::Segment
            | boostvoronoi::prelude::SourceCategory::SinglePoint => None,
        }
    }
}

pub(super) fn collect_segments(polygons: &[Polygon]) -> Vec<PolygonSegmentIndex> {
    polygons
        .iter()
        .enumerate()
        .flat_map(|(polygon_index, polygon)| {
            (0..polygon.points().len()).map(move |point_index| PolygonSegmentIndex {
                polygon_index,
                point_index,
            })
        })
        .collect()
}
