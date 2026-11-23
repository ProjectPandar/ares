use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

use super::super::value::{Config, Value};

#[derive(Default)]
pub(super) struct Mode {
    name: String,
    index: usize,
    pub(super) script: String,
}

impl Mode {
    pub(super) fn resolve(traversal: &PreparedPostClassicTraversal) -> Self {
        let printer = &traversal.resolved.views.full.printer.gcode;
        if !printer.is_imex.0 || traversal.objects.is_empty() {
            return Self::default();
        }
        let (first, _) = traversal.print_object_config(0);
        let mut name = first.object.imex_parallel_mode.0.as_str();
        const PRIMARY: &str = "primary";
        if name.is_empty() {
            name = PRIMARY;
        }
        // IMEXHelpers.cpp:648-678: exact name lookup with independent sibling arrays.
        let find = |name| printer.imex_mode_names.0.iter().position(|row| row == name);
        let mut row = find(name);
        // GCode.cpp:4141-4219: unsupported parallel modes fall back to Primary.
        if name != PRIMARY
            && row
                .and_then(|index| printer.imex_mode_active_tools.0.get(index))
                .is_none_or(String::is_empty)
        {
            name = PRIMARY;
            row = find(name);
        }
        Self {
            name: name.to_owned(),
            index: row.unwrap_or_default(),
            script: row
                .and_then(|index| printer.imex_mode_gcodes.0.get(index))
                .cloned()
                .unwrap_or_default(),
        }
    }

    pub(super) fn insert(&self, config: &mut Config) {
        config.insert("imex_mode", Value::String(self.name.clone()));
        config.insert("imex_mode_index", Value::Integer(self.index as i64));
        config.insert("imex_mode_gcode", Value::String(self.script.clone()));
    }
}
