# Task 22O.93 implementation plan

1. Add RED KSR perimeter ownership/inventory assertion.
2. Extend layer output with retained perimeter collections.
3. Move aligned source collections after successful fill generation.
4. Freeze exact inventory; run O91/O92 regressions and strict gates.
5. Update evidence, commit, and push.

## Completed evidence

Compile RED proved missing perimeter ownership. GREEN freezes the KSR 2,881 /
5,243 / 5,483 / 111,933 collection/loop/path/point inventory and predecessor
drain. Three O91 lifecycle/repeatability tests and strict core Clippy, rustfmt,
diff, and LOC gates pass.

No island ordering, motion, fallback, or G-code.
