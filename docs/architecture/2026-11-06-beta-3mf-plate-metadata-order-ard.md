# Plate metadata across interleaved model instances

## Upstream boundary

Port the plate-child ordering accepted by OrcaSlicer 2.5.0-beta revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`, not a new archive format.
The owner is `src/libslic3r/Format/bbs_3mf.cpp`:

| Source | Semantics |
| --- | --- |
| 3664–3673 | Each metadata, plate and model_instance start tag dispatches its handler. Metadata need not form one contiguous block. |
| 4577–4583 | Read each metadata key/value; an empty key is ignored. |
| 4613–4625 | Object/volume metadata is collected in vectors. This is distinct from plate scalar/config assignment. |
| 4630–4661 | Plate identity/name and recognized configuration entries are assigned as events arrive. Repeated recognized valid keys overwrite earlier values, including all three sequence keys at 4654–4661. |
| 4738–4748 | Repeated instance_id, identify_id or object_id metadata replaces that current-instance field: last assignment wins, not first or a list of IDs. |
| 4953–4970 | The current plate remains active until its closing tag, including metadata after an instance closes. |
| 4974–5005 | Instance start does not end the plate. Instance end inserts its completed record and resets the current instance. Repeated completed records are a separate issue from repeated metadata fields. |

`DynamicConfig::set_key_value` in `src/libslic3r/Config.hpp:3022–3032`
replaces the stored option on an existing key, confirming overwrite semantics.
`obj_inst_map.emplace` at `bbs_3mf.cpp:5002` does not imply last-instance-record
wins for the same object: record insertion and metadata-key assignment must not
be conflated.

## Rust boundary and decision

Only `crates/ares-core/src/project/model_settings.rs::PlateSettings` and its
private deserialization helpers change. Keep the existing `PlateSettings`
fields, visibility and consumers. This wire structure is a compatibility shell
for the cited importer events, not a separate plate-processing pipeline.

The existing two derived vector fields treat a second noncontiguous metadata
run as a duplicate struct field. Decode the plate's child stream once into a
private tagged vector, then move metadata and instance values into their
existing vectors in encounter order. Do not sort, deduplicate, rewrite XML,
retry another parser or synthesize defaults beyond existing empty-vector
semantics. Keep unknown-child ignoring; malformed recognized children remain
errors. No dependency features, public API or visibility changes are needed.

Preserving metadata order lets the existing
`project/plate_config.rs:21–81` assignment loop retain its last-valid-write
sequence semantics. Do not change `PlateModelInstance`, object/part parsing,
`project/load/metadata.rs`, option application, map recommendation, refresh or
G-code generation. No FlushMode behavior is part of this parser slice.

Included: noncontiguous plate metadata, all existing model_instance children,
relative order within each retained vector, and public selected-plate sequence
values for the four original inputs. Deferred: object/part interleaving,
repeated completed instance records, unknown-extension interpretation, malformed
value policy and broader duplicate-key identity parity. In particular,
`project/load/metadata.rs:134–142` currently rejects repeated identity keys;
this structural hunk does not claim to fix that distinct difference from
Native's last-assignment behavior. The four inputs have unique keys per scope.

## Exact production hunk

The following is the entire proposed production change; existing imports
already provide `Deserialize` and `Deserializer`.

```diff
--- a/crates/ares-core/src/project/model_settings.rs
+++ b/crates/ares-core/src/project/model_settings.rs
@@ -103,7 +103,42 @@
-#[derive(Clone, Debug, Deserialize, PartialEq)]
-pub(crate) struct PlateSettings {
-    #[serde(rename = "metadata", default)]
-    pub metadata: Vec<Metadata>,
-    #[serde(rename = "model_instance", default)]
-    pub model_instances: Vec<PlateModelInstance>,
-}
+#[derive(Clone, Debug, PartialEq)]
+pub(crate) struct PlateSettings {
+    pub metadata: Vec<Metadata>,
+    pub model_instances: Vec<PlateModelInstance>,
+}
+
+#[derive(Deserialize)]
+struct PlateSettingsWire {
+    #[serde(rename = "$value", default)]
+    children: Vec<PlateChild>,
+}
+
+#[derive(Deserialize)]
+enum PlateChild {
+    #[serde(rename = "metadata")]
+    Metadata(Metadata),
+    #[serde(rename = "model_instance")]
+    ModelInstance(PlateModelInstance),
+    #[serde(other)]
+    Other,
+}
+
+impl<'de> Deserialize<'de> for PlateSettings {
+    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
+    where
+        D: Deserializer<'de>,
+    {
+        let wire = PlateSettingsWire::deserialize(deserializer)?;
+        let mut plate = Self {
+            metadata: Vec::new(),
+            model_instances: Vec::new(),
+        };
+        for child in wire.children {
+            match child {
+                PlateChild::Metadata(metadata) => plate.metadata.push(metadata),
+                PlateChild::ModelInstance(instance) => plate.model_instances.push(instance),
+                PlateChild::Other => {}
+            }
+        }
+        Ok(plate)
+    }
+}
```

## Public RED and exit criteria

Use the unchanged archives in `tests/parity/filament-group-flush-mechanisms/`:

| Identity | SHA-256 |
| --- | --- |
| x2d_pla_other_layers_ascending_custom_order | 1604901c251e5d056b5565ff48b657ee3b9ba8175b09d921de477ce645912062 |
| x2d_pla_other_layers_middle_last_custom_order | 744be8eb5dd26e978b5873716f8e03563d36c90cb45067aee1c3d7b395ffe342 |
| x2d_pla_short_objects_ascending_prior_state | ea67a836adc5533c0eca444bf9892825dec35c316f3a01357ac0dda76b5693b8 |
| x2d_pla_short_objects_descending_prior_state | 0cfd5f48f43152f251b2b466437bdcf40b1d7a8923adbb980927de52eae4e15b |

Keep their existing expectations and full FlushMode assertions unchanged.
The independent `plate_metadata_order` tests qualify Native double replay on
each archive, check Native's three sequence fields against the existing
`ordering.json`, then require public `load_project` and `select_plate(None)`
to succeed. The selected project must retain one plate with three instances
and three objects, and its three sequence fields must equal Native exactly.
The import tests only assert; they do not write comparison JSON. Failures
report the input digest, Native artifact location and mismatching sequence
key/values. The external runner retains diagnostics. No Native-only
observation counts as an import pass.

Failure cases to distinguish: rejection of the second metadata run, loss of
trailing sequence metadata, dropped model instances, reordered sequence values,
and invalid recognized XML being silently accepted. These four unique-key
inputs establish the import/sequence seam, not runtime duplicate-key precedence
or unknown-extension coverage. Project settings also mirror sequence values;
field equality alone cannot prove trailing metadata retention, so acceptance
also requires the exact structural hunk and unchanged full assertions.

Exit: both Native modes pass all four import/sequence tests on unchanged
bytes; the original four full tests progress beyond import without weakened
assertions, and residual map failures remain independently classified. Preserve
raw Native streams and replay diagnostics. Run rustfmt, workspace/all-target
Clippy with warnings denied, and browser WASM compilation on the implementation.
Do not claim grouping GREEN, complete API/CLI output parity or whole-output
Native parity from import success.
