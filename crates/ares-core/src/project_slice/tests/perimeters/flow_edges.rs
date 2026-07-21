use std::panic::catch_unwind;

use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, Percent, SliceError,
    project_slice::{layers::PlannedLayer, perimeters::flow::resolve_perimeter_flows},
};

use super::fixture::flow_options;

#[test]
fn task22n_release_decrease_rounding_returns_bridge_flow_error_without_panic() {
    let (mut region, mut object) = flow_options();
    region.inner_wall_line_width = FloatOrPercent::Percent(Percent(500.0));
    region.bridge_line_width = FloatOrPercent::Float(0.0);
    region.bridge_flow = OrcaFloat(f64::MIN_POSITIVE);
    object.thick_bridges = OrcaBool(false);
    let layer = PlannedLayer {
        id: 0,
        height: 2e-7,
        print_z: 2e-7,
        slice_z: 1e-7,
    };
    let nozzles = OrcaFloats(vec![OrcaFloat(100.0)]);

    let result = catch_unwind(|| {
        resolve_perimeter_flows(
            &layer,
            FloatOrPercent::Percent(Percent(500.0)),
            &region,
            &object,
            &nozzles,
        )
    });
    let Ok(result) = result else {
        panic!("release decrease-rounding Flow resolution must not panic")
    };

    assert_eq!(
        result.unwrap_err(),
        SliceError::InvalidInput("invalid Orca option bridge_flow".to_owned())
    );
}
