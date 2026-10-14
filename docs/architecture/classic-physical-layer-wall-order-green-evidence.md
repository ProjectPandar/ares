# Classic physical-layer sandwich ordering: RED → GREEN evidence

Base main29f4a49c18139925fec238d31d2fdb6b790fbc30.
Tested production commit53edb48bd52b837fafabe57cad0e67c8e985483c.
Approved SPEC137b3e01c (rebased1530a7148); conditional brim deferral729b94370
(rebased7430aa2ca). Separate public RED freeze7c4e2784f→64f869a37.

## Exact source and scope

Upstream8500fcdccaa10b5099ac20d252af3a7c560046f1,
`PerimeterGenerator.cpp:1463–1471`, actual physical-layer sandwich gate.
Rust `classic/perimeter_append.rs::transform_object` changes ONE argument:
`index` → `traversal_record.layer_id` passed to `transform_record`.
No new algorithm, helper, allocation, validation, fallback or core I/O.
First-layer brim `index==0` and `lower_slices(index)` are UNCHANGED and DEFERRED;
no independently proven brim RED was used to authorize their correction.
No existing assertions/comparator changed; public test fixtures remain raw-identical.

## New-main RED before production

`/tmp/worker4-classic-layer-id-red-full9-aTog`, HEAD64f869a37:
- official2 true DIVERGENT / seeded2 true DIVERGENT; no Native failures;
- all4 Native status.code0 and complete immutable snapshots;
- Filled45008 expected/45014 actual bytes; None44320/44326;
- both first difference byte2283, line106, column6: Native travel
  `X138.164 Y143.539 Z.72`, Ares `X142.418 Y141.501 Z.72`;
- fmt0, strictclippy0 and shared Native SHA invariant.

Ares-only in-memory localization (complete-output invariant, archived separately)
established actual branch layers0/1/0/1 vs flattened record arguments0/1/2/3.
The later-owner physical layer0 incorrectly took the later-layer sandwich path.
Seed chain c0→c1→h1→h0, all local chains[(0,false)], thin-hole reverse=false;
unchanged traversal should emit Inner hole first as Native public output does.
No Native internal data was collected. All temporary core probes were removed.

## Directed GREEN

`/tmp/worker4-classic-layer-id-green-full9-BaHD`, tested53edb48bd:
- official29/29 PASS (29.746s), seeded29/29 PASS (36.293s);
- includes both formerly RED Classic M cases and counterbore controls, including
  new-main N-split regression;29 result records and33 successful Native commands
  per mode, no failed/incomplete Native snapshots;
- complete ordered comparison retains arcs, movement, roles, M73, statistics and
  footer; only established generator identity normalization applies;
- scoped ares-core `test(perimeters)`441/441 PASS (665.366s),6723 skipped;
- cargo fmt --check0; strict cargo clippy -p ares-cli --test orca_parity -- -D warnings0;
- cargo check -p ares-wasm --target wasm32-unknown-unknown0;
- official/seeded Ares raw SHA equal for each fixed case:
  None `92fcd4cff73c94d355f508e493938084559f50f32fcb943d88a56b39c0b41dd3`;
  Filled `5ab77cdbab6449d4e0d5471a6c7debfc5d7fdb23e8b456ed414bb7144bb1948a`;
- shared Native before/after SHA identical. CPU15/jobs1/nextest threads1,
  fresh official then seeded cache/artifacts; no full/workspace run.

Supplemental original M-v3 companions `/tmp/worker4-classic-m-companion-controls-full9-qkNr`:
official7/7 PASS and seeded7/7 PASS (one seven-case collector test per mode),
all14 Native status.code0 with complete snapshots. Cases: first-owner Arachne
Filled/None, incompatible Arachne Filled/None, later-owner Arachne Filled/None/
Bridges. Reused frozen pre-production discovery inputs/harness; no invented
Classic owner control or inferred group branch. Temporary CLI registration
removed after completion; no temporary core probe remains.

Supplemental archive `classic-m-companion-controls-full9-qkNr.tar.gz`, SHA256
`af6a7d06840ef5c75d1843b33ae18803fc9f2d29840264016253b9b99119925f`, under
the same durable directory. These are separate observed controls, not counted
inside the primary29-test totals.

## Durable artifacts and rebasing

Archives under `/home/indexyz/.ares-evidence/worker4/` (outside /tmp):
- `classic-layer-id-red-full9-aTog.tar.gz`, SHA256
  `194c166a5cf0282bb19f18fdbbddfa202d9a06e24068b1e4ac14f35e5357d6d5`;
- `classic-layer-id-green-full9-BaHD.tar.gz`, SHA256
  `6009b99b2d911ef16cf5d64fe4dbbeda6fdf3b6ddff709d960d4c1e7d7a7ffbd`.
Raw streams, commands/status/timings, settings, paired hashes and logs remain
verifiable; verified-red-classification.json/verified-green-summary.json are included.
Rebase audit: seven rows,4`=`,3`!` only test-module registration context after
main introduced N-split; both main and W4 registrations retained once.
Historical57fea949 RED completed before FULL9 START; no in-flight leg was counted.
No push or full scheduling; coordinator retains exact-hash review and merge gates.
