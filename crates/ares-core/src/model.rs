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
}
