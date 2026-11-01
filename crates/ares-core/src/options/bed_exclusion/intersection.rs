//! `ModelInstance::intersects_bed_exclude_region`'s yes/no path (`Model.cpp:3753-3836` at
//! v2.5.0-beta `345efec64`) and its helpers (`:3559-3750`). See
//! `docs/architecture/web-plater-arrange-imex.md`, "A2b2". The red preview (`intersection_mesh`,
//! `append_clipped_polygon`) is GUI drawing and left out.

use super::{BedExcludeRegion, counter_clockwise};
use crate::geometry::{CoordinateScale, ExPolygon, Polygon, triangulate_expolygon_3d};
use crate::{Point3d, Transform3d};

type Vec2 = [f64; 2];
type Vec3 = [f64; 3];

/// `EPSILON` (`libslic3r.h:52`).
const EPSILON: f64 = 1e-4;
/// `NORMALS_UP` (`Tesselate.hpp:14`).
const NORMALS_UP: bool = false;

/// `ExclusionPrismFootprintTriangle` (`:3561-3566`).
pub(crate) struct Footprint {
    vertices: [Vec2; 3],
    min: Vec2,
    max: Vec2,
}

fn sub3(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `cross_2d` (`:3568-3571`).
fn cross_2d(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

/// `clip_polygon_by_signed_distance` (`:3573-3614`): keeps the part with distance >= -1e-8.
fn clip_polygon_by_signed_distance(input: &[Vec3], distance: impl Fn(Vec3) -> f64) -> Vec<Vec3> {
    const CLIP_EPSILON: f64 = 1e-8;
    const DUPLICATE_EPSILON_SQ: f64 = 1e-16;
    let squared_norm = |v: Vec3| dot3(v, v);
    let mut output: Vec<Vec3> = Vec::new();
    let Some(&last) = input.last() else {
        return output;
    };
    let append_point = |output: &mut Vec<Vec3>, point: Vec3| {
        if output
            .last()
            .is_none_or(|&back| squared_norm(sub3(back, point)) > DUPLICATE_EPSILON_SQ)
        {
            output.push(point);
        }
    };
    let (mut prev, mut prev_distance) = (last, distance(last));
    let mut prev_inside = prev_distance >= -CLIP_EPSILON;
    for &curr in input {
        let curr_distance = distance(curr);
        let curr_inside = curr_distance >= -CLIP_EPSILON;
        if curr_inside != prev_inside {
            let denom = prev_distance - curr_distance;
            if denom.abs() > CLIP_EPSILON {
                let t = prev_distance / denom;
                let step = sub3(curr, prev);
                append_point(
                    &mut output,
                    [
                        prev[0] + t * step[0],
                        prev[1] + t * step[1],
                        prev[2] + t * step[2],
                    ],
                );
            }
        }
        if curr_inside {
            append_point(&mut output, curr);
        }
        (prev, prev_distance, prev_inside) = (curr, curr_distance, curr_inside);
    }
    if output.len() > 1
        && squared_norm(sub3(output[0], output[output.len() - 1])) <= DUPLICATE_EPSILON_SQ
    {
        output.pop();
    }
    output
}

/// `triangulated_prism_footprints` (`:3616-3654`): the polygon tessellated counter-clockwise, a fan
/// when the tessellator gives nothing; slivers of area <= `EPSILON` dropped.
pub(crate) fn triangulated_prism_footprints(polygon: &Polygon) -> Vec<Footprint> {
    let points = polygon.points();
    if points.len() < 3 {
        return Vec::new();
    }
    let scale = CoordinateScale::Normal;
    let expolygon = ExPolygon::new(counter_clockwise(polygon.clone()), Vec::new());
    let mut triangles: Vec<Vec2> = triangulate_expolygon_3d(&expolygon, 0.0, NORMALS_UP, scale)
        .into_iter()
        .map(|[x, y, _]| [x, y])
        .collect();
    if triangles.is_empty() {
        let unscaled = |i: usize| [scale.unscale(points[i].x()), scale.unscale(points[i].y())];
        for i in 1..points.len() - 1 {
            triangles.extend([unscaled(0), unscaled(i), unscaled(i + 1)]);
        }
    }
    triangles
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|&triangle| {
            let mut vertices = triangle;
            let edge = |i: usize| {
                [
                    vertices[i][0] - vertices[0][0],
                    vertices[i][1] - vertices[0][1],
                ]
            };
            let area = cross_2d(edge(1), edge(2));
            if area.abs() <= EPSILON {
                return None;
            }
            if area < 0.0 {
                vertices.swap(1, 2);
            }
            let fold = |f: fn(f64, f64) -> f64| {
                [0, 1].map(|axis| f(f(vertices[0][axis], vertices[1][axis]), vertices[2][axis]))
            };
            Some(Footprint {
                vertices,
                min: fold(f64::min),
                max: fold(f64::max),
            })
        })
        .collect()
}

/// `triangle_bbox_intersects_prism_footprint` (`:3656-3668`).
fn triangle_bbox_intersects_prism_footprint(
    [p0, p1, p2]: &[Vec3; 3],
    footprint: &Footprint,
) -> bool {
    const BBOX_EPSILON: f64 = 1e-8;
    let lo = |axis: usize| p0[axis].min(p1[axis]).min(p2[axis]);
    let hi = |axis: usize| p0[axis].max(p1[axis]).max(p2[axis]);
    hi(0) >= footprint.min[0] - BBOX_EPSILON
        && lo(0) <= footprint.max[0] + BBOX_EPSILON
        && hi(1) >= footprint.min[1] - BBOX_EPSILON
        && lo(1) <= footprint.max[1] + BBOX_EPSILON
}

/// `clip_model_triangle_to_prism` (`:3670-3692`): the triangle cut to the z slab, then to each
/// footprint edge's left side.
fn clip_model_triangle_to_prism(
    triangle: &[Vec3; 3],
    (z_min, z_max): (f64, f64),
    footprint: &Footprint,
) -> Vec<Vec3> {
    let mut clipped = clip_polygon_by_signed_distance(triangle, |p| p[2] - z_min);
    clipped = clip_polygon_by_signed_distance(&clipped, |p| z_max - p[2]);
    for i in 0..3 {
        if clipped.len() < 3 {
            break;
        }
        let a = footprint.vertices[i];
        let b = footprint.vertices[(i + 1) % 3];
        let edge = [b[0] - a[0], b[1] - a[1]];
        clipped = clip_polygon_by_signed_distance(&clipped, |p| {
            cross_2d(edge, [p[0] - a[0], p[1] - a[1]])
        });
    }
    clipped
}

/// `polygon_3d_area2` (`:3694-3703`): the norm of the fan's summed cross products.
fn polygon_3d_area2(polygon: &[Vec3]) -> f64 {
    if polygon.len() < 3 {
        return 0.0;
    }
    let mut normal = [0.0; 3];
    for i in 1..polygon.len() - 1 {
        let c = cross3(
            sub3(polygon[i], polygon[0]),
            sub3(polygon[i + 1], polygon[0]),
        );
        normal = [normal[0] + c[0], normal[1] + c[1], normal[2] + c[2]];
    }
    dot3(normal, normal).sqrt()
}

/// `AABBTreeIndirect::detail::intersect_triangle` (`AABBTreeIndirect.hpp:319-367`), Möller's
/// `intersect_triangle1`: the ray's distance to the triangle when it hits.
fn intersect_triangle(
    orig: Vec3,
    dir: Vec3,
    [vert0, vert1, vert2]: [Vec3; 3],
    eps: f64,
) -> Option<f64> {
    let edge1 = sub3(vert1, vert0);
    let edge2 = sub3(vert2, vert0);
    let pvec = cross3(dir, edge2);
    let det = dot3(edge1, pvec);
    let tvec = sub3(orig, vert0);
    let u = dot3(tvec, pvec);
    let qvec = cross3(tvec, edge1);
    let v = dot3(dir, qvec);
    if det > eps {
        if u < 0.0 || u > det || v < 0.0 || u + v > det {
            return None;
        }
    } else if det < -eps {
        if u > 0.0 || u < det || v > 0.0 || u + v < det {
            return None;
        }
    } else {
        return None;
    }
    Some(dot3(edge2, qvec) * (1.0 / det))
}

fn vec3(point: Point3d) -> Vec3 {
    [point.x, point.y, point.z]
}

/// `mesh_contains_point_linear` (`:3723-3750`): the nearest face a +X ray hits faces along it.
fn mesh_contains_point_linear(vertices: &[Point3d], indices: &[[u32; 3]], point: Vec3) -> bool {
    const RAY_EPSILON: f64 = 1e-6;
    const RAY_DIRECTION: Vec3 = [1.0, 0.0, 0.0];
    let corners = |face: &[u32; 3]| face.map(|index| vec3(vertices[index as usize]));
    let nearest = indices
        .iter()
        .filter_map(|face| {
            intersect_triangle(point, RAY_DIRECTION, corners(face), RAY_EPSILON)
                .filter(|&distance| distance > 0.0)
                .map(|distance| (distance, face))
        })
        .fold(None::<(f64, &[u32; 3])>, |best, hit| match best {
            Some(best) if best.0 <= hit.0 => Some(best),
            _ => Some(hit),
        });
    nearest.is_some_and(|(_, face)| {
        // `its_unnormalized_normal`, in the mesh's `float`s.
        let [a, b, c] = face.map(|index| {
            let p = vertices[index as usize];
            [p.x as f32, p.y as f32, p.z as f32]
        });
        let (u, v) = (
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
            [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
        );
        f64::from(u[1] * v[2] - u[2] * v[1]) > 0.0
    })
}

/// Eigen's `Transform3d::inverse()` (`Affine`) applied to `point`: the 3x3 cofactor inverse
/// (`compute_inverse_size3_helper`) of the linear part, then `-L⁻¹ t`.
fn inverse_transform_point(matrix: Transform3d, point: Vec3) -> Vec3 {
    let m = matrix.as_matrix();
    let cofactor = |i: usize, j: usize| {
        let (i1, i2, j1, j2) = ((i + 1) % 3, (i + 2) % 3, (j + 1) % 3, (j + 2) % 3);
        m[i1][j1] * m[i2][j2] - m[i1][j2] * m[i2][j1]
    };
    let invdet =
        1.0 / (cofactor(0, 0) * m[0][0] + cofactor(1, 0) * m[1][0] + cofactor(2, 0) * m[2][0]);
    let inverse: [[f64; 3]; 3] =
        std::array::from_fn(|i| std::array::from_fn(|j| cofactor(j, i) * invdet));
    let translation = [m[0][3], m[1][3], m[2][3]];
    std::array::from_fn(|row| dot3(inverse[row], point) - dot3(inverse[row], translation))
}

/// The region's ordered z slab, or `None` when it is degenerate (`:3765-3769`).
pub(crate) fn z_slab(region: &BedExcludeRegion) -> Option<(f64, f64)> {
    let (z_min, z_max) = if region.z_min > region.z_max {
        (region.z_max, region.z_min)
    } else {
        (region.z_min, region.z_max)
    };
    (z_max > z_min + EPSILON).then_some((z_min, z_max))
}

/// One model part of the instance loop (`:3778-3832`): any clipped surface piece of area above
/// `1e-8`, else the first footprint's centroid, mid slab, inside the part's mesh.
pub(crate) fn part_intersects(
    (vertices, indices): (&[Point3d], &[[u32; 3]]),
    matrix: Transform3d,
    slab: (f64, f64),
    footprints: &[Footprint],
) -> bool {
    if vertices.is_empty() || indices.is_empty() {
        return false;
    }
    for face in indices {
        let triangle = face.map(|index| vec3(matrix.transform_point(vertices[index as usize])));
        let z = triangle.map(|p| p[2]);
        if z[0].max(z[1]).max(z[2]) < slab.0 || z[0].min(z[1]).min(z[2]) > slab.1 {
            continue;
        }
        for footprint in footprints {
            if triangle_bbox_intersects_prism_footprint(&triangle, footprint)
                && polygon_3d_area2(&clip_model_triangle_to_prism(&triangle, slab, footprint))
                    > 1e-8
            {
                return true;
            }
        }
    }
    let [a, b, c] = footprints[0].vertices;
    let probe_world = [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        slab.0 + 0.5 * (slab.1 - slab.0),
    ];
    let probe_local = inverse_transform_point(matrix, probe_world);
    // `TriangleMesh::bounding_box()`: the `float` vertices' box.
    let (mut min, mut max) = ([f64::MAX; 3], [f64::MIN; 3]);
    for vertex in vertices {
        for (axis, value) in vec3(*vertex).into_iter().enumerate() {
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }
    (0..3).all(|axis| probe_local[axis] >= min[axis] && probe_local[axis] <= max[axis])
        && mesh_contains_point_linear(vertices, indices, probe_local)
}

/// Whether the model part with mesh `vertices` / `indices`, placed by `matrix` (`instance matrix *
/// volume matrix`), reaches into `region`'s prism.
pub fn model_part_intersects_region(
    vertices: &[Point3d],
    indices: &[[u32; 3]],
    matrix: Transform3d,
    region: &BedExcludeRegion,
) -> bool {
    if region.polygon.points().len() < 3 {
        return false;
    }
    let Some(slab) = z_slab(region) else {
        return false;
    };
    let footprints = triangulated_prism_footprints(&region.polygon);
    !footprints.is_empty() && part_intersects((vertices, indices), matrix, slab, &footprints)
}
