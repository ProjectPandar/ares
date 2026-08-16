# Spec: Task 22O.107 print-space bridge

Port centered Ares extrusion coordinates to transformed print space using the
3MF model instance/volume transform. Use the translated first-layer extrusion
bounds for runtime placeholders. Defer the source-cited convex-hull union
needed for the final adaptive-bed mesh envelope; do not add fixture constants.
