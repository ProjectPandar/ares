# Legal coInts comma-list regression

Frozen original Native-exported BBL discovery archive; do not regenerate.

seam-variant.3mf SHA256:
`3e392f7a34ea2b869722aa768ff729b51d1bf0c99d9f1be708671e618f06eebf`.
Source `${WORKTREE_ROOT}/ares-bbl-option-discovery-7m2p9k4/seam.3mf`.

The historical discovery label is seam variant; effective machine is BBL X1C,
Classic walls, aligned seam. The one object contains two normal parts. Valid
registered Ints options have multiple values, including nozzle temperatures,
print_extruder_id and printer_extruder_id. No archived G-code or Native output
is embedded. The public test compares dynamic Native1/2 and actual byte API/CLI
with the unchanged complete ordered generator comparator.

The current coInts ingress parses a comma-containing segment as one i64 before
splitting, causing the actual API panic RED. The correction only changes Ints
split-before-leaf-parse order. Any later independent strict residual is STOP,
not permission to alter the input, expected output or another producer.
