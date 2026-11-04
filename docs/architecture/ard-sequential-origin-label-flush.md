# ARD — sequential pending-label consumption

Status: accepted bounded source port, not combined-stack integration.

Decision: upstream8500fcdcc `GCode.cpp::travel_to`7466–7467 and sequential `_do_export`3321–3327 own the label lifecycle. Rust transition.rs::emit must consume existing queued end-before-start labels after travel acceleration/retraction, before the origin hop, via the existing motion label writer. Preserve GCodeWriter.cpp1181–1200 relative/absolute-E behavior without duplicating writer logic.

The existing sequential origin-travel code is only a compatibility shell for this named upstream boundary. Port the missing call, do not design Ares-owned transition orchestration, postprocess output, move generic layer queueing, change formatting/IDs/M73, or add fallback/helper abstraction. Included only the literal consumption/citation; adjacent geometry, generic travel, collision, priming/multi-tool, tower and other flavors deferred.

Acceptance: existing native_stream_case public full-output seam, original target+by-layer2object control, official/seeded Native-first replay before successful/raw-equal public calls. True tests-only RED before minimal GREEN; unchanged comparator/tests through GREEN, unchanged raw control outputs. Preserve frozen 4eeda42be and complete evidence.
