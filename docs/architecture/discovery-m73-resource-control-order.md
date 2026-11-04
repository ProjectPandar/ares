# Deferred discovery: M73/G1 insertion order in resource-build control

Status: discovery only; independent existing divergence.
Not a qualified resource-build RED and not a new milestone.
Observed on main3ee8f272c6d8e2db22dde9003a01afeb7201e36a with frozen test-only
e1de695aa8cfbffb99cd971f86468037f6df4766. No production correction is part of this record.

Official public empty-build control. Input SHA
`d951edca050a77d6bf7a0c90a14de35ceac8804104af7bb5d3cc2e40ad67157c`.
Native official+seeded three references each exit0/complete and full normalized
positive/repeat/control neutral, SHA
`6c41f187921fde85c60e8d9593f215c2519ee671d33fa59108b66594ba8d260e`.
Official control API+CLI admitted/raw equal, SHA
`14261facd18982eb5cf5212b642cbb36c08596d722bfceba3e5be0ee8462c79f`.

Strict generator-only first unequal byte **5217**, line224,column1;
Native/Ares lengths69655/69655. After identical line223
`G1 X154.651 Y143.607 E.09711`:

```
Native224: M73 P67 R2
Native225: G1 X154.037 Y143.607 E.0347
Ares224:   G1 X154.037 Y143.607 E.0347
Ares225:   M73 P67 R2
```

No equality claim for later bytes or inference that this is the only difference.
No public seeded execution: the official control failed. Original
resource build positive/repeat admission errors remain recorded but e1de695aa
RED was rejected because control did not satisfy full parity.

Owning upstream boundary for a future investigation:
Orca8500fcdcc `src/libslic3r/GCode/GCodeProcessor.cpp::GCodeProcessor::run_post_process`
981 onward; local `process_line_move:1232-1260` advances g1_times_cache by move ID,
computes integer percent/minutes and inserts changed M73; consumers1458-1466
append G-code and call it with move counters. This localizes the insertion owner,
**not the root cause** (timing/cache/counter attribution unproved). No Rust fix,
new time tolerance, dropping/reordering M73, fixture tweak or comparator weakening.

The next resource-build step is a new input SPEC using an existing current-main
strict-PASS control, not a repair of this discovery.
