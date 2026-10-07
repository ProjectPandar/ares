; ColorChange processor parity fixture: upstream reserved-tag semantics
G90
M83
T0
G92 E0
;LAYER_CHANGE
;HEIGHT:0.2
;WIDTH:0.4
;TYPE:External perimeter
G1 Z0.2 F600
;TYPE:Support
G1 X10 E1 F600
G1 X20 E1
;COLOR_CHANGE,T0,#FF0000
M600
G1 X30 E1 F600
G1 X40 E1
