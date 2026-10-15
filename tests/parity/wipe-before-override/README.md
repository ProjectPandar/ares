# Before-wipe small-positive override inputs

Approved SPECc3adae5e950cd8de3084790535d8f5a5c29cac4d.
Seven exact input-only3MFs frozen from external discoveryvZg5EQ; source/input
SHA256 in manifest.json. No Native artifacts or expected G-code committed.
Build on approved b91d9bd97 (stack15 pending), basec251aa616.

Existing admitted overhang mesh/non-config bytes unchanged. Fix length0.8,
wipe1/wipe_speed30/retraction_speed60/role_based_wipe_speed0/comments1;
staticPA/adaptive0, Classic/no scarf/tower.
Distances0.3999/0.39995/0.399975/0.4/0.400025 before0%; controls distance2
before50% and before0%. Actual Native settings verified before comparison.

Public RED expected1FAIL+6PASS/mode, sole positive distance0.399975:
Native no before-wipe E, during E-.79995, post-wipe E-.00005. Ares puts the
small.00005 before wipe instead; first complete-output mismatch byte2031line81.
Source GCodeWriter.cpp1004-1011 explicit override>EPSILON else configured
factor*full (zero for before0%). Do not infer an exact threshold classification
for0.39995 from nominal arithmetic; retain actual whole-output passing control.

OrcaRunner::from_env/stages requiredNative, frozen input/config assertions,
actualWIPEmarkers/positiveXY/E and positive pre/during/post ordering gate;
then admitted publicAPI+realCLI and unchanged generator-only whole comparator.
Runtime launch/witness/comparison/fullNativeAPI+CLI streams retained externally.
Numeric footer exact equality is a precondition, not stream tolerance.
Independent public RED review BEFORE touching before-consumer. Same-input
controls raw API/CLI must stay unchanged for GREEN. CanonicalALLoracleenv
includingprocgcode, CPU18jobs2threads2 freshserialofficialseeded namespaces.
