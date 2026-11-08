# Exclude-object tolerance and closed-polygon simplification

## Upstream boundary

OrcaSlicer source: `345efec64c7b95a05ce5cf96df0dde66a158835c`, `src/libslic3r`.

- `PrintConfig.cpp:4707–4716` defines `exclude_object_tolerance` as coFloat in millimeters, default 0.1, range 0..5. `PrintConfig.hpp:2154` contains the PrintConfig member.
- `Print.cpp:7122–7125`, `PrintInstance::get_convex_hull_2d`, obtains the model hull and calls `Polygon::douglas_peucker(scale_(print.config.exclude_object_tolerance.value))`.
- `Polygon.cpp:112–117` appends the first vertex, calls `MultiPoint::_douglas_peucker` (`MultiPoint.cpp:175+`), then removes the closing duplicate.
- `Model.cpp:1819+` and `2818–2837` implement model and volume hull construction. `GCode.cpp:11395–11406` emits the per-instance Klipper outline, with its centre from the instance bounding box.

The polygon wrapper matters independently of tolerance units: an open Douglas–Peucker chain fixes both endpoints, whereas the polygon wrapper can omit the original final vertex.

## Rust boundary and behavior

`ares-core::options::process_options::print_source` holds the scalar as `OrcaFloat`, default 0.1. Its own serializer and the flattened process serializer expose the same key. The option registry, metadata and process refresh key already contain this option.

`project_slice/gcode_emit/footprint.rs` obtains the value from the global full-process print view and passes it to the per-instance outline consumer. The consumer converts millimeters with `CoordinateScale::scaled_delta`, without integer quantization, and calls the existing `geometry::Polygon::douglas_peucker` on the convex hull. It uses the resulting polygon points for the Klipper outline.

This implements the PrintInstance tolerance and polygon-wrapper behavior. Mesh vertices, instance/volume transforms, convex-hull construction, bounding boxes, centres, plate offsets and label formatting stay unchanged. Motion, timing and configuration-block generation are outside this change.

## Behavior outside this slice

Replacing the raw-mesh hull scaffold with ModelVolume's cached 3D hull is a separate port. The scaffold remains a temporary compatibility shell around that upstream concept; equality of effective projected inputs for one fixture does not establish full 3D hull parity. Changes to hull construction, polygon simplification algorithms and other geometry consumers are also outside this slice.

The implementation stays in the platform-neutral core and adds no filesystem, terminal, viewer or native-runtime dependency.

## Runtime test premise

`native_exclude_outline_migration.rs` compares the runtime Native CLI with the public `slice_project_plate` API using identical 3MF bytes, default generation metadata, no plate override, default ConfigOverrides and context seed 0.

The rotation fixture has six pre-simplification hull vertices, including small edge residues. With tolerance 0.1 mm, Native's closed-polygon simplification keeps four corners; with tolerance 0, it keeps all six. The original last vertex can disappear at 0.1 mm, so tolerance scaling with an open-chain simplifier is insufficient. An identity-transform fixture supplies a four-corner control.

The activation test constructs 0.1 and 0 variants by changing only `exclude_object_tolerance` in `Metadata/project_settings.config`, copying all other ZIP entries verbatim. Each variant goes to both runtimes. The test checks that changing the scalar changes the Native outline, that Ares follows the same change, and that both outline definitions match. It also retains complete output line comparisons and line-count assertions, using only the existing generator-line exemption. Full-stream configuration differences remain visible rather than being removed from the comparison.
