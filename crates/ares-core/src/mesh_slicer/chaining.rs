use crate::geometry::{Point, Polygon};

use super::{EndpointReference, IntersectionLine};

mod exact;
mod gaps;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OpenPolyline {
    start: EndpointReference,
    end: EndpointReference,
    points: Vec<Point>,
    length: f64,
    consumed: bool,
}

impl OpenPolyline {
    fn new(start: EndpointReference, end: EndpointReference, points: Vec<Point>) -> Self {
        let length = open_length(&points);
        Self {
            start,
            end,
            points,
            length,
            consumed: false,
        }
    }

    #[cfg(test)]
    pub(crate) const fn start(&self) -> EndpointReference {
        self.start
    }

    #[cfg(test)]
    pub(crate) const fn end(&self) -> EndpointReference {
        self.end
    }

    #[cfg(test)]
    pub(crate) fn points(&self) -> &[Point] {
        &self.points
    }

    #[cfg(test)]
    pub(crate) const fn length(&self) -> f64 {
        self.length
    }

    #[cfg(test)]
    pub(crate) const fn consumed(&self) -> bool {
        self.consumed
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ChainedLayer {
    polygons: Vec<Polygon>,
    open_polylines: Vec<OpenPolyline>,
}

impl ChainedLayer {
    #[cfg(test)]
    pub(crate) fn polygons(&self) -> &[Polygon] {
        &self.polygons
    }

    #[cfg(test)]
    pub(crate) fn open_polylines(&self) -> &[OpenPolyline] {
        &self.open_polylines
    }

    fn into_parts(self) -> (Vec<Polygon>, Vec<OpenPolyline>) {
        (self.polygons, self.open_polylines)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LoopedLayer {
    polygons: Vec<Polygon>,
}

impl LoopedLayer {
    pub(crate) fn polygons(&self) -> &[Polygon] {
        &self.polygons
    }
}

#[derive(Clone, Copy)]
struct StartRecord {
    identity_id: u32,
    raw_index: usize,
}

struct StartRange {
    identity_id: u32,
    end: usize,
    cursor: usize,
}

struct ReferenceIndex {
    records: Vec<StartRecord>,
    ranges: Vec<StartRange>,
}

impl ReferenceIndex {
    fn new(mut records: Vec<StartRecord>) -> Self {
        records.sort_unstable_by_key(|record| (record.identity_id, record.raw_index));
        let mut ranges = Vec::new();
        let mut start = 0;
        while start < records.len() {
            let identity_id = records[start].identity_id;
            let mut end = start + 1;
            while end < records.len() && records[end].identity_id == identity_id {
                end += 1;
            }
            ranges.push(StartRange {
                identity_id,
                end,
                cursor: start,
            });
            start = end;
        }
        Self { records, ranges }
    }

    fn take_next(&mut self, identity_id: u32, consumed: &[bool]) -> Option<usize> {
        let range_index = self
            .ranges
            .binary_search_by_key(&identity_id, |range| range.identity_id)
            .ok()?;
        let range = &mut self.ranges[range_index];
        while range.cursor < range.end {
            let raw_index = self.records[range.cursor].raw_index;
            range.cursor += 1;
            if !consumed[raw_index] {
                return Some(raw_index);
            }
        }
        None
    }
}

struct StartIndex {
    edges: ReferenceIndex,
    vertices: ReferenceIndex,
}

impl StartIndex {
    fn new(lines: &[IntersectionLine]) -> Self {
        let mut edges = Vec::new();
        let mut vertices = Vec::new();
        for (raw_index, line) in lines.iter().enumerate() {
            let record = |identity_id| StartRecord {
                identity_id,
                raw_index,
            };
            match line.a().reference() {
                EndpointReference::Edge(identity_id) => edges.push(record(identity_id)),
                EndpointReference::Vertex(identity_id) => vertices.push(record(identity_id)),
            }
        }
        Self {
            edges: ReferenceIndex::new(edges),
            vertices: ReferenceIndex::new(vertices),
        }
    }

    fn take_next(&mut self, reference: EndpointReference, consumed: &[bool]) -> Option<usize> {
        match reference {
            EndpointReference::Edge(identity_id) => self.edges.take_next(identity_id, consumed),
            EndpointReference::Vertex(identity_id) => {
                self.vertices.take_next(identity_id, consumed)
            }
        }
    }
}

pub(crate) fn chain_lines_by_triangle_connectivity(lines: Vec<IntersectionLine>) -> ChainedLayer {
    let mut starts = StartIndex::new(&lines);
    let mut consumed = vec![false; lines.len()];
    let mut polygons = Vec::new();
    let mut open_polylines = Vec::new();

    for seed_index in 0..lines.len() {
        if consumed[seed_index] {
            continue;
        }

        let seed = lines[seed_index];
        consumed[seed_index] = true;
        let mut points = vec![seed.a().point()];
        let mut last = seed;

        while let Some(next_index) = starts.take_next(last.b().reference(), &consumed) {
            let next = lines[next_index];
            debug_assert_eq!(last.b().point(), next.a().point());
            points.push(next.a().point());
            consumed[next_index] = true;
            last = next;
        }

        if seed.a().reference() == last.b().reference() {
            debug_assert_eq!(seed.a().point(), last.b().point());
            polygons.push(Polygon::new(points));
        } else {
            points.push(last.b().point());
            open_polylines.push(OpenPolyline::new(
                seed.a().reference(),
                last.b().reference(),
                points,
            ));
        }
    }

    ChainedLayer {
        polygons,
        open_polylines,
    }
}

pub(crate) fn make_loops(
    chained: ChainedLayer,
    max_gap_scaled: crate::geometry::Coord,
) -> LoopedLayer {
    let (mut polygons, mut open_polylines) = chained.into_parts();
    exact::chain_open_polylines_exact(&mut open_polylines, &mut polygons, false);
    exact::chain_open_polylines_exact(&mut open_polylines, &mut polygons, true);
    gaps::chain_open_polylines_close_gaps(
        &mut open_polylines,
        &mut polygons,
        max_gap_scaled,
        false,
    );
    gaps::chain_open_polylines_close_gaps(&mut open_polylines, &mut polygons, max_gap_scaled, true);
    LoopedLayer { polygons }
}

fn open_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| {
            let dx = (i128::from(pair[1].x()) - i128::from(pair[0].x())) as f64;
            let dy = (i128::from(pair[1].y()) - i128::from(pair[0].y())) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

fn signed_area(points: &[Point]) -> f64 {
    let mut area = 0.0;
    for index in 0..points.len() {
        let previous = if index == 0 {
            points.len() - 1
        } else {
            index - 1
        };
        let x_sum = i128::from(points[index].x()) + i128::from(points[previous].x());
        let y_difference = i128::from(points[index].y()) - i128::from(points[previous].y());
        area += x_sum as f64 * y_difference as f64;
    }
    area
}
