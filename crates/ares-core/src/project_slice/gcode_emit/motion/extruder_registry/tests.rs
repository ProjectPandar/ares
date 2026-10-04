use super::*;
use crate::project_slice::gcode_emit::{
    motion::{EmitState, apply_writer_state, refresh_writer_state},
    value::{Config, Value},
};

fn state() -> EmitState {
    EmitState {
        extruders: ExtruderRegistry::new(vec![0, 2], &config(), 1),
        ..EmitState::default()
    }
}

fn config() -> GCodeOptions {
    GCodeOptions::from_sources(
        &crate::options::PrinterGCodeSourceOptions::default(),
        &crate::options::ProcessGCodeSourceOptions::default(),
        &crate::options::FilamentGCodeSourceOptions::default(),
        &crate::options::ProjectGCodeSourceOptions::default(),
    )
}

fn assign(config: &mut Config, name: &str, id: usize, value: f64) {
    let Some(Value::List(mut values)) = config.get(name).cloned() else {
        panic!("missing writer vector")
    };
    values[id] = Value::number(value);
    config.insert(name, Value::List(values));
}

#[test]
fn priming_order_is_sorted_for_writer_registration() {
    let registry = ExtruderRegistry::new(vec![2, 0, 1], &config(), 1);
    assert_eq!(registry.ids(), [0, 1, 2]);
    assert_eq!(registry.vector_size(), 3);
    assert!(registry.multiple());
    assert_eq!(
        registry.read(2, &ExtruderState::default(), false).position,
        0.0
    );
}

#[test]
fn template_writeback_preserves_inactive_tool_and_unused_holes() {
    let mut state = state();
    let mut config = Config::default();
    refresh_writer_state(&mut config, &state);
    assign(&mut config, "e_position", 2, 17.0);
    assign(&mut config, "e_retracted", 2, 2.0);
    assign(&mut config, "e_restart_extra", 2, 0.5);
    assign(&mut config, "e_position", 1, 99.0);
    apply_writer_state("test", &config, &mut state).unwrap();
    refresh_writer_state(&mut config, &state);
    assert_eq!(
        config
            .get("e_position")
            .unwrap()
            .index(2)
            .unwrap()
            .as_number(),
        Some(17.0)
    );
    assert_eq!(
        config
            .get("e_retracted")
            .unwrap()
            .index(2)
            .unwrap()
            .as_number(),
        Some(2.0)
    );
    assert_eq!(
        config
            .get("e_restart_extra")
            .unwrap()
            .index(2)
            .unwrap()
            .as_number(),
        Some(0.5)
    );
    assert_eq!(
        config
            .get("e_position")
            .unwrap()
            .index(1)
            .unwrap()
            .as_number(),
        Some(0.0)
    );
    state.extruders.activate(2, &mut state.extruder, false);
    assert_eq!(state.extruder.e_position, 17.0);
    assert_eq!(state.extruder.retracted_amount, 2.0);
    assert_eq!(state.extruder.restart_extra, 0.5);
}

#[test]
fn inactive_tool_negative_retraction_fails_template() {
    let mut state = state();
    let mut config = Config::default();
    refresh_writer_state(&mut config, &state);
    assign(&mut config, "e_retracted", 2, -1.0);
    assert_eq!(
        apply_writer_state("test", &config, &mut state),
        Err("Custom G-code reports negative z_retracted.".to_owned())
    );
}

#[test]
fn statistics_vectors_keep_unused_holes_and_sum_all_registered_tools() {
    let mut state = state();
    state.options.filament_area = 2.0;
    state.options.filament_density = 1.0;
    state.extruder.filament_used = 3.0;
    state.extruders.activate(2, &mut state.extruder, false);
    state.extruder.filament_used = 4.0;
    let mut config = Config::default();
    refresh_writer_state(&mut config, &state);
    let Some(Value::List(volumes)) = config.get("extruded_volume") else {
        panic!("missing volumes")
    };
    assert_eq!(volumes.len(), 3);
    assert_eq!(volumes[1].as_number(), Some(0.0));
    let first = 3.0 * (1.75 * 1.75 * 0.25 * std::f64::consts::PI);
    assert_eq!(volumes[0].as_number(), Some(first));
    assert_eq!(volumes[2].as_number(), Some(8.0));
    assert_eq!(
        config.get("extruded_volume_total").unwrap().as_number(),
        Some(first + 8.0)
    );
}

#[test]
fn empty_registry_does_not_publish_extruder_output_variables() {
    let state = EmitState {
        extruders: ExtruderRegistry::new(Vec::new(), &config(), 1),
        ..EmitState::default()
    };
    let mut config = Config::default();
    refresh_writer_state(&mut config, &state);
    for key in ["e_position", "e_retracted", "e_restart_extra"] {
        assert!(config.get(key).is_none(), "unexpected output {key}");
    }
    assert_eq!(
        config.get("extruded_volume"),
        Some(&Value::List(Vec::new()))
    );
    assert_eq!(
        config.get("extruded_volume_total").unwrap().as_number(),
        Some(0.0)
    );
}

#[test]
fn empty_registry_does_not_validate_unregistered_extruder_variables() {
    let mut state = EmitState {
        extruders: ExtruderRegistry::new(Vec::new(), &config(), 1),
        ..EmitState::default()
    };
    let mut config = Config::default();
    refresh_writer_state(&mut config, &state);
    config.insert("e_retracted", Value::List(vec![Value::number(-2.0)]));
    assert_eq!(apply_writer_state("test", &config, &mut state), Ok(()));
}

#[test]
fn relative_e_does_not_validate_absolute_position_output() {
    let mut state = state();
    state.options.use_relative_e_distances = true;
    let mut config = Config::default();
    refresh_writer_state(&mut config, &state);
    assert!(config.get("e_position").is_none());
    config.insert("e_position", Value::List(vec![Value::number(17.0)]));
    assert_eq!(apply_writer_state("test", &config, &mut state), Ok(()));
    assert_eq!(state.extruder.e_position, 0.0);
}

#[test]
fn later_tool_error_preserves_earlier_and_current_position_writeback() {
    let mut state = state();
    let mut config = Config::default();
    refresh_writer_state(&mut config, &state);
    assign(&mut config, "e_position", 0, 23.0);
    assign(&mut config, "e_position", 2, 17.0);
    assign(&mut config, "e_retracted", 2, -1.0);
    assert!(apply_writer_state("test", &config, &mut state).is_err());
    refresh_writer_state(&mut config, &state);
    assert_eq!(
        config
            .get("e_position")
            .unwrap()
            .index(0)
            .unwrap()
            .as_number(),
        Some(23.0)
    );
    assert_eq!(
        config
            .get("e_position")
            .unwrap()
            .index(2)
            .unwrap()
            .as_number(),
        Some(17.0)
    );
}

#[test]
fn semm_shares_physical_axes_but_keeps_filament_private_counters() {
    let mut config = config();
    config.filament_map.0 = vec![crate::OrcaInt(1), crate::OrcaInt(1), crate::OrcaInt(2)];
    let mut state = EmitState {
        extruders: ExtruderRegistry::new(vec![0, 1, 2], &config, 2),
        ..EmitState::default()
    };
    state.options.single_extruder_multi_material = true;
    state.extruder.e_position = 10.0;
    state.extruder.retracted_amount = 2.0;
    state.extruder.retracted = true;
    state.extruder.private_e_position = 3.0;
    state.extruder.private_retracted = 0.25;
    state.extruder.restart_extra = 0.5;
    state.extruder.filament_used = 5.0;
    state.extruders.activate(1, &mut state.extruder, true);
    assert_eq!(state.extruder.e_position, 10.0);
    assert_eq!(state.extruder.retracted_amount, 2.0);
    assert_eq!(state.extruder.private_e_position, 0.0);
    assert_eq!(state.extruder.private_retracted, 0.0);
    assert_eq!(state.extruder.restart_extra, 0.0);
    assert_eq!(state.extruder.filament_used, 0.0);
    state.extruder.e_position = 17.0;
    state.extruder.retracted_amount = 4.0;
    state.extruder.private_e_position = 8.0;
    state.extruder.private_retracted = 0.75;
    state.extruder.restart_extra = 1.0;
    state.extruder.filament_used = 9.0;
    state.extruders.activate(2, &mut state.extruder, true);
    assert_eq!(state.extruder.e_position, 0.0);
    assert_eq!(state.extruder.retracted_amount, 0.0);
    state.extruder.e_position = 19.0;
    state.extruder.retracted_amount = 6.0;
    state.extruder.retracted = true;
    state.extruders.activate(0, &mut state.extruder, true);
    assert_eq!(state.extruder.e_position, 17.0);
    assert_eq!(state.extruder.retracted_amount, 4.0);
    assert_eq!(state.extruder.private_e_position, 3.0);
    assert_eq!(state.extruder.private_retracted, 0.25);
    assert_eq!(state.extruder.restart_extra, 0.5);
    assert_eq!(state.extruder.filament_used, 5.0);
    state.extruders.activate(1, &mut state.extruder, true);
    assert_eq!(state.extruder.private_e_position, 8.0);
    assert_eq!(state.extruder.private_retracted, 0.75);
    assert_eq!(state.extruder.restart_extra, 1.0);
    assert_eq!(state.extruder.filament_used, 9.0);
    let mut output = Config::default();
    refresh_writer_state(&mut output, &state);
    assert_eq!(
        output
            .get("e_position")
            .unwrap()
            .index(0)
            .unwrap()
            .as_number(),
        Some(3.0)
    );
    assert_eq!(
        output
            .get("e_position")
            .unwrap()
            .index(1)
            .unwrap()
            .as_number(),
        Some(8.0)
    );
    assign(&mut output, "e_position", 2, 21.0);
    assign(&mut output, "e_retracted", 2, 1.0);
    assign(&mut output, "e_restart_extra", 2, 0.7);
    apply_writer_state("test", &output, &mut state).unwrap();
    state.extruders.activate(2, &mut state.extruder, true);
    assert_eq!(state.extruder.e_position, 19.0);
    assert_eq!(state.extruder.retracted_amount, 6.0);
    assert!(state.extruder.retracted);
    assert_eq!(state.extruder.private_e_position, 21.0);
    assert_eq!(state.extruder.private_retracted, 1.0);
    assert_eq!(state.extruder.restart_extra, 0.7);
}

#[test]
fn missing_filament_map_entry_uses_physical_zero_not_first_mapping() {
    let mut config = config();
    config.filament_map.0 = vec![crate::OrcaInt(2)];
    let mut registry = ExtruderRegistry::new(vec![0, 1], &config, 2);
    let mut active = ExtruderState {
        e_position: 17.0,
        ..ExtruderState::default()
    };
    registry.activate(1, &mut active, true);
    assert_eq!(active.e_position, 0.0);
    active.e_position = 9.0;
    registry.activate(0, &mut active, true);
    assert_eq!(active.e_position, 17.0);
}
