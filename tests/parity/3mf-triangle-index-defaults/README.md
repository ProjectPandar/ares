# Shared triangle vertex-index zero-default public RED

Approved SPEC `a1c1b81dac38fba08e170bc75ab3b8724755677f`; public RED authorized.
Main `bbs_3mf.cpp:3813-3819` and resource `:5526-5532` use
`bbs_get_attribute_value_int:508-514`: absent index returns zero; present strings
use boost::spirit qi::int_, not atoi. Malformed/negative/overflow stay deferred.

- control SHA `adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`
- positive SHA `a19533ceee3dea510848bf4ee578a3b48f5266139c79fd288ea15da546c8e20d`

Only `3D/Objects/overhang.stl_1.model` changes: first triangle
`<triangle v1="0" v2="1" v3="2"/>` loses ` v1="0"`. Vertex0 is already its
first index: Native recreates identical non-newly-degenerate geometry. All other
entry bytes are unchanged (full entry manifest committed). Control is an exact
copy of `wipe-before-override/control-before50.3mf`. No Native output committed.

`resource_triangle_missing_index_matches_native_public_slicing` uses from_env
else-return and the established reference-enabled gate. Enabled tests generate
fresh Native control/positive/repeat, assert Native determinism/zero neutrality,
then exercise public control first. Each public case executes real bare CLI and
validated API before target-success assertions; records both outcomes before
failing. Successful cases assert raw API==CLI and complete generator-only strict
Native/API/CLI equality including M73/config/motion/arcs/statistics/footer.

Before GREEN, positive missing @v1 must FAIL target success after control strict
PASS. It must not pass an expected-error branch, is_err or inequality assertion.
No native-only/external-reference bypass; missing streams mean comparison NOT
EVALUATED. Public repeat cannot be reached after positive target failure, but
Native repeat completes first. No production/mesh-repair change in this RED.
