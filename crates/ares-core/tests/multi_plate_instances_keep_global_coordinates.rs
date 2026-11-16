//! O6 multi-plate: instances stay in global coordinates upstream (no-arrange
//! assembly translates each plate's objects by the plate origin,
//! OrcaSlicer.cpp:4837-4845), the volume gate tests them against the bed
//! shape translated to the plate's grid position (PartPlate.cpp:3179-3187),
//! and the G-code emitter subtracts the plate origin again
//! (`set_plate_origin` PartPlate.cpp:2516, `set_gcode_offset`
//! Print.cpp:2636-2638) so plate N's G-code is plate-local coordinates.
//! The -min fixture is a single-filament variant of lint's two-plate
//! project exercising the same origin path without the multi-filament
//! bridge features of the base fixture.

#[path = "../../../tests/support/slice_project_plate.rs"]
mod direct_project;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use direct_project::slice_project_plate;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../tests/parity/validation/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[tokio::test]
async fn empty_plate_one_uses_model_read_error() {
    let error = slice_project_plate(
        fixture("print-bed-two-plates.3mf"),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    // Selected importer projection is empty: Model.cpp:500-501 read error,
    // not the later print-volume rejection. CLI exit parity remains deferred.
    assert_eq!(
        error.to_string(),
        "The supplied file couldn't be read because it's empty."
    );
}
