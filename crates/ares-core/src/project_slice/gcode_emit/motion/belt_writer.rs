//! BeltKinematics::to_machine and GCodeWriter's coupled-axis words.

use crate::{
    Point3d, options::PrinterRemainingOptions, project_slice::belt::kinematics::BeltKinematics,
};

use super::{EmitState, format::push_axis};

pub(in crate::project_slice::gcode_emit) struct BeltWriter {
    kinematics: BeltKinematics,
}

impl BeltWriter {
    pub(in crate::project_slice::gcode_emit) fn new(
        printer: &PrinterRemainingOptions,
    ) -> Option<Self> {
        BeltKinematics::from_printer(printer).map(|kinematics| Self { kinematics })
    }

    pub(super) fn origin(&self, origin: (f64, f64)) -> (f64, f64) {
        self.kinematics.project_origin(origin)
    }

    fn apply(&self, point: Point3d) -> [f64; 3] {
        self.kinematics.to_machine(point)
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

    pub(super) fn spiral_travel_words(&self, point: Point3d) -> String {
        let Some(belt) = &self.belt else {
            return self.xy_words((point.x, point.y), Some(point.z));
        };
        let (x, y) = (point.x - self.xy_offset.0, point.y - self.xy_offset.1);
        // `GCodeWriter::travel_to_xyz` emits the transformed destination
        // (`GCodeWriter.cpp:1113–1122`), including machine Z under a belt.
        let [x, y, z] = belt.apply(Point3d::new(x, y, point.z));
        let mut buf = Vec::with_capacity(32);
        buf.push(b'X');
        push_axis(&mut buf, x);
        buf.extend_from_slice(b" Y");
        push_axis(&mut buf, y);
        buf.extend_from_slice(b" Z");
        push_axis(&mut buf, z);
        String::from_utf8(buf).expect("spiral travel words are ASCII")
    }

    pub(in crate::project_slice::gcode_emit) fn z_words(&self, z: f64) -> String {
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
