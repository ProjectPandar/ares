use crate::InputFormat;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    vertices: [Point3; 3],
}

impl Triangle {
    pub const fn new(vertices: [Point3; 3]) -> Self {
        Self { vertices }
    }

    pub const fn vertices(&self) -> &[Point3; 3] {
        &self.vertices
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZBounds {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct XyBounds {
    pub(crate) min_x: f64,
    pub(crate) max_x: f64,
    pub(crate) min_y: f64,
    pub(crate) max_y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    format: InputFormat,
    triangles: Vec<Triangle>,
}

impl Model {
    pub fn new(format: InputFormat, triangles: Vec<Triangle>) -> Self {
        Self { format, triangles }
    }

    pub const fn format(&self) -> InputFormat {
        self.format
    }

    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn z_bounds(&self) -> Option<ZBounds> {
        let mut vertices = self
            .triangles
            .iter()
            .flat_map(|triangle| triangle.vertices().iter());
        let first = vertices.next()?;
        if !first.z.is_finite() {
            return None;
        }
        let mut min = first.z;
        let mut max = first.z;
        for vertex in vertices {
            if !vertex.z.is_finite() {
                return None;
            }
            min = min.min(vertex.z);
            max = max.max(vertex.z);
        }
        Some(ZBounds { min, max })
    }

    pub(crate) fn xy_bounds(&self) -> Option<XyBounds> {
        let mut vertices = self
            .triangles
            .iter()
            .flat_map(|triangle| triangle.vertices().iter());
        let first = vertices.next()?;
        if !first.x.is_finite() || !first.y.is_finite() {
            return None;
        }
        let mut min_x = first.x;
        let mut max_x = first.x;
        let mut min_y = first.y;
        let mut max_y = first.y;
        for vertex in vertices {
            if !vertex.x.is_finite() || !vertex.y.is_finite() {
                return None;
            }
            min_x = min_x.min(vertex.x);
            max_x = max_x.max(vertex.x);
            min_y = min_y.min(vertex.y);
            max_y = max_y.max(vertex.y);
        }
        Some(XyBounds {
            min_x: f64::from(min_x),
            max_x: f64::from(max_x),
            min_y: f64::from(min_y),
            max_y: f64::from(max_y),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InputFormat;

    #[test]
    fn model_reports_z_bounds() {
        let model = Model::new(
            InputFormat::Stl,
            vec![Triangle::new([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.2),
                Point3::new(0.0, 1.0, 0.4),
            ])],
        );
        let bounds = model.z_bounds().unwrap();

        assert_eq!(bounds.min, 0.0);
        assert_eq!(bounds.max, 0.4);
    }

    #[test]
    fn model_rejects_non_finite_z_bounds() {
        let model = Model::new(
            InputFormat::Stl,
            vec![Triangle::new([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, f32::NAN),
                Point3::new(0.0, 1.0, 0.4),
            ])],
        );

        assert_eq!(model.z_bounds(), None);
    }

    #[test]
    fn model_reports_xy_bounds() {
        let model = Model::new(
            InputFormat::Stl,
            vec![
                Triangle::new([
                    Point3::new(-2.0, 1.0, 0.0),
                    Point3::new(3.0, -4.0, 0.2),
                    Point3::new(0.0, 2.0, 0.4),
                ]),
                Triangle::new([
                    Point3::new(1.0, 5.0, 0.0),
                    Point3::new(-1.0, -3.0, 0.2),
                    Point3::new(2.0, 0.0, 0.4),
                ]),
            ],
        );
        let bounds = model.xy_bounds().unwrap();

        assert_eq!(bounds.min_x, -2.0);
        assert_eq!(bounds.max_x, 3.0);
        assert_eq!(bounds.min_y, -4.0);
        assert_eq!(bounds.max_y, 5.0);
    }
}
