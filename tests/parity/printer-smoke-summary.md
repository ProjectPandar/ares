# OrcaSlicer printer smoke summary

638 of 1001 printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; classic wall generator baseline; cube model).

Statuses: `PASS` (normalized byte equality), `DIVERGENT` (first byte difference), `ORCA_ERROR` (the upstream OrcaSlicer 2.4.2 reference binary itself failed, so no reference stream exists), `VENDOR_INCOMPLETE` (the vendor profile tree does not ship the machine's referenced default process preset), `ARES_ERROR` (Ares failed to load or slice the case).

| status | printer | first divergence |
|---|---|---|
| DIVERGENT | Afinia/Afinia H+1(HS) 0.4 nozzle | first difference at byte 24148 (line 1022, column 1; expected 107465 bytes, actual 107605 bytes) context:   line 1021: "G1 X108.29 Y132.29 Z3.4"   line 1022: expected "M73 P29 R2"; actual "G1 Z3"   line 1023: expected "G1 Z3"; actual "M73 P29 R2" |
| PASS | Afinia/Afinia H+1(HS) 0.6 nozzle |  |
| PASS | Anker/Anker M5 0.2 nozzle |  |
| PASS | Anker/Anker M5 0.25 nozzle |  |
| PASS | Anker/Anker M5 0.4 nozzle |  |
| PASS | Anker/Anker M5 0.6 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.2 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.25 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.4 nozzle |  |
| DIVERGENT | Anker/Anker M5 All-Metal 0.6 nozzle | first difference at byte 1810 (line 80, column 1; expected 96012 bytes, actual 96238 bytes) context:   line 79: "G1 F3000"   line 80: expected "M73 P1 R4"; actual "G1 X110.693 Y109.753 E.02489"   line 81: expected "G1 X110.693 Y109.753 E.02489"; actual "G1 X112.5 Y109.211 E.08871" |
| PASS | Anker/Anker M5C 0.2 nozzle |  |
| PASS | Anker/Anker M5C 0.25 nozzle |  |
| PASS | Anker/Anker M5C 0.4 nozzle |  |
| PASS | Anker/Anker M5C 0.6 nozzle |  |
| PASS | Anycubic/Anycubic 4Max Pro 0.4 nozzle |  |
| PASS | Anycubic/Anycubic 4Max Pro 2 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Chiron 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra 2 0.4 nozzle | first difference at byte 3191 (line 152, column 7; expected 98580 bytes, actual 99710 bytes) context:   line 151: ";WIDTH:0.5"   line 152: expected "G1 F1755"; actual "G1 F1760"   line 153: "G1 X106.087 Y113.913 E.29148" |
| PASS | Anycubic/Anycubic Kobra 2 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Neo 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Plus 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Pro 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra 3 0.2 nozzle | first difference at byte 18994 (line 661, column 29; expected 454235 bytes, actual 454235 bytes) context:   line 660: ";WIPE_START"   line 661: expected "G1 X121.761 Y120.566 E-.70072"; actual "G1 X121.761 Y120.566 E-.70071"   line 662: expected "G1 X121.88 Y120.53 E-.09928"; actual "G1 X121.88 Y120.53 E-.09929" |
| PASS | Anycubic/Anycubic Kobra 3 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.8 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra Max 0.4 nozzle | first difference at byte 2908 (line 123, column 1; expected 111491 bytes, actual 112752 bytes) context:   line 122: "G1 X205 Y210.536 E.06851"   line 123: expected "M73 P9 R7"; actual "G1 X195 Y210.536 E.29097"   line 124: expected "G1 X195 Y210.536 E.29097"; actual "M73 P9 R7" |
| PASS | Anycubic/Anycubic Kobra Neo 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra Plus 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.25 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra X 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Predator 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Vyper 0.4 nozzle |  |
| PASS | Anycubic/Anycubic i3 Mega S 0.4 nozzle |  |
| PASS | Artillery/Artillery Genius 0.4 nozzle |  |
| PASS | Artillery/Artillery Genius Pro 0.4 nozzle |  |
| DIVERGENT | Artillery/Artillery Hornet 0.4 nozzle | first difference at byte 1856 (line 84, column 1; expected 106043 bytes, actual 106435 bytes) context:   line 83: "G1 X108.484 Y122.722 E.0712"   line 84: expected "M73 P40 R8"; actual "G1 X107.351 Y121.641 E.0712"   line 85: expected "G1 X107.351 Y121.641 E.0712"; actual "G1 X106.884 Y120 E.07757" |
| PASS | Artillery/Artillery M1 Pro 0.2 nozzle |  |
| DIVERGENT | Artillery/Artillery M1 Pro 0.4 nozzle | first difference at byte 5414 (line 266, column 8; expected 124187 bytes, actual 125688 bytes) context:   line 265: ";WIDTH:0.45"   line 266: expected "G1 F2694"; actual "G1 F2696"   line 267: "G1 X125.602 Y134.398 E.27685" |
| PASS | Artillery/Artillery M1 Pro 0.6 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.8 nozzle |  |
| DIVERGENT | Artillery/Artillery Sidewinder X1 0.4 nozzle | first difference at byte 4008 (line 167, column 1; expected 104247 bytes, actual 104247 bytes) context:   line 166: "G1 X154.75 Y145.25 E.43192"   line 167: expected "M73 P4 R8"; actual "G1 X154.75 Y154.71 E.4301"   line 168: expected "G1 X154.75 Y154.71 E.4301"; actual "M73 P4 R8" |
| DIVERGENT | Artillery/Artillery Sidewinder X2 0.4 nozzle | first difference at byte 2278 (line 96, column 1; expected 104148 bytes, actual 104344 bytes) context:   line 95: "G1 X157.722 Y156.516 E.0712"   line 96: expected "M73 P36 R8"; actual "G1 X156.641 Y157.649 E.0712"   line 97: expected "G1 X156.641 Y157.649 E.0712"; actual "G1 X155 Y158.116 E.07757" |
| PASS | Artillery/Artillery Sidewinder X3 Plus 0.4 nozzle |  |
| DIVERGENT | Artillery/Artillery Sidewinder X3 Pro 0.4 nozzle | first difference at byte 3178 (line 146, column 6; expected 94385 bytes, actual 94901 bytes) context:   line 145: "G1 X115.602 Y118.102 E3.05979"   line 146: expected "M73 P3 R6"; actual "M73 P2 R6"   line 147: "G1 X124.398 Y118.102 E3.35156" |
| PASS | Artillery/Artillery Sidewinder X4 Plus 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X4 Pro 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.2 nozzle |  |
| DIVERGENT | BBL/Bambu Lab A1 0.4 nozzle | first difference at byte 51756 (line 1057, column 1; expected 199495 bytes, actual 199921 bytes) context:   line 1056: "G1 Z1.300 F1200"   line 1057: expected "M73 P13 R10"; actual "G1 Y262.5 F6000"   line 1058: expected "G1 Y262.5 F6000"; actual "M73 P13 R10" |
| PASS | BBL/Bambu Lab A1 0.6 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.8 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.2 nozzle |  |
| DIVERGENT | BBL/Bambu Lab A1 mini 0.4 nozzle | first difference at byte 48082 (line 907, column 1; expected 193093 bytes, actual 193672 bytes) context:   line 906: "G2 I-0.75 J0 X-1.5"   line 907: expected "M73 P43 R7"; actual "G2 I1 J0 X2"   line 908: expected "G2 I1 J0 X2"; actual "M73 P43 R7" |
| PASS | BBL/Bambu Lab A1 mini 0.6 nozzle |  |
| DIVERGENT | BBL/Bambu Lab A1 mini 0.8 nozzle | first difference at byte 109819 (line 4135, column 7; expected 118035 bytes, actual 118329 bytes) context:   line 4134: "; LINE_WIDTH: 0.82"   line 4135: expected "G1 F1747"; actual "G1 F1755"   line 4136: "M106 S201" |
| PASS | BBL/Bambu Lab H2D 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.8 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.6 nozzle |  |
| DIVERGENT | BBL/Bambu Lab H2D Pro 0.8 nozzle | first difference at byte 55096 (line 1214, column 1; expected 103719 bytes, actual 104058 bytes) context:   line 1213: "G1 X178.069 Y158.84 E.32903"   line 1214: expected "M73 P71 R2"; actual "G1 X178.069 Y159.897 E.12884"   line 1215: expected "G1 X178.069 Y159.897 E.12884"; actual "M73 P71 R2" |
| PASS | BBL/Bambu Lab H2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab P1P 0.2 nozzle | first difference at byte 93 (line 3, column 28; expected 371083 bytes, actual 371106 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 15m 25s"; actual "; model printing time: 8m 39s; total estimated time: 15m 28s"   line 4: "; estimated first layer printing time (normal mode) = 6m 48s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.4 nozzle | first difference at byte 93 (line 3, column 28; expected 143333 bytes, actual 143848 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 37s; total estimated time: 11m 52s"; actual "; model printing time: 5m 39s; total estimated time: 11m 54s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 101993 bytes, actual 101993 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 42s; total estimated time: 9m 56s"; actual "; model printing time: 3m 44s; total estimated time: 9m 57s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.8 nozzle | first difference at byte 93 (line 3, column 28; expected 87104 bytes, actual 87104 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 43s; total estimated time: 9m 0s"; actual "; model printing time: 2m 44s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.2 nozzle | first difference at byte 93 (line 3, column 28; expected 371249 bytes, actual 371272 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 15m 25s"; actual "; model printing time: 8m 39s; total estimated time: 15m 28s"   line 4: "; estimated first layer printing time (normal mode) = 6m 48s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 143988 bytes, actual 143988 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 5s; total estimated time: 10m 20s"; actual "; model printing time: 4m 8s; total estimated time: 10m 23s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 101946 bytes, actual 102145 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 46s; total estimated time: 9m 0s"; actual "; model printing time: 2m 48s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.8 nozzle | first difference at byte 92 (line 3, column 27; expected 87316 bytes, actual 87327 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 7s; total estimated time: 8m 25s"; actual "; model printing time: 2m 9s; total estimated time: 8m 27s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| PASS | BBL/Bambu Lab P2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.4 nozzle |  |
| DIVERGENT | BBL/Bambu Lab P2S 0.6 nozzle | first difference at byte 60100 (line 1374, column 8; expected 124435 bytes, actual 124609 bytes) context:   line 1373: "; LINE_WIDTH: 0.62"   line 1374: expected "G1 F2710"; actual "G1 F2717"   line 1375: "G1 X123.93 Y132.07 E.55282" |
| PASS | BBL/Bambu Lab P2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab X1 0.2 nozzle | first difference at byte 92 (line 3, column 27; expected 396001 bytes, actual 395885 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 37s; total estimated time: 18m 36s"; actual "; model printing time: 8m 40s; total estimated time: 18m 39s"   line 4: "; estimated first layer printing time (normal mode) = 9m 58s" |
| DIVERGENT | BBL/Bambu Lab X1 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 169245 bytes, actual 169164 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 38s; total estimated time: 12m 54s"; actual "; model printing time: 5m 40s; total estimated time: 12m 57s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 127211 bytes, actual 127139 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 43s; total estimated time: 11m 1s"; actual "; model printing time: 3m 45s; total estimated time: 11m 2s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
| DIVERGENT | BBL/Bambu Lab X1 0.8 nozzle | first difference at byte 93 (line 3, column 28; expected 112013 bytes, actual 111945 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 44s; total estimated time: 10m 7s"; actual "; model printing time: 2m 45s; total estimated time: 10m 9s"   line 4: "; estimated first layer printing time (normal mode) = 7m 23s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.2 nozzle | first difference at byte 92 (line 3, column 27; expected 396062 bytes, actual 395946 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 37s; total estimated time: 18m 36s"; actual "; model printing time: 8m 40s; total estimated time: 18m 39s"   line 4: "; estimated first layer printing time (normal mode) = 9m 58s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 169250 bytes, actual 169180 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 7s; total estimated time: 11m 23s"; actual "; model printing time: 4m 9s; total estimated time: 11m 25s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 127323 bytes, actual 127220 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 47s; total estimated time: 10m 4s"; actual "; model printing time: 2m 49s; total estimated time: 10m 6s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.8 nozzle | first difference at byte 92 (line 3, column 27; expected 112166 bytes, actual 112077 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 9s; total estimated time: 9m 32s"; actual "; model printing time: 2m 10s; total estimated time: 9m 34s"   line 4: "; estimated first layer printing time (normal mode) = 7m 23s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.2 nozzle | first difference at byte 93 (line 3, column 28; expected 394389 bytes, actual 394309 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 19m 11s"; actual "; model printing time: 8m 39s; total estimated time: 19m 15s"   line 4: "; estimated first layer printing time (normal mode) = 10m 35s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 167733 bytes, actual 167663 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 6s; total estimated time: 11m 33s"; actual "; model printing time: 4m 8s; total estimated time: 11m 35s"   line 4: "; estimated first layer printing time (normal mode) = 7m 27s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 125678 bytes, actual 125595 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 46s; total estimated time: 10m 17s"; actual "; model printing time: 2m 48s; total estimated time: 10m 19s"   line 4: expected "; estimated first layer printing time (normal mode) = 7m 30s"; actual "; estimated first layer printing time (normal mode) = 7m 31s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.8 nozzle | first difference at byte 92 (line 3, column 27; expected 110611 bytes, actual 110521 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 8s; total estimated time: 9m 45s"; actual "; model printing time: 2m 9s; total estimated time: 9m 47s"   line 4: "; estimated first layer printing time (normal mode) = 7m 37s" |
| PASS | BBL/Bambu Lab X2D 0.2 nozzle |  |
| PASS | BBL/Bambu Lab X2D 0.4 nozzle |  |
| PASS | BBL/Bambu Lab X2D 0.6 nozzle |  |
| PASS | BBL/Bambu Lab X2D 0.8 nozzle |  |
| PASS | BIQU/BIQU B1 (0.4 nozzle) |  |
| DIVERGENT | BIQU/BIQU BX (0.4 nozzle) | first difference at byte 5448 (line 235, column 1; expected 103754 bytes, actual 104046 bytes) context:   line 234: "G1 X120.2 Y120.2 E.27934"   line 235: expected "M73 P55 R5"; actual "G1 X129.8 Y120.2 E.27934"   line 236: expected "G1 X129.8 Y120.2 E.27934"; actual "M73 P55 R5" |
| PASS | BIQU/BIQU Hurakan (0.4 nozzle) |  |
| PASS | Blocks/BLOCKS Pro S100 0.4 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 0.6 nozzle |  |
| DIVERGENT | Blocks/BLOCKS Pro S100 0.8 nozzle | first difference at byte 2393 (line 105, column 1; expected 74144 bytes, actual 75208 bytes) context:   line 104: "G1 X503.67 Y503.67 E.65717"   line 105: expected "M73 P5 R2"; actual "G1 X496.33 Y503.67 E.65717"   line 106: expected "G1 X496.33 Y503.67 E.65717"; actual "M73 P5 R2" |
| PASS | Blocks/BLOCKS Pro S100 1.0 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 1.2 nozzle |  |
| DIVERGENT | Blocks/BLOCKS RD50 V2 0.4 nozzle | first difference at byte 2890 (line 121, column 1; expected 126266 bytes, actual 127984 bytes) context:   line 120: "G1 X252.63 Y246.39 E.01645"   line 121: expected "M73 P2 R4"; actual "G1 X253.61 Y247.37 E.04199"   line 122: expected "G1 X253.61 Y247.37 E.04199"; actual "M73 P2 R4" |
| DIVERGENT | Blocks/BLOCKS RD50 V2 0.6 nozzle | first difference at byte 4104 (line 175, column 5; expected 95456 bytes, actual 96532 bytes) context:   line 174: ";WIDTH:0.62"   line 175: expected "G1 F2941"; actual "G1 F3000"   line 176: "G1 X245 Y257.289 E.14411" |
| PASS | Blocks/BLOCKS RD50 V2 0.8 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.4 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.6 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.8 nozzle |  |
| PASS | CONSTRUCT3D/Construct 1 0.4 nozzle |  |
| DIVERGENT | CONSTRUCT3D/Construct 1 XL 0.6 nozzle | first difference at byte 2726 (line 122, column 1; expected 86942 bytes, actual 88044 bytes) context:   line 121: "G1 X166.293 Y182.576 Z.52"   line 122: expected "M73 P6 R3"; actual "G1 Z.32"   line 123: expected "G1 Z.32"; actual "M73 P6 R3" |
| PASS | Chuanying/Chuanying X1 0.25 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.4 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.6 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.8 Nozzle |  |
| DIVERGENT | Co Print/Co Print ChromaSet 0.4 nozzle | first difference at byte 56826 (line 2303, column 1; expected 157126 bytes, actual 157364 bytes) context:   line 2302: "G1 X113.268 Y117.119 Z4.48571"   line 2303: expected "M73 P43 R3"; actual "G1 X113.268 Y116.591 Z4.51429"   line 2304: expected "G1 X113.268 Y116.591 Z4.51429"; actual "M73 P43 R3" |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 |  |
| DIVERGENT | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 Plus | first difference at byte 6646 (line 252, column 7; expected 156577 bytes, actual 157344 bytes) context:   line 251: ";WIDTH:0.45"   line 252: expected "G1 F3249"; actual "G1 F3254"   line 253: "G1 X140.645 Y145.855 E.26561" |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle fast |  |
| PASS | CoLiDo/CoLiDo 160 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo SR1 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo X16 0.4 nozzle |  |
| DIVERGENT | Comgrow/Comgrow T300 0.4 nozzle | first difference at byte 19753 (line 768, column 7; expected 107462 bytes, actual 107602 bytes) context:   line 767: ";WIDTH:0.42"   line 768: expected "G1 F2558"; actual "G1 F2562"   line 769: "G1 X145.21 Y154.79 E.29437" |
| DIVERGENT | Comgrow/Comgrow T500 0.4 nozzle | first difference at byte 6518 (line 274, column 8; expected 106321 bytes, actual 107055 bytes) context:   line 273: ";TYPE:Inner wall"   line 274: expected "G1 F3181"; actual "G1 F3186"   line 275: "G1 X245.557 Y254.443 E.25856" |
| PASS | Comgrow/Comgrow T500 0.6 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.8 nozzle |  |
| DIVERGENT | Creality/Creality CR-10 Max 0.4 nozzle | first difference at byte 2744 (line 134, column 10; expected 102301 bytes, actual 102790 bytes) context:   line 133: "G1 Z.2"   line 134: expected "M73 P11 R7"; actual "M73 P11 R8"   line 135: "G1 E5 F2400" |
| PASS | Creality/Creality CR-10 SE 0.2 nozzle |  |
| DIVERGENT | Creality/Creality CR-10 SE 0.4 nozzle | first difference at byte 55029 (line 2554, column 1; expected 89542 bytes, actual 89542 bytes) context:   line 2553: "G1 X114.79 Y114.75 E.29314"   line 2554: expected "G1 E-.61523 F1800"; actual "M73 P88 R0"   line 2555: expected "M73 P88 R0"; actual "G1 E-.61523 F1800" |
| PASS | Creality/Creality CR-10 SE 0.6 nozzle |  |
| DIVERGENT | Creality/Creality CR-10 SE 0.8 nozzle | first difference at byte 5179 (line 210, column 1; expected 71857 bytes, actual 72371 bytes) context:   line 209: "G1 X110.158 Y106.88 E.31448"   line 210: expected "M73 P7 R5"; actual "G1 X108.981 Y106.88 E.07988"   line 211: expected "G1 X108.981 Y106.88 E.07988"; actual "M73 P7 R5" |
| DIVERGENT | Creality/Creality CR-10 V2 0.4 nozzle | first difference at byte 3543 (line 161, column 1; expected 99979 bytes, actual 100244 bytes) context:   line 160: "G1 X153.987 Y152.91 E.01673"   line 161: expected "M73 P5 R7"; actual "G1 X147.09 Y146.013 E.30281"   line 162: expected "G1 X147.09 Y146.013 E.30281"; actual "G1 X146.552 Y146.013 E.01673" |
| PASS | Creality/Creality CR-10 V3 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 V3 0.6 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 Max 0.2 nozzle | first difference at byte 7041 (line 291, column 29; expected 394066 bytes, actual 394066 bytes) context:   line 290: "G1 X195.605 Y204.196 E-.58892"   line 291: expected "G1 X195.605 Y203.947 E-.37351"; actual "G1 X195.605 Y203.947 E-.37352"   line 292: "G1 X195.858 Y204.2 E-.53756" |
| PASS | Creality/Creality CR-6 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.8 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 SE 0.2 nozzle | first difference at byte 3772 (line 171, column 1; expected 393805 bytes, actual 394163 bytes) context:   line 170: "G1 X117.463 Y113.105 E.00214"   line 171: expected "G1 X121.895 Y117.537 E.05396"; actual "M73 P4 R22"   line 172: expected "G1 X121.895 Y117.786 E.00214"; actual "G1 X121.895 Y117.537 E.05396" |
| PASS | Creality/Creality CR-6 SE 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 SE 0.6 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 SE 0.8 nozzle | first difference at byte 3692 (line 182, column 7; expected 62671 bytes, actual 63170 bytes) context:   line 181: ";WIDTH:0.8"   line 182: expected "G1 F1142"; actual "G1 F1150"   line 183: "G1 X114.431 Y120.569 E.5852" |
| PASS | Creality/Creality CR-M4 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 0.2 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 0.4 nozzle | first difference at byte 2990 (line 138, column 1; expected 100617 bytes, actual 100727 bytes) context:   line 137: "G1 E4 F2400"   line 138: expected "M73 P10 R7"; actual ";TYPE:Bottom surface"   line 139: expected ";TYPE:Bottom surface"; actual ";WIDTH:0.423911" |
| PASS | Creality/Creality Ender-3 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 S1 Pro 0.4 nozzle | first difference at byte 5030 (line 217, column 7; expected 100723 bytes, actual 101353 bytes) context:   line 216: ";WIDTH:0.42"   line 217: expected "G1 F2875"; actual "G1 F2883"   line 218: "G1 X105 Y117.566 E.04899" |
| PASS | Creality/Creality Ender-3 V2 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V2 Neo 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 0.6 nozzle | first difference at byte 6413 (line 258, column 8; expected 86228 bytes, actual 86321 bytes) context:   line 257: ";WIDTH:0.65"   line 258: expected "G1 F1301"; actual "G1 F1304"   line 259: "G1 X106.543 Y113.457 E.40458" |
| DIVERGENT | Creality/Creality Ender-3 V3 KE 0.2 nozzle | first difference at byte 6170 (line 239, column 8; expected 171777 bytes, actual 171903 bytes) context:   line 238: ";WIDTH:0.225"   line 239: expected "G1 F8219"; actual "G1 F8215"   line 240: "G1 X105.323 Y114.678 E.14163" |
| PASS | Creality/Creality Ender-3 V3 KE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 Plus 0.4 nozzle | first difference at byte 22406 (line 893, column 1; expected 123624 bytes, actual 123624 bytes) context:   line 892: "G1 X146.052 Y146.052 E.26192"   line 893: expected "M73 P21 R4"; actual "G1 X153.948 Y146.052 E.26192"   line 894: expected "G1 X153.948 Y146.052 E.26192"; actual "M73 P21 R4" |
| PASS | Creality/Creality Ender-3 V3 Plus 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V4 0.4 nozzle | first difference at byte 12940 (line 554, column 1; expected 108071 bytes, actual 108165 bytes) context:   line 553: "G1 X105.602 Y105.602 E.28284"   line 554: expected "M73 P37 R5"; actual "G1 X114.398 Y105.602 E.28284"   line 555: expected "G1 X114.398 Y105.602 E.28284"; actual "M73 P37 R5" |
| PASS | Creality/Creality Ender-5 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-5 Max 0.4 nozzle | first difference at byte 4525 (line 170, column 8; expected 125115 bytes, actual 126411 bytes) context:   line 169: ";WIDTH:0.45"   line 170: expected "G1 F3262"; actual "G1 F3269"   line 171: "G1 X195.602 Y204.398 E.27402" |
| DIVERGENT | Creality/Creality Ender-5 Max 0.6 nozzle | first difference at byte 8622 (line 357, column 5; expected 76620 bytes, actual 77176 bytes) context:   line 356: "G1 X196.143 Y202.845 E.0942"   line 357: expected "G1 X202.666 Y196.413 F30000"; actual "G1 X196.477 Y203.496 F30000"   line 358: ";TYPE:Internal solid infill" |
| DIVERGENT | Creality/Creality Ender-5 Max 0.8 nozzle | first difference at byte 6461 (line 270, column 1; expected 64111 bytes, actual 64184 bytes) context:   line 269: "G1 X203.116 Y203.856 E.08583"   line 270: expected "M73 P17 R2"; actual "SET_VELOCITY_LIMIT ACCEL=12000 ACCEL_TO_DECEL=6000"   line 271: expected "SET_VELOCITY_LIMIT ACCEL=12000 ACCEL_TO_DECEL=6000"; actual "G1 E-1 F2100" |
| PASS | Creality/Creality Ender-5 Plus 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.2 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.25 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.3 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.5 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.6 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-5 Pro (2019) 1.0 nozzle | first difference at byte 5045 (line 244, column 6; expected 58214 bytes, actual 58548 bytes) context:   line 243: ";WIDTH:1.05"   line 244: expected "G1 F867"; actual "G1 F875"   line 245: "G1 X105.525 Y114.475 E1.01074" |
| PASS | Creality/Creality Ender-5 S1 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5S 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-6 0.4 nozzle | first difference at byte 5935 (line 256, column 1; expected 99900 bytes, actual 100559 bytes) context:   line 255: "G1 X130.765 Y132.051 E.04019"   line 256: expected "M73 P12 R8"; actual "G1 X130 Y132.189 E.02388"   line 257: expected "G1 X130 Y132.189 E.02388"; actual "M73 P12 R8" |
| DIVERGENT | Creality/Creality Hi 0.2 nozzle | first difference at byte 19881 (line 757, column 8; expected 342423 bytes, actual 342635 bytes) context:   line 756: ";WIDTH:0.22"   line 757: expected "G1 F2192"; actual "G1 F2195"   line 758: "G1 X125.727 Y134.273 E.06359" |
| PASS | Creality/Creality Hi 0.4 nozzle |  |
| PASS | Creality/Creality Hi 0.6 nozzle |  |
| PASS | Creality/Creality Hi 0.8 nozzle |  |
| PASS | Creality/Creality K1 (0.4 nozzle) |  |
| PASS | Creality/Creality K1 (0.6 nozzle) |  |
| PASS | Creality/Creality K1 (0.8 nozzle) |  |
| DIVERGENT | Creality/Creality K1 Max (0.4 nozzle) | first difference at byte 85262 (line 3754, column 1; expected 112955 bytes, actual 112955 bytes) context:   line 3753: "G1 X145.877 Y152.634 E.01639"   line 3754: expected "G1 X147.366 Y154.123 E.06472"; actual "M73 P98 R0"   line 3755: expected "M73 P98 R0"; actual "G1 X147.366 Y154.123 E.06472" |
| PASS | Creality/Creality K1 Max (0.6 nozzle) |  |
| DIVERGENT | Creality/Creality K1 Max (0.8 nozzle) | first difference at byte 3354 (line 147, column 1; expected 56304 bytes, actual 56525 bytes) context:   line 146: "G1 X154.59 Y154.59 F30000"   line 147: expected "M73 P47 R2"; actual ";TYPE:Outer wall"   line 148: expected ";TYPE:Outer wall"; actual "G1 F1200" |
| PASS | Creality/Creality K1 Max_CFS-C 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K1 SE 0.4 nozzle | first difference at byte 36756 (line 1561, column 1; expected 115805 bytes, actual 116076 bytes) context:   line 1560: "G1 X105.892 Y108.061 E.05401"   line 1561: expected "M73 P56 R3"; actual "; stop printing object cube10.stl id:0 copy 0"   line 1562: expected "; stop printing object cube10.stl id:0 copy 0"; actual ";LAYER_CHANGE" |
| PASS | Creality/Creality K1 SE 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Creality/Creality K1 SE 0.8 nozzle | Creality/Creality K1 SE 0.8 nozzle process: default preset "0.40mm Standard @Creality K1 SE 0.8 nozzle" not found |
| DIVERGENT | Creality/Creality K1 SE_CFS-C 0.4 nozzle | first difference at byte 4005 (line 173, column 8; expected 115849 bytes, actual 116018 bytes) context:   line 172: ";WIDTH:0.45"   line 173: expected "G1 F1363"; actual "G1 F1364"   line 174: "G1 X105.602 Y114.398 E.26837" |
| PASS | Creality/Creality K1C 0.4 nozzle |  |
| PASS | Creality/Creality K1C 0.6 nozzle |  |
| DIVERGENT | Creality/Creality K1C 0.8 nozzle | first difference at byte 2682 (line 107, column 1; expected 72120 bytes, actual 72630 bytes) context:   line 106: "G1 X107.305 Y107.305 E.07912"   line 107: expected "M73 P41 R3"; actual "G1 X112.695 Y112.695 E.65107"   line 108: expected "G1 X112.695 Y112.695 E.65107"; actual "M73 P41 R3" |
| DIVERGENT | Creality/Creality K1C_CFS-C 0.4 nozzle | first difference at byte 6572 (line 269, column 1; expected 116650 bytes, actual 117121 bytes) context:   line 268: "G1 X108.432 Y105.877 E.01588"   line 269: expected "M73 P30 R5"; actual "G1 X105.877 Y108.432 E.10766"   line 270: expected "G1 X105.877 Y108.432 E.10766"; actual "M73 P30 R5" |
| DIVERGENT | Creality/Creality K1_CFS-C 0.4 nozzle | first difference at byte 4145 (line 180, column 8; expected 112109 bytes, actual 112859 bytes) context:   line 179: ";WIDTH:0.45"   line 180: expected "G1 F1361"; actual "G1 F1364"   line 181: "G1 X105.602 Y114.398 E.26837" |
| PASS | Creality/Creality K2 0.2 nozzle |  |
| PASS | Creality/Creality K2 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K2 0.6 nozzle | first difference at byte 3240 (line 134, column 1; expected 74638 bytes, actual 75615 bytes) context:   line 133: "G1 X126.432 Y130.48 E.05491"   line 134: expected "G1 X129.52 Y133.568 E.29616"; actual "M73 P40 R2"   line 135: expected "M73 P40 R2"; actual "G1 X129.52 Y133.568 E.29616" |
| PASS | Creality/Creality K2 0.8 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.2 nozzle |  |
| DIVERGENT | Creality/Creality K2 Plus 0.4 nozzle | first difference at byte 23851 (line 983, column 1; expected 133842 bytes, actual 134475 bytes) context:   line 982: "G1 X171.256 Y179.094 E.19669"   line 983: expected "M73 P26 R3"; actual "G1 X170.906 Y178.744 E.013"   line 984: expected "G1 X170.906 Y178.744 E.013"; actual "M73 P26 R3" |
| DIVERGENT | Creality/Creality K2 Plus 0.6 nozzle | first difference at byte 3372 (line 142, column 1; expected 95769 bytes, actual 95769 bytes) context:   line 141: "G1 X174.52 Y178.568 E.29616"   line 142: expected "G1 X173.71 Y178.568 E.05491"; actual "M73 P16 R3"   line 143: expected "M73 P16 R3"; actual "G1 X173.71 Y178.568 E.05491" |
| DIVERGENT | Creality/Creality K2 Plus 0.8 nozzle | first difference at byte 1052 (line 46, column 1; expected 78834 bytes, actual 79002 bytes) context:   line 45: "G1 X0 Y150 F6000"   line 46: expected "M73 P13 R2"; actual "G1 X0 Y0 E15 F6000"   line 47: expected "G1 X0 Y0 E15 F6000"; actual "G1 X150 Y0 E15 F6000" |
| PASS | Creality/Creality K2 Pro 0.2 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K2 Pro 0.6 nozzle | first difference at byte 4108 (line 170, column 7; expected 75414 bytes, actual 75726 bytes) context:   line 169: ";WIDTH:0.62"   line 170: expected "G1 F2824"; actual "G1 F2830"   line 171: "G1 X145.866 Y154.134 E.54437" |
| PASS | Creality/Creality K2 Pro 0.8 nozzle |  |
| PASS | Creality/Creality K2 SE 0.4 nozzle |  |
| DIVERGENT | Creality/Creality SPARKX i7 0.2 nozzle | first difference at byte 6790 (line 271, column 8; expected 361252 bytes, actual 361673 bytes) context:   line 270: ";WIDTH:0.22"   line 271: expected "G1 F3811"; actual "G1 F3812"   line 272: "G1 X125.706 Y134.294 E.06607" |
| DIVERGENT | Creality/Creality SPARKX i7 0.4 nozzle | first difference at byte 2827 (line 132, column 1; expected 137363 bytes, actual 137362 bytes) context:   line 131: "G1 X129.221 Y126.2 E.26305"   line 132: expected "M73 P9 R6"; actual "G1 X128.55 Y126.2 E.02728"   line 133: expected "G1 X128.55 Y126.2 E.02728"; actual "M73 P9 R6" |
| DIVERGENT | Creality/Creality SPARKX i7 0.6 nozzle | first difference at byte 2217 (line 107, column 1; expected 103458 bytes, actual 104148 bytes) context:   line 106: "G1 X134.123 Y127.322 E.1484"   line 107: expected "M73 P13 R4"; actual "G1 X134.123 Y128.122 E.05811"   line 108: expected "G1 X134.123 Y128.122 E.05811"; actual "M73 P13 R4" |
| DIVERGENT | Creality/Creality SPARKX i7 0.8 nozzle | first difference at byte 954 (line 46, column 7; expected 88493 bytes, actual 88503 bytes) context:   line 45: "G1 X115 E.3742  F1600"   line 46: expected "M73 P15 R3"; actual "M73 P14 R3"   line 47: "G1 X110 E.3742  F6400" |
| DIVERGENT | Creality/Creality Sermoon V1 0.4 nozzle | first difference at byte 2015 (line 53, column 9; expected 162827 bytes, actual 167626 bytes) context:   line 52: "G1 Z2.0 F3000                          ; Move Z Axis up"   line 53: expected "M73 P3 R6"; actual "M73 P3 R7"   line 54: "G92 E0" |
| ORCA_ERROR | Cubicon/Cubicon xCeler-I 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-4mT1uG") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Mini 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-KWIlFj") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Plus 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-OnFRvY") |
| PASS | Custom/MyKlipper 0.2 nozzle |  |
| PASS | Custom/MyKlipper 0.4 nozzle |  |
| DIVERGENT | Custom/MyKlipper 0.6 nozzle | first difference at byte 6351 (line 267, column 6; expected 108872 bytes, actual 109530 bytes) context:   line 266: ";WIDTH:0.66"   line 267: expected "G1 F2470"; actual "G1 F2501"   line 268: "G1 X121.547 Y128.453 E.34725" |
| PASS | Custom/MyKlipper 0.8 nozzle |  |
| PASS | Custom/MyMarlin 0.4 nozzle |  |
| PASS | Custom/MyRRF 0.4 nozzle |  |
| PASS | Custom/MyRepetier 0.4 nozzle |  |
| DIVERGENT | Custom/MyToolChanger 0.2 nozzle | first difference at byte 209688 (line 8001, column 1; expected 462943 bytes, actual 462943 bytes) context:   line 8000: "G1 X170.662 Y177.272 E.0618"   line 8001: expected "G1 X170.662 Y176.461 E.00536"; actual "M73 P50 R5"   line 8002: expected "M73 P50 R5"; actual "G1 X170.662 Y176.461 E.00536" |
| PASS | Custom/MyToolChanger 0.4 nozzle |  |
| DIVERGENT | Custom/MyToolChanger 0.6 nozzle | first difference at byte 8420 (line 355, column 1; expected 118098 bytes, actual 118518 bytes) context:   line 354: "G1 X172.222 Y177.748 F21000"   line 355: expected "M73 P9 R3"; actual "SET_VELOCITY_LIMIT ACCEL=5000 ACCEL_TO_DECEL=2500"   line 356: expected "SET_VELOCITY_LIMIT ACCEL=5000 ACCEL_TO_DECEL=2500"; actual ";WIDTH:0.694762" |
| PASS | Custom/MyToolChanger 0.8 nozzle |  |
| DIVERGENT | DeltaMaker/DeltaMaker 2 0.35 nozzle | first difference at byte 7872 (line 343, column 6; expected 187118 bytes, actual 187126 bytes) context:   line 342: ";WIDTH:0.385"   line 343: expected "G1 F2292"; actual "G1 F2313"   line 344: "G1 X-4.111 Y64.111 E.21036" |
| PASS | DeltaMaker/DeltaMaker 2T 0.5 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2XT 0.5 nozzle |  |
| PASS | Dremel/Dremel 3D20 0.4 nozzle |  |
| PASS | Dremel/Dremel 3D40 0.4 nozzle |  |
| DIVERGENT | Dremel/Dremel 3D45 0.4 nozzle | first difference at byte 2078 (line 94, column 1; expected 112422 bytes, actual 113300 bytes) context:   line 93: "G1 X-23.325 Y5 E4.71029"   line 94: expected "M73 P3 R7"; actual "G1 X-23.325 Y-5 E4.86452"   line 95: expected "G1 X-23.325 Y-5 E4.86452"; actual "G1 X-22.931 Y-6.57 E4.88949" |
| PASS | Elegoo/Elegoo Centauri 0.2 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri 0.4 nozzle | first difference at byte 9251 (line 404, column 1; expected 100894 bytes, actual 101211 bytes) context:   line 403: "G1 X123.935 Y131.109 E.01589"   line 404: expected "M73 P10 R3"; actual "G1 X125.061 Y132.234 E.04745"   line 405: expected "G1 X125.061 Y132.234 E.04745"; actual "M73 P10 R3" |
| PASS | Elegoo/Elegoo Centauri 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.2 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri 2 0.4 nozzle | first difference at byte 3131 (line 148, column 1; expected 106087 bytes, actual 106814 bytes) context:   line 147: "G1 X127.439 Y130.259 E.16797"   line 148: expected "M73 P15 R3"; actual "G1 X126.775 Y130.259 E.02468"   line 149: expected "G1 X126.775 Y130.259 E.02468"; actual "M73 P15 R3" |
| PASS | Elegoo/Elegoo Centauri 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.6 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 0.8 nozzle | first difference at byte 3154 (line 160, column 1; expected 57784 bytes, actual 58424 bytes) context:   line 159: "G1 X127.893 Y131.008 E.12058"   line 160: expected "M73 P27 R1"; actual "G1 X124.992 Y128.107 E.47629"   line 161: expected "G1 X124.992 Y128.107 E.47629"; actual "M73 P27 R1" |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 2 0.2 nozzle | first difference at byte 4012 (line 175, column 1; expected 249177 bytes, actual 250239 bytes) context:   line 174: "G1 X124.358 Y125.569 E.00629"   line 175: expected "M73 P8 R8"; actual "G1 X128.931 Y130.142 E.105"   line 176: expected "G1 X128.931 Y130.142 E.105"; actual "G1 X128.544 Y130.142 E.00629" |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 0.4 nozzle | first difference at byte 1131 (line 48, column 9; expected 104581 bytes, actual 105522 bytes) context:   line 47: "G1 Z0.6 F120.0 ;Move to side a little"   line 48: expected "M73 P3 R7"; actual "M73 P3 R8"   line 49: "G1 X152 F3000" |
| PASS | Elegoo/Elegoo Neptune 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.4 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 2 0.6 nozzle | first difference at byte 5593 (line 271, column 8; expected 66280 bytes, actual 66448 bytes) context:   line 270: ";WIDTH:0.62"   line 271: expected "G1 F1505"; actual "G1 F1509"   line 272: "G1 X113.366 Y121.634 E.54472" |
| PASS | Elegoo/Elegoo Neptune 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 0.4 nozzle | first difference at byte 1920 (line 95, column 1; expected 104040 bytes, actual 105580 bytes) context:   line 94: "G1 X122.15 Y112.85 E.336"   line 95: expected "M73 P4 R7"; actual "G1 X122.15 Y122.11 E.33455"   line 96: expected "G1 X122.15 Y122.11 E.33455"; actual "M73 P4 R7" |
| DIVERGENT | Elegoo/Elegoo Neptune 3 0.6 nozzle | first difference at byte 8004 (line 382, column 8; expected 66163 bytes, actual 66448 bytes) context:   line 381: ";WIDTH:0.62"   line 382: expected "G1 F1493"; actual "G1 F1497"   line 383: "G1 X113.366 Y121.634 E.54472" |
| DIVERGENT | Elegoo/Elegoo Neptune 3 0.8 nozzle | first difference at byte 3157 (line 159, column 8; expected 56067 bytes, actual 56235 bytes) context:   line 158: ";WIDTH:0.82"   line 159: expected "G1 F1143"; actual "G1 F1146"   line 160: "G1 X113.644 Y121.356 E.89502" |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.6 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Max 0.8 nozzle | first difference at byte 4964 (line 250, column 7; expected 51015 bytes, actual 51172 bytes) context:   line 249: ";WIDTH:0.82"   line 250: expected "G1 F1087"; actual "G1 F1098"   line 251: "G1 X208.644 Y216.356 E.89502" |
| PASS | Elegoo/Elegoo Neptune 3 Max 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.6 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Plus 0.8 nozzle | first difference at byte 2429 (line 119, column 1; expected 50811 bytes, actual 51315 bytes) context:   line 118: "G1 X163.431 Y165.508 E.64682"   line 119: expected "M73 P12 R3"; actual "G1 X162.393 Y165.508 E.12058"   line 120: expected "G1 X162.393 Y165.508 E.12058"; actual "M73 P12 R3" |
| PASS | Elegoo/Elegoo Neptune 3 Plus 1.0 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Pro 0.2 nozzle | first difference at byte 6410 (line 285, column 1; expected 225473 bytes, actual 225658 bytes) context:   line 284: "G1 X119.126 Y114.919 E.0022"   line 285: expected "M73 P3 R16"; actual "G1 X114.919 Y119.126 E.04668"   line 286: expected "G1 X114.919 Y119.126 E.04668"; actual "G1 X114.638 Y119.126 E.0022" |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 1.0 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 0.2 nozzle | first difference at byte 23782 (line 986, column 1; expected 237739 bytes, actual 237923 bytes) context:   line 985: "G1 X120.493 Y121.493 F30000"   line 986: expected "M73 P8 R13"; actual "G1 F3867"   line 987: expected "G1 F3867"; actual "M73 P8 R13" |
| PASS | Elegoo/Elegoo Neptune 4 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 1.0 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Max 0.2 nozzle | first difference at byte 4294 (line 176, column 1; expected 237854 bytes, actual 237946 bytes) context:   line 175: "G1 X211.558 Y214.642 E.01608"   line 176: expected "M73 P3 R14"; actual "G1 X211.171 Y214.642 E.00629"   line 177: expected "G1 X211.171 Y214.642 E.00629"; actual "M73 P3 R14" |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Max 1.0 nozzle | first difference at byte 6344 (line 307, column 1; expected 45693 bytes, actual 46016 bytes) context:   line 306: "G1 X210.893 Y207.393 E1.27268"   line 307: expected "M73 P33 R1"; actual "G1 X218.107 Y207.393 E1.27268"   line 308: expected "G1 X218.107 Y207.393 E1.27268"; actual "M73 P33 R1" |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.2 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Pro 0.4 nozzle | first difference at byte 3695 (line 170, column 8; expected 94680 bytes, actual 95669 bytes) context:   line 169: ";WIDTH:0.45"   line 170: expected "G1 F2000"; actual "G1 F2006"   line 171: "G1 X113.102 Y122.898 E.28302" |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune X 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune X 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune X 0.8 nozzle |  |
| PASS | Elegoo/Elegoo OrangeStorm Giga 0.4 nozzle |  |
| PASS | Elegoo/Elegoo OrangeStorm Giga 0.6 nozzle |  |
| PASS | Elegoo/Elegoo OrangeStorm Giga 0.8 nozzle |  |
| PASS | Elegoo/Elegoo OrangeStorm Giga 1.0 nozzle |  |
| PASS | Eryone/Eryone ER20 0.2 nozzle |  |
| PASS | Eryone/Eryone ER20 0.4 nozzle |  |
| DIVERGENT | Eryone/Eryone ER20 0.5 nozzle | first difference at byte 2090 (line 90, column 7; expected 112154 bytes, actual 113351 bytes) context:   line 89: "G1 X125.739 Y114.239"   line 90: expected "G1 X126.225 Y114.725"; actual "G1 X125.739 Y114.239"   line 91: expected "M205 X9 Y9"; actual "G1 X126.225 Y114.725" |
| DIVERGENT | Eryone/Eryone ER20 0.6 nozzle | first difference at byte 2079 (line 90, column 7; expected 93467 bytes, actual 94160 bytes) context:   line 89: "G1 X125.667 Y114.167"   line 90: expected "G1 X126.19 Y114.69"; actual "G1 X125.667 Y114.167"   line 91: expected "M205 X9 Y9"; actual "G1 X126.19 Y114.69" |
| DIVERGENT | Eryone/Eryone ER20 0.8 nozzle | first difference at byte 2110 (line 92, column 7; expected 67425 bytes, actual 67929 bytes) context:   line 91: "G1 X125.399 Y113.899"   line 92: expected "G1 X126.09 Y114.59"; actual "G1 X125.399 Y113.899"   line 93: expected "G1 Z.4"; actual "G1 X126.09 Y114.59" |
| PASS | Eryone/Eryone ER20 Klipper 0.2 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.4 nozzle |  |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.5 nozzle | first difference at byte 1564 (line 60, column 7; expected 144405 bytes, actual 145287 bytes) context:   line 59: "G1 X125.784 Y114.284"   line 60: expected "G1 X126.24 Y114.74"; actual "G1 X125.784 Y114.284"   line 61: expected "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 SQUARE_CORNER_VELOCITY=9"; actual "G1 X126.24 Y114.74" |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.6 nozzle | first difference at byte 1571 (line 61, column 7; expected 114496 bytes, actual 115504 bytes) context:   line 60: "G1 X125.667 Y114.167"   line 61: expected "G1 X126.19 Y114.69"; actual "G1 X125.667 Y114.167"   line 62: expected "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 SQUARE_CORNER_VELOCITY=9"; actual "G1 X126.19 Y114.69" |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.8 nozzle | first difference at byte 1602 (line 63, column 7; expected 80922 bytes, actual 81741 bytes) context:   line 62: "G1 X125.399 Y113.899"   line 63: expected "G1 X126.09 Y114.59"; actual "G1 X125.399 Y113.899"   line 64: expected "G1 Z.4"; actual "G1 X126.09 Y114.59" |
| DIVERGENT | Eryone/Thinker X400 0.2 nozzle | first difference at byte 58879 (line 2288, column 1; expected 336452 bytes, actual 336566 bytes) context:   line 2287: "G1 X204.493 Y204.473 E.07253"   line 2288: expected "M73 P20 R7"; actual "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000"   line 2289: expected "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000"; actual "G1 X204.691 Y204.691 F24000" |
| PASS | Eryone/Thinker X400 0.4 nozzle |  |
| PASS | FLSun/FLSun Q5 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun QQ-S Pro 0.4 nozzle | first difference at byte 25213 (line 1260, column 1; expected 103841 bytes, actual 103931 bytes) context:   line 1259: "G1 X1.036 Y2.093 E.04955"   line 1260: expected "M73 P32 R5"; actual "G1 X3.109 Y2.052 E.06876"   line 1261: expected "G1 X3.109 Y2.052 E.06876"; actual "M73 P32 R5" |
| PASS | FLSun/FLSun S1 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun Super Racer 0.4 nozzle | first difference at byte 2574 (line 105, column 1; expected 104847 bytes, actual 104847 bytes) context:   line 104: "G1 X5 Y10.611 E.08056"   line 105: expected "M73 P5 R7"; actual "G1 X-5 Y10.611 E.33172"   line 106: expected "G1 X-5 Y10.611 E.33172"; actual "G1 X-7.072 Y10.214 E.06996" |
| PASS | FLSun/FLSun T1 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun V400 0.4 nozzle | first difference at byte 3993 (line 170, column 8; expected 121658 bytes, actual 122071 bytes) context:   line 169: ";WIDTH:0.45"   line 170: expected "G1 F2060"; actual "G1 F2063"   line 171: "G1 X-3.968 Y3.968 E.26325" |
| PASS | Flashforge/Flashforge AD5X 0.25 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.4 nozzle |  |
| DIVERGENT | Flashforge/Flashforge AD5X 0.6 nozzle | first difference at byte 6164 (line 271, column 8; expected 107184 bytes, actual 108034 bytes) context:   line 270: ";WIDTH:0.62"   line 271: expected "G1 F1804"; actual "G1 F1809"   line 272: "G1 X105.93 Y114.07 E.34707" |
| PASS | Flashforge/Flashforge AD5X 0.8 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.4 Nozzle | first difference at byte 5552 (line 276, column 15; expected 106902 bytes, actual 107888 bytes) context:   line 275: "G1 X4.464 Y4.464 F4800"   line 276: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 277: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.6 Nozzle | first difference at byte 4936 (line 253, column 9; expected 71444 bytes, actual 72396 bytes) context:   line 252: "G1 X4.197 Y4.197 F6000"   line 253: expected "G1 X4.196 Y4.164"; actual "G1 X4.197 Y4.197"   line 254: expected "G1 X4.164 Y4.164"; actual "G1 X4.196 Y4.164" |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.3 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series 0.4 Nozzle | first difference at byte 5126 (line 242, column 15; expected 92758 bytes, actual 94238 bytes) context:   line 241: "G1 X4.464 Y4.464 F4800"   line 242: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 243: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series 0.6 Nozzle | first difference at byte 4222 (line 216, column 1; expected 61636 bytes, actual 61824 bytes) context:   line 215: "G1 Z.6"   line 216: expected "M73 P10 R4"; actual "G1 E5 F2100"   line 217: expected "G1 E5 F2100"; actual "M73 P10 R4" |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series HS Nozzle | first difference at byte 5404 (line 256, column 15; expected 103767 bytes, actual 105025 bytes) context:   line 255: "G1 X4.464 Y4.464 F9000"   line 256: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 257: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| PASS | Flashforge/Flashforge Adventurer 5M 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.4 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 5M 0.6 Nozzle | first difference at byte 3325 (line 168, column 7; expected 70251 bytes, actual 71509 bytes) context:   line 167: ";WIDTH:0.62"   line 168: expected "G1 F1919"; actual "G1 F1924"   line 169: "G1 X-4.07 Y4.07 E.55282" |
| PASS | Flashforge/Flashforge Adventurer 5M 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.8 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Artemis 0.4 Nozzle | first difference at byte 5282 (line 252, column 15; expected 110781 bytes, actual 112430 bytes) context:   line 251: "G1 X4.458 Y4.458 F6000"   line 252: expected "G1 X4.458 Y4.439"; actual "G1 X4.458 Y4.458"   line 253: expected "G1 X4.439 Y4.439"; actual "G1 X4.458 Y4.439" |
| PASS | Flashforge/Flashforge Creator 5 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Creator 5 0.6 nozzle |  |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-39tnUM") |
| DIVERGENT | Flashforge/Flashforge Creator 5 Pro 0.4 nozzle | first difference at byte 63203 (line 3064, column 8; expected 98311 bytes, actual 98511 bytes) context:   line 3063: ";WIDTH:0.45"   line 3064: expected "G1 F2016"; actual "G1 F2019"   line 3065: "M106 S178" |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-SwIfZX") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-nEixhU") |
| PASS | Flashforge/Flashforge Guider 2s 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.25 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.4 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.4 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 0.6 HF nozzle | first difference at byte 3838 (line 177, column 4; expected 81788 bytes, actual 82170 bytes) context:   line 176: "SET_VELOCITY_LIMIT ACCEL=30000"   line 177: expected "G1 E-1.2 F1800"; actual "G1 X153.439 Y153.303 F33000"   line 178: expected ";WIPE_START"; actual "SET_VELOCITY_LIMIT ACCEL=10000" |
| PASS | Flashforge/Flashforge Guider4 0.6 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 0.8 HF nozzle | first difference at byte 1237 (line 61, column 1; expected 61625 bytes, actual 61771 bytes) context:   line 60: "G1 E1.5 F1800"   line 61: expected "SET_VELOCITY_LIMIT ACCEL=800"; actual "M73 P26 R2"   line 62: expected ";TYPE:Inner wall"; actual "SET_VELOCITY_LIMIT ACCEL=800" |
| PASS | Flashforge/Flashforge Guider4 Pro 0.25 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 Pro 0.4 HF nozzle | first difference at byte 3560 (line 166, column 7; expected 100221 bytes, actual 101010 bytes) context:   line 165: ";WIDTH:0.45"   line 166: expected "G1 F4178"; actual "G1 F4187"   line 167: "G1 X145.602 Y154.398 E.29177" |
| PASS | Flashforge/Flashforge Guider4 Pro 0.4 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 Pro 0.6 HF nozzle | first difference at byte 1822 (line 91, column 1; expected 80827 bytes, actual 82184 bytes) context:   line 90: "G1 E1.2 F1800"   line 91: expected "M73 P20 R4"; actual "SET_VELOCITY_LIMIT ACCEL=800"   line 92: expected "SET_VELOCITY_LIMIT ACCEL=800"; actual ";TYPE:Bottom surface" |
| PASS | Flashforge/Flashforge Guider4 Pro 0.6 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.8 HF nozzle |  |
| PASS | FlyingBear/FlyingBear Ghost 6 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear Ghost7 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear Reborn3 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear S1 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-5 0.4 nozzle |  |
| DIVERGENT | Folgertech/Folgertech FT-5 0.6 nozzle | first difference at byte 7559 (line 344, column 1; expected 92339 bytes, actual 92627 bytes) context:   line 343: "G1 F3067"   line 344: expected "M73 P15 R4"; actual "G1 X153.513 Y146.487 E.03552"   line 345: expected "G1 X153.513 Y146.487 E.03552"; actual "G1 X152.604 Y146.487 E.03552" |
| DIVERGENT | Folgertech/Folgertech FT-6 0.4 nozzle | first difference at byte 1910 (line 89, column 9; expected 94952 bytes, actual 95580 bytes) context:   line 88: "G1 X345.9 Y170.9 E.30542"   line 89: expected "M73 P9 R4"; actual "M73 P9 R5"   line 90: "G1 X354.1 Y170.9 E.30542" |
| DIVERGENT | Folgertech/Folgertech FT-6 0.6 nozzle | first difference at byte 3765 (line 174, column 5; expected 91240 bytes, actual 91524 bytes) context:   line 173: ";WIDTH:0.62"   line 174: expected "G1 F2990"; actual "G1 F3006"   line 175: "G1 X345.93 Y179.07 E.34707" |
| DIVERGENT | Folgertech/Folgertech i3 0.4 nozzle | first difference at byte 1793 (line 84, column 2; expected 93437 bytes, actual 94049 bytes) context:   line 83: "G1 E5 F2400"   line 84: expected "M73 P9 R4"; actual "M204 S500"   line 85: expected "M204 S500"; actual ";TYPE:Inner wall" |
| PASS | Folgertech/Folgertech i3 0.6 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A10 M 0.4 nozzle | first difference at byte 4709 (line 199, column 1; expected 111471 bytes, actual 112135 bytes) context:   line 198: "G1 X106.032 Y106.032 E.26325"   line 199: expected "M73 P6 R5"; actual "G1 X113.968 Y106.032 E.26325"   line 200: expected "G1 X113.968 Y106.032 E.26325"; actual "M73 P6 R5" |
| PASS | Geeetech/Geeetech A10 Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A10 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.2 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A20 0.4 nozzle | first difference at byte 6258 (line 268, column 1; expected 111214 bytes, actual 111663 bytes) context:   line 267: "G1 X121.326 Y128.674 E.00357"   line 268: expected "M73 P8 R5"; actual "G1 X121.326 Y128.291 E.01112"   line 269: expected "G1 X121.326 Y128.291 E.01112"; actual "G1 X128.291 Y121.326 E.28662" |
| PASS | Geeetech/Geeetech A20 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A20 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.6 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A30 Pro 0.8 nozzle | first difference at byte 4564 (line 197, column 2; expected 58133 bytes, actual 58322 bytes) context:   line 196: "G1 X164.59 Y164.51 E1.08881"   line 197: expected "M73 P13 R3"; actual "M204 S700"   line 198: expected "M204 S700"; actual "G1 E-6.33333 F1200" |
| DIVERGENT | Geeetech/Geeetech A30 T 0.4 nozzle | first difference at byte 2621 (line 109, column 1; expected 111254 bytes, actual 111715 bytes) context:   line 108: "G1 F900"   line 109: expected "M73 P4 R6"; actual "G1 X155.9 Y164.1 E.30542"   line 110: expected "G1 X155.9 Y164.1 E.30542"; actual "G1 X155.9 Y155.9 E.30542" |
| DIVERGENT | Geeetech/Geeetech M1 0.2 nozzle | first difference at byte 7948 (line 341, column 1; expected 296646 bytes, actual 297109 bytes) context:   line 340: "G1 X52.717 Y56.62 E.00227"   line 341: expected "M73 P5 R9"; actual "G1 X56.62 Y52.717 E.04465"   line 342: expected "G1 X56.62 Y52.717 E.04465"; actual "M73 P5 R9" |
| DIVERGENT | Geeetech/Geeetech M1 0.4 nozzle | first difference at byte 4541 (line 188, column 1; expected 105976 bytes, actual 106300 bytes) context:   line 187: "G1 X49.101 Y56.643 E.02474"   line 188: expected "M73 P7 R3"; actual "; stop printing object cube10.stl id:0 copy 0"   line 189: expected "; stop printing object cube10.stl id:0 copy 0"; actual "M106 S255" |
| DIVERGENT | Geeetech/Geeetech M1 0.6 nozzle | first difference at byte 4335 (line 188, column 7; expected 70808 bytes, actual 71073 bytes) context:   line 187: ";WIDTH:0.62"   line 188: expected "G1 F2775"; actual "G1 F2788"   line 189: "G1 X48.986 Y56.014 E.47735" |
| PASS | Geeetech/Geeetech M1 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.2 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar 0.4 nozzle | first difference at byte 3713 (line 157, column 1; expected 111197 bytes, actual 111657 bytes) context:   line 156: "G1 X111.956 Y113.3 E.28786"   line 157: expected "M73 P6 R6"; actual "G1 X111.284 Y113.3 E.02603"   line 158: expected "G1 X111.284 Y113.3 E.02603"; actual "M73 P6 R6" |
| PASS | Geeetech/Geeetech Mizar 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.8 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar M 0.4 nozzle | first difference at byte 7906 (line 337, column 1; expected 111521 bytes, actual 111689 bytes) context:   line 336: "G1 X132.3 Y132.3 F9000"   line 337: expected "M73 P10 R6"; actual ";TYPE:Outer wall"   line 338: expected ";TYPE:Outer wall"; actual ";WIDTH:0.4" |
| PASS | Geeetech/Geeetech Mizar Max 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.6 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar Max 0.8 nozzle | first difference at byte 4574 (line 203, column 9; expected 57422 bytes, actual 58366 bytes) context:   line 202: ";WIPE_END"   line 203: expected "G1 X162.801 Y161.387 F9000"; actual "G1 X162.375 Y162.259 F9000"   line 204: "G1 E7 F1200" |
| PASS | Geeetech/Geeetech Mizar Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.8 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar S 0.2 nozzle | first difference at byte 54428 (line 2284, column 1; expected 316240 bytes, actual 316240 bytes) context:   line 2283: "G1 X125.256 Y128.005 E.00324"   line 2284: expected "M73 P19 R13"; actual "G1 X125.7 Y126.995 E.00893"   line 2285: expected "G1 X125.7 Y126.995 E.00893"; actual "M73 P19 R13" |
| PASS | Geeetech/Geeetech Mizar S 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.2 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Thunder 0.4 nozzle | first difference at byte 4001 (line 169, column 1; expected 108815 bytes, actual 109961 bytes) context:   line 168: "G1 X121.243 Y121.575 E.38878"   line 169: expected "M73 P10 R4"; actual "G1 X121.243 Y122.239 E.02543"   line 170: expected "G1 X121.243 Y122.239 E.02543"; actual "M73 P10 R4" |
| PASS | Geeetech/Geeetech Thunder 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.8 nozzle |  |
| DIVERGENT | Ginger Additive/Ginger G1 1.2 nozzle | first difference at byte 596 (line 19, column 9; expected 39732 bytes, actual 39065 bytes) context:   line 18: "EXCLUDE_OBJECT_DEFINE NAME=cube10.stl_id_0_copy_0 CENTER=500,500 POLYGON=[[495,495],[505,495],[505,505],[495,505],[495,495]]"   line 19: expected "M73 P0 R10"; actual "M73 P0 R5"   line 20: ";TYPE:Custom" |
| PASS | Ginger Additive/Ginger G1 3.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 5.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 8.0 nozzle |  |
| PASS | InfiMech/InfiMech EX 0.4 nozzle |  |
| DIVERGENT | InfiMech/InfiMech EX+APS 0.4 nozzle | first difference at byte 4215 (line 187, column 8; expected 102522 bytes, actual 102627 bytes) context:   line 186: ";WIDTH:0.45"   line 187: expected "G1 F2007"; actual "G1 F2009"   line 188: "G1 X120.602 Y129.398 E.29177" |
| PASS | InfiMech/InfiMech TX 0.4 nozzle |  |
| DIVERGENT | InfiMech/InfiMech TX HSN 0.4 nozzle | first difference at byte 9273 (line 398, column 1; expected 106167 bytes, actual 106650 bytes) context:   line 397: "G1 X105.935 Y109.376 E.19544"   line 398: expected "M73 P13 R4"; actual "G1 X105.935 Y109.909 E.01572"   line 399: expected "G1 X105.935 Y109.909 E.01572"; actual "M73 P13 R4" |
| DIVERGENT | Kingroon/Kingroon KLP1 0.4 nozzle | first difference at byte 62381 (line 2765, column 1; expected 112197 bytes, actual 112310 bytes) context:   line 2764: "G1 X118.682 Y118.682 E.34544"   line 2765: expected "G1 X114.537 Y118.682 E.1375"; actual "M73 P70 R1"   line 2766: expected "M73 P70 R1"; actual "G1 X114.537 Y118.682 E.1375" |
| DIVERGENT | Kingroon/Kingroon KP3S 3.0 0.4 nozzle | first difference at byte 513 (line 21, column 9; expected 92286 bytes, actual 90491 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R3"; actual "M73 P0 R2"   line 22: "M201 X4000 Y4000 Z1100 E10000" |
| DIVERGENT | Kingroon/Kingroon KP3S PRO S1 0.4 nozzle | first difference at byte 11591 (line 536, column 1; expected 101730 bytes, actual 101858 bytes) context:   line 535: "G1 X96.009 Y96.009 E.26477"   line 536: expected "M73 P15 R4"; actual "G1 X103.991 Y96.009 E.26477"   line 537: expected "G1 X103.991 Y96.009 E.26477"; actual "M73 P15 R4" |
| DIVERGENT | Kingroon/Kingroon KP3S PRO V2 0.4 nozzle | first difference at byte 940 (line 39, column 3; expected 103315 bytes, actual 103327 bytes) context:   line 38: ""   line 39: expected "G10 ; retract"; actual "G1 E-.8 F2700"   line 40: ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Kingroon/Kingroon KP3S V1 0.4 nozzle | first difference at byte 5243 (line 217, column 5; expected 107950 bytes, actual 107939 bytes) context:   line 216: "G1 F4075"   line 217: expected "G1 X94.398 Y85.602 E.29177"; actual "G1 X85.602 Y94.398 E.29177"   line 218: "G1 X85.602 Y85.602 E.29177" |
| DIVERGENT | LH/LH Stinger 0.4 nozzle | first difference at byte 27518 (line 1151, column 1; expected 134927 bytes, actual 135127 bytes) context:   line 1150: "G1 X118 Y140.359 E.04049"   line 1151: expected "M73 P22 R5"; actual "G1 X117.18 Y139.538 E.03272"   line 1152: expected "G1 X117.18 Y139.538 E.03272"; actual "M73 P22 R5" |
| DIVERGENT | LH/LH Stinger MMU 0.4 nozzle | first difference at byte 627 (line 23, column 1; expected 135321 bytes, actual 135322 bytes) context:   line 22: ";TYPE:Custom"   line 23: expected "_SP_PRINT_START LANE=0 TEMP=230"; actual " _SP_PRINT_START LANE=0 TEMP=230"   line 24: "" |
| DIVERGENT | LONGER/LONGER LK10 (0.2 nozzle) | first difference at byte 8231 (line 312, column 8; expected 191653 bytes, actual 191970 bytes) context:   line 311: ";WIDTH:0.25"   line 312: expected "G1 F10268"; actual "G1 F10271"   line 313: "G1 X108.057 Y116.943 E.153" |
| PASS | LONGER/LONGER LK10 (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 (0.8 nozzle) | first difference at byte 51264 (line 2164, column 29; expected 103095 bytes, actual 103095 bytes) context:   line 2163: "G1 X110.137 Y110.137 E-.05789"   line 2164: expected "G1 X110.944 Y110.137 E-.24211"; actual "G1 X110.944 Y110.137 E-.24212"   line 2165: ";WIPE_END" |
| PASS | LONGER/LONGER LK10 Plus (0.2 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.4 nozzle) | first difference at byte 5867 (line 228, column 7; expected 124393 bytes, actual 124532 bytes) context:   line 227: ";WIDTH:0.45"   line 228: expected "G1 F5311"; actual "G1 F5322"   line 229: "G1 X156.032 Y163.968 E.26862" |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.6 nozzle) | first difference at byte 39728 (line 1653, column 1; expected 109239 bytes, actual 109497 bytes) context:   line 1652: "G1 X164.075 Y164.015 E.40837"   line 1653: expected "M73 P45 R1"; actual "G1 X164.7 Y164.7 F21000"   line 1654: expected "G1 X164.7 Y164.7 F21000"; actual "M73 P45 R1" |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.8 nozzle) | first difference at byte 16056 (line 659, column 1; expected 102536 bytes, actual 103059 bytes) context:   line 658: "G1 X157.637 Y157.637 E.44845"   line 659: expected "M73 P16 R2"; actual "G1 X160.866 Y157.637 E.21665"   line 660: expected "G1 X160.866 Y157.637 E.21665"; actual "M73 P16 R2" |
| PASS | Lulzbot/Lulzbot Taz 4 or 5 0.5 nozzle |  |
| DIVERGENT | Lulzbot/Lulzbot Taz 6 0.5 nozzle | first difference at byte 1531 (line 50, column 1; expected 93350 bytes, actual 93806 bytes) context:   line 49: "G1 X0 Y0 Z15 F5000; move up off last probe point"   line 50: expected "M73 P48 R4"; actual "G4 S1; pause"   line 51: expected "G4 S1; pause"; actual "M400; wait for moves to finish" |
| PASS | Lulzbot/Lulzbot Taz Pro Dual 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz Pro S 0.5 nozzle |  |
| PASS | M3D/M3D Enabler D8500 MM |  |
| DIVERGENT | MagicMaker/MM BoneKing 0.4 nozzle | first difference at byte 738 (line 32, column 1; expected 201125 bytes, actual 203961 bytes) context:   line 31: "G1 X0 Y100 F6000"   line 32: expected "M73 P0 R6"; actual "G92 E0"   line 33: expected "G92 E0"; actual "G0 Z0.5 F300" |
| PASS | MagicMaker/MM hj SK 0.4 nozzle |  |
| DIVERGENT | MagicMaker/MM hqs SF 0.4 nozzle | first difference at byte 1076 (line 45, column 9; expected 179630 bytes, actual 180583 bytes) context:   line 44: "G1 F9000"   line 45: expected "M73 P3 R7"; actual "M73 P3 R8"   line 46: "M117 Printing..." |
| PASS | MagicMaker/MM hqs hj 0.4 nozzle |  |
| DIVERGENT | MagicMaker/MM slb 0.4 nozzle | first difference at byte 4679 (line 232, column 1; expected 173578 bytes, actual 174196 bytes) context:   line 231: "G1 X57.7 Y57.7 E.14806"   line 232: expected "M73 P5 R8"; actual "G1 X67.3 Y57.7 E.14806"   line 233: expected "G1 X67.3 Y57.7 E.14806"; actual "M73 P5 R8" |
| DIVERGENT | Mellow/M1 0.2 nozzle | first difference at byte 375170 (line 15639, column 62; expected 395090 bytes, actual 395090 bytes) context:   line 15638: "; estimated printing time (normal mode) = 10m 11s"   line 15639: expected "; estimated first layer printing time (normal mode) = 0.438319s"; actual "; estimated first layer printing time (normal mode) = 0.438318s"   line 15640: "" |
| PASS | Mellow/M1 0.4 nozzle |  |
| DIVERGENT | Mellow/M1 0.6 nozzle | first difference at byte 4614 (line 205, column 1; expected 100473 bytes, actual 101062 bytes) context:   line 204: "G1 X55.968 Y50.602 E.05283"   line 205: expected "M73 P4 R3"; actual "G1 X50.602 Y55.968 E.4187"   line 206: expected "G1 X50.602 Y55.968 E.4187"; actual "M73 P4 R3" |
| PASS | Mellow/M1 0.8 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.2 nozzle |  |
| DIVERGENT | OpenEYE/OpenEYE Peacock V2 0.4 nozzle | first difference at byte 8255 (line 298, column 8; expected 126429 bytes, actual 127362 bytes) context:   line 297: ";WIDTH:0.45"   line 298: expected "G1 F4023"; actual "G1 F4029"   line 299: "G1 X113.895 Y125.355 E.28893" |
| PASS | OpenEYE/OpenEYE Peacock V2 0.6 nozzle |  |
| DIVERGENT | OpenEYE/OpenEYE Peacock V2 0.8 nozzle | first difference at byte 4156 (line 148, column 7; expected 68964 bytes, actual 69374 bytes) context:   line 147: ";WIDTH:0.82"   line 148: expected "G1 F2326"; actual "G1 F2338"   line 149: "G1 X114.48 Y124.77 E.90216" |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.2 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.4 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.6 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.8 nozzle |  |
| DIVERGENT | Peopoly/Peopoly Magneto X 0.4 nozzle | first difference at byte 66946 (line 2459, column 1; expected 205814 bytes, actual 206009 bytes) context:   line 2458: "G1 X151.942 Y198.648 E.00598"   line 2459: expected "M73 P36 R4"; actual "G1 X151.664 Y197.746 E.01278"   line 2460: expected "G1 X151.664 Y197.746 E.01278"; actual "M73 P36 R4" |
| PASS | Peopoly/Peopoly Magneto X 0.6 nozzle |  |
| PASS | Peopoly/Peopoly Magneto X 0.8 nozzle |  |
| PASS | Phrozen/Phrozen Arco 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.2 nozzle |  |
| DIVERGENT | Positron3D/The Positron 0.4 nozzle | first difference at byte 16026 (line 647, column 1; expected 127016 bytes, actual 127568 bytes) context:   line 646: "G1 X93.983 Y93.943 E.25646"   line 647: expected "M73 P14 R3"; actual "SET_VELOCITY_LIMIT ACCEL=7000 ACCEL_TO_DECEL=3500 SQUARE_CORNER_VELOCITY=12"   line 648: expected "SET_VELOCITY_LIMIT ACCEL=7000 ACCEL_TO_DECEL=3500 SQUARE_CORNER_VELOCITY=12"; actual "G1 X94.38 Y94.38 F25200" |
| DIVERGENT | Positron3D/The Positron 0.6 nozzle | first difference at byte 21192 (line 875, column 10; expected 112317 bytes, actual 112586 bytes) context:   line 874: "G1 X94.7 Y85.3 E.42671"   line 875: expected "M73 P22 R2"; actual "M73 P22 R3"   line 876: "G1 X94.7 Y94.64 E.42399" |
| PASS | Positron3D/The Positron 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.25 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One 0.4 nozzle | first difference at byte 6667 (line 300, column 1; expected 96233 bytes, actual 96233 bytes) context:   line 299: "G1 X121.021 Y111.083 E.24231"   line 300: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 301: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| DIVERGENT | Prusa/Prusa CORE One 0.5 nozzle | first difference at byte 5025 (line 234, column 2; expected 97548 bytes, actual 98188 bytes) context:   line 233: "G1 X129.725 Y114.725 F21000"   line 234: expected "M73 P86 R5"; actual "M204 P3000"   line 235: expected "M204 P3000"; actual ";TYPE:Outer wall" |
| DIVERGENT | Prusa/Prusa CORE One 0.6 nozzle | first difference at byte 3587 (line 174, column 1; expected 81571 bytes, actual 81995 bytes) context:   line 173: "G1 X128.302 Y111.039 E.04817"   line 174: expected "M73 P88 R3"; actual "G1 X123.961 Y106.698 E.32677"   line 175: expected "G1 X123.961 Y106.698 E.32677"; actual "G1 X123.056 Y106.698 E.04817" |
| DIVERGENT | Prusa/Prusa CORE One 0.8 nozzle | first difference at byte 25339 (line 1332, column 1; expected 55739 bytes, actual 56063 bytes) context:   line 1331: "G1 X129.55 Y114.55 E1.2321"   line 1332: expected "M73 P99 R0"; actual "G1 X120.45 Y114.55 E1.2321"   line 1333: expected "G1 X120.45 Y114.55 E1.2321"; actual "M73 P99 R0" |
| DIVERGENT | Prusa/Prusa CORE One HF 0.4 nozzle | first difference at byte 6631 (line 297, column 1; expected 95737 bytes, actual 95737 bytes) context:   line 296: "G1 X121.021 Y111.083 E.24231"   line 297: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 298: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa CORE One HF 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One L 0.4 nozzle | first difference at byte 7475 (line 343, column 6; expected 96077 bytes, actual 96077 bytes) context:   line 342: ";WIDTH:0.45"   line 343: expected "G1 F1912"; actual "G1 F1899"   line 344: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L 0.5 nozzle | first difference at byte 514 (line 21, column 9; expected 90693 bytes, actual 90702 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L 0.6 nozzle | first difference at byte 2106 (line 106, column 7; expected 75897 bytes, actual 75886 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P89 R3"; actual "M73 P88 R3"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L 0.8 nozzle | first difference at byte 2106 (line 106, column 7; expected 51843 bytes, actual 51842 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.4 nozzle | first difference at byte 7439 (line 340, column 6; expected 95301 bytes, actual 95609 bytes) context:   line 339: ";WIDTH:0.45"   line 340: expected "G1 F1912"; actual "G1 F1899"   line 341: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.5 nozzle | first difference at byte 514 (line 21, column 9; expected 89238 bytes, actual 89697 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.6 nozzle | first difference at byte 2107 (line 106, column 7; expected 67236 bytes, actual 67235 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P91 R2"; actual "M73 P90 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.8 nozzle | first difference at byte 2106 (line 106, column 7; expected 51902 bytes, actual 51901 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| PASS | Prusa/Prusa MINI 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MINI 0.4 nozzle | first difference at byte 3563 (line 162, column 1; expected 102434 bytes, actual 102841 bytes) context:   line 161: "G1 X86.485 Y93.019 E.02231"   line 162: expected "M73 P40 R7"; actual "G1 X87.178 Y93.712 E.03514"   line 163: expected "G1 X87.178 Y93.712 E.03514"; actual "M73 P40 R7" |
| DIVERGENT | Prusa/Prusa MINI 0.6 nozzle | first difference at byte 21606 (line 1187, column 2; expected 89826 bytes, actual 89826 bytes) context:   line 1186: "G1 E3.2 F1800"   line 1187: expected "M73 P57 R4"; actual "M205 X8 Y8"   line 1188: expected "M205 X8 Y8"; actual ";TYPE:Inner wall" |
| PASS | Prusa/Prusa MINI 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MINIIS 0.25 nozzle | first difference at byte 21777 (line 975, column 1; expected 215474 bytes, actual 215474 bytes) context:   line 974: "G1 X89.28 Y94.131 E.00421"   line 975: expected "M73 P29 R13"; actual "G1 X94.131 Y89.28 E.0836"   line 976: expected "G1 X94.131 Y89.28 E.0836"; actual "M73 P29 R13" |
| DIVERGENT | Prusa/Prusa MINIIS 0.4 nozzle | first difference at byte 2782 (line 133, column 1; expected 107296 bytes, actual 107307 bytes) context:   line 132: "G1 X92.051 Y86.75 E.06609"   line 133: expected "M73 P35 R8"; actual "G1 X91.388 Y86.75 E.02581"   line 134: expected "G1 X91.388 Y86.75 E.02581"; actual "M73 P35 R8" |
| DIVERGENT | Prusa/Prusa MINIIS 0.6 nozzle | first difference at byte 12080 (line 617, column 1; expected 78011 bytes, actual 78246 bytes) context:   line 616: "G1 X90.961 Y86.584 E.1302"   line 617: expected "M73 P49 R5"; actual "; stop printing object cube10.stl id:0 copy 0"   line 618: expected "; stop printing object cube10.stl id:0 copy 0"; actual ";LAYER_CHANGE" |
| PASS | Prusa/Prusa MINIIS 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3.5 0.4 nozzle | first difference at byte 4896 (line 228, column 1; expected 111095 bytes, actual 111095 bytes) context:   line 227: "G1 X129.775 Y100.225 E.32326"   line 228: expected "M73 P72 R8"; actual "G1 X129.775 Y109.735 E.3219"   line 229: expected "G1 X129.775 Y109.735 E.3219"; actual "M73 P72 R8" |
| PASS | Prusa/Prusa MK3.5 0.6 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.4 nozzle | first difference at byte 55392 (line 2695, column 1; expected 95415 bytes, actual 95415 bytes) context:   line 2694: "G1 X129.368 Y109.328 E.28846"   line 2695: expected "M73 P85 R1"; actual "; stop printing object cube10.stl id:0 copy 0"   line 2696: expected "; stop printing object cube10.stl id:0 copy 0"; actual ";LAYER_CHANGE" |
| PASS | Prusa/Prusa MK3S 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.8 nozzle | first difference at byte 3732 (line 178, column 1; expected 64861 bytes, actual 64861 bytes) context:   line 177: "G1 X120 Y97.582 E.10208"   line 178: expected "G1 X130 Y97.582 E1.02139"; actual "M73 P6 R4"   line 179: expected "M73 P6 R4"; actual "G1 X130 Y97.582 E1.02139" |
| PASS | Prusa/Prusa MK4 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4 0.4 nozzle | first difference at byte 7823 (line 368, column 7; expected 110151 bytes, actual 110799 bytes) context:   line 367: ";WIDTH:0.44"   line 368: expected "G1 F1856"; actual "G1 F1861"   line 369: "G1 X121.017 Y108.983 E.25775" |
| PASS | Prusa/Prusa MK4 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4 0.8 nozzle | first difference at byte 4186 (line 220, column 6; expected 66465 bytes, actual 67355 bytes) context:   line 219: ";WIDTH:0.88"   line 220: expected "G1 F978"; actual "G1 F982"   line 221: "G1 X122.034 Y107.966 E.76772" |
| DIVERGENT | Prusa/Prusa MK4S 0.25 nozzle | first difference at byte 3440 (line 169, column 1; expected 214106 bytes, actual 214246 bytes) context:   line 168: "G1 X126.728 Y100.965 E.00929"   line 169: expected "M73 P72 R9"; actual "G1 X129.035 Y103.272 E.07624"   line 170: expected "G1 X129.035 Y103.272 E.07624"; actual "M73 P72 R9" |
| PASS | Prusa/Prusa MK4S 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S 0.4 nozzle | first difference at byte 4002 (line 188, column 1; expected 92187 bytes, actual 92830 bytes) context:   line 187: "G1 X124.443 Y108.707 E.17184"   line 188: expected "M73 P84 R5"; actual "G1 X123.787 Y108.707 E.0253"   line 189: expected "G1 X123.787 Y108.707 E.0253"; actual "M73 P84 R5" |
| PASS | Prusa/Prusa MK4S 0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S HF0.4 nozzle | first difference at byte 7240 (line 326, column 7; expected 92234 bytes, actual 92234 bytes) context:   line 325: ";WIDTH:0.45"   line 326: expected "G1 F2560"; actual "G1 F2544"   line 327: "G1 X120.675 Y109.325 E.29279" |
| PASS | Prusa/Prusa MK4S HF0.5 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S HF0.6 nozzle | first difference at byte 2804 (line 140, column 1; expected 67822 bytes, actual 69103 bytes) context:   line 139: "G1 X129.46 Y100.54 E.47252"   line 140: expected "M73 P86 R4"; actual "G1 X129.46 Y109.4 E.46934"   line 141: expected "G1 X129.46 Y109.4 E.46934"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa MK4S HF0.8 nozzle |  |
| PASS | Prusa/Prusa XL 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.3 nozzle | first difference at byte 2216 (line 81, column 1; expected 125488 bytes, actual 125536 bytes) context:   line 80: "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"   line 81: expected "M73 P77 R9"; actual "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"   line 82: expected "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"; actual "G92 E0 ; reset extruder position" |
| DIVERGENT | Prusa/Prusa XL 0.4 nozzle | first difference at byte 3176 (line 135, column 1; expected 93107 bytes, actual 93155 bytes) context:   line 134: "G1 E.8 F1800"   line 135: expected "M73 P83 R6"; actual ";TYPE:Bottom surface"   line 136: expected ";TYPE:Bottom surface"; actual ";WIDTH:0.50675" |
| DIVERGENT | Prusa/Prusa XL 0.5 nozzle | first difference at byte 7476 (line 323, column 8; expected 76300 bytes, actual 76498 bytes) context:   line 322: ";WIDTH:0.55"   line 323: expected "G1 F1225"; actual "G1 F1227"   line 324: "G1 X175.825 Y184.175 E.43077" |
| PASS | Prusa/Prusa XL 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.8 nozzle | first difference at byte 4422 (line 198, column 22; expected 53778 bytes, actual 53824 bytes) context:   line 197: ";WIPE_END"   line 198: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 199: expected "G1 E.6 F1800"; actual "G1 Z.6" |
| PASS | Prusa/Prusa XL 5T 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 5T 0.3 nozzle | first difference at byte 7601 (line 353, column 1; expected 136093 bytes, actual 136141 bytes) context:   line 352: "G1 X183.94 Y176.494 E.01038"   line 353: expected "M73 P78 R8"; actual "G1 X176.494 Y183.94 E.26012"   line 354: expected "G1 X176.494 Y183.94 E.26012"; actual "G1 X176.074 Y183.94 E.01038" |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-Q1ym2c") |
| DIVERGENT | Prusa/Prusa XL 5T 0.5 nozzle | first difference at byte 44317 (line 2223, column 8; expected 86730 bytes, actual 87082 bytes) context:   line 2222: ";WIDTH:0.55"   line 2223: expected "G1 F1222"; actual "G1 F1224"   line 2224: "M106 S255" |
| DIVERGENT | Prusa/Prusa XL 5T 0.6 nozzle | first difference at byte 2568 (line 125, column 1; expected 81138 bytes, actual 81336 bytes) context:   line 124: "G0 X70 E9 F800 ; continue purging and wipe the nozzle"   line 125: expected "M73 P87 R4"; actual "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"   line 126: expected "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"; actual "M73 P87 R4" |
| DIVERGENT | Prusa/Prusa XL 5T 0.8 nozzle | first difference at byte 4856 (line 248, column 22; expected 64354 bytes, actual 64400 bytes) context:   line 247: ";WIPE_END"   line 248: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 249: expected "G1 E.8 F1800"; actual "G1 Z.6" |
| PASS | Qidi/Qidi Q1 Pro 0.2 nozzle |  |
| DIVERGENT | Qidi/Qidi Q1 Pro 0.4 nozzle | first difference at byte 3370 (line 147, column 1; expected 122691 bytes, actual 123544 bytes) context:   line 146: "G1 X118.7 Y125.079 E.06683"   line 147: expected "M73 P6 R5"; actual "G1 X118.7 Y125.75 E.02598"   line 148: expected "G1 X118.7 Y125.75 E.02598"; actual "M73 P6 R5" |
| PASS | Qidi/Qidi Q1 Pro 0.6 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.8 nozzle |  |
| DIVERGENT | Qidi/Qidi Q2 0.2 nozzle | first difference at byte 2920 (line 123, column 1; expected 489412 bytes, actual 489604 bytes) context:   line 122: "G1 X138.971 Y131.494 Z.5"   line 123: expected "M73 P5 R23"; actual "G1 Z.1"   line 124: expected "G1 Z.1"; actual "M73 P5 R23" |
| PASS | Qidi/Qidi Q2 0.4 nozzle |  |
| PASS | Qidi/Qidi Q2 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2 0.8 nozzle |  |
| DIVERGENT | Qidi/Qidi Q2C 0.2 nozzle | first difference at byte 6603 (line 265, column 7; expected 480104 bytes, actual 480272 bytes) context:   line 264: ";WIDTH:0.22"   line 265: expected "G1 F3682"; actual "G1 F3667"   line 266: "G1 X130.731 Y139.269 E.04323" |
| PASS | Qidi/Qidi Q2C 0.4 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.8 nozzle |  |
| PASS | Qidi/Qidi X-CF Pro 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 0.4 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Max 3 0.2 nozzle | first difference at byte 23738 (line 930, column 1; expected 299355 bytes, actual 299357 bytes) context:   line 929: "G1 X166.626 Y163.985 E.00227"   line 930: expected "M73 P28 R13"; actual "G1 X161.015 Y158.374 E.06418"   line 931: expected "G1 X161.015 Y158.374 E.06418"; actual "G1 X160.734 Y158.374 E.00227" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.4 nozzle | first difference at byte 6582 (line 282, column 1; expected 120893 bytes, actual 121138 bytes) context:   line 281: "G1 X158.435 Y163.124 E.01639"   line 282: expected "M73 P48 R5"; actual "G1 X163.124 Y158.435 E.20376"   line 283: expected "G1 X163.124 Y158.435 E.20376"; actual "G1 X162.591 Y158.435 E.01639" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.6 nozzle | first difference at byte 3973 (line 173, column 1; expected 75688 bytes, actual 75690 bytes) context:   line 172: "G1 Z.6"   line 173: expected "M73 P58 R3"; actual "G1 E1.4 F1800"   line 174: expected "G1 E1.4 F1800"; actual "M73 P58 R3" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.8 nozzle | first difference at byte 2982 (line 134, column 1; expected 69152 bytes, actual 69154 bytes) context:   line 133: "G1 X161.32 Y165.655 E.34889"   line 134: expected "M73 P63 R2"; actual "G1 X160.236 Y165.655 E.13536"   line 135: expected "G1 X160.236 Y165.655 E.13536"; actual "M73 P63 R2" |
| DIVERGENT | Qidi/Qidi X-Max 4 0.2 nozzle | first difference at byte 1421 (line 73, column 1; expected 311355 bytes, actual 311359 bytes) context:   line 72: "G1 X145 F5000"   line 73: expected "G1 X175 F6000"; actual "M73 P6 R15"   line 74: expected "G1 X163"; actual "G1 X175 F6000" |
| DIVERGENT | Qidi/Qidi X-Max 4 0.4 nozzle | first difference at byte 1536 (line 84, column 2; expected 127426 bytes, actual 129250 bytes) context:   line 83: "G1 Z-0.2 F480"   line 84: expected "M73 P11 R9"; actual "M106 S255"   line 85: expected "M106 S255"; actual "M109.1 S150" |
| PASS | Qidi/Qidi X-Max 4 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 0.4 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Plus 3 0.2 nozzle | first difference at byte 6345 (line 252, column 7; expected 299297 bytes, actual 299323 bytes) context:   line 251: ";WIDTH:0.22"   line 252: expected "G1 F3843"; actual "G1 F3824"   line 253: "G1 X135.706 Y144.294 E.06948" |
| PASS | Qidi/Qidi X-Plus 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.8 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Smart 3 0.2 nozzle | first difference at byte 20892 (line 866, column 8; expected 287656 bytes, actual 287721 bytes) context:   line 865: ";WIDTH:0.22"   line 866: expected "G1 F3855"; actual "G1 F3857"   line 867: "G1 X83.206 Y94.294 E.06948" |
| PASS | Qidi/Qidi X-Smart 3 0.4 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Smart 3 0.6 nozzle | first difference at byte 6287 (line 277, column 8; expected 73788 bytes, actual 74032 bytes) context:   line 276: ";WIDTH:0.62"   line 277: expected "G1 F1431"; actual "G1 F1436"   line 278: "G1 X83.366 Y94.134 E.56156" |
| PASS | Qidi/Qidi X-Smart 3 0.8 nozzle |  |
| DIVERGENT | RH3D/E3NG v1.2S - 0.2 nozzle | first difference at byte 13777 (line 523, column 11; expected 291096 bytes, actual 291096 bytes) context:   line 522: "G1 X121 Y105.338 E.10775"   line 523: expected "G1 X123.081 Y105.734 E.02283"; actual "G1 X123.082 Y105.734 E.02283"   line 524: "G1 X124.872 Y106.868 E.02283" |
| PASS | RH3D/E3NG v1.2S - 0.3 nozzle |  |
| PASS | RH3D/E3NG v1.2S - 0.4 nozzle |  |
| DIVERGENT | RH3D/E3NG v1.2S - 0.5 nozzle | first difference at byte 6247 (line 241, column 1; expected 111046 bytes, actual 111310 bytes) context:   line 240: "G1 X107.113 Y109.203 E.07137"   line 241: expected "M73 P4 R5"; actual "G1 X107.911 Y108.153 E.05116"   line 242: expected "G1 X107.911 Y108.153 E.05116"; actual "M73 P4 R5" |
| PASS | RH3D/E3NG v1.2S - 0.6 nozzle |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Dual) |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Left) |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Right) |  |
| DIVERGENT | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Dual) | first difference at byte 6199 (line 285, column 1; expected 109796 bytes, actual 110522 bytes) context:   line 284: "G1 X172.575 Y153.658 E.04572"   line 285: expected "M73 P18 R5"; actual "G1 X173.372 Y153.319 F9000"   line 286: expected "G1 X173.372 Y153.319 F9000"; actual "M73 P18 R5" |
| PASS | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Left) |  |
| DIVERGENT | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Right) | first difference at byte 1626 (line 87, column 1; expected 109494 bytes, actual 110740 bytes) context:   line 86: "G1 X176.615 Y141.891 E.06819"   line 87: expected "M73 P9 R5"; actual "G1 X177.866 Y142.985 E.06819"   line 88: expected "G1 X177.866 Y142.985 E.06819"; actual "M73 P9 R5" |
| PASS | Ratrig/RatRig V-Cast 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Cast 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 3 200 0.4 nozzle | first difference at byte 33025 (line 1441, column 1; expected 113832 bytes, actual 113971 bytes) context:   line 1440: "G1 E-.7 F2400"   line 1441: expected "M73 P37 R1"; actual ";WIPE_START"   line 1442: expected ";WIPE_START"; actual "G1 F7200" |
| PASS | Ratrig/RatRig V-Core 3 300 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 3 400 0.4 nozzle | first difference at byte 4201 (line 165, column 6; expected 115504 bytes, actual 115902 bytes) context:   line 164: ";WIDTH:0.4"   line 165: expected "G1 F5786"; actual "G1 F5800"   line 166: "G1 X195.957 Y204.043 E.23528" |
| PASS | Ratrig/RatRig V-Core 3 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 300 0.5 nozzle | first difference at byte 28345 (line 1150, column 1; expected 108126 bytes, actual 108386 bytes) context:   line 1149: "G1 X153.282 Y149.378 E.11015"   line 1150: expected "M73 P32 R1"; actual "G1 X153.282 Y147.708 E.12632"   line 1151: expected "G1 X153.282 Y147.708 E.12632"; actual "M73 P32 R1" |
| PASS | Ratrig/RatRig V-Core 4 300 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 300 0.8 nozzle | Ratrig/RatRig V-Core 4 300 0.8 nozzle process: no compatible preset |
| DIVERGENT | Ratrig/RatRig V-Core 4 400 0.4 nozzle | first difference at byte 6003 (line 230, column 7; expected 113960 bytes, actual 114195 bytes) context:   line 229: ";WIDTH:0.45"   line 230: expected "G1 F4961"; actual "G1 F4971"   line 231: "G1 X196.082 Y203.918 E.25993" |
| PASS | Ratrig/RatRig V-Core 4 400 0.5 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 400 0.6 nozzle | first difference at byte 8381 (line 332, column 1; expected 88683 bytes, actual 89201 bytes) context:   line 331: "G1 X204.025 Y203.965 E.48534"   line 332: expected "M73 P10 R2"; actual "SET_VELOCITY_LIMIT ACCEL=4000 ACCEL_TO_DECEL=2000"   line 333: expected "SET_VELOCITY_LIMIT ACCEL=4000 ACCEL_TO_DECEL=2000"; actual "G1 X204.675 Y204.675 F30000" |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 400 0.8 nozzle | Ratrig/RatRig V-Core 4 400 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.5 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 500 0.6 nozzle | first difference at byte 2955 (line 112, column 1; expected 88605 bytes, actual 89313 bytes) context:   line 111: "G1 X236.163 Y239.519 E.18689"   line 112: expected "M73 P4 R2"; actual "G1 X237.783 Y237.573 E.16444"   line 113: expected "G1 X237.783 Y237.573 E.16444"; actual "M73 P4 R2" |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 500 0.8 nozzle | Ratrig/RatRig V-Core 4 500 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 300 0.5 nozzle | first difference at byte 8301 (line 326, column 7; expected 109449 bytes, actual 109617 bytes) context:   line 325: ";WIDTH:0.55"   line 326: expected "G1 F4073"; actual "G1 F4084"   line 327: "G1 X146.302 Y153.698 E.3056" |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 300 0.6 nozzle | first difference at byte 5566 (line 218, column 7; expected 102166 bytes, actual 102863 bytes) context:   line 217: ";WIDTH:0.6"   line 218: expected "G1 F3671"; actual "G1 F3683"   line 219: "G1 X146.437 Y153.563 E.32348" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.8 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 400 0.4 nozzle | first difference at byte 6044 (line 231, column 8; expected 115215 bytes, actual 115437 bytes) context:   line 230: ";WIDTH:0.45"   line 231: expected "G1 F4963"; actual "G1 F4967"   line 232: "G1 X196.082 Y203.918 E.25993" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 400 0.8 nozzle | first difference at byte 3050 (line 126, column 7; expected 82259 bytes, actual 83126 bytes) context:   line 125: ";WIDTH:0.75"   line 126: expected "G1 F2205"; actual "G1 F2222"   line 127: "G1 X196.771 Y203.229 E.46736" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 500 0.5 nozzle | first difference at byte 10104 (line 396, column 1; expected 109450 bytes, actual 109618 bytes) context:   line 395: "G1 F4084"   line 396: expected "M73 P9 R2"; actual "G1 X246.785 Y253.126 E.00279"   line 397: expected "G1 X246.785 Y253.126 E.00279"; actual "G1 X246.752 Y253.183 E.00279" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 500 0.8 nozzle | first difference at byte 62871 (line 2488, column 61; expected 83126 bytes, actual 83126 bytes) context:   line 2487: "; estimated printing time (normal mode) = 2m 48s"   line 2488: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2489: "" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 0.4 nozzle | first difference at byte 6252 (line 236, column 7; expected 118801 bytes, actual 119592 bytes) context:   line 235: ";WIDTH:0.45"   line 236: expected "G1 F4976"; actual "G1 F4987"   line 237: "G1 X146.082 Y153.918 E.25993" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 0.8 nozzle | first difference at byte 3125 (line 127, column 7; expected 83615 bytes, actual 84753 bytes) context:   line 126: ";WIDTH:0.75"   line 127: expected "G1 F2209"; actual "G1 F2226"   line 128: "G1 X146.771 Y153.229 E.46736" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.8 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 0.4 nozzle | first difference at byte 6253 (line 236, column 8; expected 119023 bytes, actual 119592 bytes) context:   line 235: ";WIDTH:0.45"   line 236: expected "G1 F4981"; actual "G1 F4987"   line 237: "G1 X196.082 Y203.918 E.25993" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.8 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.4 nozzle | first difference at byte 7956 (line 298, column 1; expected 113285 bytes, actual 113827 bytes) context:   line 297: "G1 X196.416 Y199.175 E.01949"   line 298: expected "M73 P7 R3"; actual "G1 X199.175 Y196.416 E.13208"   line 299: expected "G1 X199.175 Y196.416 E.13208"; actual "M73 P7 R3" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.5 nozzle | first difference at byte 5367 (line 210, column 7; expected 105668 bytes, actual 106172 bytes) context:   line 209: ";WIDTH:0.55"   line 210: expected "G1 F3028"; actual "G1 F3036"   line 211: "G1 X181.302 Y203.698 E.31184" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.6 nozzle | first difference at byte 18638 (line 744, column 1; expected 98929 bytes, actual 99098 bytes) context:   line 743: "G1 X189.71 Y204.71 F30000"   line 744: expected "M73 P21 R2"; actual ";TYPE:Outer wall"   line 745: expected ";TYPE:Outer wall"; actual ";WIDTH:0.58" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.8 nozzle | first difference at byte 13892 (line 557, column 1; expected 84520 bytes, actual 84819 bytes) context:   line 556: "G1 X189.65 Y195.35 E.62478"   line 557: expected "M73 P20 R2"; actual "G1 X189.65 Y204.57 E.6194"   line 558: expected "G1 X189.65 Y204.57 E.6194"; actual "M73 P20 R2" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.8 nozzle | first difference at byte 63890 (line 2567, column 61; expected 84764 bytes, actual 84764 bytes) context:   line 2566: "; estimated printing time (normal mode) = 2m 48s"   line 2567: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2568: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.8 nozzle | first difference at byte 63904 (line 2569, column 61; expected 84822 bytes, actual 84822 bytes) context:   line 2568: "; estimated printing time (normal mode) = 2m 48s"   line 2569: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2570: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Minion 0.4 nozzle |  |
| DIVERGENT | RolohaunDesign/Rolohaun Delta Flyer Refit 0.4 nozzle | first difference at byte 6910 (line 261, column 7; expected 125621 bytes, actual 126094 bytes) context:   line 260: ";WIDTH:0.44"   line 261: expected "G1 F3779"; actual "G1 F3790"   line 262: "G1 X-1.586 Y6.38 E.25775" |
| PASS | RolohaunDesign/Rook MK1 LDO 0.2 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.4 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.6 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.8 nozzle |  |
| DIVERGENT | SecKit/SecKit Go3 0.4 nozzle | first difference at byte 39422 (line 1726, column 1; expected 110732 bytes, actual 111003 bytes) context:   line 1725: "G1 X147.273 Y148.591 E.01975"   line 1726: expected "M73 P46 R3"; actual "G1 X149.091 Y147.774 E.05563"   line 1727: expected "G1 X149.091 Y147.774 E.05563"; actual "M73 P46 R3" |
| PASS | SecKit/SecKit SK-Tank 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.4 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC Artemis 0.5 nozzle | first difference at byte 1498 (line 47, column 9; expected 112314 bytes, actual 112668 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R5"; actual "M73 P2 R6"   line 48: "G3 X50 Y-129.9 R139.2 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.5 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.7 nozzle | first difference at byte 3053 (line 133, column 1; expected 76804 bytes, actual 77500 bytes) context:   line 132: "G1 E-3.5 F2700"   line 133: expected "M73 P11 R3"; actual ";WIPE_START"   line 134: expected ";WIPE_START"; actual "G1 F1200" |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.5 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.7 nozzle | first difference at byte 5085 (line 261, column 1; expected 77079 bytes, actual 77540 bytes) context:   line 260: "G1 X-4.65 Y-4.65 E.84565"   line 261: expected "M73 P15 R3"; actual "G1 X4.65 Y-4.65 E.84565"   line 262: expected "G1 X4.65 Y-4.65 E.84565"; actual "M73 P15 R3" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 1.0 nozzle | first difference at byte 6613 (line 345, column 5; expected 60306 bytes, actual 60468 bytes) context:   line 344: "M205 X6 Y6"   line 345: expected "G1 X.155 Y-1.706 F6000"; actual "G1 X-1.653 Y1.612 F6000"   line 346: "M205 X5 Y5" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.4 nozzle | first difference at byte 8500 (line 422, column 1; expected 149768 bytes, actual 150720 bytes) context:   line 421: "G1 X4.6 Y4.454 F9000"   line 422: expected "M73 P11 R7"; actual ";TYPE:Outer wall"   line 423: expected ";TYPE:Outer wall"; actual "G1 F1200" |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.5 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.7 nozzle | first difference at byte 6136 (line 315, column 1; expected 77348 bytes, actual 77540 bytes) context:   line 314: "G1 F1200"   line 315: expected "M73 P17 R3"; actual "G1 X-2.736 Y-2.765 E.00726"   line 316: expected "G1 X-2.736 Y-2.765 E.00726"; actual "G1 X-2.776 Y-2.697 E.00726" |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 1.0 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.4 nozzle | first difference at byte 7416 (line 363, column 1; expected 150535 bytes, actual 150698 bytes) context:   line 362: "G1 X-3.937 Y-2.642 E.04992"   line 363: expected "; stop printing object cube10.stl id:0 copy 0"; actual "M205 X10 Y10"   line 364: expected ";LAYER_CHANGE"; actual "G1 X-3.661 Y-3.723 F9000" |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.7 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v3.2 1.0 nozzle | first difference at byte 4080 (line 197, column 2; expected 60200 bytes, actual 60470 bytes) context:   line 196: "G1 E5 F2100"   line 197: expected "M73 P19 R2"; actual "M205 X3 Y3"   line 198: expected "M205 X3 Y3"; actual "G1 F750" |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 1.0 nozzle |  |
| PASS | Snapmaker/Snapmaker A250 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 BKit (0.2 nozzle) | first difference at byte 36710 (line 1607, column 1; expected 186177 bytes, actual 186302 bytes) context:   line 1606: "G1 Z2.14 F7200"   line 1607: expected "M73 P22 R10"; actual "G1 X119.129 Y122.776 Z2.14"   line 1608: expected "G1 X119.129 Y122.776 Z2.14"; actual "M73 P22 R10" |
| PASS | Snapmaker/Snapmaker A250 BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 Dual (0.2 nozzle) | first difference at byte 5665 (line 272, column 1; expected 193337 bytes, actual 194111 bytes) context:   line 271: "G1 X111.871 Y128.956 E.00584"   line 272: expected "M73 P8 R12"; actual "G1 X111.044 Y128.129 E.02312"   line 273: expected "G1 X111.044 Y128.129 E.02312"; actual "M73 P8 R12" |
| PASS | Snapmaker/Snapmaker A250 Dual (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 Dual (0.6 nozzle) | first difference at byte 1343 (line 81, column 9; expected 99036 bytes, actual 99335 bytes) context:   line 80: "G0 Y0 F3420.0"   line 81: expected "M73 P4 R7"; actual "M73 P4 R8"   line 82: "" |
| DIVERGENT | Snapmaker/Snapmaker A250 Dual (0.8 nozzle) | first difference at byte 3550 (line 212, column 5; expected 79607 bytes, actual 80231 bytes) context:   line 211: ";WIDTH:0.82"   line 212: expected "G1 F998"; actual "G1 F1001"   line 213: "G1 X111.23 Y128.77 E.56661" |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.6 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.8 nozzle) | first difference at byte 2184 (line 146, column 1; expected 79637 bytes, actual 80368 bytes) context:   line 145: "G1 Z.7 F5760"   line 146: expected "M73 P13 R5"; actual "G1 X119.49 Y129.49 Z.7"   line 147: expected "G1 X119.49 Y129.49 Z.7"; actual "M73 P13 R5" |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 QS+B Kit (0.2 nozzle) | first difference at byte 36714 (line 1607, column 1; expected 186232 bytes, actual 186357 bytes) context:   line 1606: "G1 Z2.14 F7200"   line 1607: expected "M73 P22 R10"; actual "G1 X119.129 Y122.776 Z2.14"   line 1608: expected "G1 X119.129 Y122.776 Z2.14"; actual "M73 P22 R10" |
| DIVERGENT | Snapmaker/Snapmaker A250 QS+B Kit (0.4 nozzle) | first difference at byte 9284 (line 441, column 8; expected 117717 bytes, actual 117844 bytes) context:   line 440: ";WIDTH:0.45"   line 441: expected "G1 F1940"; actual "G1 F1943"   line 442: "G1 X111.061 Y128.939 E.21349" |
| DIVERGENT | Snapmaker/Snapmaker A250 QS+B Kit (0.6 nozzle) | first difference at byte 3657 (line 198, column 7; expected 92337 bytes, actual 92891 bytes) context:   line 197: ";WIDTH:0.62"   line 198: expected "G1 F1339"; actual "G1 F1344"   line 199: "G1 X110.93 Y129.07 E.34707" |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 QSKit (0.4 nozzle) | first difference at byte 9281 (line 441, column 8; expected 117666 bytes, actual 117793 bytes) context:   line 440: ";WIDTH:0.45"   line 441: expected "G1 F1940"; actual "G1 F1943"   line 442: "G1 X111.061 Y128.939 E.21349" |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.6 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 QSKit (0.8 nozzle) | first difference at byte 2911 (line 168, column 1; expected 73209 bytes, actual 73653 bytes) context:   line 167: "G1 X111.897 Y126.181 E.11899"   line 168: expected "M73 P14 R5"; actual "G1 X113.819 Y128.103 E.30197"   line 169: expected "G1 X113.819 Y128.103 E.30197"; actual "M73 P14 R5" |
| PASS | Snapmaker/Snapmaker A350 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 BKit (0.4 nozzle) | first difference at byte 17404 (line 826, column 1; expected 117636 bytes, actual 117763 bytes) context:   line 825: "G1 X164.355 Y170.645 E.23602"   line 826: expected "M73 P19 R8"; actual "G1 X164.355 Y179.295 E.23439"   line 827: expected "G1 X164.355 Y179.295 E.23439"; actual "M73 P19 R8" |
| PASS | Snapmaker/Snapmaker A350 BKit (0.6 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 BKit (0.8 nozzle) | first difference at byte 3430 (line 194, column 7; expected 73407 bytes, actual 73575 bytes) context:   line 193: ";WIDTH:0.82"   line 194: expected "G1 F1008"; actual "G1 F1011"   line 195: "G1 X156.23 Y178.77 E.56661" |
| PASS | Snapmaker/Snapmaker A350 Dual (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual (0.4 nozzle) | first difference at byte 4014 (line 228, column 8; expected 124123 bytes, actual 124379 bytes) context:   line 227: ";WIDTH:0.45"   line 228: expected "G1 F1933"; actual "G1 F1936"   line 229: "G1 X156.061 Y178.939 E.21349" |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual (0.6 nozzle) | first difference at byte 1529 (line 103, column 1; expected 99020 bytes, actual 99287 bytes) context:   line 102: "G1 E3 F200"   line 103: expected "M73 P9 R7"; actual "G92 E0"   line 104: expected "G92 E0"; actual "G1 X0 E6.23628 F3420.0" |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual (0.8 nozzle) | first difference at byte 5580 (line 308, column 7; expected 79620 bytes, actual 80180 bytes) context:   line 307: ";WIDTH:0.82"   line 308: expected "G1 F1006"; actual "G1 F1019"   line 309: "G1 X156.23 Y178.77 E.56661" |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.6 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual BKit (0.8 nozzle) | first difference at byte 3116 (line 189, column 1; expected 79955 bytes, actual 80235 bytes) context:   line 188: "G1 X158.819 Y178.103 E.30197"   line 189: expected "M73 P16 R5"; actual "G1 X157.748 Y178.103 E.11899"   line 190: expected "G1 X157.748 Y178.103 E.11899"; actual "M73 P16 R5" |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.6 nozzle) | first difference at byte 7481 (line 386, column 1; expected 98859 bytes, actual 99413 bytes) context:   line 385: "G1 X156.424 Y176.28 E.03506"   line 386: expected "M73 P16 R7"; actual "G1 X158.72 Y178.576 E.13846"   line 387: expected "G1 X158.72 Y178.576 E.13846"; actual "M73 P16 R7" |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 QS+B Kit (0.4 nozzle) | first difference at byte 3873 (line 208, column 8; expected 117691 bytes, actual 117818 bytes) context:   line 207: ";WIDTH:0.45"   line 208: expected "G1 F1940"; actual "G1 F1943"   line 209: "G1 X156.061 Y178.939 E.21349" |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 QSKit (0.2 nozzle) | first difference at byte 3472 (line 182, column 1; expected 185912 bytes, actual 186287 bytes) context:   line 181: "G1 X159.292 Y171.044 E.00603"   line 182: expected "M73 P7 R12"; actual "G1 X163.956 Y175.708 E.13463"   line 183: expected "G1 X163.956 Y175.708 E.13463"; actual "G1 X163.956 Y176.004 E.00603" |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.6 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 QSKit (0.8 nozzle) | first difference at byte 3641 (line 205, column 1; expected 73131 bytes, actual 73578 bytes) context:   line 204: "G1 X161.77 Y178.847 Z.94"   line 205: expected "M73 P16 R5"; actual "G1 X163.847 Y178.847"   line 206: expected "G1 X163.847 Y178.847"; actual "M73 P16 R5" |
| DIVERGENT | Snapmaker/Snapmaker Artisan (0.2 nozzle) | first difference at byte 13345 (line 589, column 1; expected 188207 bytes, actual 188207 bytes) context:   line 588: "G1 X204.129 Y204.114 E.00282"   line 589: expected "G1 X195.871 Y195.871 E.12255"; actual "M73 P12 R11"   line 590: expected "M73 P12 R11"; actual "G1 X195.871 Y195.871 E.12255" |
| DIVERGENT | Snapmaker/Snapmaker Artisan (0.4 nozzle) | first difference at byte 6846 (line 349, column 7; expected 126579 bytes, actual 126689 bytes) context:   line 348: ";WIDTH:0.45"   line 349: expected "G1 F1938"; actual "G1 F1942"   line 350: "G1 X196.061 Y203.939 E.21349" |
| PASS | Snapmaker/Snapmaker Artisan (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.8 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.2 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-qJqLtV") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-1YjjEw") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4+0.6 nozzle) | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-WTzWA9") |
| PASS | Snapmaker/Snapmaker U1 (0.6 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.8 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-YKDRoo") |
| DIVERGENT | Sovol/Sovol SV01 0.4 nozzle | first difference at byte 6782 (line 294, column 9; expected 98420 bytes, actual 98814 bytes) context:   line 293: "G1 X142.286 Y123.518 E.05765"   line 294: expected "G1 X143.817 Y121.706 F9000"; actual "G1 X143.191 Y123.13 F9000"   line 295: expected "G1 F2400"; actual ";WIDTH:0.40039" |
| DIVERGENT | Sovol/Sovol SV01 Pro 0.4 nozzle | first difference at byte 9317 (line 402, column 1; expected 98709 bytes, actual 98832 bytes) context:   line 401: "G1 X143.638 Y119.239 E.01817"   line 402: expected "M73 P20 R5"; actual "G1 X140.761 Y116.362 E.13165"   line 403: expected "G1 X140.761 Y116.362 E.13165"; actual "M73 P20 R5" |
| PASS | Sovol/Sovol SV02 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV05 0.4 nozzle | first difference at byte 1296 (line 58, column 7; expected 98613 bytes, actual 98870 bytes) context:   line 57: "G1 E-2 F3600"   line 58: expected "M73 P11 R6"; actual "M73 P10 R6"   line 59: ";_SET_FAN_SPEED_CHANGING_LAYER" |
| DIVERGENT | Sovol/Sovol SV06 0.4 High-Speed nozzle | first difference at byte 1666 (line 90, column 19; expected 101958 bytes, actual 101958 bytes) context:   line 89: "G1 X117.495 Y97.735 E.05224"   line 90: expected "G1 X118.551 Y98.188 E.05224"; actual "G1 X118.551 Y98.189 E.05224"   line 91: "G1 X119.527 Y98.794 E.05224" |
| PASS | Sovol/Sovol SV06 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.2 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.6 nozzle |  |
| DIVERGENT | Sovol/Sovol SV06 ACE 0.8 nozzle | first difference at byte 3558 (line 162, column 8; expected 61531 bytes, actual 61729 bytes) context:   line 161: ";WIDTH:0.82"   line 162: expected "G1 F1441"; actual "G1 F1446"   line 163: "G1 X106.21 Y113.79 E.90694" |
| PASS | Sovol/Sovol SV06 Plus 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus ACE 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV07 0.4 nozzle | first difference at byte 4721 (line 204, column 1; expected 93336 bytes, actual 93542 bytes) context:   line 203: "G1 X115.78 Y118.616 E.02925"   line 204: expected "M73 P5 R5"; actual "G1 X114.973 Y118.702 E.04322"   line 205: expected "G1 X114.973 Y118.702 E.04322"; actual "M73 P5 R5" |
| DIVERGENT | Sovol/Sovol SV07 Plus 0.4 nozzle | first difference at byte 4032 (line 174, column 1; expected 106361 bytes, actual 107311 bytes) context:   line 173: "G1 X146.375 Y150.578 E5.51963"   line 174: expected "G1 X149.422 Y153.625 E5.67617"; actual "M106 S255"   line 175: expected "G1 X148.896 Y153.625 E5.69525"; actual "G1 X149.422 Y153.625 E5.67617" |
| PASS | Sovol/Sovol SV08 0.2 nozzle |  |
| PASS | Sovol/Sovol SV08 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV08 0.6 nozzle | first difference at byte 2007 (line 99, column 1; expected 81298 bytes, actual 81822 bytes) context:   line 98: "G1 X184.461 Y169.095 E.04549"   line 99: expected "M73 P13 R3"; actual "G1 X184.553 Y170 E.05953"   line 100: expected "G1 X184.553 Y170 E.05953"; actual "M73 P13 R3" |
| DIVERGENT | Sovol/Sovol SV08 0.8 nozzle | first difference at byte 43282 (line 1886, column 1; expected 70048 bytes, actual 70246 bytes) context:   line 1885: "G1 Z10 F36000"   line 1886: expected "M73 P93 R0"; actual "G1 X178.8 Y178.8 Z10"   line 1887: expected "G1 X178.8 Y178.8 Z10"; actual "M73 P93 R0" |
| PASS | Sovol/Sovol SV08 MAX 0.4 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.6 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.8 nozzle |  |
| DIVERGENT | Sovol/Sovol Zero 0.4 nozzle | first difference at byte 9131 (line 393, column 8; expected 123889 bytes, actual 124438 bytes) context:   line 392: ";WIDTH:0.4"   line 393: expected "G1 F3124"; actual "G1 F3127"   line 394: "G1 X71.8 Y80.6 E.25606" |
| PASS | Tiertime/Tiertime UP300 HS 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP310 Pro 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.6 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.8 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.6 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.8 nozzle |  |
| DIVERGENT | Tronxy/Tronxy X5SA 400 0.4 nozzle | first difference at byte 3486 (line 159, column 8; expected 94789 bytes, actual 94961 bytes) context:   line 158: ";WIDTH:0.45"   line 159: expected "G1 F4232"; actual "G1 F4238"   line 160: "G1 X195.625 Y204.375 E.29025" |
| DIVERGENT | TwoTrees/TwoTrees SK1 0.4 nozzle | first difference at byte 1414 (line 60, column 1; expected 101783 bytes, actual 102276 bytes) context:   line 59: "G1 X190 Y12 F6000 ;Wipe"   line 60: expected "M73 P66 R2"; actual "G1 X180 Y8 F6000 ;Wipe"   line 61: expected "G1 X180 Y8 F6000 ;Wipe"; actual "G1 X170 Y12 F6000 ;Wipe" |
| PASS | TwoTrees/TwoTrees SP-5 Klipper 0.4 nozzle |  |
| PASS | UltiMaker/UltiMaker 2 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 Klipper 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 RRF 0.4 nozzle |  |
| DIVERGENT | Volumic/EXO42 (0.4 nozzle) | first difference at byte 40140 (line 1760, column 1; expected 120293 bytes, actual 120292 bytes) context:   line 1759: "G1 X214.328 Y205.672"   line 1760: expected "M73 P38 R3"; actual "G1 X208.497 Y205.672"   line 1761: expected "G1 X208.497 Y205.672"; actual "M73 P38 R3" |
| PASS | Volumic/EXO42 IDRE (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO42 IDRE COPY MODE (0.4 nozzle) | first difference at byte 1049 (line 49, column 1; expected 102207 bytes, actual 102946 bytes) context:   line 48: "G1 X126.292 Y217.072 E.04787"   line 49: expected "M73 P2 R3"; actual "G1 X125.5 Y217.219 E.02929"   line 50: expected "G1 X125.5 Y217.219 E.02929"; actual "M73 P2 R3" |
| DIVERGENT | Volumic/EXO42 IDRE MIRROR MODE (0.4 nozzle) | first difference at byte 2766 (line 123, column 1; expected 102513 bytes, actual 102943 bytes) context:   line 122: "G1 X101.985 Y206.485 E.01138"   line 123: expected "M73 P4 R3"; actual "G1 X101.985 Y206.796 E.01139"   line 124: expected "G1 X101.985 Y206.796 E.01139"; actual "G1 X108.704 Y213.515 E.34766" |
| PASS | Volumic/EXO42 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Stage 2 (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO65 (0.6 nozzle) | first difference at byte 39837 (line 1917, column 1; expected 115024 bytes, actual 115034 bytes) context:   line 1916: "G1 X328.968 Y328.968"   line 1917: expected "M73 P41 R2"; actual "G1 X329.64 Y329.64"   line 1918: expected "G1 X329.64 Y329.64"; actual "M73 P41 R2" |
| DIVERGENT | Volumic/EXO65 IDRE (0.4 nozzle) | first difference at byte 2366 (line 109, column 1; expected 102457 bytes, actual 102973 bytes) context:   line 108: "G1 X312.53 Y321.485 E.01545"   line 109: expected "M73 P5 R3"; actual "G1 X311.907 Y321.485 E.00228"   line 110: expected "G1 X311.907 Y321.485 E.00228"; actual "M73 P5 R3" |
| DIVERGENT | Volumic/EXO65 IDRE COPY MODE (0.4 nozzle) | first difference at byte 1203 (line 55, column 1; expected 102095 bytes, actual 102917 bytes) context:   line 54: "G1 X170.781 Y320 E.36343"   line 55: expected "M73 P3 R3"; actual "G1 X171.172 Y318.742 E.04787"   line 56: expected "G1 X171.172 Y318.742 E.04787"; actual "M73 P3 R3" |
| PASS | Volumic/EXO65 IDRE MIRROR MODE (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO65 Performance (0.4 nozzle) | first difference at byte 1500 (line 70, column 1; expected 101614 bytes, actual 102651 bytes) context:   line 69: "G1 X328.843 Y321.157 E.27933"   line 70: expected "M73 P4 R3"; actual "G1 X328.843 Y328.803 E.27788"   line 71: expected "G1 X328.843 Y328.803 E.27788"; actual "M73 P4 R3" |
| PASS | Volumic/EXO65 Performance (0.6 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.8 nozzle) |  |
| DIVERGENT | Volumic/EXO65 Stage 2 (0.6 nozzle) | first difference at byte 3298 (line 153, column 7; expected 90224 bytes, actual 91004 bytes) context:   line 152: ";WIDTH:0.72"   line 153: expected "G1 F2700"; actual "G1 F2712"   line 154: "G1 X321.757 Y328.243 E.36515" |
| DIVERGENT | Volumic/SH65 (0.4 nozzle) | first difference at byte 1950 (line 98, column 1; expected 120082 bytes, actual 120290 bytes) context:   line 97: "G1 X329.76 Y145.24 E.26586"   line 98: expected "M73 P3 R5"; actual "G1 X329.76 Y154.72 E.26474"   line 99: expected "G1 X329.76 Y154.72 E.26474"; actual "M73 P3 R5" |
| PASS | Volumic/SH65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE MIRROR MODE (0.4 nozzle) |  |
| DIVERGENT | Volumic/SH65 Performance (0.4 nozzle) | first difference at byte 2918 (line 129, column 1; expected 102146 bytes, actual 102670 bytes) context:   line 128: "G1 X326.959 Y153.515 E.28326"   line 129: expected "M73 P5 R3"; actual "G1 X326.337 Y153.515 E.02277"   line 130: expected "G1 X326.337 Y153.515 E.02277"; actual "M73 P5 R3" |
| PASS | Volumic/SH65 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/VS20MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK3 (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30MK3 Stage 2 (0.4 nozzle) | first difference at byte 7654 (line 328, column 1; expected 101006 bytes, actual 101231 bytes) context:   line 327: "G1 X146.507 Y96.822 E.34288"   line 328: expected "M73 P6 R3"; actual "G1 X146.507 Y97.44 E.02246"   line 329: expected "G1 X146.507 Y97.44 E.02246"; actual "M73 P6 R3" |
| PASS | Volumic/VS30SC (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30SC2 (0.4 nozzle) | first difference at byte 3028 (line 142, column 1; expected 118006 bytes, actual 119020 bytes) context:   line 141: "G1 X150.261 Y103.496 E.01846"   line 142: expected "M73 P2 R4"; actual "G1 X146.504 Y99.739 E.15161"   line 143: expected "G1 X146.504 Y99.739 E.15161"; actual "G1 X146.504 Y100.386 E.01846" |
| PASS | Volumic/VS30SC2 Performance (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30SC2 Stage 2 (0.4 nozzle) | first difference at byte 3754 (line 163, column 7; expected 100924 bytes, actual 101231 bytes) context:   line 162: ";WIDTH:0.48"   line 163: expected "G1 F3663"; actual "G1 F3674"   line 164: "G1 X146.157 Y103.843 E.27933" |
| PASS | Volumic/VS30ULTRA (0.4 nozzle) |  |
| PASS | Voron/Voron 0.1 0.15 nozzle |  |
| DIVERGENT | Voron/Voron 0.1 0.2 nozzle | first difference at byte 333887 (line 12739, column 62; expected 354370 bytes, actual 354370 bytes) context:   line 12738: "; estimated printing time (normal mode) = 8m 24s"   line 12739: expected "; estimated first layer printing time (normal mode) = 0.486746s"; actual "; estimated first layer printing time (normal mode) = 0.486747s"   line 12740: "" |
| PASS | Voron/Voron 0.1 0.25 nozzle |  |
| PASS | Voron/Voron 0.1 0.4 nozzle |  |
| PASS | Voron/Voron 0.1 0.5 nozzle |  |
| PASS | Voron/Voron 0.1 0.6 nozzle |  |
| PASS | Voron/Voron 0.1 0.8 nozzle |  |
| PASS | Voron/Voron 0.1 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.2 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 250 0.25 nozzle | first difference at byte 20535 (line 747, column 8; expected 315819 bytes, actual 316030 bytes) context:   line 746: ";WIDTH:0.27"   line 747: expected "G1 F6234"; actual "G1 F6238"   line 748: "G1 X120.654 Y129.346 E.08803" |
| PASS | Voron/Voron 2.4 250 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.6 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 250 0.8 nozzle | first difference at byte 1582 (line 60, column 1; expected 78029 bytes, actual 79008 bytes) context:   line 59: "G1 X128.036 Y121.964 E.72647"   line 60: expected "M73 P1 R1"; actual "G1 X128.036 Y127.956 E.7169"   line 61: expected "G1 X128.036 Y127.956 E.7169"; actual "M73 P1 R1" |
| PASS | Voron/Voron 2.4 250 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.2 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 300 0.25 nozzle | first difference at byte 10841 (line 394, column 8; expected 315819 bytes, actual 316030 bytes) context:   line 393: ";WIDTH:0.27"   line 394: expected "G1 F6234"; actual "G1 F6238"   line 395: "G1 X145.654 Y154.346 E.08803" |
| PASS | Voron/Voron 2.4 300 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.6 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.8 nozzle |  |
| PASS | Voron/Voron 2.4 300 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.2 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.25 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.6 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.8 nozzle |  |
| PASS | Voron/Voron 2.4 350 1.0 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.15 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.2 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.25 nozzle |  |
| DIVERGENT | Voron/Voron Switchwire 250 0.4 nozzle | first difference at byte 18985 (line 735, column 1; expected 131585 bytes, actual 131811 bytes) context:   line 734: "G1 F2136"   line 735: expected "M73 P16 R3"; actual "G1 X120.6 Y109.4 E.25606"   line 736: expected "G1 X120.6 Y109.4 E.25606"; actual "G1 X120.6 Y100.6 E.25606" |
| PASS | Voron/Voron Switchwire 250 0.5 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.6 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.8 nozzle |  |
| DIVERGENT | Voron/Voron Switchwire 250 1.0 nozzle | first difference at byte 5544 (line 223, column 1; expected 66631 bytes, actual 66995 bytes) context:   line 222: "G1 X125.852 Y102.717 E.7165"   line 223: expected "; stop printing object cube10.stl id:0 copy 0"; actual "SET_VELOCITY_LIMIT ACCEL=7000 ACCEL_TO_DECEL=3500"   line 224: expected ";LAYER_CHANGE"; actual "G1 X123.387 Y103.239 F21000" |
| DIVERGENT | Voron/Voron Trident 250 0.15 nozzle | first difference at byte 14770 (line 524, column 8; expected 542026 bytes, actual 542253 bytes) context:   line 523: ";WIDTH:0.17"   line 524: expected "G1 F10570"; actual "G1 F10557"   line 525: "G1 X120.406 Y129.594 E.05077" |
| PASS | Voron/Voron Trident 250 0.2 nozzle |  |
| PASS | Voron/Voron Trident 250 0.25 nozzle |  |
| DIVERGENT | Voron/Voron Trident 250 0.4 nozzle | first difference at byte 17669 (line 679, column 1; expected 131581 bytes, actual 131806 bytes) context:   line 678: "G1 E-.56 F1800"   line 679: expected "M73 P15 R3"; actual ";WIPE_START"   line 680: expected ";WIPE_START"; actual "G1 F7200" |
| PASS | Voron/Voron Trident 250 0.5 nozzle |  |
| PASS | Voron/Voron Trident 250 0.6 nozzle |  |
| PASS | Voron/Voron Trident 250 0.8 nozzle |  |
| PASS | Voron/Voron Trident 250 1.0 nozzle |  |
| PASS | Voron/Voron Trident 300 0.15 nozzle |  |
| PASS | Voron/Voron Trident 300 0.2 nozzle |  |
| DIVERGENT | Voron/Voron Trident 300 0.25 nozzle | first difference at byte 3879 (line 138, column 1; expected 315165 bytes, actual 316037 bytes) context:   line 137: "G1 X145.854 Y145.854 E.00274"   line 138: expected "M73 P1 R7"; actual "G1 X145.854 Y146.032 E.00274"   line 139: expected "G1 X145.854 Y146.032 E.00274"; actual "G1 X153.968 Y154.146 E.17648" |
| PASS | Voron/Voron Trident 300 0.4 nozzle |  |
| PASS | Voron/Voron Trident 300 0.5 nozzle |  |
| DIVERGENT | Voron/Voron Trident 300 0.6 nozzle | first difference at byte 4183 (line 157, column 1; expected 89096 bytes, actual 89722 bytes) context:   line 156: "G1 X146.486 Y146.486 E.47735"   line 157: expected "M73 P5 R2"; actual "G1 X153.514 Y146.486 E.47735"   line 158: expected "G1 X153.514 Y146.486 E.47735"; actual "M73 P5 R2" |
| PASS | Voron/Voron Trident 300 0.8 nozzle |  |
| DIVERGENT | Voron/Voron Trident 300 1.0 nozzle | first difference at byte 8073 (line 326, column 6; expected 66717 bytes, actual 66990 bytes) context:   line 325: "SET_VELOCITY_LIMIT ACCEL=7000 ACCEL_TO_DECEL=3500"   line 326: expected "G1 X150.128 Y148.346 F21000"; actual "G1 X148.387 Y151.574 F21000"   line 327: "SET_VELOCITY_LIMIT ACCEL=5000 ACCEL_TO_DECEL=2500" |
| PASS | Voron/Voron Trident 350 0.15 nozzle |  |
| PASS | Voron/Voron Trident 350 0.2 nozzle |  |
| DIVERGENT | Voron/Voron Trident 350 0.25 nozzle | first difference at byte 3908 (line 139, column 1; expected 315390 bytes, actual 316037 bytes) context:   line 138: "G1 X170.854 Y171.032 E.00274"   line 139: expected "M73 P1 R7"; actual "G1 X178.968 Y179.146 E.17648"   line 140: expected "G1 X178.968 Y179.146 E.17648"; actual "M73 P1 R7" |
| PASS | Voron/Voron Trident 350 0.4 nozzle |  |
| PASS | Voron/Voron Trident 350 0.5 nozzle |  |
| PASS | Voron/Voron Trident 350 0.6 nozzle |  |
| PASS | Voron/Voron Trident 350 0.8 nozzle |  |
| PASS | Voron/Voron Trident 350 1.0 nozzle |  |
| DIVERGENT | Voxelab/Voxelab Aquila X2 0.4 nozzle | first difference at byte 1470 (line 69, column 1; expected 95553 bytes, actual 95829 bytes) context:   line 68: "G1 X105 Y101.389 E.07389"   line 69: expected "M73 P13 R5"; actual "G1 X115 Y101.389 E.33172"   line 70: expected "G1 X115 Y101.389 E.33172"; actual "M73 P13 R5" |
| PASS | Vzbot/Vzbot 235 AWD 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.5 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.6 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.5 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.6 nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.2mm nozzle | first difference at byte 3985 (line 171, column 7; expected 213124 bytes, actual 213150 bytes) context:   line 170: "G1 E-1.2 F7200"   line 171: expected "G1 X148.896 Y148.972 Z.56 F15000"; actual "G1 X145.864 Y150.863 Z.56 F15000"   line 172: expected "G1 X153.319 Y153.319 Z.56"; actual "G1 X146.681 Y153.319 Z.56" |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.3mm nozzle | first difference at byte 4761 (line 204, column 1; expected 246343 bytes, actual 246953 bytes) context:   line 203: "G1 X147.175 Y147.698 E.01184"   line 204: expected "M73 P28 R13"; actual "G1 X152.302 Y152.825 E.16399"   line 205: expected "G1 X152.302 Y152.825 E.16399"; actual "M73 P28 R13" |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.4mm nozzle | first difference at byte 3428 (line 154, column 1; expected 123591 bytes, actual 123779 bytes) context:   line 153: "G1 X148.961 Y147.393 E.03627"   line 154: expected "M73 P44 R6"; actual "G1 X152.607 Y151.039 E.29817"   line 155: expected "G1 X152.607 Y151.039 E.29817"; actual "M73 P44 R6" |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.6mm nozzle | first difference at byte 1690 (line 83, column 1; expected 109754 bytes, actual 109765 bytes) context:   line 82: "G1 X157.576 Y156.477 E.10062"   line 83: expected "M73 P44 R6"; actual "G1 X156.501 Y157.562 E.10062"   line 84: expected "G1 X156.501 Y157.562 E.10062"; actual "M73 P44 R6" |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.2mm nozzle | first difference at byte 8286 (line 354, column 8; expected 138701 bytes, actual 139401 bytes) context:   line 353: ";WIDTH:0.4"   line 354: expected "G1 F4404"; actual "G1 F4407"   line 355: "G1 X100.868 Y100.868 E.1512" |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.3mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.4mm nozzle | first difference at byte 1985 (line 95, column 1; expected 132373 bytes, actual 132656 bytes) context:   line 94: "G1 X100 Y97.784 E.04516"   line 95: expected "M73 P43 R6"; actual "G1 X110 Y97.784 E.56236"   line 96: expected "G1 X110 Y97.784 E.56236"; actual "G1 X111.257 Y98.175 E.07402" |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.6mm nozzle | first difference at byte 1562 (line 78, column 1; expected 106407 bytes, actual 107568 bytes) context:   line 77: "G1 X111.477 Y97.424 E.10062"   line 78: expected "M73 P44 R6"; actual "G1 X112.562 Y98.499 E.10062"   line 79: expected "G1 X112.562 Y98.499 E.10062"; actual "M73 P44 R6" |
| DIVERGENT | Wanhao/Wanhao D12-300 0.4 nozzle | first difference at byte 3754 (line 125, column 1; expected 129272 bytes, actual 129750 bytes) context:   line 124: "G1 X154.25 Y145.75 E.31659"   line 125: expected "M73 P47 R6"; actual "G1 X154.25 Y154.21 E.3151"   line 126: expected "G1 X154.25 Y154.21 E.3151"; actual "M73 P47 R6" |
| PASS | Wanhao France/D12 230 PRO M2 DIRECT 0.4 nozzle |  |
| DIVERGENT | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle | first difference at byte 15853 (line 686, column 2; expected 94911 bytes, actual 95041 bytes) context:   line 685: "G1 X119.444 Y110.41 F24000"   line 686: expected "M73 P18 R4"; actual "M204 P1000"   line 687: expected "M204 P1000"; actual ";TYPE:Outer wall" |
| DIVERGENT | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle PoopTool | first difference at byte 7704 (line 310, column 6; expected 96666 bytes, actual 97197 bytes) context:   line 309: ";WIDTH:0.45"   line 310: expected "G1 F2434"; actual "G1 F2215"   line 311: "G1 X119.355 Y110.645 E.274" |
| PASS | Wanhao France/D12 230 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| DIVERGENT | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle PoopTool | first difference at byte 12696 (line 433, column 6; expected 118554 bytes, actual 119047 bytes) context:   line 432: ";WIDTH:0.45"   line 433: expected "G1 F2435"; actual "G1 F2217"   line 434: "G1 X110.645 Y110.645 E.274" |
| PASS | Wanhao France/D12 300 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle |  |
| DIVERGENT | Wanhao France/D12 500 PRO M2 DIRECT 0.4 nozzle | first difference at byte 21191 (line 974, column 1; expected 93517 bytes, actual 93630 bytes) context:   line 973: "G1 X245.974 Y245.974 E.37777"   line 974: expected "M73 P28 R4"; actual "G1 X245.974 Y254.026 E.26712"   line 975: expected "G1 X245.974 Y254.026 E.26712"; actual "M73 P28 R4" |
| DIVERGENT | Wanhao France/D12 500 PRO M2 MONO DUAL 0.4 nozzle | first difference at byte 7725 (line 312, column 8; expected 94320 bytes, actual 94942 bytes) context:   line 311: ";WIDTH:0.45"   line 312: expected "G1 F2221"; actual "G1 F2225"   line 313: "G1 X254.355 Y245.645 E.274" |
| DIVERGENT | Wanhao France/D12 500 PRO M2 MONO DUAL PoopTool 0.4 nozzle | first difference at byte 31530 (line 1514, column 1; expected 97074 bytes, actual 97187 bytes) context:   line 1513: "G1 X250.21 Y259.79 E.29437"   line 1514: expected "M73 P44 R3"; actual "G1 X250.21 Y250.21 E.29437"   line 1515: expected "G1 X250.21 Y250.21 E.29437"; actual "M73 P44 R3" |
| DIVERGENT | Wanhao France/D12 500 PRO SMARTPAD DIRECT 0.4 nozzle | first difference at byte 45139 (line 1664, column 1; expected 117382 bytes, actual 117647 bytes) context:   line 1663: "G1 F1200"   line 1664: expected "M73 P50 R3"; actual "G1 X245.645 Y254.355 E.274"   line 1665: expected "G1 X245.645 Y254.355 E.274"; actual "G1 X245.645 Y245.645 E.28893" |
| DIVERGENT | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL 0.4 nozzle | first difference at byte 12784 (line 435, column 8; expected 118736 bytes, actual 118990 bytes) context:   line 434: ";WIDTH:0.45"   line 435: expected "G1 F2225"; actual "G1 F2229"   line 436: "G1 X245.645 Y245.645 E.274" |
| PASS | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.2 nozzle |  |
| DIVERGENT | WonderMaker/WonderMaker ZR 0.4 nozzle | first difference at byte 593 (line 19, column 9; expected 123060 bytes, actual 123495 bytes) context:   line 18: "EXCLUDE_OBJECT_DEFINE NAME=cube10.stl_id_0_copy_0 CENTER=150,150 POLYGON=[[145,145],[155,145],[155,155],[145,155],[145,145]]"   line 19: expected "M73 P0 R3"; actual "M73 P0 R4"   line 20: "M106 S0" |
| PASS | WonderMaker/WonderMaker ZR 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.8 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.2 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.4 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-cFahrf") |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-FktOA5/command-qE2HYi") |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.2 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.6 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S1000 0.8 nozzle | first difference at byte 3031 (line 125, column 7; expected 79189 bytes, actual 79358 bytes) context:   line 124: ";WIDTH:0.82"   line 125: expected "G1 F1954"; actual "G1 F1961"   line 126: "G1 X496.23 Y503.77 E.56661" |
| DIVERGENT | Z-Bolt/Z-Bolt S1000 Dual 0.4 nozzle | first difference at byte 10588 (line 440, column 1; expected 145193 bytes, actual 145487 bytes) context:   line 439: "G1 X502.599 Y498.55 E.02712"   line 440: expected "M73 P10 R3"; actual "G1 X502.168 Y498.113 E.02036"   line 441: expected "G1 X502.168 Y498.113 E.02036"; actual "M73 P10 R3" |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.8 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S300 0.4 nozzle | first difference at byte 13016 (line 534, column 1; expected 149799 bytes, actual 150093 bytes) context:   line 533: "G1 X150.297 Y153.369 E.01449"   line 534: expected "M73 P11 R3"; actual "G1 X149.423 Y153.656 E.0305"   line 535: expected "G1 X149.423 Y153.656 E.0305"; actual "M73 P11 R3" |
| DIVERGENT | Z-Bolt/Z-Bolt S300 0.6 nozzle | first difference at byte 2538 (line 102, column 1; expected 85453 bytes, actual 85873 bytes) context:   line 101: "G1 X146.497 Y148.885 E.05477"   line 102: expected "M73 P4 R2"; actual "G1 X151.115 Y153.503 E.44937"   line 103: expected "G1 X151.115 Y153.503 E.44937"; actual "M73 P4 R2" |
| DIVERGENT | Z-Bolt/Z-Bolt S300 0.8 nozzle | first difference at byte 2284 (line 93, column 1; expected 59837 bytes, actual 60328 bytes) context:   line 92: "G1 X146.931 Y149.045 E.12884"   line 93: expected "M73 P5 R1"; actual "G1 X150.955 Y153.069 E.69343"   line 94: expected "G1 X150.955 Y153.069 E.69343"; actual "M73 P5 R1" |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.6 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S300 Dual 0.8 nozzle | first difference at byte 2307 (line 93, column 1; expected 60601 bytes, actual 60782 bytes) context:   line 92: "G1 X146.931 Y149.045 E.12884"   line 93: expected "M73 P5 R1"; actual "G1 X150.955 Y153.069 E.69343"   line 94: expected "G1 X150.955 Y153.069 E.69343"; actual "M73 P5 R1" |
| PASS | Z-Bolt/Z-Bolt S400 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.8 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S400 Dual 0.4 nozzle | first difference at byte 3757 (line 151, column 5; expected 150260 bytes, actual 150563 bytes) context:   line 150: ";WIDTH:0.45"   line 151: expected "G1 F3994"; actual "G1 F4005"   line 152: "G1 X195.645 Y204.355 E.28893" |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.8 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S600 Dual 0.4 nozzle | first difference at byte 3753 (line 151, column 5; expected 145180 bytes, actual 145483 bytes) context:   line 150: ";WIDTH:0.45"   line 151: expected "G1 F3994"; actual "G1 F4005"   line 152: "G1 X295.645 Y304.355 E.28893" |
| DIVERGENT | Z-Bolt/Z-Bolt S600 Dual 0.6 nozzle | first difference at byte 4904 (line 202, column 1; expected 82865 bytes, actual 83032 bytes) context:   line 201: "G1 X299.383 Y296.402 E.28624"   line 202: expected "M73 P8 R2"; actual "G1 X298.597 Y296.402 E.05336"   line 203: expected "G1 X298.597 Y296.402 E.05336"; actual "M73 P8 R2" |
| DIVERGENT | Z-Bolt/Z-Bolt S600 Dual 0.8 nozzle | first difference at byte 13682 (line 615, column 1; expected 58001 bytes, actual 58311 bytes) context:   line 614: "G1 X302.899 Y302.466 E.03816"   line 615: expected "M73 P37 R1"; actual "G1 X302.111 Y301.65 E.13567"   line 616: expected "G1 X302.111 Y301.65 E.13567"; actual "M73 P37 R1" |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.4 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S800 Dual 0.6 nozzle | first difference at byte 2128 (line 86, column 1; expected 82454 bytes, actual 82874 bytes) context:   line 85: "G1 X404.503 Y401.319 E.05477"   line 86: expected "M73 P3 R2"; actual "G1 X400.681 Y397.497 E.37193"   line 87: expected "G1 X400.681 Y397.497 E.37193"; actual "M73 P3 R2" |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.8 nozzle |  |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.25 Nozzle | iQ/iQ TiQ2 0.25 Nozzle process: no compatible preset |
| PASS | iQ/iQ TiQ2 0.4 Nozzle |  |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.6 Nozzle | iQ/iQ TiQ2 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.8 Nozzle | iQ/iQ TiQ2 0.8 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.25 Nozzle | iQ/iQ TiQ8 0.25 Nozzle process: no compatible preset |
| DIVERGENT | iQ/iQ TiQ8 0.4 Nozzle | first difference at byte 21659 (line 859, column 1; expected 129665 bytes, actual 130358 bytes) context:   line 858: "G1 X245.736 Y197.238 E.0621"   line 859: expected ";LAYER_CHANGE"; actual "G1 X246.055 Y195.983 F18000"   line 860: expected ";Z:0.6"; actual ";WIDTH:0.463894" |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.6 Nozzle | iQ/iQ TiQ8 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.8 Nozzle | iQ/iQ TiQ8 0.8 Nozzle process: no compatible preset |
| DIVERGENT | re3D/re3D Gigabot 4 0.4 nozzle | first difference at byte 3995 (line 169, column 1; expected 85968 bytes, actual 86538 bytes) context:   line 168: "G1 E1 F1800"   line 169: expected "M73 P4 R5"; actual "SET_VELOCITY_LIMIT ACCEL=500"   line 170: expected "SET_VELOCITY_LIMIT ACCEL=500"; actual ";TYPE:Bottom surface" |
| DIVERGENT | re3D/re3D Gigabot 4 0.8 nozzle | first difference at byte 11237 (line 535, column 1; expected 61291 bytes, actual 61613 bytes) context:   line 534: "G1 E-.7 F1800"   line 535: expected "M73 P23 R2"; actual ";WIPE_START"   line 536: expected ";WIPE_START"; actual "G1 F1200" |
| PASS | re3D/re3D Gigabot 4 XLT 0.4 nozzle |  |
| DIVERGENT | re3D/re3D Gigabot 4 XLT 0.8 nozzle | first difference at byte 19152 (line 951, column 1; expected 61281 bytes, actual 61621 bytes) context:   line 950: "G1 E1 F1800"   line 951: expected "G1 F1200"; actual "M73 P42 R1"   line 952: expected "G1 X298.586 Y383.586 E.40697"; actual "G1 F1200" |
| PASS | re3D/re3D GigabotX 2 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 1.75 nozzle |  |
| PASS | re3D/re3D GigabotX 2 XLT 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 XLT 1.75 nozzle |  |
| PASS | re3D/re3D Terabot 4 0.4 nozzle |  |
| PASS | re3D/re3D Terabot 4 0.8 nozzle |  |
| PASS | re3D/re3D TerabotX 2 0.8 nozzle |  |
| PASS | re3D/re3D TerabotX 2 1.75 nozzle |  |
