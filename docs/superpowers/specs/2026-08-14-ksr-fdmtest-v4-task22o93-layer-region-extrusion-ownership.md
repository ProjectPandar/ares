# Task 22O.93 — layer-region extrusion ownership

Port pinned `Layer.hpp:43-76`. Move retained perimeter collections into every
aligned O92 layer output beside generated fills and moved thin fills. Preserve
all tree, path, role, flow, and point order; drain predecessor ownership.

Focused KSR tests freeze exact perimeter collection/node/path/point inventory,
repeatability, disposal, and public lifecycle. Separate modules, <400 LOC, no
source-splitting macros.

Deferred: island sorting/chaining, motion, and G-code.
