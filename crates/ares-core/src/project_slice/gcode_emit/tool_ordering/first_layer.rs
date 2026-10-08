//! `generate_first_layer_tool_order` and `handle_dontcare_extruder`.

use std::collections::BTreeMap;

use super::LayerTools;
use crate::geometry::{ExPolygon, JoinType, fixed_gcc_sort_by, offset_expolygon};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::project_slice::perimeters::types::PostPerimeterInputPrintObject;
use crate::{FloatOrPercent, SliceError};

pub(super) fn generate(traversal: &PreparedPostClassicTraversal) -> Result<Vec<usize>, SliceError> {
    let configured = traversal
        .resolved
        .views
        .full
        .process
        .print
        .initial_layer_line_width;
    let (raw_width, width) = match configured {
        FloatOrPercent::Float(value) => (value, value),
        FloatOrPercent::Percent(value) => (
            value.0,
            traversal
                .resolved
                .views
                .full
                .project
                .print
                .nozzle_diameter
                .0[0]
                .0
                * value.0
                / 100.0,
        ),
    };
    let mut areas = BTreeMap::<usize, f64>::new();
    for object in &traversal.objects {
        let input = &object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let Some(layer) = target_layer(
            input,
            object.physical_layers().len(),
            raw_width,
            traversal.scale.factor(),
        )?
        else {
            return Ok(Vec::new());
        };
        collect_areas(input, layer, width, traversal.scale.factor(), &mut areas)?;
    }
    let mut tools = Vec::with_capacity(areas.len());
    for (&id, &area) in &areas {
        let before = tools
            .iter()
            .position(|other| areas[other] < area)
            .unwrap_or(tools.len());
        tools.insert(before, id);
    }
    apply_configured_order(&mut tools, traversal);
    Ok(tools)
}

fn target_layer(
    input: &PostPerimeterInputPrintObject,
    count: usize,
    width: f64,
    scale: f64,
) -> Result<Option<usize>, SliceError> {
    let regions = input.as_parts().0.as_parts().0.as_parts().2;
    for layer in 0..count {
        for polygon in (0..regions.len()).flat_map(|region| input.raw_slices(region, layer)) {
            if survives(polygon, width, scale)? {
                return Ok(Some(layer));
            }
        }
    }
    Ok(None)
}

fn collect_areas(
    input: &PostPerimeterInputPrintObject,
    layer: usize,
    width: f64,
    scale: f64,
    areas: &mut BTreeMap<usize, f64>,
) -> Result<(), SliceError> {
    let regions = input.as_parts().0.as_parts().0.as_parts().2;
    for (index, region) in regions.iter().enumerate() {
        let id = region.options.outer_wall_filament_id.0 as usize;
        for polygon in input.raw_slices(index, layer) {
            if survives(polygon, width, scale)? {
                let area = polygon.contour().area();
                areas
                    .entry(id)
                    .and_modify(|minimum| *minimum = minimum.min(area))
                    .or_insert(area);
            }
        }
    }
    Ok(())
}

fn survives(polygon: &ExPolygon, width: f64, scale: f64) -> Result<bool, SliceError> {
    offset_expolygon(
        polygon,
        (-0.2 * (width / scale)) as f32,
        JoinType::Miter,
        3.0,
    )
    .map(|polygons| !polygons.is_empty())
    .map_err(|_| {
        SliceError::InvalidInput(
            "first-layer tool ordering geometry is outside the Clipper range".into(),
        )
    })
}

fn apply_configured_order(tools: &mut [usize], traversal: &PreparedPostClassicTraversal) {
    let configured = &traversal
        .resolved
        .views
        .full
        .project
        .print
        .first_layer_print_sequence
        .0;
    if configured.len() >= tools.len() {
        fixed_gcc_sort_by(tools, |left, right| {
            let left = configured.iter().position(|id| id.0 as usize == *left);
            let right = configured.iter().position(|id| id.0 as usize == *right);
            matches!((left, right), (Some(left), Some(right)) if left < right)
        });
    }
}

pub(super) fn handle_dontcare(
    layers: &mut [LayerTools],
    first_order: &[usize],
    traversal: &PreparedPostClassicTraversal,
) {
    let Some(first) = layers.first_mut() else {
        return;
    };
    let mut last;
    let start;
    if !first_order.is_empty() {
        let mut remaining = std::mem::take(&mut first.extruders);
        for &id in first_order {
            if let Some(index) = remaining.iter().position(|&value| value == id) {
                first.extruders.push(id);
                remaining[index] = usize::MAX;
            }
        }
        first.extruders.extend(
            remaining
                .into_iter()
                .filter(|&id| id != 0 && id != usize::MAX),
        );
        if first.extruders.is_empty() {
            first.extruders.push(first_order[0]);
        }
        last = *first
            .extruders
            .last()
            .expect("first-layer preference seeds tools");
        start = 1;
    } else {
        let Some(id) = layers
            .iter()
            .flat_map(|layer| &layer.extruders)
            .copied()
            .find(|&id| id > 0)
        else {
            return;
        };
        last = id;
        start = 0;
    }
    for (index, layer) in layers.iter_mut().enumerate().skip(start) {
        if layer.extruders.is_empty() {
            continue;
        }
        if layer.extruders.len() == 1 && layer.extruders[0] == 0 {
            layer.extruders[0] = last;
        } else {
            if layer.extruders[0] == 0 {
                layer.extruders.remove(0);
            }
            if let Some(index) = layer.extruders.iter().position(|&id| id == last) {
                layer.extruders[..=index].rotate_right(1);
            }
            if index == 0 {
                prefer_first_layer(&mut layer.extruders, traversal);
            }
        }
        last = *layer.extruders.last().expect("nonempty layer owns a tool");
    }
    for id in layers.iter_mut().flat_map(|layer| &mut layer.extruders) {
        *id -= 1;
    }
}

fn prefer_first_layer(tools: &mut [usize], traversal: &PreparedPostClassicTraversal) {
    let runtime = &traversal.resolved.views.runtime_gcode;
    if traversal
        .resolved
        .views
        .full
        .process
        .print
        .enable_prime_tower
        .0
        && let Some(index) = tools.iter().position(|&id| {
            runtime
                .filament_soluble
                .0
                .get(id - 1)
                .unwrap_or(&runtime.filament_soluble.0[0])
                .0
        })
    {
        tools.swap(0, index);
    }
    apply_configured_order(tools, traversal);
}

/// `ToolOrdering::handle_dontcare_extruder(last_extruder_id)`
/// (`ToolOrdering.cpp:253-300`), the per-object seed path of
/// `ToolOrdering(object, first_extruder)` (`ToolOrdering.cpp:396-436`):
/// the seeded extruder leads every layer's tool order and resolves the
/// dontcare placeholder. Ares stores 0-based ids (upstream is 1-based
/// with 0 as dontcare), so the caller's `++` conversion is absorbed.
pub(super) fn handle_dontcare_seed(layers: &mut [LayerTools], seed: Option<usize>) {
    // `ToolOrdering.cpp:258-271`: an unset seed resolves to the first
    // extruder used anywhere in the print; with none, nothing changes.
    let seed = match seed {
        Some(seed) => seed,
        None => {
            let found = layers
                .iter()
                .flat_map(|layer| layer.extruders.iter().copied())
                .find(|id| *id != 0);
            let Some(found) = found else { return };
            found
        }
    };
    for layer in layers.iter_mut() {
        if layer.extruders.is_empty() {
            continue;
        }
        if layer.extruders.len() == 1 && layer.extruders[0] == 0 {
            layer.extruders[0] = seed;
        } else {
            if layer.extruders[0] == 0 {
                layer.extruders.remove(0);
            }
            if let Some(position) = layer.extruders.iter().position(|&id| id == seed) {
                layer.extruders[..=position].rotate_right(1);
            }
        }
    }
}
