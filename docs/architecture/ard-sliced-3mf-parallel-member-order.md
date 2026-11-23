# Sliced 3MF parity preserves archive order outside two parallel writes

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) appends two groups of ZIP members in parallel in `src/libslic3r/Format/bbs_3mf.cpp`:

- `_BBS_3MF_Exporter::_add_model_file_to_archive` (`7467–7497`) writes relationships first, then uses `tbb::parallel_for` (`7472`) to build and append split-model entries `3D/Objects/*.model`.
- `_BBS_3MF_Exporter::_add_gcode_file_to_archive` (`9005–9065`) collects the sliced plates, then uses `tbb::parallel_for` (`9026`) to build and append `Metadata/plate_N.gcode`. The caller writes `Metadata/plate_N.gcode.md5` in a sequential loop before invoking it (`6789–6828`). Statistics, when present, are appended sequentially after the parallel loop (`9066–9074`).

In each loop a mutex serializes appends, but the acquisition order depends on scheduling. Each call finishes its group before the exporter adds other members. Identical inputs can therefore produce different permutations **within** a group, not different placement of the group. The model export path, filenames, relationships, plate IDs, checksums, payloads, and other ZIP entries are not interchangeable.

## Decision

Change only the sliced-3MF parity comparator. Read ZIP members in central-directory order. Recognize only case-sensitive direct-child `3D/Objects/<name>.model` files and exact `Metadata/plate_<positive decimal plate number>.gcode` files as the two parallel-written groups. Compare each as an unordered set of full member names, preserving cardinality and requiring the same payload and existing ZIP member metadata for each matching name. A comparison must not discard entries or collapse repeated names into one.

Each group must occupy at most one contiguous span in **each** archive; both archives must have the same span start and length for that group, or both must omit it. Permute only the members in those spans for comparison (for example, sort each verified span by full name). Compare the resulting sequences positionally, with the existing per-member assertions. Do not globally sort the ZIP or extract groups from the sequence and compare the leftovers. In particular, moving a checksum into the G-code span, moving a whole span past another member, or swapping the two spans is a mismatch.

All other member names, positions, payloads and metadata remain subject to the existing exact checks. Apply the existing plate 1 G-code producer/timestamp rule to every numbered plate: verify the original digest against the G-code, normalize exactly one producer banner, then recompute the digest for comparison. This is not a new payload normalization; other G-code bytes and the existing model-settings object-record checks are unchanged. `Metadata/plate_N.gcode.md5`, the main model, relationships, and all other metadata remain strictly ordered. This applies only to comparing sliced exports, not unsliced exports, input rewriting, backup handling, or the archive writers. Ares keeps its sequential writer order and byte output independent of core count; neither the Rust writer nor the upstream oracle changes.

## Comparison sites

Paths below are under `crates/ares-cli/tests/orca_parity/`.

| Comparison | Scope |
|---|---|
| `export_3mf_sliced.rs`: `read_members`, `compare_members`, called by `sliced_archive_matches_native` | Shared ordered ZIP comparison for direct STL, saved single-object, saved two-object, and saved translated-part exports. Apply the span rule here. |
| `export_3mf_sliced/comparator.rs`: `compare_members`, used by the saved/direct cases and `cli_sliced_3mf_from_arranged_stls_matches_native_archive` in the arranged multi-plate extension | Same comparison boundary after extraction to a module. Use the shared span rule instead of splitting plate G-code out of the remaining sequence; no separate arranged comparator. |
| `export_3mf.rs`: `export_matches_native` / `archive_members` | Unsliced STL and 3MF export: strict ordered member comparison and full Ares byte repeatability. Unchanged. |
| `export_3mf_sliced_rejection.rs`: `cli_sliced_3mf_rejects_multiple_instances_of_one_object` | Rejection and directory inventory only; reuse of the invocation helper does not compare member order. Unchanged. |

Saved-project fixture rewriters and member-by-name readers are not output order comparators. On the base revision only the unsliced comparator is present; the sliced comparator is supplied by the sliced-export change. The arranged extension must consume the shared comparison rather than add a separate rule.

## Verification

- Exercise the sliced comparison through its actual ZIP reader and comparator with generated archives. Positives: swap only two object models; swap only two plate G-code members; permute both groups while retaining their spans. Degenerate zero- and one-member spans retain the same membership and location rules.
- Negatives: remove or add a member in either group; substitute a different name without changing the count; change model or plate G-code payload bytes (a plate mutation must still fail with a valid recomputed digest); move a non-group member; interleave a checksum with the plate G-code; split the object group with another member; move an entire contiguous group to another position. Non-group order failures must use unchanged payloads and valid checksums so they exercise placement rather than integrity checks.
- Replay the retained opposite-order Native/Ares pairs for `cli_sliced_3mf_with_two_objects_matches_native_archive` and `cli_sliced_3mf_with_translated_model_part_matches_native_archive` through the same comparator before and after the change. Record the order-only mismatch before, equality after, and continued rejection of modified payloads. Perform mutations on copies. Do not commit retained exports as fixtures.
- Run the affected CLI parity cases with both official and seeded Native references, including the arranged multi-plate case when present. Check that repeated Ares exports and pre-/post-change Ares artifacts retain identical bytes for the same inputs. Do not retry comparisons to obtain a particular Native member order.
