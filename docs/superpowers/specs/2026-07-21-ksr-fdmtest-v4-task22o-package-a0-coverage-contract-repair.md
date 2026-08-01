# Task 22O Package A0 Coverage Contract Repair

## Scope and immutable parents

This amendment repairs only two gaps discovered before Package A0 formal
qualification:

1. the approved direct MedialAxis corpus does not exercise validation branch 1;
2. the approved plan names `corpus/corpus-v1.bin` without defining its byte
   grammar.

It also records why chaining action 3 is structurally unreachable and therefore
is not a required coverage branch. It does not change the frozen fixed Orca
derivative, the `ORCA22V` or `ARES22V` wire, the eleven CLI cases, the 71-process
role plan, or any tracked Rust production file.

The immutable parent A0 spec and plan are byte-identical at SHA-256
`f40807bd9d891f8d38a7fb82bb2c2db74294ab67e38c02fd8e6a903224221200`
and `9f84c95dc9a2dbf4c55f4b3d381455921c13f40c180e50230dd76830310538f5`.
Their detached document approval envelope remains byte-identical at SHA-256
`b234da531b0e1a9d9b681d059717a6c5fb564e57beefec257d4109665d894890`.
All original Task 22O, Package 0, fixture, fixed Orca commit, and fixed Ares
baseline identities remain those bound by the parent A0 amendment.

The fixed source is independently frozen as:

- Package0-relative nine-path patch: 62,560 bytes, SHA-256
  `269841a4842970cb2046b048bece3fcf416b7230b25854a7051e1b35354ad5df`;
- twelve-path source ledger: 1,262 bytes, SHA-256
  `d85b2b35fd788f332a1a7e29ba7f94c9be8c085195f5e5016d21d8969a69c5c4`;
- canonical source status: 4,797 bytes, SHA-256
  `8e8cca81ba0494a0d0e6e853a8bd562d2fb676e282a72cdc93675114d362971a`;
- source review: 4,796 bytes, SHA-256
  `f0bf0de7f3c9b56fab569424fb3d8445393a927c8039e55219525eeb921bec2f`;
- detached source approval envelope: 1,897 bytes, SHA-256
  `5fb414d7e09ea188ca78da54800a5a89fa662e3a31d779381255e391b7f9f9ef`.

The envelope binds the other four subjects and literal `VERDICT: APPROVE`.
No repair execution may modify those subjects, their Package 0 parents, or any
of the twelve source bytes. A fresh formal build must apply only the frozen
Package 0 patch followed by this exact nine-path patch, then regenerate and
match both twelve-path subjects before configuration. This amendment pair must
be frozen and receive two independent document approvals before the direct
probe corpus or repair tooling is changed.

## Pre-qualification evidence

No Package A0 formal qualification process has run. The frozen development-only
campaign summary reports one attempt for each of all eleven CLI cases, with
exact approved `ORCA22O`, a strictly parsed `ORCA22V`, one G-code output, and
clean process/temp residue. That summary is 80,594 bytes, SHA-256
`dd5f7c25d2c2656679f0240a5a910b1f67f540585290a99f08f43e4fcf3cc41d`;
its 329-byte identity record has SHA-256
`61e23cfe39cddcca134cdc5a8d8e806b0de84cbf0a35abc8b2b4e11059d7f3ee`.
They are discovery evidence only and do not substitute for any formal leaf.
The corresponding coverage report is 24,816 bytes, SHA-256
`566abd211000a1e88cfbf62b069980e219ee69aaf194e94f2ea19e678c0d14bd`.
Across 2,350 MedialAxis records it observed validation branches 0, 2, and 3 and
chaining actions 0, 1, and 2, but not validation branch 1 or chaining action 3.

The approved five-case direct probe run is 305,036 bytes, SHA-256
`b267e41b6788de1bf8d1dcba427f7a3ed57eaf3f0593e36a39452c9dd6470ed1`.
It observes validation branches 0, 2, and 3; chaining actions 0, 1, and 2; and
the required closing transition from first construction state/issue `2:1` to
second construction state/issue `1:0`. It does not observe validation branch 1
or chaining action 3.

A deterministic development search against the same reviewed derivative found
a valid branch-1 input. The search source, executable, complete `ORCA22V`,
stdout, empty stderr, and strict parser have SHA-256 identities respectively:

- 3,987-byte source:
  `fab0e122b0c7caf93fefdfe0bb366764b42fa22518f4c2434df862d8deb3c1d9`;
- 789,504-byte executable:
  `84ba353cb7588ac8db195ba5174b5afc21281a60cfc9d7b2ebc489042e59cfaf`;
- 983,547-byte wire:
  `b42b3062f9eb07e79448df30e439ca335ddf157693c9295cdee0b9900cdeacf4`;
- 7,631-byte stdout:
  `b92976fdf1a57828a409e5e3531b5b0dd14ebd3c0e277772b1386ba9b52e20be`;
- zero-byte stderr:
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`;
- 24,819-byte strict parser:
  `98e6157d213fe1776e64efdd7565ccc3eaebb778ee76259418a28f0c4bbf6b5c`.

The canonical discovery result is 3,557 bytes, SHA-256
`d2d32f94deacfaa4b5fc3782f6d4fbeb6b2536db3e5334f21bd571b301662ba8`.

The mixed-case search is discovery evidence only. Formal evidence comes from
the amended fixed direct probe, executed twice without retry after a fresh
fixed-source build.

## Validation branch repair

The four validation branch identities retain the reviewed meanings:

- 0: reject at the segment/segment angle and minimum-length filter;
- 1: reject a point/segment edge because either endpoint width is below
  `SCALED_EPSILON`;
- 2: accept at the minimum/maximum width range;
- 3: reject at the minimum/maximum width range.

All four are required in aggregate across the eleven qualified CLI wires and
the direct probe wire. Add exactly one sixth direct MedialAxis case named
`endpoint_epsilon_notch` after the existing five cases. Its exact input is:

- minimum width bits `0x3ff0000000000000` (`1.0`);
- maximum width bits `0x4197d78400000000` (`100000000.0`);
- no holes;
- contour, in order:
  `(-10000000,-10000000)`, `(10000000,-10000000)`,
  `(10000000,10000000)`, `(1,10000000)`, `(1,9999999)`,
  `(-1,9999999)`, `(-1,10000000)`, `(-10000000,10000000)`.

The fixed probe contract requires first construction state/issue `0:0`, no
closing for this case, and at least one validation branch-1 decision with
`accepted == false` and `active == false`. It does not pin edge indices, counts,
or output bytes before the formal build. The complete formal wire remains the
oracle.

Appending case six may not rewrite the approved direct corpus. Before accepting
the amended probe, the strict parser must extract the original 16 ordered record
bodies from the approved 305,036-byte wire and require the amended wire's first
16 record bodies to be byte-identical, tag-identical, and order-identical. The
seventeenth record is the sole append and must be the exact
`endpoint_epsilon_notch` MedialAxis case. Coordinated edits to an old JSON case
and its C++ input are therefore rejected even if aggregate coverage still
passes.

This changes only the ignored fixed-probe `medial_cases` source, its direct-case
JSON, and focused probe tests. It does not add an Orca source path, change the
twelve-path derivative status, or change process cardinality: the direct probe
still runs in exactly two fresh processes, so the qualification remains 69 CLI
processes plus two direct processes.

## Unreachable chaining action

The fixed `src/libslic3r/Geometry/MedialAxis.cpp` blob is
`7fece75e633653dccf59b21657e59de5f202ef3f`. In fixed lines 576-581,
`process_edge_neighbors` counts active neighbors and stores the sole active
neighbor. Fixed lines 584-586 immediately retrieve the same per-instance
`EdgeData` and test the same `active` bit. There is no call, callback, yield,
write, or aliasing mutation between those operations. Fixed `ExPolygon.cpp`
blob `185e92508449a425064b26690e3d74d06a16fda8`, lines 261-268,
stack-constructs a new `MedialAxis` for each synchronous invocation. Fixed
`MedialAxis.hpp` blob `cd1404f915b5857130e4ce77aa35ea02d3526935`, lines
35-45, owns its private `m_edge_data`. The function itself is sequential, so no
other invocation can alias or concurrently mutate that entry.

The reviewed instrumentation does not alter that invariant. It only copies the
rotation and active-neighbor lists before the count is evaluated and records a
decision after the same read. Therefore the instrumented action 3, "one active
neighbor was counted but the immediate re-read is inactive", cannot occur in
the fixed execution model.

Required chaining coverage is exactly:

- action 0: follow the sole active neighbor;
- action 1: stop with no active neighbor;
- action 2: stop at a multi-neighbor branch.

Action 3 remains a valid wire discriminant so the strict parser and comparator
must decode and compare it if a wire contains it. It is excluded only from the
minimum runtime coverage set. Production code may not synthesize it, delete the
second source read, or normalize actions to satisfy the gate.

A valid synthetic action-3 record is a mandatory wire-contract subject. Strict
parse must accept it, the comparator must distinguish the same record with
action 0 from action 3, coverage observation must retain its actual action-3
count, and the minimum-coverage validator must continue to require exactly
actions 0, 1, and 2 rather than action 3.

## Minimum semantic coverage

Before freezing the sidecar manifest, the union of the eleven qualified run-1
wires and direct probe run-1 must contain at least:

- record tags `medial_axis`, `raw_points`, `raw_segments`, and
  `wrapped_segments`;
- all ten source inventory claims: `ordinary`, `raw_point`,
  `raw_or_open_segment`, `hole`, `multiple_hole`, `edge_collapse`, `duplicate`,
  `intersecting`, `missing_vertex`, and `repair`;
- construction state/issue pairs `0:0`, `1:0`, and `2:1`, including one exact
  closing transition `2:1 -> 1:0`;
- cell source categories 0, 1, 2, 8, and 9; point and segment containment;
  cell annotations 0, 1, and 2; and both degenerate values;
- directed-edge annotations 0, 1, and 2 and the five observed
  `(finite,primary,curved)` classes `(0,0,0)`, `(0,1,0)`, `(1,0,0)`, `(1,1,0)`,
  and `(1,1,1)`;
- vertex annotations 0, 1, and 2;
- validation predicates 0, 1, and 2 on both sides and validation branches
  0, 1, 2, and 3;
- chaining directions 0 and 1, actions 0, 1, and 2, and active-neighbor
  cardinalities 0, 1, and at least one value greater than 1; action-0 decisions
  must include both `chosen_reversed == false` and `chosen_reversed == true`;
- both observed ThickPolyline endpoint classes `(false,true)` and
  `(true,true)`, plus nonempty point and exact width-cardinality transcripts.

These are minimum sets, not normalization targets. The manifest records all
actual categories and counts. A formal wire may contain additional valid source
states or cardinalities; tooling must retain them and reviewers must inspect
them. Missing any minimum item blocks sidecar-manifest creation.

## `corpus-v1.bin` contract

`corpus-v1.bin` is an ignored evidence container, not an Orca or Ares runtime
wire. It copies already-qualified subjects byte-for-byte and never parses and
re-encodes their records.

Its byte grammar is exactly:

1. eight-byte ASCII magic `A0C22V1\0`;
2. little-endian `u32` version 1;
3. little-endian `u32` entry count 12;
4. twelve entries, each encoded as:
   - little-endian `u16` UTF-8 identifier byte length;
   - the exact UTF-8 identifier bytes;
   - `u8` kind, where 1 is `ARES22V` and 2 is `ORCA22V`;
   - 32 raw SHA-256 bytes for the following payload;
   - little-endian `u64` payload length;
   - the exact payload bytes;
5. eight-byte ASCII trailer `A0C2EOF\0`, then physical EOF.

The parser uses checked offset arithmetic for every variable-length field. For
entry index `i`, define `later_reserve(i)` as eight trailer bytes plus, for every
later table row `j`, exactly
`2 + len(expected_literal_identifier[j]) + 1 + 32 + 8` bytes: its `u16`, known
literal identifier bytes, kind, raw hash, and `u64`, with a zero-byte payload.
Before reading entry `i`'s `u16`, the parser requires enough remaining bytes for
that field, `len(expected_literal_identifier[i])`, the current `1 + 32 + 8`
fixed suffix, and `later_reserve(i)`. It then requires the `u16` value to equal
the expected literal byte length before accessing any identifier byte. Only
after proving that same exact reserve may it slice, decode UTF-8, allocate, or
copy the identifier. After reading `payload_length`, it requires remaining bytes
to be at least `payload_length + later_reserve(i)` before addition, slicing,
hashing, allocation, or copying. It hashes payloads in their already-bounded
slice; an untrusted length never drives an allocation. Identifier length
`0xffff`, any identifier length that consumes the current suffix, even one byte
of a later literal identifier, any later fixed field, or the trailer,
`payload_length == 0xffffffffffffffff`, any length beyond the remaining bytes,
truncated/invalid UTF-8, offset overflow, or insufficient reserved bytes is
rejected.

The repair document, not mutable ignored tooling, freezes the exact entry
mapping below. Identifier bytes are the literal ASCII shown in the first
column. `a0_cases.CASE_IDS` must equal the first eleven identifiers in this
order; its file identity is also bound by the final manifest.

| Identifier | Source role | Kind | Exact payload subject | Required parent role |
| --- | --- | ---: | --- | --- |
| `ksr_fdmtest_v4` | `qualified/ksr_fdmtest_v4/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `A1-topology-gap-base` | `qualified/A1-topology-gap-base/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `A2-precise-off` | `qualified/A2-precise-off/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `A3-one-wall` | `qualified/A3-one-wall/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `A4-gap-speed-zero` | `qualified/A4-gap-speed-zero/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `A5-gap-target-everywhere` | `qualified/A5-gap-target-everywhere/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `A6-zero-speed-target-everywhere` | `qualified/A6-zero-speed-target-everywhere/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `A7-gap-filter-five` | `qualified/A7-gap-filter-five/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `B1-top-terrace` | `qualified/B1-top-terrace/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `C1-overhang-raft0` | `qualified/C1-overhang-raft0/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `D1-nested-island-traversal` | `qualified/D1-nested-island-traversal/run-1` | 1 | that role's exact `ARES22V` composite | that role's `expected_parent_ares22o` |
| `fixed-probe` | `fixed-probe/run-1` | 2 | that role's exact `ORCA22V` wire | not applicable |

For each kind-1 entry, the embedded `ARES22V` parent must equal the same table
row's run-1 `expected_parent_ares22o`; no other approved parent is accepted.
Run 1 is fixed before execution and is not selected after results are known.
Run-2 byte equality is a prerequisite for assembly and both run identities are
bound separately in the sidecar manifest. Substituting an otherwise valid
run-2 payload for run 1, swapping parents or payloads across cases, changing a
kind, or permuting identifiers is rejected.

`corpus-v1.json` is canonical ASCII JSON with sorted keys, two-space indentation,
and one final newline. It records this grammar version, every ordered identifier,
role, kind, source identity, paired-equality proof, decoded summary, aggregate
coverage, and the binary container identity. The assembler must build both
outputs twice in memory and require byte equality before create-once publication.
The verifier reparses every embedded payload, checks its internal physical EOF,
hash, kind, parent binding where applicable, exact approved `ARES22O`, entry
order/count, outer trailer, and outer physical EOF.

## Formal candidate identity and root

The candidate ID is exactly the first 16 lowercase hexadecimal characters of
`SHA-256(raw source-files.sha256 bytes || installed executable bytes)`. Its sole
allowed resolved candidate root is
`.superpowers/sdd/task22o-oracle/voronoi-a0/runs/qualified/<candidate-id>`.
Before creating that directory or launching any process, the runner independently
recomputes the ID, resolves the existing authorized parent, and rejects a wrong
leaf, sibling root, alias path, or pre-existing candidate root with zero
filesystem mutation. The result collector and sidecar-manifest assembler each
independently recompute the same formula and require the recorded root's resolved
parent and literal leaf to match before publishing any collector or manifest
artifact. For the runner, collector, and assembler separately, every invalid
ID/root/alias case requires byte-for-byte before/after filesystem snapshots and
zero mutation. A runner-only assertion is not sufficient.

## Repair evidence and final-manifest binding

The exact ignored-path delta is listed in the implementation plan and is part of
this amendment frame. No unlisted authored source, test, review, or evidence
path is authorized. Generated build/cache trees and formal run children under
the parent's already authorized roots are not manifest additions and are never
accepted as source subjects.

The closed-set verifier classifies exactly these seven existing files, with no
directory wildcard, as excluded development residue:

- `fixed-probe/evidence/final-smoke/run-1.orca22v`;
- `fixed-probe/evidence/final-smoke/run-2.orca22v`;
- `fixed-probe/runs/explore-1.orca22v`;
- `fixed-probe/evidence/post-review-final/run-1.stderr.log`;
- `fixed-probe/evidence/post-review-final/run-1.stdout.log`;
- `fixed-probe/evidence/post-review-final/run-2.stderr.log`;
- `fixed-probe/evidence/post-review-final/run-2.stdout.log`.

The two adjacent `.orca22v` files remain authorized frozen subjects. None of the
excluded paths may be bound, copied into formal evidence, or treated as an
approved result. Any other unlisted authored path fails the closed-set check.

The final `sidecar-manifest-v1.json` must bind, by exact path, byte length, and
SHA-256, every non-excluded authored path in the plan's exact repair addition
list. This includes the repair spec/plan/reviews/envelope, all five frozen source
package subjects, all nine retained `coverage-repair/exploration/` subjects, all
five `coverage-repair/direct-probe/` RED/GREEN subjects, tooling-review
subjects, and every repair tooling/test source. Each subject has individual
remove, substitute, and one-byte mutation tests.

The closed-set correspondence is exhaustive: parent-manifest paths retain their
parent binding; every authorized repair addition is final-manifest-bound;
machine-generated build/cache trees are non-subjects and contain no authorized
authored path; and only the exact seven files above are excluded development
residue. No retained evidence is unclassified. Missing, substituted,
self-referential, mutated, unapproved, or newly injected authored subjects block
manifest creation and verification.

## Review and exit criteria

Two independent reviewers inspect this exact spec/plan frame before any repair
edit. One reviews fixed-source reachability and the branch-1 input; the other
reviews corpus framing, qualification isolation, and manifest implications.
Any rejection returns a concrete repair list to the main thread; the frame is
refrozen and reviewed again.

This repair is complete only when:

1. immutable parent hashes and all five frozen source-package subjects remain
   exact, and a fresh worktree regenerated the twelve-path ledger/status after
   applying the exact frozen patch;
2. the amendment pair has two detached approvals and an approval envelope;
3. focused REDs fail before and pass after the sixth direct case while the
   original 16 direct records remain byte-identical;
4. the amended direct probe runs twice from the fresh formal build, without
   retry, and produces byte-identical strict wires containing branch 1;
5. the complete formal union satisfies every minimum semantic coverage item;
6. `corpus-v1.bin` and `.json` satisfy the grammar and two-pass determinism;
7. action 3 remains positively parseable/comparable/observable but is not a
   minimum runtime requirement;
8. the Package A0 sidecar manifest binds the complete parent and repair frames
   and its unchanged two-review protocol approves; and
9. no tracked production, Cargo, lockfile, notice, architecture, roadmap, or
   workflow file changes in this repair.
