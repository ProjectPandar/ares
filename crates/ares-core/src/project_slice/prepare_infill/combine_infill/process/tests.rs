use super::groups;
use super::{EPSILON, LayerConfig};
use crate::{ProcessInfillPattern, project_slice::perimeters::types::PerimeterFlows};

fn config(height: f64, maximum: f64) -> LayerConfig {
    LayerConfig {
        active: true,
        kind: crate::project_slice::region_slices::RegionSurfaceKind::Internal,
        pattern: ProcessInfillPattern::CrossHatch,
        maximum_height: maximum,
        area_threshold: 0.0,
        layer_height: height,
        flows: PerimeterFlows {
            perimeter_flow: flow(),
            ext_perimeter_flow: flow(),
            overhang_flow: flow(),
            solid_infill_flow: flow(),
        },
    }
}

fn flow() -> crate::project_slice::perimeters::types::Flow {
    crate::project_slice::perimeters::types::Flow {
        width: 0.4,
        height: 0.2,
        spacing: 0.36,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.08,
    }
}

#[test]
fn grouping_skips_first_layer_and_caps_combined_height() {
    let configs = vec![config(0.2, 0.4); 6];

    assert_eq!(groups(&configs), [0, 0, 2, 0, 2, 1]);
}

#[test]
fn grouping_combines_layers_strictly_below_cap_plus_epsilon() {
    let configs = vec![config(0.1, 0.4 + 2.0 * EPSILON); 5];

    assert_eq!(groups(&configs), [0, 0, 0, 0, 4]);
}
