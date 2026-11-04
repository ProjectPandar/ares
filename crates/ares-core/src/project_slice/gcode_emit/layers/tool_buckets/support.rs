//! Support selectors in `GCode.cpp:4870–4967`, including wipe overrides.

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{ExtrusionRole, ObjectOptions};

pub(in crate::project_slice::gcode_emit::layers) fn selectors(
    roles: impl Iterator<Item = ExtrusionRole>,
    options: &ObjectOptions,
    traversal: &PreparedPostClassicTraversal,
    tools: &[usize],
    overrides: [Option<usize>; 2],
) -> Vec<(usize, ExtrusionRole)> {
    let role = roles.fold(ExtrusionRole::None, |current, next| {
        if current == ExtrusionRole::None || current == next {
            next
        } else {
            ExtrusionRole::Mixed
        }
    });
    if role == ExtrusionRole::None {
        return Vec::new();
    }
    let has_base = matches!(
        role,
        ExtrusionRole::Mixed | ExtrusionRole::SupportMaterial | ExtrusionRole::SupportTransition
    );
    let has_interface = matches!(
        role,
        ExtrusionRole::Mixed | ExtrusionRole::SupportMaterialInterface
    );
    let mut base_dontcare = options.support_filament.0 == 0;
    let mut interface_dontcare = options.support_interface_filament.0 == 0;
    let mut base = (options.support_filament.0 - 1) as usize;
    let mut interface = (options.support_interface_filament.0 - 1) as usize;
    if base_dontcare && let Some(tool) = overrides[0] {
        base = tool;
        base_dontcare = false;
    }
    if interface_dontcare && let Some(tool) = overrides[1] {
        interface = tool;
        interface_dontcare = false;
    }
    let runtime = &traversal.resolved.views.runtime_gcode;
    let soluble = |id: usize| {
        runtime
            .filament_soluble
            .0
            .get(id)
            .unwrap_or(&runtime.filament_soluble.0[0])
            .0
    };
    let support = |id: usize| {
        runtime
            .filament_is_support
            .0
            .get(id)
            .unwrap_or(&runtime.filament_is_support.0[0])
            .0
    };
    let mut dontcare = tools[0];
    if base_dontcare && !interface_dontcare {
        if let Some(&id) = tools.iter().find(|&&id| !soluble(id) && id != interface) {
            dontcare = id;
        }
        base = dontcare;
    } else if base_dontcare || interface_dontcare {
        if soluble(dontcare)
            && let Some(&id) = tools.iter().find(|&&id| !soluble(id))
        {
            dontcare = id;
        }
        if support(dontcare)
            && let Some(&id) = tools.iter().find(|&&id| !support(id))
        {
            dontcare = id;
        }
        if base_dontcare {
            base = dontcare;
        }
        if interface_dontcare {
            interface = dontcare;
        }
    }
    let single = !has_base || base == interface;
    let mut result = vec![(
        if has_base { base } else { interface },
        if single {
            ExtrusionRole::Mixed
        } else {
            ExtrusionRole::SupportMaterial
        },
    )];
    if !single && has_interface {
        result.push((interface, ExtrusionRole::SupportMaterialInterface));
    }
    result
}
