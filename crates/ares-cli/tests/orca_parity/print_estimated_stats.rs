//! `DoExport::update_print_estimated_stats` (`GCode.cpp:1904-1940`): the
//! `PrintStatistics` totals the preview Summary legend reads
//! (`GCodeViewer.cpp:4003-4024`), against Native `print.print_statistics()`
//! after export for the inputs in `tests/parity/summary/` (see its README),
//! compared bit for bit.
use crate::runner::{self, OrcaRunner};

fn processed(project: &[u8]) -> ares_core::ProcessedGCode {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_processed(
            project,
            ares_core::GenerationMetadata::default(),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap()
}

fn assert_native(fixture: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let input = runner::repo_root().join(format!("tests/parity/summary/{fixture}.3mf"));
    let Some((native, entry)) = runner
        .print_statistics_reference(&input)
        .unwrap_or_else(|error| panic!("{error}"))
    else {
        return;
    };
    let output = processed(&std::fs::read(&input).unwrap());
    let statistics = &output.result.print_statistics;
    // The shared sources must already agree before the totals can.
    // Native's std::map holds only extruders that extruded.
    let volumes = statistics
        .total_volumes_per_extruder
        .iter()
        .enumerate()
        .filter(|(_, volume)| **volume != 0.0)
        .map(|(id, volume)| (id.to_string(), volume.to_bits()))
        .collect::<Vec<_>>();
    let native_volumes = native["total_volumes_per_extruder"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(id, volume)| (id.clone(), volume.as_f64().unwrap().to_bits()))
        .collect::<Vec<_>>();
    assert_eq!(
        volumes, native_volumes,
        "{fixture}: total_volumes_per_extruder; {entry:?}"
    );
    assert_eq!(
        statistics.normal_time.to_bits(),
        (native["normal_time"].as_f64().unwrap() as f32).to_bits(),
        "{fixture}: modes[Normal].time {}; {entry:?}",
        statistics.normal_time
    );
    // The footer's Prepare and Model printing time rows
    // (`GCodeViewer.cpp:4627-4640`); only Normal mode is ported.
    let native_prepare = native["normal_prepare_time"].as_f64().unwrap() as f32;
    assert!(
        native_prepare > 0.0 && native["stealth_time"].as_f64().unwrap() == 0.0,
        "{fixture}: fixture must have a prepare stage and no stealth machine; {entry:?}"
    );
    assert_eq!(
        statistics.normal_prepare_time.to_bits(),
        native_prepare.to_bits(),
        "{fixture}: modes[Normal].prepare_time {} != native {native_prepare}; {entry:?}",
        statistics.normal_prepare_time
    );
    let actual = output.print_statistics;
    for (field, value) in [
        ("total_extruded_volume", actual.total_extruded_volume),
        ("total_used_filament", actual.total_used_filament),
        ("total_weight", actual.total_weight),
        ("total_cost", actual.total_cost),
    ] {
        let expected = native[field].as_f64().unwrap();
        assert_eq!(
            value.to_bits(),
            expected.to_bits(),
            "{fixture}: {field} {value:?} != native {expected:?}; {entry:?}"
        );
    }
}

/// Two used filaments with distinct densities and costs, `time_cost` 1.7.
#[test]
fn print_estimated_stats_time_cost() {
    assert_native("time-cost");
}

/// Filament 2 is configured but unused (not in `m_writer.extruders()`).
#[test]
fn print_estimated_stats_unused_filament() {
    assert_native("unused-filament");
}
