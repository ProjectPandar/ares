# Spec: Task 22O.106 first-layer print bounds

Port `GCode.cpp` first-layer convex-hull placeholder setup into the typed
project emitter. Include transformed model-part bounds from the 3MF-derived
project and keep all output generic. Defer exact hull/offset parity to a
follow-up source-cited slice; no fixture-specific constants are allowed.
