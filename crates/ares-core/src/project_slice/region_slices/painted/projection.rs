//! MultiMaterialSegmentation.cpp482–489,2029–2120 (ordinary f32 facet projection).

use super::{painted_lines::PaintedLine, preprocess::Layer, projection_visitor};
use crate::{
    ProjectObject, ProjectVolume, ProjectVolumeType, SliceError, Transform3d,
    geometry::{BoundingBox, CoordinateScale, Line, Point, fixed_gcc_sort_by},
    project::facets_annotation::ordinary_facets,
};

pub(super) enum Painting {
    Mmu,
    Fuzzy,
}

pub(super) struct Input<'a> {
    pub(super) painting: Painting,
    pub(super) source: &'a ProjectObject,
    pub(super) transform: Transform3d,
    pub(super) center: Point,
    pub(super) planes: &'a [f64],
    pub(super) states: usize,
    pub(super) scale: CoordinateScale,
}

fn affine(a: Transform3d, b: Transform3d) -> [[f32; 4]; 3] {
    let a = a.as_matrix().map(|row| row.map(|x| x as f32));
    let b = b.as_matrix().map(|row| row.map(|x| x as f32));
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            let linear =
                a[row][0] * b[0][column] + a[row][1] * b[1][column] + a[row][2] * b[2][column];
            if column == 3 {
                linear + a[row][3]
            } else {
                linear
            }
        })
    })
}

fn transformed(matrix: &[[f32; 4]; 3], p: [f32; 3]) -> [f32; 3] {
    matrix.map(|row| (row[0] * p[0] + row[1] * p[1] + row[2] * p[2]) + row[3])
}

fn interpolate(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + t * (b[i] - a[i]))
}

fn equal(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-3_f32
}

fn triangle(
    input: &Input<'_>,
    mut facet: [[f32; 3]; 3],
    layers: &[Layer],
    color: usize,
    output: &mut [Vec<PaintedLine>],
) -> Result<(), SliceError> {
    let min = facet.iter().map(|p| p[2]).fold(f32::MAX, f32::min);
    let max = facet.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
    if equal(min, max) {
        return Ok(());
    }
    fixed_gcc_sort_by(&mut facet, |a, b| a[2] < b[2]);
    let first = input
        .planes
        .partition_point(|&z| z <= f64::from((f64::from(min) - 1e-4) as f32));
    let stop = input
        .planes
        .partition_point(|&z| z <= f64::from((f64::from(max) + 1e-4) as f32));
    for index in first..stop {
        let z = input.planes[index];
        let zf = z as f32;
        if layers[index].input.is_empty()
            || zf + 1e-3_f32 < facet[0][2]
            || facet[2][2] + 1e-3_f32 < zf
        {
            continue;
        }
        let t = (zf - facet[0][2]) / (facet[2][2] - facet[0][2]);
        let a = interpolate(facet[0], facet[2], t);
        let b = if (equal(facet[0][2], facet[1][2]) && equal(facet[1][2], zf))
            || (equal(facet[1][2], facet[2][2]) && equal(facet[1][2], zf))
        {
            facet[1]
        } else if f64::from(facet[1][2]) > z {
            let t = (zf - facet[0][2]) / (facet[1][2] - facet[0][2]);
            interpolate(facet[0], facet[1], t)
        } else {
            let t = (zf - facet[1][2]) / (facet[2][2] - facet[1][2]);
            interpolate(facet[1], facet[2], t)
        };
        let lattice = |p: [f32; 3]| {
            Point::new(
                input.scale.scaled_delta(f64::from(p[0])).round() as i64 - input.center.x(),
                input.scale.scaled_delta(f64::from(p[1])).round() as i64 - input.center.y(),
            )
        };
        let line = Line::new(lattice(a), lattice(b));
        let grid = layers[index].grid.as_ref().unwrap();
        if let Some(line) = clipped(line, grid.bounds(), input.scale) {
            projection_visitor::visit(
                grid,
                line,
                color,
                50.0 * input.scale.scaled_delta(1e-4),
                &mut output[index],
            )
            .map_err(|e| SliceError::InvalidInput(format!("painted facet projection: {e:?}")))?;
        }
    }
    Ok(())
}

fn clipped(line: Line, (min, max): (Point, Point), scale: CoordinateScale) -> Option<Line> {
    let bbox = BoundingBox::new(min, max).inflated((10.0 * scale.scaled_delta(1e-4)) as i64);
    if bbox.contains(line.a) && bbox.contains(line.b) {
        return Some(line);
    }
    if !bbox.overlap(BoundingBox::from_points([line.a, line.b].into_iter()).unwrap()) {
        return None;
    }
    line.clip_with_bbox(bbox)
}

fn volume(
    input: &Input<'_>,
    volume: &ProjectVolume,
    layers: &[Layer],
    output: &mut [Vec<PaintedLine>],
) -> Result<(), SliceError> {
    for color in 1..input.states {
        let facets = ordinary_facets(
            volume.mesh(),
            match input.painting {
                Painting::Mmu => &volume.mesh().mmu_segmentation_facets,
                Painting::Fuzzy => &volume.mesh().fuzzy_skin_facets,
            },
            color as u8,
        )?;
        if volume.volume_type() != ProjectVolumeType::ModelPart || facets.triangles.is_empty() {
            continue;
        }
        let transform = affine(input.transform, volume.transform());
        for indices in facets.triangles {
            let facet = indices.map(|i| transformed(&transform, facets.vertices[i as usize]));
            triangle(input, facet, layers, color, output)?;
        }
    }
    Ok(())
}

pub(super) fn project(
    input: &Input<'_>,
    layers: &[Layer],
) -> Result<Vec<Vec<PaintedLine>>, SliceError> {
    let mut output = vec![Vec::new(); layers.len()];
    for source in input.source.volumes() {
        volume(input, source, layers, &mut output)?;
    }
    Ok(output)
}
