//! `MachineFrameTransform` (`GCode/MachineFrameTransform.{hpp,cpp}`),
//! `CartesianKinematics` axis remap (`GCode/MachineKinematics.{hpp,cpp}`),
//! `BeltKinematics` (`GCode/BeltKinematics.{hpp,cpp}`), and `BeltGCode::on_set_origin`
//! (`BeltGCode.cpp:34-48`).

use crate::{
    BeltRotationAxis, Point3d, RemapAxis, options::PrinterRemainingOptions, project::Transform3d,
};

use super::{
    EPSILON,
    back_transform::{BeltBackTransform, build_forward_transform, deg2rad, is_approx_identity},
    remap_axes,
};

/// Post-stage shear/scale machine-frame transform (`GCode/MachineFrameTransform.{hpp,cpp}`).
#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct MachineFrameTransform {
    matrix: Option<Transform3d>,
}

impl MachineFrameTransform {
    /// `MachineFrameTransform::init_from_config` (`GCode/MachineFrameTransform.cpp:11-73`).
    pub(in crate::project_slice) fn from_printer(printer: &PrinterRemainingOptions) -> Self {
        if !printer.belt_printer.0 {
            return Self { matrix: None };
        }
        let axis = printer.belt_slice_rotation;
        if !matches!(axis, BeltRotationAxis::X | BeltRotationAxis::Y) {
            return Self { matrix: None };
        }
        let angle_deg = if printer.belt_frame_tilt_decouple.0 {
            printer.belt_frame_tilt_angle.0
        } else {
            printer.belt_slice_rotation_angle.0
        };
        if angle_deg.abs() <= EPSILON {
            return Self { matrix: None };
        }
        let (sin_a, cos_a) = deg2rad(angle_deg).sin_cos();
        if sin_a.abs() <= EPSILON {
            return Self { matrix: None };
        }
        let cot_a = cos_a / sin_a;
        let inv_sin = 1.0 / sin_a.abs();

        // `MachineFrameTransform.cpp:50-65`: combined = scale * shear.
        // X-axis rotation: (x, y, z) -> (x, inv_sin * y, cot_a * y + z).
        // Y-axis rotation: (x, y, z) -> (inv_sin * x, y, -cot_a * x + z).
        let combined = Transform3d::from_rows(match axis {
            BeltRotationAxis::X => [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, inv_sin, 0.0, 0.0],
                [0.0, cot_a, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            BeltRotationAxis::Y => [
                [inv_sin, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-cot_a, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            BeltRotationAxis::None | BeltRotationAxis::Z => unreachable!("checked above"),
        });
        if is_approx_identity(combined) {
            return Self { matrix: None };
        }
        Self {
            matrix: Some(combined),
        }
    }

    /// `MachineFrameTransform::apply` (`GCode/MachineFrameTransform.cpp:75-80`).
    #[inline]
    pub(in crate::project_slice) fn apply(self, pos: Point3d) -> Point3d {
        self.matrix
            .map_or(pos, |matrix| matrix.transform_point(pos))
    }
}

/// Belt-printer coordinate pipeline (`GCode/BeltKinematics.{hpp,cpp}`):
/// `machine = MachineFrameTransform(axis_remap(BeltBackTransform(logical)))`.
#[derive(Clone, Debug)]
pub(in crate::project_slice) struct BeltKinematics {
    forward: Transform3d,
    back_transform: BeltBackTransform,
    machine_frame: MachineFrameTransform,
    axes: [(usize, RemapAxis); 3],
    has_axis_remap: bool,
    build_vol_max: [f64; 3],
}

impl BeltKinematics {
    /// Installs `BeltKinematics` (`GCode/BeltKinematics.cpp:10-27`, `GCode.cpp:3465-3474`).
    pub(in crate::project_slice) fn from_printer(
        printer: &PrinterRemainingOptions,
    ) -> Option<Self> {
        if !printer.belt_printer.0 {
            return None;
        }
        let forward = build_forward_transform(printer);
        let back_transform = BeltBackTransform::from_forward(true, forward);
        let machine_frame = MachineFrameTransform::from_printer(printer);
        let axes = remap_axes(printer);
        let has_axis_remap = printer.gcode_remap_x != RemapAxis::PosX
            || printer.gcode_remap_y != RemapAxis::PosY
            || printer.gcode_remap_z != RemapAxis::PosZ;
        let (max_x, max_y) = if printer.printable_area.0.is_empty() {
            (0.0, 0.0)
        } else {
            printer
                .printable_area
                .0
                .iter()
                .fold((f64::NEG_INFINITY, f64::NEG_INFINITY), |(x, y), point| {
                    (x.max(point.x), y.max(point.y))
                })
        };
        Some(Self {
            forward,
            back_transform,
            machine_frame,
            axes,
            has_axis_remap,
            build_vol_max: [max_x, max_y, printer.printable_height.0],
        })
    }

    /// `BeltGCode::on_set_origin` (`BeltGCode.cpp:34-48`): projects the 2D origin through
    /// `BeltTransformPipeline::build_forward_transform` so `BeltBackTransform` recovers it.
    #[inline]
    pub(in crate::project_slice) fn project_origin(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let adjusted = self.forward.transform_point(Point3d::new(x, y, 0.0));
        (adjusted.x, adjusted.y)
    }

    /// `CartesianKinematics::apply_axis_remap` (`GCode/MachineKinematics.cpp:7-18`).
    #[inline]
    fn apply_axis_remap(&self, pos: Point3d) -> Point3d {
        if !self.has_axis_remap {
            return pos;
        }
        let coords = [pos.x, pos.y, pos.z];
        let [x, y, z] = self.axes.map(|(index, axis)| match axis {
            RemapAxis::PosX | RemapAxis::PosY | RemapAxis::PosZ => coords[index],
            RemapAxis::NegX | RemapAxis::NegY | RemapAxis::NegZ => -coords[index],
            RemapAxis::RevX | RemapAxis::RevY | RemapAxis::RevZ => {
                self.build_vol_max[index] - coords[index]
            }
        });
        Point3d::new(x, y, z)
    }

    /// `BeltKinematics::to_machine` (`GCode/BeltKinematics.cpp:17-22`).
    #[inline]
    pub(in crate::project_slice) fn to_machine(&self, pos: Point3d) -> [f64; 3] {
        let after_back = self.back_transform.apply(pos);
        let after_remap = self.apply_axis_remap(after_back);
        let machine = self.machine_frame.apply(after_remap);
        [machine.x, machine.y, machine.z]
    }
}
