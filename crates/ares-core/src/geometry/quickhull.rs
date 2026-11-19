//! Ordered `Qt` hull for Orca's `TriangleMesh::convex_hull_3d`
//! (`TriangleMesh.cpp:1358–1434`). Predicates use double precision; mesh
//! coordinates and returned facet normals retain upstream float storage.

mod precision;
mod qt;

#[derive(Clone, Debug, PartialEq)]
pub struct ConvexHull3d {
    pub vertices: Vec<[f32; 3]>,
    pub facets: Vec<[u32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

/// An ordered, outward-wound triangulation of the three-dimensional hull.
/// Non-finite and lower-dimensional inputs have no three-dimensional hull.
pub fn convex_hull_3d(input: &[[f32; 3]]) -> Option<ConvexHull3d> {
    try_convex_hull_3d(input).ok().flatten()
}

/// Why the ordered hull could not be constructed for three-dimensional input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HullError {
    DuplicateRidge,
    Precision,
    Topology,
}

impl std::fmt::Display for HullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DuplicateRidge => "Qt hull requires duplicate-ridge repair",
            Self::Precision => "Qt hull exceeded supported precision bounds",
            Self::Topology => "Qt hull encountered an unresolved topology",
        })
    }
}

impl std::error::Error for HullError {}

/// Like [`convex_hull_3d`], but distinguishes an empty hull from a failed construction.
pub fn try_convex_hull_3d(input: &[[f32; 3]]) -> Result<Option<ConvexHull3d>, HullError> {
    if input.len() < 4 || input.iter().flatten().any(|value| !value.is_finite()) {
        return Ok(None);
    }
    qt::triangulate(input)
}
