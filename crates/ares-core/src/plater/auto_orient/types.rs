use crate::ProjectMesh;

/// `Orient.hpp:34-56`: a mesh with the class constructor's per-object context.
#[derive(Clone, Copy, Debug)]
pub struct OrientMesh<'a> {
    pub mesh: &'a ProjectMesh,
    pub overhang_angle: f64,
    pub cooling_direction: Option<[f64; 3]>,
}

/// Active `OrientParams` fields (`Orient.hpp:59-96`). Scheduling is external:
/// worker count never selects whether cooling/Euler results are calculated.
#[derive(Clone, Copy, Debug)]
pub struct OrientParams {
    pub tar_a: f32,
    pub tar_b: f32,
    pub relative_f: f32,
    pub contour_f: f32,
    pub bottom_f: f32,
    pub bottom_hull_f: f32,
    pub tar_c: f32,
    pub tar_d: f32,
    pub tar_e: f32,
    pub first_layer_height: f32,
    pub ascent: f32,
    pub low_angle_max: f32,
    pub low_angle_min: f32,
    pub tar_low_angle: f32,
    pub tar_projected_area: f32,
    pub bottom_min: f32,
    pub appearance_face_support: f32,
    pub use_low_angle_face: bool,
    pub min_volume: bool,
}
impl Default for OrientParams {
    fn default() -> Self {
        Self {
            tar_a: 0.01,
            tar_b: 0.177,
            relative_f: 6.610_621,
            contour_f: 0.232_286_23,
            bottom_f: 1.167_152,
            bottom_hull_f: 0.1,
            tar_c: 0.243_080_7,
            tar_d: 0.628_451_5,
            tar_e: 0.,
            first_layer_height: 0.2,
            ascent: -0.5,
            low_angle_max: 0.999,
            low_angle_min: 0.9703,
            tar_low_angle: 0.01,
            tar_projected_area: 0.1,
            bottom_min: 0.1,
            appearance_face_support: 3.,
            use_low_angle_face: true,
            min_volume: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoolingOrientation {
    pub direction: [f64; 3],
    pub axis: [f64; 3],
    pub angle: f64,
    pub rotation_matrix: [[f64; 3]; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Orientation {
    pub direction: [f64; 3],
    pub axis: [f64; 3],
    pub angle: f64,
    pub rotation_matrix: [[f64; 3]; 3],
    pub euler_angles: [f64; 3],
    pub cooling: Option<CoolingOrientation>,
}
