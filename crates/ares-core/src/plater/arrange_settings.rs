//! The arrange settings popup's model (`GLCanvas3D::_render_arrange_menu`,
//! `GLCanvas3D.cpp:6654-6778` at v2.5.0-beta `345efec64`): its defaults, the rules it applies
//! while shown, and the app-config set the settings belong to.

use super::{ArrangeSettings, PlaterSession};
use crate::options::{PrinterStructure, ProcessPrintSequence};
use crate::project::raw_settings::RawConfigValue;

impl PlaterSession {
    /// The settings after "Reset" (`GLCanvas3D.cpp:6757-6767`) and after a printer change
    /// (`Plater.cpp:3994-3999`): the `ArrangeSettings` defaults (`GLCanvas3D.hpp:603-614`) with
    /// align to Y on an i3 printer.
    pub fn default_arrange_settings(&self) -> ArrangeSettings {
        ArrangeSettings {
            distance: 0.0,
            enable_rotation: false,
            allow_multi_materials_on_same_plate: true,
            avoid_extrusion_cali_region: true,
            align_to_y_axis: self.project().settings().printer.gcode.printer_structure
                == PrinterStructure::I3,
        }
    }

    /// Whether the popup offers "Avoid extrusion calibration region": a Bambu Lab printer that
    /// scans the first layer (`GLCanvas3D.cpp:6725-6734`). Bambu Lab is told by `printer_model`,
    /// as the CLI does (`WipeTowerEstimate.cpp:64-67`).
    pub fn shows_calibration_region(&self) -> bool {
        let project = self.project();
        let scans = project
            .documents()
            .project_settings_raw
            .get("scan_first_layer")
            .is_some_and(|value| match value {
                RawConfigValue::Text(text) => text == "1",
                RawConfigValue::Flag(flag) => *flag,
                _ => false,
            });
        scans
            && project
                .settings()
                .printer
                .remaining
                .printer_model
                .0
                .starts_with("Bambu Lab")
    }

    /// `settings` as the popup leaves them (`GLCanvas3D.cpp:6712-6745`): spacing of at least 0,
    /// no calibration region where the option is not shown, and no align to Y while auto rotate
    /// is on.
    pub fn shown_arrange_settings(&self, settings: ArrangeSettings) -> ArrangeSettings {
        ArrangeSettings {
            distance: settings.distance.max(0.0),
            avoid_extrusion_cali_region: settings.avoid_extrusion_cali_region
                && self.shows_calibration_region(),
            align_to_y_axis: settings.align_to_y_axis && !settings.enable_rotation,
            ..settings
        }
    }

    /// The app-config postfix of the settings set in use (`GLCanvas3D.cpp:6690-6700`): the
    /// global print sequence picks it (`:1563-1580`).
    pub fn arrange_settings_key(&self) -> &'static str {
        if self.project().settings().process.print.print_sequence == ProcessPrintSequence::ByObject
        {
            "_fff_seq_print"
        } else {
            "_fff"
        }
    }
}
