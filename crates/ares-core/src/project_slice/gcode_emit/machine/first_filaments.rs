use crate::{
    OrcaInts, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::super::{tool_ordering::ToolOrdering, value};

/// `ToolOrdering::cal_non_support_filaments` and the physical-hotend remap in
/// `GCode.cpp:2721-2851`. Unused physical hotends remain `-1`.
/// This export metadata shell also caches the non-tower total-change
/// prediction installed by `DoExport` (`GCode.cpp:2289–2309,2896`).
#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct InitialFilaments {
    initial: usize,
    initial_bed_temperature: i32,
    first_layer_tools: Vec<usize>,
    non_support: usize,
    initial_hotend: i32,
    non_support_hotend: i32,
    first: Vec<i32>,
    first_non_support: Vec<i32>,
    total_toolchanges: usize,
    used_extruders: Option<value::Value>,
}

impl InitialFilaments {
    pub(in crate::project_slice::gcode_emit) fn total_toolchanges(&self) -> usize {
        self.total_toolchanges
    }

    pub(in crate::project_slice::gcode_emit) fn initial(&self) -> usize {
        self.initial
    }

    pub(in crate::project_slice::gcode_emit) fn initial_bed_temperature(&self) -> i32 {
        self.initial_bed_temperature
    }

    pub(in crate::project_slice::gcode_emit) fn first_layer_tools(&self) -> &[usize] {
        &self.first_layer_tools
    }

    pub(in crate::project_slice::gcode_emit) fn non_support(&self) -> usize {
        self.non_support
    }

    pub(in crate::project_slice::gcode_emit) fn insert(&self, config: &mut value::Config) {
        config.insert(
            "bed_temperature_initial_layer_single",
            value::Value::Integer(i64::from(self.initial_bed_temperature)),
        );
        if let Some(used) = &self.used_extruders {
            config.insert("is_extruder_used", used.clone());
        }
        config.insert(
            "total_toolchanges",
            value::Value::Integer(self.total_toolchanges as i64),
        );
        for (name, id) in [
            ("initial_tool", self.initial),
            ("initial_extruder", self.initial),
            ("initial_no_support_tool", self.non_support),
            ("initial_no_support_extruder", self.non_support),
        ] {
            config.insert(name, value::Value::Integer(id as i64));
        }
        config.insert(
            "initial_no_support_hotend",
            value::Value::Integer(i64::from(self.non_support_hotend)),
        );
        for name in ["first_tools", "first_filaments"] {
            config.insert(name, as_value(&self.first));
        }
        for name in ["first_non_support_tools", "first_non_support_filaments"] {
            config.insert(name, as_value(&self.first_non_support));
        }
    }

    pub(super) fn insert_machine(&self, config: &mut value::Config) {
        self.insert(config);
        for name in ["current_extruder", "filament_extruder_id"] {
            config.insert(name, value::Value::Integer(self.initial as i64));
        }
        config.insert(
            "current_hotend",
            value::Value::Integer(i64::from(self.initial_hotend)),
        );
    }
}

pub(in crate::project_slice::gcode_emit) fn resolve(
    traversal: &PreparedPostClassicTraversal,
    ordering: &ToolOrdering,
) -> InitialFilaments {
    let settings = &traversal.resolved.views.full;
    let physical_count = settings.project.print.nozzle_diameter.0.len().max(1);
    let mut first = vec![-1; physical_count];
    let mut first_non_support = vec![-1; physical_count];
    let filament_map = &settings.project.gcode.filament_map;

    let is_support = |filament| {
        settings
            .filament
            .gcode
            .filament_is_support
            .0
            .get(filament)
            .or_else(|| settings.filament.gcode.filament_is_support.0.first())
            .is_some_and(|value| value.0)
    };
    let initial = ordering.initial_extruder(settings);
    let has_non_support = ordering.all_extruders().iter().any(|&id| !is_support(id));
    let mut non_support = None;
    let mut found_first = 0;
    let mut found_non_support = 0;
    for filament in ordering.ordered_tools() {
        let slot = (!filament_map.0.is_empty()).then(|| (filament_map.0[filament].0 - 1) as usize);
        if let Some(slot) = slot
            && first[slot] == -1
        {
            first[slot] = filament as i32;
            found_first += 1;
        }
        if !has_non_support {
            if found_first == physical_count || slot.is_none() {
                break;
            }
            continue;
        }
        if is_support(filament) {
            continue;
        }
        non_support.get_or_insert(filament);
        let Some(slot) = slot else {
            break;
        };
        if first_non_support[slot] == -1 {
            first_non_support[slot] = filament as i32;
            found_non_support += 1;
        }
        if found_non_support == physical_count {
            break;
        }
    }

    // `GCode.cpp:2834-2842` applies the physical-extruder remap
    // UNCONDITIONALLY (`physical[map.get_at(i)] = filaments[i]`) — no
    // heterogeneous-extruder gate.
    first = apply_physical_map(first, &settings.printer.gcode.physical_extruder_map);
    first_non_support = apply_physical_map(
        first_non_support,
        &settings.printer.gcode.physical_extruder_map,
    );

    let non_support = non_support.unwrap_or(initial);
    let hotend = |filament: usize| {
        if settings.printer.remaining.printer_model.0 == "Bambu Lab X2D" {
            -1
        } else {
            filament_map.0.get(filament).map_or(0, |value| value.0 - 1)
        }
    };
    let first_layer_tools = ordering
        .layers
        .first()
        .map_or_else(Vec::new, |layer| layer.extruders.clone());
    let initial_bed_temperature =
        super::temperature::first_layer_bed_temperature(traversal, initial, &first_layer_tools);
    InitialFilaments {
        initial,
        initial_bed_temperature,
        first_layer_tools,
        non_support,
        initial_hotend: hotend(initial),
        non_support_hotend: hotend(non_support),
        first,
        first_non_support,
        // `GCode.cpp:2901-2904`: by-layer export uses finalized logical tools,
        // not the configured object-extruder candidate set.
        used_extruders: (settings.process.print.print_sequence
            == crate::ProcessPrintSequence::ByLayer)
            .then(|| {
                let mut used = vec![
                    value::Value::option_bool(false);
                    settings.filament.gcode.filament_diameter.0.len().max(64)
                ];
                for &id in ordering.all_extruders() {
                    used[id] = value::Value::option_bool(true);
                }
                value::Value::List(used)
            }),
        // Sequential upstream leaves the global Print tool ordering empty.
        total_toolchanges: match settings.process.print.print_sequence {
            crate::ProcessPrintSequence::ByLayer => ordering.total_toolchanges(),
            crate::ProcessPrintSequence::ByObject => 0,
        },
    }
}

fn int_at(values: &OrcaInts, index: usize) -> i32 {
    values
        .0
        .get(index)
        .or_else(|| values.0.first())
        .map_or(0, |value| value.0)
}

fn apply_physical_map(filaments: Vec<i32>, physical_map: &OrcaInts) -> Vec<i32> {
    let mut remapped = vec![-1; filaments.len()];
    for (index, filament) in filaments.into_iter().enumerate() {
        let target = int_at(physical_map, index);
        if let Ok(target) = usize::try_from(target)
            && target < remapped.len()
        {
            remapped[target] = filament;
        }
    }
    remapped
}

fn as_value(filaments: &[i32]) -> value::Value {
    value::Value::List(
        filaments
            .iter()
            .map(|&filament| value::Value::Integer(i64::from(filament)))
            .collect(),
    )
}
