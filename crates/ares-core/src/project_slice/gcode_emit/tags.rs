//! Processor reserved-tag vocabulary, selected by printer family
//! (`GCodeProcessor.hpp:375`, `GCode.cpp:4617-4625`). Bambu Lab printers
//! emit the BBL tag set (`; CHANGE_LAYER`, `; Z_HEIGHT:`, `; LAYER_HEIGHT:`,
//! `; FEATURE: `); every other printer emits the compatible set
//! (`;LAYER_CHANGE`, `;Z:`, `;HEIGHT:`, `;TYPE:`).

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) const CHANGE_LAYER: &str = "; CHANGE_LAYER";
pub(super) const LAYER_CHANGE: &str = ";LAYER_CHANGE";
pub(super) const FEATURE: &str = "; FEATURE: ";
pub(super) const TYPE: &str = ";TYPE:";

#[derive(Clone, Copy)]
pub(super) struct Tags {
    bbl: bool,
}

impl Default for Tags {
    fn default() -> Self {
        Self { bbl: true }
    }
}

impl Tags {
    pub(super) fn of(traversal: &PreparedPostClassicTraversal) -> Self {
        Self {
            bbl: traversal
                .resolved
                .views
                .full
                .printer
                .remaining
                .printer_model
                .0
                .starts_with("Bambu Lab"),
        }
    }

    pub(super) fn layer_change(&self) -> &str {
        if self.bbl { CHANGE_LAYER } else { LAYER_CHANGE }
    }

    pub(super) fn z(&self, z: &str) -> String {
        if self.bbl {
            format!("; Z_HEIGHT: {z}")
        } else {
            format!(";Z:{z}")
        }
    }

    pub(super) fn height(&self, height: &str) -> String {
        if self.bbl {
            format!("; LAYER_HEIGHT: {height}")
        } else {
            format!(";HEIGHT:{height}")
        }
    }

    pub(super) fn feature(&self, feature: &str) -> String {
        if self.bbl {
            format!("{FEATURE}{feature}")
        } else {
            format!("{TYPE}{feature}")
        }
    }

    pub(super) fn width(&self, width: &str) -> String {
        if self.bbl {
            format!("; LINE_WIDTH: {width}")
        } else {
            format!(";WIDTH:{width}")
        }
    }

    pub(super) fn custom(&self) -> String {
        self.feature("Custom")
    }

    pub(super) const fn is_bbl(&self) -> bool {
        self.bbl
    }
}
