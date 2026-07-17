use crate::geometry::Point;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndpointReference {
    Vertex(u32),
    Edge(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FacetEdgeType {
    General,
    Top,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IntersectionPoint {
    point: Point,
    reference: EndpointReference,
}

impl IntersectionPoint {
    const fn new(point: Point, reference: EndpointReference) -> Self {
        Self { point, reference }
    }

    pub(crate) const fn point(self) -> Point {
        self.point
    }

    pub(crate) const fn reference(self) -> EndpointReference {
        self.reference
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IntersectionLine {
    a: IntersectionPoint,
    b: IntersectionPoint,
    edge_type: FacetEdgeType,
}

impl IntersectionLine {
    const fn new(a: IntersectionPoint, b: IntersectionPoint, edge_type: FacetEdgeType) -> Self {
        Self { a, b, edge_type }
    }

    pub(crate) const fn a(self) -> IntersectionPoint {
        self.a
    }

    pub(crate) const fn b(self) -> IntersectionPoint {
        self.b
    }

    pub(crate) const fn edge_type(self) -> FacetEdgeType {
        self.edge_type
    }
}

pub(crate) fn intersect_facet(
    slice_z: f32,
    vertices: &[[f32; 3]; 3],
    vertex_ids: [u32; 3],
    edge_ids: [u32; 3],
) -> Option<IntersectionLine> {
    let min_z = vertices[0][2].min(vertices[1][2].min(vertices[2][2]));
    let max_z = vertices[0][2].max(vertices[1][2].max(vertices[2][2]));
    if min_z == max_z {
        return None;
    }

    let lowest = lowest_vertex_index(vertices, min_z);
    let mut points = [None; 3];
    let mut point_count = 0;
    let mut point_on_layer = None;

    for offset in 0..3 {
        let a_index = (lowest + offset) % 3;
        let b_index = (a_index + 1) % 3;
        let c_index = (a_index + 2) % 3;
        let a = vertices[a_index];
        let b = vertices[b_index];
        let a_id = vertex_ids[a_index];
        let b_id = vertex_ids[b_index];

        if a[2] == slice_z && b[2] == slice_z {
            return (vertices[c_index][2] < slice_z).then(|| {
                IntersectionLine::new(
                    inherited_point(b, b_id),
                    inherited_point(a, a_id),
                    FacetEdgeType::Top,
                )
            });
        }

        if a[2] == slice_z {
            add_inherited_point(&mut points, &mut point_count, &mut point_on_layer, a, a_id);
        } else if b[2] == slice_z {
            add_inherited_point(&mut points, &mut point_count, &mut point_on_layer, b, b_id);
        } else if (a[2] < slice_z && b[2] > slice_z) || (b[2] < slice_z && a[2] > slice_z) {
            let (a, a_id, b, b_id) = if a_id <= b_id {
                (a, a_id, b, b_id)
            } else {
                (b, b_id, a, a_id)
            };
            let t = (f64::from(slice_z) - f64::from(b[2])) / (f64::from(a[2]) - f64::from(b[2]));
            if t <= 0.0 {
                add_inherited_point(&mut points, &mut point_count, &mut point_on_layer, a, a_id);
            } else if t >= 1.0 {
                add_inherited_point(&mut points, &mut point_count, &mut point_on_layer, b, b_id);
            } else {
                points[point_count] = Some(IntersectionPoint::new(
                    Point::new(
                        interpolate_coordinate(a[0], b[0], t),
                        interpolate_coordinate(a[1], b[1], t),
                    ),
                    EndpointReference::Edge(edge_ids[a_index]),
                ));
                point_count += 1;
            }
        }
    }

    (point_count == 2).then(|| {
        IntersectionLine::new(
            points[1].unwrap(),
            points[0].unwrap(),
            FacetEdgeType::General,
        )
    })
}

pub(super) fn lowest_vertex_index(vertices: &[[f32; 3]; 3], min_z: f32) -> usize {
    if vertices[1][2] == min_z {
        1
    } else if vertices[2][2] == min_z {
        2
    } else {
        0
    }
}

fn add_inherited_point(
    points: &mut [Option<IntersectionPoint>; 3],
    point_count: &mut usize,
    point_on_layer: &mut Option<u32>,
    vertex: [f32; 3],
    vertex_id: u32,
) {
    if *point_on_layer != Some(vertex_id) {
        points[*point_count] = Some(inherited_point(vertex, vertex_id));
        *point_count += 1;
        *point_on_layer = Some(vertex_id);
    }
}

fn inherited_point(vertex: [f32; 3], vertex_id: u32) -> IntersectionPoint {
    IntersectionPoint::new(
        Point::new(vertex[0] as i64, vertex[1] as i64),
        EndpointReference::Vertex(vertex_id),
    )
}

fn interpolate_coordinate(a: f32, b: f32, t: f64) -> i64 {
    (f64::from(b) + (f64::from(a) - f64::from(b)) * t + 0.5).floor() as i64
}
