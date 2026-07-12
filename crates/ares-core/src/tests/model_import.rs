use super::*;
#[test]
fn load_model_imports_ascii_stl_triangle() {
    let model = load_model(one_triangle_ascii_stl("\n")).unwrap();

    assert_eq!(model.format(), InputFormat::Stl);
    assert_eq!(model.triangles().len(), 1);
    assert_eq!(
        model.triangles()[0].vertices()[1],
        Point3::new(1.0, 0.0, 0.0)
    );
}

#[test]
fn load_model_imports_crlf_ascii_stl_triangle() {
    let model = load_model(one_triangle_ascii_stl("\r\n")).unwrap();

    assert_eq!(model.format(), InputFormat::Stl);
    assert_eq!(model.triangles().len(), 1);
}

#[test]
fn load_model_imports_binary_stl_triangle() {
    let mut bytes = vec![0; 80];
    bytes.extend_from_slice(&2_u32.to_le_bytes());
    append_binary_triangle(
        &mut bytes,
        [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    );
    append_binary_triangle(
        &mut bytes,
        [[0.0_f32, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]],
    );

    let model = load_model(bytes).unwrap();

    assert_eq!(model.format(), InputFormat::Stl);
    assert_eq!(model.triangles().len(), 2);
    assert_eq!(
        model.triangles()[0].vertices()[2],
        Point3::new(0.0, 1.0, 0.0)
    );
    assert_eq!(
        model.triangles()[1].vertices()[2],
        Point3::new(0.0, 1.0, 1.0)
    );
}

#[test]
fn load_model_rejects_ascii_stl_without_required_structure() {
    let err = load_model(b"solid broken\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n").unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn load_model_rejects_ascii_stl_with_non_finite_coordinate() {
    let err = load_model(
        b"solid broken\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex inf 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid broken\n",
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn load_model_rejects_binary_stl_with_non_finite_coordinate() {
    let mut bytes = vec![0; 80];
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    append_binary_triangle(
        &mut bytes,
        [[0.0_f32, 0.0, 0.0], [f32::NAN, 0.0, 0.0], [0.0, 1.0, 0.0]],
    );

    let err = load_model(bytes).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

#[test]
fn load_model_rejects_zero_triangle_binary_stl() {
    let mut bytes = vec![0; 80];
    bytes.extend_from_slice(&0_u32.to_le_bytes());

    let err = load_model(bytes).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

fn append_binary_triangle(bytes: &mut Vec<u8>, triangle: [[f32; 3]; 3]) {
    for value in [0.0_f32, 0.0, 1.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in triangle.into_iter().flatten() {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0_u16.to_le_bytes());
}

#[test]
fn load_model_rejects_malformed_stl() {
    let err = load_model(b"solid broken\nvertex 0 0 0\nendsolid broken\n").unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
}

fn one_triangle_ascii_stl(newline: &str) -> Vec<u8> {
    [
        "solid one",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 1 0 0",
        "vertex 0 1 0",
        "endloop",
        "endfacet",
        "endsolid one",
    ]
    .join(newline)
    .into_bytes()
}
