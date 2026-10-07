; ColorChange processor parity fixture: upstream reserved-tag semantics
G90
M83
T0
G92 E0
; CHANGE_LAYER
; LAYER_HEIGHT: 0.2
; LINE_WIDTH: 0.4
; FEATURE: Outer wall
G1 Z0.2 F600
G1 X5 F600
G1 X8
; COLOR_CHANGE,T0,#FF0000
M600
G1 X10 F600
G1 X20
