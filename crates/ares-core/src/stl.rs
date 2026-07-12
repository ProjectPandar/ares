use crate::{InputFormat, Model, Point3, SliceError, Triangle};

const BINARY_HEADER_LEN: usize = 80;
const TRIANGLE_COUNT_LEN: usize = 4;
const BINARY_TRIANGLE_LEN: usize = 50;

pub fn load(input: &[u8]) -> Result<Model, SliceError> {
    if looks_like_binary(input) {
        parse_binary(input)
    } else {
        parse_ascii(input)
    }
}

pub fn looks_like_binary(input: &[u8]) -> bool {
    if input.len() < BINARY_HEADER_LEN + TRIANGLE_COUNT_LEN {
        return false;
    }

    let count = u32::from_le_bytes(
        input[BINARY_HEADER_LEN..BINARY_HEADER_LEN + TRIANGLE_COUNT_LEN]
            .try_into()
            .unwrap(),
    ) as usize;
    BINARY_HEADER_LEN + TRIANGLE_COUNT_LEN + count.saturating_mul(BINARY_TRIANGLE_LEN)
        == input.len()
}

fn parse_binary(input: &[u8]) -> Result<Model, SliceError> {
    let count = u32::from_le_bytes(
        input[BINARY_HEADER_LEN..BINARY_HEADER_LEN + TRIANGLE_COUNT_LEN]
            .try_into()
            .unwrap(),
    ) as usize;
    if count == 0 {
        return Err(malformed_stl());
    }

    let mut triangles = Vec::with_capacity(count);
    let mut offset = BINARY_HEADER_LEN + TRIANGLE_COUNT_LEN;

    for _ in 0..count {
        let vertex_offset = offset + 12;
        let vertices = [
            read_point(input, vertex_offset)?,
            read_point(input, vertex_offset + 12)?,
            read_point(input, vertex_offset + 24)?,
        ];
        triangles.push(Triangle::new(vertices));
        offset += BINARY_TRIANGLE_LEN;
    }

    Ok(Model::new(InputFormat::Stl, triangles))
}

fn read_point(input: &[u8], offset: usize) -> Result<Point3, SliceError> {
    Ok(Point3::new(
        read_finite_f32(input, offset)?,
        read_finite_f32(input, offset + 4)?,
        read_finite_f32(input, offset + 8)?,
    ))
}

fn read_finite_f32(input: &[u8], offset: usize) -> Result<f32, SliceError> {
    finite_f32(f32::from_le_bytes(
        input[offset..offset + 4].try_into().unwrap(),
    ))
}

fn parse_ascii(input: &[u8]) -> Result<Model, SliceError> {
    let text = std::str::from_utf8(input).map_err(|_| malformed_stl())?;
    let mut triangles = Vec::new();
    let mut in_solid = false;
    let mut in_facet = false;
    let mut in_loop = false;
    let mut vertices = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("solid") if !in_solid && !in_facet && !in_loop => in_solid = true,
            Some("facet") if in_solid && !in_facet && !in_loop => {
                if parts.next() != Some("normal") {
                    return Err(malformed_stl());
                }
                for _ in 0..3 {
                    parse_ascii_f32(parts.next().ok_or_else(malformed_stl)?)?;
                }
                if parts.next().is_some() {
                    return Err(malformed_stl());
                }
                in_facet = true;
            }
            Some("outer") if in_facet && !in_loop => {
                if parts.next() != Some("loop") || parts.next().is_some() {
                    return Err(malformed_stl());
                }
                in_loop = true;
                vertices.clear();
            }
            Some("vertex") if in_loop => {
                let vertex = Point3::new(
                    parse_ascii_f32(parts.next().ok_or_else(malformed_stl)?)?,
                    parse_ascii_f32(parts.next().ok_or_else(malformed_stl)?)?,
                    parse_ascii_f32(parts.next().ok_or_else(malformed_stl)?)?,
                );
                if parts.next().is_some() {
                    return Err(malformed_stl());
                }
                vertices.push(vertex);
            }
            Some("endloop") if in_loop => {
                if parts.next().is_some() || vertices.len() != 3 {
                    return Err(malformed_stl());
                }
                in_loop = false;
            }
            Some("endfacet") if in_facet && !in_loop => {
                if parts.next().is_some() || vertices.len() != 3 {
                    return Err(malformed_stl());
                }
                triangles.push(Triangle::new([vertices[0], vertices[1], vertices[2]]));
                vertices.clear();
                in_facet = false;
            }
            Some("endsolid") if in_solid && !in_facet && !in_loop => {
                in_solid = false;
            }
            _ => return Err(malformed_stl()),
        }
    }

    if in_solid || in_facet || in_loop || !vertices.is_empty() || triangles.is_empty() {
        return Err(malformed_stl());
    }

    Ok(Model::new(InputFormat::Stl, triangles))
}

fn parse_ascii_f32(value: &str) -> Result<f32, SliceError> {
    finite_f32(value.parse().map_err(|_| malformed_stl())?)
}

fn finite_f32(value: f32) -> Result<f32, SliceError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(malformed_stl())
    }
}

fn malformed_stl() -> SliceError {
    SliceError::InvalidInput("malformed STL input".to_owned())
}
