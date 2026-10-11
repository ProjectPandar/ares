; Native actual-speed-profile oracle: arcs at changing speeds, an outer wall
; closed where it started (GCodeProcessor seam), wipe, retract, travels faster than any extrusion (the shown
; travel range differs), zero-length moves and a trailing arc.
G90
M83
T0
G92 E0
; CHANGE_LAYER
; LAYER_HEIGHT: 0.2
; LINE_WIDTH: 0.4
; FEATURE: Outer wall
G1 X20 Y20 Z0.2 F600
G1 X20 Y0 E1 F3000
G2 X40 Y20 I0 J20 E2 F12000
G1 X40 Y40 E1 F600
G3 X20 Y40 I-10 J0 E1 F9000
G1 X20 Y20 E2 F6000
; WIPE_START
G1 X25 Y0 F3000
; WIPE_END
G1 E-0.8 F2400
G1 X60 Y0 F18000
G1 E0.8 F2400
G1 X60 Y0 F1800
; FEATURE: Sparse infill
G3 X80 Y20 I20 J0 E2 F4800
G1 X100 Y20 E1 F4800
G1 X100 Y20 E0.01
G1 X120 Y20 E1 F4800
; CHANGE_LAYER
G1 Z0.4 F600
G1 X0 Y0 F9000
G2 X20 Y20 I20 J0 E2 F3000
