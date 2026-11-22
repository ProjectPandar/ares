//! BeltKinematics::to_machine and GCodeWriter's coupled-axis words.

use crate::{Point3d, RemapAxis, options::PrinterRemainingOptions, project_slice::belt};

use super::{EmitState, format::axis};

pub(in crate::project_slice::gcode_emit) struct BeltWriter {
    rotation: belt::BeltRotation,
    axes: [(usize, RemapAxis); 3],
    max: [f64; 3],
}

impl BeltWriter {
    pub(in crate::project_slice::gcode_emit) fn new(
        printer: &PrinterRemainingOptions,
    ) -> Option<Self> {
        let rotation = belt::BeltRotation::from_printer(printer)?;
        let axes = belt::remap_axes(printer);
        let (max_x, max_y) = printer
            .printable_area
            .0
            .iter()
            .fold((f64::NEG_INFINITY, f64::NEG_INFINITY), |(x, y), point| {
                (x.max(point.x), y.max(point.y))
            });
        Some(Self {
            rotation,
            axes,
            max: [max_x, max_y, printer.printable_height.0],
        })
    }

    pub(super) fn origin(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let point = self
            .rotation
            .forward()
            .transform_point(Point3d::new(x, y, 0.));
        (point.x, point.y)
    }

    fn apply(&self, point: Point3d) -> [f64; 3] {
        let point = self.rotation.inverse().transform_point(point);
        let point = [point.x, point.y, point.z];
        let remapped = self.axes.map(|(index, axis)| match axis {
            RemapAxis::PosX | RemapAxis::PosY | RemapAxis::PosZ => point[index],
            RemapAxis::NegX | RemapAxis::NegY | RemapAxis::NegZ => -point[index],
            RemapAxis::RevX | RemapAxis::RevY | RemapAxis::RevZ => self.max[index] - point[index],
        });
        self.rotation.machine_frame(remapped)
    }
}

impl EmitState {
    pub(super) fn xy_words(&self, (x, y): (f64, f64), z: Option<f64>) -> String {
        let (x, y) = (x - self.xy_offset.0, y - self.xy_offset.1);
        if let Some(belt) = &self.belt {
            let z = z.or(self.writer_z).unwrap_or(self.layer_z);
            let [x, y, z] = belt.apply(Point3d::new(x, y, z));
            format!("X{} Y{} Z{}", axis(x), axis(y), axis(z))
        } else if let Some(z) = z {
            format!("X{} Y{} Z{}", axis(x), axis(y), axis(z))
        } else {
            format!("X{} Y{}", axis(x), axis(y))
        }
    }

    pub(super) fn z_words(&self, z: f64) -> String {
        if self.belt.is_some() {
            self.xy_words((self.x, self.y), Some(z))
        } else {
            format!("Z{}", axis(z))
        }
    }
}
