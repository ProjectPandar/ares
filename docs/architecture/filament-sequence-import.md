# Optional 3MF metadata and filament sequences

## Upstream boundary

OrcaSlicer `40ecc66f7`, `src/libslic3r/Format/bbs_3mf.cpp:3500-3553`,
`_BBS_3MF_Importer::_extract_filament_sequence_from_archive`, owns sequence
import. Archive entry dispatch at `:1680-1710` and `:2042-2114` invokes
project settings, model settings, slice information and sequence readers only
when their entries exist. Absence does not select a different importer route.

The Rust destination is `ares-core::project::{load,filament_sequence}`.
The strict serde map and plate-set equality check are temporary scaffolding
replaced by lookup against the imported model-settings plates.

## Included behavior

- Missing model settings leave objects and plates metadata empty; existing
  model/build assembly supplies geometry without object overrides or plate labels.
- Missing project settings leave the embedded configuration empty. Normal
  retained defaults and caller overrides still apply. Substitutions are empty.
- Missing slice information leaves its header empty.
- Missing sequence metadata leaves sequences at their empty defaults.
- Only canonical keys for existing plates are inspected. Other members and
  non-object plate members are ignored, including invalid values inside them.
- Prefer `filament_sequence`; use `sequence` only if the former is absent.
  Both the selected filament list and `nozzle_sequence` must be arrays;
  otherwise the plate keeps its defaults without inspecting list items.
- Store filament IDs minus one, saturating zero at zero. Keep nozzle IDs
  unchanged. Read integer assignment arrays only; empty assignments do not
  replace defaults.
- Empty or malformed JSON and nonnumeric list items remain import errors.

## Deferred behavior

The exporter at `:8662-8690` writes one-based filament IDs under
`filament_sequence` for dynamic-map plates and `sequence` otherwise. Exporter
changes, dynamic nozzle mapping, sequence-driven tool ordering, and unrelated
3MF route differences are outside this import slice. Ares currently retains
these documents privately rather than exposing an editable sequence API.
Upstream undefined numeric conversions are not reproduced.

## Failure modes and verification

Public project loading and slicing consume synthetic archive variants with
incomplete, alternate, unknown, non-object and malformed sequence members.
Additional variants remove each optional metadata entry. Invalid items are
checked only when both required arrays exist on a known plate. Complete
G-code streams for reachable Native CLI imports are compared with the existing
generator-only byte comparator. Browser drag-drop verifies that a synthetic
incomplete sequence imports successfully and produces a visible object list.
No user document or Native output is stored in the source tree.
