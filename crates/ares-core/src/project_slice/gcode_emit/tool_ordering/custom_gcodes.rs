//! `ToolOrdering::assign_custom_gcodes` (`ToolOrdering.cpp:1454-1516`).
use super::ToolOrdering;
use crate::{
    CustomGCodeMode,
    gcode_processor::CustomGCodeType,
    project_slice::{
        extruders::collect_project_object_extruders,
        perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

impl ToolOrdering {
    /// By-layer orderings only: `GCode.cpp:2764-2767`. The sequential
    /// branch (`:2758-2760`) never assigns.
    pub(in crate::project_slice::gcode_emit) fn assign_custom_gcodes(
        &mut self,
        traversal: &PreparedPostClassicTraversal,
    ) {
        let info = traversal.project.curr_plate_custom_gcodes();
        if info.gcodes.is_empty() {
            return;
        }
        let num_filaments = traversal
            .resolved
            .views
            .full
            .filament
            .gcode
            .filament_diameter
            .0
            .len();
        let mode = if num_filaments == 1 {
            CustomGCodeMode::SingleExtruder
        } else if object_extruder_count(traversal) == 1 {
            CustomGCodeMode::MultiAsSingle
        } else {
            CustomGCodeMode::MultiExtruder
        };
        let ignore_tool_and_color_changes = (mode == CustomGCodeMode::MultiExtruder)
            != (info.mode == CustomGCodeMode::MultiExtruder);
        let mut extruder_printing_above = vec![false; num_filaments];
        let mut items = info.gcodes.into_iter().rev().peekable();
        let mut print_z_above = f64::MIN;
        for index in (0..self.layers.len()).rev() {
            let print_z = self.layers[index].print_z;
            for &extruder in &self.layers[index].extruders {
                extruder_printing_above[extruder] = true;
            }
            // Skip events above this layer's upper midpoint (only when a
            // layer is above) and every ToolChange (`:1485-1487`).
            while items
                .next_if(|item| {
                    (print_z_above > print_z && item.print_z > 0.5 * (print_z + print_z_above))
                        || item.kind == CustomGCodeType::ToolChange
                })
                .is_some()
            {}
            print_z_above = print_z;
            if items.peek().is_none() {
                break;
            }
            let print_z_below = index
                .checked_sub(1)
                .map_or(0.0, |below| self.layers[below].print_z);
            // Applied or not, the event is consumed (`:1513`).
            let Some(item) = items.next_if(|item| item.print_z > 0.5 * (print_z_below + print_z))
            else {
                continue;
            };
            let applies = match item.kind {
                CustomGCodeType::ColorChange => {
                    !ignore_tool_and_color_changes
                        && (mode == CustomGCodeMode::SingleExtruder
                            || (item.extruder as usize <= num_filaments
                                && extruder_printing_above[item.extruder as usize - 1]))
                }
                CustomGCodeType::ToolChange => {
                    unreachable!("the skip loop consumes every ToolChange")
                }
                CustomGCodeType::PausePrint
                | CustomGCodeType::Template
                | CustomGCodeType::Custom
                | CustomGCodeType::Unknown => true,
            };
            if applies {
                self.layers[index].custom_gcode = Some(item);
            }
        }
    }
}

/// `Print::object_extruders().size()` (`Print.cpp:451-483`): region and
/// volume extruders plus the layer-range `extruder` values above 0.
fn object_extruder_count(traversal: &PreparedPostClassicTraversal) -> usize {
    let mut extruders = collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    )
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    for resolved in &traversal.resolved.objects {
        let source = &traversal.project.objects()[resolved.source_object_index];
        extruders.extend(
            source
                .layer_config_ranges()
                .iter()
                .filter_map(|range| range.region_overrides().extruder)
                .filter(|value| value.0 > 0)
                .map(|value| value.0 as usize - 1),
        );
    }
    extruders.sort_unstable();
    extruders.dedup();
    extruders.len()
}
