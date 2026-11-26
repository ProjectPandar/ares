//! BeltKinematics::to_machine and GCodeWriter's coupled-axis words.

use crate::{Point3d, RemapAxis, options::PrinterRemainingOptions, project_slice::belt};

use super::{EmitState, format::push_axis};

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
    pub(super) fn push_xy_words(&self, output: &mut Vec<u8>, (x, y): (f64, f64), z: Option<f64>) {
        let (x, y) = (x - self.xy_offset.0, y - self.xy_offset.1);
        let (x, y, z) = if let Some(belt) = &self.belt {
            let z = z.or(self.writer_z).unwrap_or(self.layer_z);
            let [x, y, z] = belt.apply(Point3d::new(x, y, z));
            (x, y, Some(z))
        } else {
            (x, y, z)
        };
        output.push(b'X');
        push_axis(output, x);
        output.extend_from_slice(b" Y");
        push_axis(output, y);
        if let Some(z) = z {
            output.extend_from_slice(b" Z");
            push_axis(output, z);
        }
    }

    pub(super) fn xy_words(&self, (x, y): (f64, f64), z: Option<f64>) -> String {
        let mut buf = Vec::with_capacity(32);
        self.push_xy_words(&mut buf, (x, y), z);
        String::from_utf8(buf).expect("XY words are ASCII")
    }

    pub(super) fn z_words(&self, z: f64) -> String {
        if self.belt.is_some() {
            self.xy_words((self.x, self.y), Some(z))
        } else {
            let mut buf = Vec::with_capacity(16);
            buf.push(b'Z');
            push_axis(&mut buf, z);
            String::from_utf8(buf).expect("Z word is ASCII")
        }
    }
}
