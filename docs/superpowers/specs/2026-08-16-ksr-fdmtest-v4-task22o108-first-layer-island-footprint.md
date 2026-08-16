# Spec: Task 22O.108 first-layer island footprint

Use the first compensated layer slices retained by the classic perimeter input
as the model-island component of Orca's first-layer convex hull. Translate the
bounds with the same project model center used by G-code motion. Defer
skirt/brim/support/wipe-tower hull union without introducing fixture constants.
