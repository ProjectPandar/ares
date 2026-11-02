use crate::geometry::Point;

/// `Brim.cpp:1004-1010`: one map entry's infill — an uncombined owner's
/// copy emits exactly these paths at its own chunk
/// (`GCode.cpp:5991-6082` keys emission on `ObjectInstanceID`).
pub(crate) struct InstanceBrim {
    pub(crate) copy: usize,
    pub(crate) paths: Vec<Vec<Point>>,
}

/// `m_objectBrimAreasByInstance` entry (`Brim.cpp:984-996`): the
/// per-instance plate-frame brim area consumed by skirt grouping
/// (`Print.cpp:4195-4202`).
pub(crate) struct InstanceArea {
    pub(crate) copy: usize,
    pub(crate) area: Vec<crate::geometry::ExPolygon>,
}

pub(crate) struct BrimPlan {
    pub(crate) area: Vec<crate::geometry::ExPolygon>,
    pub(crate) instances: Vec<InstanceBrim>,
    pub(crate) instance_areas: Vec<InstanceArea>,
    pub(crate) paths: Vec<Vec<Point>>,
    pub(crate) covered_hull: Vec<Point>,
    pub(crate) width: f32,
    pub(crate) spacing: f32,
    pub(crate) height: f32,
    pub(crate) mm3_per_mm: f64,
}
