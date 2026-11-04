# KSR historical Native product retirement

KSR, CONFIG_BLOCK, classic mutation and browser consumers now obtain their
Native reference at runtime. The retired files have no executable consumers.

The source boundary remains Print.cpp:2618-2638 Print::export_gcode,
GCode::do_export, and GCode.cpp:5591-5643 GCode::append_full_config. See
[the authority decision](ard-ksr-initialized-runtime-authority.md) and
[the migration contract](native-artifact-cleanup-ksr-reference-spec.md).
No production behavior, input geometry or comparator changes belong here.

## Removed products and remaining authority

Delete the original and initialized KSR G-code, initialized historical receipt,
and classic replay G-code. The original producer could not reproduce its body
or CONFIG_BLOCK. Initialized producer is reproducible from the committed recipe,
so consumers obtain authoritative bytes at runtime through existing OrcaRunner
producer/cache/receipt checks. Historical fixed hashes are not rebound.
KSR external 3MF and upstream DRC remain unchanged; classic 3MF remains unchanged.

## Original CLI self-check assertion text

The following is the exact old function. Project SHA and ZIP entry-count
assertions remain active in the same test name. Every other assertion solely
authenticated a historical output or its receipt; the corresponding file is
removed, so its self-check has no remaining subject. This retires file-self-checks,
not an Ares-vs-Native comparison. Runtime Native/full CLI comparison, changed-ID
negative control, classic 33 mutations, and real browser ordered comparison
keep the unchanged strict generator-only comparator.

```rust
#[test]
fn fixture_contract_is_stable() {
    let project = project();
    let reference = reference();

    assert_eq!(sha256(&project), PROJECT_SHA256);
    assert_eq!(sha256(&reference), REFERENCE_SHA256);
    let initialized = fs::read(fixture_path("ksr_fdmtest_v4.initialized-orca.gcode")).unwrap();
    let identity: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture_path("initialized-reference.json")).unwrap())
            .unwrap();
    assert_eq!(sha256(&initialized), identity["output_sha256"]);
    assert_eq!(PROJECT_SHA256, identity["input_sha256"]);
    assert_eq!(
        zip::ZipArchive::new(Cursor::new(project)).unwrap().len(),
        15
    );
    assert_eq!(
        reference.iter().filter(|byte| **byte == b'\n').count(),
        269_330
    );
    assert_eq!(count_bytes(&reference, b"; CHANGE_LAYER\n"), 460);

    let normalized = normalize_one_generator_line(&reference, GeneratorKind::Orca).unwrap();
    assert_eq!(sha256(&normalized), NORMALIZED_REFERENCE_SHA256);
}

```

| Original assertion | Disposition and reason |
| --- | --- |
| `assert_eq!(sha256(&project), PROJECT_SHA256);` | Keep: retained source input contract. |
| `assert_eq!(sha256(&reference), REFERENCE_SHA256);` | Remove: original historical file integrity; producer is irreproducible. |
| `assert_eq!(sha256(&initialized), identity["output_sha256"]);` | Remove: authentication of deleted historical file against deleted receipt; runtime cache validates fresh bytes. |
| `assert_eq!(PROJECT_SHA256, identity["input_sha256"]);` | Remove: deleted receipt self-consistency; direct input SHA remains active and runtime consumers independently check it. |
| ZIP archive `.len() == 15` assertion above | Keep exactly: retained input packaging contract. |
| original `reference` newline count `269_330` assertion above | Remove: fixed original-capture self-check, not a live output comparison. |
| `assert_eq!(count_bytes(&reference, b"; CHANGE_LAYER\n"), 460);` | Remove: deleted original-capture self-check; browser's independent 460-layer assertion remains unchanged. |
| `assert_eq!(sha256(&normalized), NORMALIZED_REFERENCE_SHA256);` | Remove: irreproducible original-body hash; no new literal binding. |

## Original core CONFIG_BLOCK self-check assertion text

This function only examined the irreproducible original file. Remove it and its
now-orphaned parser/hash/constants. Its exact macro text follows, including
multi-line expressions and loop assertions. No production/core Native runner
is introduced. The separate pure Rust views/collector/writer test retains its
six sentinel assertion pairs and both bed/nozzle temperature checks verbatim.
The migrated CLI CONFIG_BLOCK test compares complete runtime block bytes and
preserves both marker uniqueness checks and first-difference diagnostics.

```rust
#[test]
fn config_export_fixture_freezes_the_committed_reference_block_contract() {
    let block = reference_block();
    assert_eq!(block.len(), 49_005);
    assert_eq!(sha256(block), BLOCK_SHA256);
    assert!(block.starts_with(START));
    assert!(block.ends_with(END));
    assert!(!block.contains(&b'\r'));

    let assignments = assignments(block);
    assert_eq!(assignments.len(), 639);
    assert_eq!(
        assignments
            .iter()
            .map(|(key, _)| *key)
            .collect::<BTreeSet<_>>()
            .len(),
        637
    );
    assert_eq!(
        assignment_lines(block, "wipe_tower_x"),
        ["; wipe_tower_x = 165.000", "; wipe_tower_x = 165"]
    );
    assert_eq!(
        assignment_lines(block, "wipe_tower_y"),
        ["; wipe_tower_y = 220.096", "; wipe_tower_y = 220.096"]
    );
    for key in ["from", "name", "version"].into_iter().chain(NIL_OMISSIONS) {
        assert!(assignments.iter().all(|(actual, _)| *actual != key), "{key}");
    }
    for key in RETAINED_EMPTY {
        assert!(
            assignments
                .iter()
                .any(|(actual, value)| *actual == key && value.is_empty()),
            "{key}"
        );
    }
    assert_eq!(
        assignments[assignments.len() - 2..],
        [
            ("first_layer_bed_temperature", "55"),
            ("first_layer_temperature", "220"),
        ]
    );
}

```

| Original assertion group | Reason for removal |
| --- | --- |
| `assert_eq!(block.len(), 49_005);` and `assert_eq!(sha256(block), BLOCK_SHA256);` | Deleted original block's size/hash; initialized block is different and is never rebound to these literals. |
| START, END and no-CR assertions above | Structural self-check of the deleted original block; runtime block extraction retains unique markers and byte equality. |
| assignments `639` and unique-key count `637` assertions above | Historical original-block inventories, not assertions against Ares output. |
| both duplicate `wipe_tower_x`/`wipe_tower_y` arrays above | Original historical serialization values; live exact block comparison remains. |
| nil/from/name/version omission loop above | Original-file self-check; runtime complete block comparison rejects changed omissions. |
| retained-empty loop above | Original-file self-check; runtime complete block comparison rejects changed empty values. |
| final two assignment tuples `55`/`220` above | Original-file order/value self-check; live block is compared exactly and pure resolved-temperature assertions remain unchanged. |
| `assert_eq!(matches.len(), 1, "marker occurrence count");` in old `find_once` | Orphaned frozen-reader helper assertion; same assertion is retained in migrated runtime CONFIG_BLOCK helper. |

## Reference audit and exclusions

The executable-reference audit in crates/, scripts/, web/ and tests/ found zero
remaining reads of the four deleted products. Architecture/history documents may
still name retired products
as evidence, without acting as reference readers. No production, shared cache,
web's Ender3 roundtrip or other consumer groups are changed. No new ignore,
probe, normalization rule, Native output or fixed output hash is introduced.

## Browser authority

The active browser consumer compares actual WASM-produced KSR bytes with the
runtime initialized Native producer using the unchanged full ordered comparator.
Only the generated-by line is normalized; no CLI capture substitutes for browser
output. The system Chrome run passed all three browser tests, including complete
Native equality and repeat-slice equality. The earlier bundled-headless-shell
launch failure is INCONCLUSIVE: missing NixOS shared libraries prevented every
test body from running. It is neither RED nor GREEN.

Without `ARES_BROWSER_GCODE`, the browser consumer reports shared visible
SKIPPED: "browser capture absent; run via project-slice.spec.mjs" and returns
before starting Native, including in required ordinary control runs. This is
not browser parity PASS. With capture present, missing Native follows the shared
optional/required dependency policy and the complete ordered comparator remains
unchanged. An unreadable configured capture still fails. The capture-absence
policy test remains active in the default environment without an artifact root.
