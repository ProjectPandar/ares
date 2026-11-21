mod geometry;
mod repair;

use crate::SliceError;

/// One object produced by the Native CLI's binary STL action.
#[derive(Debug)]
pub struct StlExport {
    /// Native per-object filename, including the one-based index and extension.
    pub filename: String,
    pub bytes: Vec<u8>,
}

/// Export the objects of one STL or plain 3MF as ordered binary STL artifacts.
/// `filename` is a source basename, not a path. This API performs no file I/O.
/// Explicit repair is limited to exact cleanup of a single convex shell per object.
/// Unsupported geometry or combinations are rejected without producing artifacts.
pub fn export_stl(
    input: &[u8],
    filename: &str,
    repair: bool,
) -> Result<Vec<StlExport>, SliceError> {
    export_stls([(input, filename)], repair)
}

/// Export the merged objects of positional STL and plain 3MF models, in input order.
/// Each input pairs its contents with its source basename.
pub fn export_stls<'a>(
    inputs: impl IntoIterator<Item = (&'a [u8], &'a str)>,
    repair: bool,
) -> Result<Vec<StlExport>, SliceError> {
    let mut objects = Vec::new();
    // OrcaSlicer.cpp:4040–4055 appends the objects of each model before dispatch.
    for (input, filename) in inputs {
        if !leaf_name(filename) {
            return Err(unsupported("STL export requires a safe source filename"));
        }
        objects.extend(geometry::load(input, filename)?);
    }
    objects
        .into_iter()
        .enumerate()
        .map(|(index, mut mesh)| {
            if repair {
                repair::exact_cleanup(&mesh.vertices, &mut mesh.faces)?;
            }
            let mut filename = format!("obj_{}_{}", index + 1, mesh.name);
            if !filename.ends_with(".stl") {
                filename.push_str(".stl");
            }
            let bytes = write_binary(&mesh.placed_vertices()?, &mesh.faces)?;
            Ok(StlExport { filename, bytes })
        })
        .collect()
}

fn leaf_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        && !name.chars().any(char::is_control)
}

fn unsupported(reason: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(reason.to_owned())
}

fn invalid(reason: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(reason.into())
}

type Vertex = [f32; 3];
type Face = [usize; 3];

fn coordinate_key(vertex: Vertex) -> [u32; 3] {
    vertex.map(|value| if value == 0.0 { 0 } else { value.to_bits() })
}

fn minus<T: Copy + std::ops::Sub<Output = T>>(a: [T; 3], b: [T; 3]) -> [T; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}

fn cross<T: Copy + std::ops::Mul<Output = T> + std::ops::Sub<Output = T>>(
    [x, y, z]: [T; 3],
    [a, b, c]: [T; 3],
) -> [T; 3] {
    [y * c - z * b, z * a - x * c, x * b - y * a]
}

fn dot<T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T>>(
    [x, y, z]: [T; 3],
    [a, b, c]: [T; 3],
) -> T {
    x * a + (y * b + z * c)
}

fn normalized(v: Vertex) -> Vertex {
    let squared = dot(v, v);
    if squared > 0.0 {
        let length = squared.sqrt();
        v.map(|x| x / length)
    } else {
        v
    }
}

fn write_binary(vertices: &[Vertex], faces: &[Face]) -> Result<Vec<u8>, SliceError> {
    // TriangleMesh::write_binary -> its_write_stl_binary: empty label and attributes.
    let count = u32::try_from(faces.len()).map_err(|_| invalid("too many STL faces"))?;
    let mut bytes = Vec::with_capacity(84 + 50 * faces.len());
    bytes.resize(80, 0);
    bytes.extend(count.to_le_bytes());
    for face in faces {
        let [a, b, c] = face.map(|index| vertices[index]);
        let normal = normalized(cross(minus(b, a), minus(c, b)));
        for component in [normal, a, b, c].into_iter().flatten() {
            if !component.is_finite() {
                return Err(invalid("non-finite STL output component"));
            }
            bytes.extend(component.to_le_bytes());
        }
        bytes.extend([0, 0]);
    }
    Ok(bytes)
}
