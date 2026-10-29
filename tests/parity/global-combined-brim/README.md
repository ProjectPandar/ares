# Global combined brim public RED inputs

Input-only 3MFs, exact SHA256 manifest; no Native outputs.
Source: Brim.cpp:958-1018; Print.cpp:2532-2543; GCode.cpp:4486-4531.
Two distinct singleton objects, ByLayer, combined skirt, outer_only brim8/gap0,
one filament, no support/raft/tower. Target combine_brims1; paired control0.
Only combine_brims differs; geometry and all other ZIP members unchanged.
Runtime Native FIRST; assert one target / two control brim blocks, positive
extrusion, effective config, then strict whole Native/API/CLI equality and raw
API==CLI. Target must FAIL actual output divergence until authorized GREEN;
control asserts equality too. No coordinate tolerance or divergence-is-success.
Area/cleanup/carrier provenance must be localized before any production change.
Copies and perobject guard admission remain deferred.
