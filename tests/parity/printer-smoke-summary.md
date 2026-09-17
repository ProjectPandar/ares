# OrcaSlicer printer smoke summary

630 of 1001 printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; classic wall generator baseline; cube model).

Statuses: `PASS` (normalized byte equality), `DIVERGENT` (first byte difference), `ORCA_ERROR` (the upstream OrcaSlicer 2.4.2 reference binary itself failed, so no reference stream exists), `VENDOR_INCOMPLETE` (the vendor profile tree does not ship the machine's referenced default process preset), `ARES_ERROR` (Ares failed to load or slice the case).

| status | printer | first divergence |
|---|---|---|
| DIVERGENT | Afinia/Afinia H+1(HS) 0.4 nozzle | first difference at byte 22174 (line 935, column 1; expected 107267 bytes, actual 107605 bytes) context:   line 934: "G1 X99.45 Y126.168 E.05401"   line 935: expected "M73 P27 R2"; actual "; stop printing object cube10.stl id:0 copy 0"   line 936: expected "; stop printing object cube10.stl id:0 copy 0"; actual ";LAYER_CHANGE" |
| PASS | Afinia/Afinia H+1(HS) 0.6 nozzle |  |
| PASS | Anker/Anker M5 0.2 nozzle |  |
| PASS | Anker/Anker M5 0.25 nozzle |  |
| DIVERGENT | Anker/Anker M5 0.4 nozzle | first difference at byte 4946 (line 218, column 7; expected 106659 bytes, actual 106930 bytes) context:   line 217: ";WIDTH:0.45"   line 218: expected "G1 F3832"; actual "G1 F3840"   line 219: "G1 X113.552 Y121.448 E.26192" |
| PASS | Anker/Anker M5 0.6 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.2 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.25 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.4 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.6 nozzle |  |
| PASS | Anker/Anker M5C 0.2 nozzle |  |
| PASS | Anker/Anker M5C 0.25 nozzle |  |
| DIVERGENT | Anker/Anker M5C 0.4 nozzle | first difference at byte 24078 (line 1166, column 1; expected 106500 bytes, actual 106649 bytes) context:   line 1165: "G1 X113.619 Y108.753 E.15851"   line 1166: expected "M73 P26 R3"; actual "G1 X113.338 Y108.472 E.01418"   line 1167: expected "G1 X113.338 Y108.472 E.01418"; actual "M73 P26 R3" |
| DIVERGENT | Anker/Anker M5C 0.6 nozzle | first difference at byte 1775 (line 78, column 2; expected 95666 bytes, actual 96182 bytes) context:   line 77: "G1 X102.835 Y102.643 F18000"   line 78: expected "M73 P1 R4"; actual "M205 X8 Y8"   line 79: expected "M205 X8 Y8"; actual "G1 F3000" |
| DIVERGENT | Anycubic/Anycubic 4Max Pro 0.4 nozzle | first difference at byte 6235 (line 282, column 7; expected 99995 bytes, actual 100651 bytes) context:   line 281: ";WIDTH:0.45"   line 282: expected "G1 F2089"; actual "G1 F2096"   line 283: "G1 X131.032 Y106.468 E.26325" |
| PASS | Anycubic/Anycubic 4Max Pro 2 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Chiron 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 0.4 nozzle |  |
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
| DIVERGENT | Anycubic/Anycubic Kobra Max 0.4 nozzle | first difference at byte 12530 (line 534, column 7; expected 112047 bytes, actual 112752 bytes) context:   line 533: ";WIDTH:0.4"   line 534: expected "G1 F2128"; actual "G1 F2131"   line 535: "G1 X195.957 Y204.043 E.23528" |
| PASS | Anycubic/Anycubic Kobra Neo 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra Plus 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.25 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra X 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Predator 0.4 nozzle | first difference at byte 3743 (line 174, column 8; expected 90314 bytes, actual 90644 bytes) context:   line 173: ";WIDTH:0.45"   line 174: expected "G1 F4161"; actual "G1 F4164"   line 175: "G1 X-3.968 Y3.968 E.26862" |
| PASS | Anycubic/Anycubic Vyper 0.4 nozzle |  |
| PASS | Anycubic/Anycubic i3 Mega S 0.4 nozzle |  |
| PASS | Artillery/Artillery Genius 0.4 nozzle |  |
| DIVERGENT | Artillery/Artillery Genius Pro 0.4 nozzle | first difference at byte 4276 (line 173, column 1; expected 87550 bytes, actual 87746 bytes) context:   line 172: "G1 X106.746 Y113.765 E.02933"   line 173: expected "M73 P40 R7"; actual "G1 X106.034 Y113.054 E.04626"   line 174: expected "G1 X106.034 Y113.054 E.04626"; actual "; stop printing object cube10.stl id:0 copy 0" |
| PASS | Artillery/Artillery Hornet 0.4 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.2 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.4 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.6 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.8 nozzle |  |
| DIVERGENT | Artillery/Artillery Sidewinder X1 0.4 nozzle | first difference at byte 4008 (line 167, column 1; expected 104247 bytes, actual 104247 bytes) context:   line 166: "G1 X154.75 Y145.25 E.43192"   line 167: expected "M73 P4 R8"; actual "G1 X154.75 Y154.71 E.4301"   line 168: expected "G1 X154.75 Y154.71 E.4301"; actual "M73 P4 R8" |
| DIVERGENT | Artillery/Artillery Sidewinder X2 0.4 nozzle | first difference at byte 2306 (line 97, column 1; expected 104344 bytes, actual 104344 bytes) context:   line 96: "G1 X156.641 Y157.649 E.0712"   line 97: expected "M73 P36 R8"; actual "G1 X155 Y158.116 E.07757"   line 98: expected "G1 X155 Y158.116 E.07757"; actual "M73 P36 R8" |
| PASS | Artillery/Artillery Sidewinder X3 Plus 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X3 Pro 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X4 Plus 0.4 nozzle |  |
| DIVERGENT | Artillery/Artillery Sidewinder X4 Pro 0.4 nozzle | first difference at byte 1859 (line 98, column 1; expected 90072 bytes, actual 90773 bytes) context:   line 97: "G1 X124.11 Y121.836 Z.6"   line 98: expected "M73 P6 R6"; actual "G1 Z.2"   line 99: expected "G1 Z.2"; actual "G1 E1.3 F2400" |
| PASS | BBL/Bambu Lab A1 0.2 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.6 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.8 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.2 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.6 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.8 nozzle |  |
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
| DIVERGENT | BBL/Bambu Lab P1P 0.4 nozzle | first difference at byte 93 (line 3, column 28; expected 143644 bytes, actual 143848 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 37s; total estimated time: 11m 52s"; actual "; model printing time: 5m 39s; total estimated time: 11m 54s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 101704 bytes, actual 101993 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 42s; total estimated time: 9m 56s"; actual "; model printing time: 3m 44s; total estimated time: 9m 57s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.8 nozzle | first difference at byte 93 (line 3, column 28; expected 87104 bytes, actual 87104 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 43s; total estimated time: 9m 0s"; actual "; model printing time: 2m 44s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.2 nozzle | first difference at byte 93 (line 3, column 28; expected 371249 bytes, actual 371272 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 15m 25s"; actual "; model printing time: 8m 39s; total estimated time: 15m 28s"   line 4: "; estimated first layer printing time (normal mode) = 6m 48s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 143988 bytes, actual 143988 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 5s; total estimated time: 10m 20s"; actual "; model printing time: 4m 8s; total estimated time: 10m 23s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 102134 bytes, actual 102145 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 46s; total estimated time: 9m 0s"; actual "; model printing time: 2m 48s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.8 nozzle | first difference at byte 92 (line 3, column 27; expected 87316 bytes, actual 87327 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 7s; total estimated time: 8m 25s"; actual "; model printing time: 2m 9s; total estimated time: 8m 27s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| PASS | BBL/Bambu Lab P2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab X1 0.2 nozzle | first difference at byte 92 (line 3, column 27; expected 396001 bytes, actual 395885 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 37s; total estimated time: 18m 36s"; actual "; model printing time: 8m 40s; total estimated time: 18m 39s"   line 4: "; estimated first layer printing time (normal mode) = 9m 58s" |
| DIVERGENT | BBL/Bambu Lab X1 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 168834 bytes, actual 169164 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 38s; total estimated time: 12m 54s"; actual "; model printing time: 5m 40s; total estimated time: 12m 57s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 126922 bytes, actual 127139 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 43s; total estimated time: 11m 1s"; actual "; model printing time: 3m 45s; total estimated time: 11m 2s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
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
| DIVERGENT | BBL/Bambu Lab X2D 0.6 nozzle | first difference at byte 95953 (line 3155, column 1; expected 121653 bytes, actual 121840 bytes) context:   line 3154: "G1 X141.848 Y130.06 E.14771"   line 3155: expected "M73 P49 R7"; actual "G1 X141.848 Y124.402 E.38422"   line 3156: expected "G1 X141.848 Y124.402 E.38422"; actual "M73 P49 R7" |
| PASS | BBL/Bambu Lab X2D 0.8 nozzle |  |
| PASS | BIQU/BIQU B1 (0.4 nozzle) |  |
| DIVERGENT | BIQU/BIQU BX (0.4 nozzle) | first difference at byte 5448 (line 235, column 1; expected 103900 bytes, actual 104046 bytes) context:   line 234: "G1 X120.2 Y120.2 E.27934"   line 235: expected "M73 P55 R5"; actual "G1 X129.8 Y120.2 E.27934"   line 236: expected "G1 X129.8 Y120.2 E.27934"; actual "M73 P55 R5" |
| DIVERGENT | BIQU/BIQU Hurakan (0.4 nozzle) | first difference at byte 9549 (line 381, column 1; expected 114618 bytes, actual 115084 bytes) context:   line 380: "G1 X106.326 Y110.243 E.01469"   line 381: expected "M73 P11 R4"; actual "G1 X109.757 Y113.674 E.14116"   line 382: expected "G1 X109.757 Y113.674 E.14116"; actual "M73 P11 R4" |
| DIVERGENT | Blocks/BLOCKS Pro S100 0.4 nozzle | first difference at byte 8481 (line 339, column 7; expected 127902 bytes, actual 128216 bytes) context:   line 338: ";WIDTH:0.42"   line 339: expected "G1 F3228"; actual "G1 F3235"   line 340: "G1 X503.993 Y496.007 E.23787" |
| PASS | Blocks/BLOCKS Pro S100 0.6 nozzle |  |
| DIVERGENT | Blocks/BLOCKS Pro S100 0.8 nozzle | first difference at byte 2393 (line 105, column 1; expected 73613 bytes, actual 75208 bytes) context:   line 104: "G1 X503.67 Y503.67 E.65717"   line 105: expected "M73 P5 R2"; actual "G1 X496.33 Y503.67 E.65717"   line 106: expected "G1 X496.33 Y503.67 E.65717"; actual "M73 P5 R2" |
| PASS | Blocks/BLOCKS Pro S100 1.0 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 1.2 nozzle |  |
| DIVERGENT | Blocks/BLOCKS RD50 V2 0.4 nozzle | first difference at byte 11986 (line 473, column 7; expected 127154 bytes, actual 127984 bytes) context:   line 472: ";WIDTH:0.42"   line 473: expected "G1 F3242"; actual "G1 F3254"   line 474: "G1 X246.007 Y246.007 E.23787" |
| PASS | Blocks/BLOCKS RD50 V2 0.6 nozzle |  |
| PASS | Blocks/BLOCKS RD50 V2 0.8 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.4 nozzle |  |
| DIVERGENT | Blocks/BLOCKS RF50 0.6 nozzle | first difference at byte 4170 (line 171, column 1; expected 94024 bytes, actual 95587 bytes) context:   line 170: "G1 X248.511 Y257.278 Z.96 F30000"   line 171: expected "M73 P4 R3"; actual "G1 Z.56"   line 172: expected "G1 Z.56"; actual "M73 P4 R3" |
| DIVERGENT | Blocks/BLOCKS RF50 0.8 nozzle | first difference at byte 3229 (line 140, column 1; expected 74407 bytes, actual 74927 bytes) context:   line 139: "G1 X249.326 Y247.765 E.51668"   line 140: expected "M73 P5 R2"; actual "G1 X248.285 Y247.765 E.13073"   line 141: expected "G1 X248.285 Y247.765 E.13073"; actual "M73 P5 R2" |
| PASS | CONSTRUCT3D/Construct 1 0.4 nozzle |  |
| PASS | CONSTRUCT3D/Construct 1 XL 0.6 nozzle |  |
| PASS | Chuanying/Chuanying X1 0.25 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.4 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.6 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.8 Nozzle |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle |  |
| DIVERGENT | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 | first difference at byte 33325 (line 1387, column 1; expected 152640 bytes, actual 152870 bytes) context:   line 1386: "G1 F21000"   line 1387: expected "M73 P25 R4"; actual "G1 X90.2442 Y83.2721 Z2.62857"   line 1388: expected "G1 X90.2442 Y83.2721 Z2.62857"; actual "G1 X89.8229 Y83.5903 Z2.65714" |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 Plus |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle fast |  |
| PASS | CoLiDo/CoLiDo 160 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo SR1 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo X16 0.4 nozzle |  |
| DIVERGENT | Comgrow/Comgrow T300 0.4 nozzle | first difference at byte 17018 (line 658, column 8; expected 107396 bytes, actual 107602 bytes) context:   line 657: ";WIDTH:0.42"   line 658: expected "G1 F2540"; actual "G1 F2544"   line 659: "G1 X145.21 Y154.79 E.29437" |
| PASS | Comgrow/Comgrow T500 0.4 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.6 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.8 nozzle |  |
| PASS | Creality/Creality CR-10 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 SE 0.2 nozzle |  |
| DIVERGENT | Creality/Creality CR-10 SE 0.4 nozzle | first difference at byte 5562 (line 224, column 1; expected 89327 bytes, actual 89542 bytes) context:   line 223: "G1 X114.061 Y106.772 E.31673"   line 224: expected "M73 P6 R6"; actual "G1 X114.061 Y106.239 E.01639"   line 225: expected "G1 X114.061 Y106.239 E.01639"; actual "M73 P6 R6" |
| PASS | Creality/Creality CR-10 SE 0.6 nozzle |  |
| DIVERGENT | Creality/Creality CR-10 SE 0.8 nozzle | first difference at byte 4238 (line 168, column 6; expected 72052 bytes, actual 72371 bytes) context:   line 167: ";WIDTH:0.85"   line 168: expected "G1 F957"; actual "G1 F961"   line 169: "G1 X106.275 Y113.725 E.48996" |
| PASS | Creality/Creality CR-10 V2 0.4 nozzle |  |
| DIVERGENT | Creality/Creality CR-10 V3 0.4 nozzle | first difference at byte 3612 (line 162, column 10; expected 99242 bytes, actual 100419 bytes) context:   line 161: "G1 X152.629 Y151.013 E.01673"   line 162: expected "M73 P14 R7"; actual "M73 P14 R8"   line 163: "G1 X158.987 Y157.371 E.27915" |
| PASS | Creality/Creality CR-10 V3 0.6 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 Max 0.2 nozzle | first difference at byte 7041 (line 291, column 29; expected 394066 bytes, actual 394066 bytes) context:   line 290: "G1 X195.605 Y204.196 E-.58892"   line 291: expected "G1 X195.605 Y203.947 E-.37351"; actual "G1 X195.605 Y203.947 E-.37352"   line 292: "G1 X195.858 Y204.2 E-.53756" |
| DIVERGENT | Creality/Creality CR-6 Max 0.4 nozzle | first difference at byte 13179 (line 628, column 1; expected 102533 bytes, actual 102827 bytes) context:   line 627: "G1 Z1.6 F9000"   line 628: expected "M73 P21 R7"; actual "G1 X203.649 Y197.875 Z1.6"   line 629: expected "G1 X203.649 Y197.875 Z1.6"; actual "M73 P21 R7" |
| DIVERGENT | Creality/Creality CR-6 Max 0.6 nozzle | first difference at byte 4016 (line 192, column 7; expected 85868 bytes, actual 86650 bytes) context:   line 191: ";WIDTH:0.66"   line 192: expected "G1 F1366"; actual "G1 F1372"   line 193: "G1 X196.607 Y203.393 E.34122" |
| DIVERGENT | Creality/Creality CR-6 Max 0.8 nozzle | first difference at byte 2071 (line 97, column 1; expected 62408 bytes, actual 63143 bytes) context:   line 96: "G1 Z.72 F9000"   line 97: expected "M73 P15 R4"; actual "G1 X203.069 Y203.069 Z.72"   line 98: expected "G1 X203.069 Y203.069 Z.72"; actual "M73 P15 R4" |
| DIVERGENT | Creality/Creality CR-6 SE 0.2 nozzle | first difference at byte 7068 (line 292, column 29; expected 394163 bytes, actual 394163 bytes) context:   line 291: "G1 X113.105 Y121.696 E-.58892"   line 292: expected "G1 X113.105 Y121.447 E-.37351"; actual "G1 X113.105 Y121.447 E-.37352"   line 293: "G1 X113.358 Y121.7 E-.53756" |
| DIVERGENT | Creality/Creality CR-6 SE 0.4 nozzle | first difference at byte 6080 (line 282, column 1; expected 102376 bytes, actual 102854 bytes) context:   line 281: "G1 X113.826 Y120.286 E.26584"   line 282: expected "M73 P14 R7"; actual "G1 X113.826 Y119.781 E.01469"   line 283: expected "G1 X113.826 Y119.781 E.01469"; actual "M73 P14 R7" |
| PASS | Creality/Creality CR-6 SE 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 SE 0.8 nozzle |  |
| PASS | Creality/Creality CR-M4 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 0.6 nozzle | first difference at byte 3159 (line 144, column 1; expected 86399 bytes, actual 86918 bytes) context:   line 143: "G1 X113.515 Y108.86 E.04127"   line 144: expected "M73 P11 R7"; actual "G1 X111.14 Y106.485 E.16378"   line 145: expected "G1 X111.14 Y106.485 E.16378"; actual "M73 P11 R7" |
| PASS | Creality/Creality Ender-3 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.6 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 Pro 0.8 nozzle | first difference at byte 8166 (line 369, column 1; expected 75867 bytes, actual 76125 bytes) context:   line 368: "G1 X113.058 Y106.911 F9000"   line 369: expected "M73 P15 R6"; actual ";WIDTH:0.72292"   line 370: expected ";WIDTH:0.72292"; actual "G1 F1073" |
| PASS | Creality/Creality Ender-3 S1 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Pro 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V2 0.4 nozzle | first difference at byte 8994 (line 405, column 1; expected 102015 bytes, actual 102181 bytes) context:   line 404: "G1 X112.384 Y106.414 E.03496"   line 405: expected "; stop printing object cube10.stl id:0 copy 0"; actual "M204 S700"   line 406: expected ";LAYER_CHANGE"; actual "G1 X113.245 Y106.738 F9000" |
| DIVERGENT | Creality/Creality Ender-3 V2 Neo 0.4 nozzle | first difference at byte 5171 (line 240, column 1; expected 102619 bytes, actual 102754 bytes) context:   line 239: "G1 X113.586 Y112.344 E.03613"   line 240: expected "M73 P40 R7"; actual "G1 X112.373 Y113.558 E.04993"   line 241: expected "G1 X112.373 Y113.558 E.04993"; actual "M73 P40 R7" |
| PASS | Creality/Creality Ender-3 V3 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.6 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 KE 0.8 nozzle | first difference at byte 4811 (line 201, column 7; expected 75368 bytes, actual 75959 bytes) context:   line 200: ";WIDTH:0.9"   line 201: expected "G1 F1836"; actual "G1 F1846"   line 202: "G1 X106.29 Y113.71 E.5288" |
| DIVERGENT | Creality/Creality Ender-3 V3 Plus 0.4 nozzle | first difference at byte 7479 (line 287, column 8; expected 123089 bytes, actual 123624 bytes) context:   line 286: ";WIDTH:0.45"   line 287: expected "G1 F1951"; actual "G1 F1956"   line 288: "G1 X146.052 Y153.948 E.26192" |
| PASS | Creality/Creality Ender-3 V3 Plus 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.2 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 SE 0.4 nozzle | first difference at byte 26 (line 2, column 12; expected 116543 bytes, actual 117482 bytes) context:   line 1: ";FLAVOR:Marlin"   line 2: expected ";TIME:358.12"; actual ";TIME:358.17"   line 3: ";Filament used:0.24m" |
| PASS | Creality/Creality Ender-3 V3 SE 0.6 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 SE 0.8 nozzle | first difference at byte 25 (line 2, column 11; expected 96857 bytes, actual 97485 bytes) context:   line 1: ";FLAVOR:Marlin"   line 2: expected ";TIME:287.89"; actual ";TIME:287.99"   line 3: ";Filament used:0.32m" |
| PASS | Creality/Creality Ender-3 V4 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-5 0.4 nozzle | first difference at byte 1716 (line 79, column 1; expected 100597 bytes, actual 100696 bytes) context:   line 78: "G1 X105 Y117.566 E.30727"   line 79: expected "M73 P9 R8"; actual "G1 X103.637 Y117.173 E.04359"   line 80: expected "G1 X103.637 Y117.173 E.04359"; actual "M73 P9 R8" |
| DIVERGENT | Creality/Creality Ender-5 Max 0.4 nozzle | first difference at byte 4524 (line 170, column 7; expected 125250 bytes, actual 126411 bytes) context:   line 169: ";WIDTH:0.45"   line 170: expected "G1 F3259"; actual "G1 F3269"   line 171: "G1 X195.602 Y204.398 E.27402" |
| PASS | Creality/Creality Ender-5 Max 0.6 nozzle |  |
| PASS | Creality/Creality Ender-5 Max 0.8 nozzle |  |
| PASS | Creality/Creality Ender-5 Plus 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.2 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.25 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.3 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.5 nozzle |  |
| DIVERGENT | Creality/Creality Ender-5 Pro (2019) 0.6 nozzle | first difference at byte 1808 (line 84, column 1; expected 87293 bytes, actual 87773 bytes) context:   line 83: "G1 X102.875 Y115.862 E.06409"   line 84: expected "M73 P10 R7"; actual "G1 X102.706 Y115 E.04201"   line 85: expected "G1 X102.706 Y115 E.04201"; actual "M73 P10 R7" |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-5 Pro (2019) 1.0 nozzle | first difference at byte 2610 (line 126, column 1; expected 57546 bytes, actual 58548 bytes) context:   line 125: "G1 X113.739 Y110.214 E.18326"   line 126: expected "M73 P17 R4"; actual "G1 X109.786 Y106.261 E.72671"   line 127: expected "G1 X109.786 Y106.261 E.72671"; actual "M73 P17 R4" |
| DIVERGENT | Creality/Creality Ender-5 S1 0.4 nozzle | first difference at byte 9958 (line 432, column 1; expected 100700 bytes, actual 100844 bytes) context:   line 431: "G1 X105.98 Y106.655 E.3455"   line 432: expected "M73 P17 R7"; actual "G1 X105.98 Y107.231 E.0191"   line 433: expected "G1 X105.98 Y107.231 E.0191"; actual "M73 P17 R7" |
| PASS | Creality/Creality Ender-5S 0.4 nozzle |  |
| PASS | Creality/Creality Ender-6 0.4 nozzle |  |
| PASS | Creality/Creality Hi 0.2 nozzle |  |
| PASS | Creality/Creality Hi 0.4 nozzle |  |
| PASS | Creality/Creality Hi 0.6 nozzle |  |
| PASS | Creality/Creality Hi 0.8 nozzle |  |
| PASS | Creality/Creality K1 (0.4 nozzle) |  |
| PASS | Creality/Creality K1 (0.6 nozzle) |  |
| DIVERGENT | Creality/Creality K1 (0.8 nozzle) | first difference at byte 2098 (line 93, column 1; expected 56077 bytes, actual 56722 bytes) context:   line 92: "G1 X113.155 Y109.904 E.13536"   line 93: expected "M73 P45 R2"; actual "G1 X110.096 Y106.845 E.54034"   line 94: expected "G1 X110.096 Y106.845 E.54034"; actual "G1 X109.012 Y106.845 E.13536" |
| DIVERGENT | Creality/Creality K1 Max (0.4 nozzle) | first difference at byte 85262 (line 3754, column 1; expected 112955 bytes, actual 112955 bytes) context:   line 3753: "G1 X145.877 Y152.634 E.01639"   line 3754: expected "G1 X147.366 Y154.123 E.06472"; actual "M73 P98 R0"   line 3755: expected "M73 P98 R0"; actual "G1 X147.366 Y154.123 E.06472" |
| PASS | Creality/Creality K1 Max (0.6 nozzle) |  |
| DIVERGENT | Creality/Creality K1 Max (0.8 nozzle) | first difference at byte 3354 (line 147, column 1; expected 56274 bytes, actual 56525 bytes) context:   line 146: "G1 X154.59 Y154.59 F30000"   line 147: expected "M73 P47 R2"; actual ";TYPE:Outer wall"   line 148: expected ";TYPE:Outer wall"; actual "G1 F1200" |
| PASS | Creality/Creality K1 Max_CFS-C 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K1 SE 0.4 nozzle | first difference at byte 76164 (line 3257, column 1; expected 115911 bytes, actual 116076 bytes) context:   line 3256: "G1 X105.935 Y113.935 E.22089"   line 3257: expected "M73 P90 R0"; actual "G1 X105.962 Y114.065 E.00367"   line 3258: expected "G1 X105.962 Y114.065 E.00367"; actual "M73 P90 R0" |
| DIVERGENT | Creality/Creality K1 SE 0.6 nozzle | first difference at byte 3433 (line 154, column 7; expected 74907 bytes, actual 75186 bytes) context:   line 153: ";WIDTH:0.65"   line 154: expected "G1 F1429"; actual "G1 F1435"   line 155: "G1 X105.881 Y114.119 E.52658" |
| VENDOR_INCOMPLETE | Creality/Creality K1 SE 0.8 nozzle | Creality/Creality K1 SE 0.8 nozzle process: default preset "0.40mm Standard @Creality K1 SE 0.8 nozzle" not found |
| DIVERGENT | Creality/Creality K1 SE_CFS-C 0.4 nozzle | first difference at byte 7280 (line 306, column 8; expected 115879 bytes, actual 116018 bytes) context:   line 305: ";WIDTH:0.45"   line 306: expected "G1 F1366"; actual "G1 F1367"   line 307: "G1 X105.602 Y114.398 E.26837" |
| DIVERGENT | Creality/Creality K1C 0.4 nozzle | first difference at byte 4266 (line 177, column 7; expected 116811 bytes, actual 117197 bytes) context:   line 176: ";WIDTH:0.45"   line 177: expected "G1 F2047"; actual "G1 F2050"   line 178: "G1 X105.602 Y114.398 E.27685" |
| DIVERGENT | Creality/Creality K1C 0.6 nozzle | first difference at byte 3744 (line 151, column 1; expected 92838 bytes, actual 93154 bytes) context:   line 150: "G1 X106.685 Y112.155 E-.04826"   line 151: expected "M73 P33 R4"; actual "G1 X106.925 Y112.396 E-.02552"   line 152: expected "G1 X106.925 Y112.396 E-.02552"; actual "M73 P33 R4" |
| PASS | Creality/Creality K1C 0.8 nozzle |  |
| DIVERGENT | Creality/Creality K1C_CFS-C 0.4 nozzle | first difference at byte 40971 (line 1746, column 1; expected 117011 bytes, actual 117121 bytes) context:   line 1745: "G1 X106.432 Y105.892 E.0246"   line 1746: expected "M73 P58 R3"; actual "G1 X105.892 Y105.892 E.0174"   line 1747: expected "G1 X105.892 Y105.892 E.0174"; actual "M73 P58 R3" |
| PASS | Creality/Creality K1_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K2 0.2 nozzle |  |
| PASS | Creality/Creality K2 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K2 0.6 nozzle | first difference at byte 3240 (line 134, column 1; expected 75107 bytes, actual 75615 bytes) context:   line 133: "G1 X126.432 Y130.48 E.05491"   line 134: expected "G1 X129.52 Y133.568 E.29616"; actual "M73 P40 R2"   line 135: expected "M73 P40 R2"; actual "G1 X129.52 Y133.568 E.29616" |
| PASS | Creality/Creality K2 0.8 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.2 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K2 Plus 0.6 nozzle | first difference at byte 3372 (line 142, column 1; expected 95315 bytes, actual 95769 bytes) context:   line 141: "G1 X174.52 Y178.568 E.29616"   line 142: expected "G1 X173.71 Y178.568 E.05491"; actual "M73 P16 R3"   line 143: expected "M73 P16 R3"; actual "G1 X173.71 Y178.568 E.05491" |
| DIVERGENT | Creality/Creality K2 Plus 0.8 nozzle | first difference at byte 5106 (line 218, column 1; expected 79002 bytes, actual 79002 bytes) context:   line 217: "G1 X175.467 Y178.313 E.38216"   line 218: expected "M73 P20 R2"; actual "G1 X174.404 Y178.313 E.1009"   line 219: expected "G1 X174.404 Y178.313 E.1009"; actual "M73 P20 R2" |
| PASS | Creality/Creality K2 Pro 0.2 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.4 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.6 nozzle |  |
| DIVERGENT | Creality/Creality K2 Pro 0.8 nozzle | first difference at byte 1092 (line 48, column 1; expected 63170 bytes, actual 63480 bytes) context:   line 47: "G1 X150 Y0 E15 F6000"   line 48: expected "M73 P44 R1"; actual "G92 E0"   line 49: expected "G92 E0"; actual "G1 Z1 F600" |
| DIVERGENT | Creality/Creality K2 SE 0.4 nozzle | first difference at byte 4394 (line 191, column 8; expected 116412 bytes, actual 116917 bytes) context:   line 190: ";WIDTH:0.45"   line 191: expected "G1 F2050"; actual "G1 F2052"   line 192: "G1 X105.602 Y111.898 E.26837" |
| DIVERGENT | Creality/Creality SPARKX i7 0.2 nozzle | first difference at byte 22156 (line 808, column 1; expected 361463 bytes, actual 361673 bytes) context:   line 807: "G1 X134.126 Y129.358 E.00227"   line 808: expected "M73 P8 R13"; actual "G1 X129.358 Y134.126 E.05455"   line 809: expected "G1 X129.358 Y134.126 E.05455"; actual "G1 X129.077 Y134.126 E.00227" |
| DIVERGENT | Creality/Creality SPARKX i7 0.4 nozzle | first difference at byte 2827 (line 132, column 1; expected 137363 bytes, actual 137362 bytes) context:   line 131: "G1 X129.221 Y126.2 E.26305"   line 132: expected "M73 P9 R6"; actual "G1 X128.55 Y126.2 E.02728"   line 133: expected "G1 X128.55 Y126.2 E.02728"; actual "M73 P9 R6" |
| DIVERGENT | Creality/Creality SPARKX i7 0.6 nozzle | first difference at byte 2217 (line 107, column 1; expected 104151 bytes, actual 104148 bytes) context:   line 106: "G1 X134.123 Y127.322 E.1484"   line 107: expected "M73 P13 R4"; actual "G1 X134.123 Y128.122 E.05811"   line 108: expected "G1 X134.123 Y128.122 E.05811"; actual "M73 P13 R4" |
| DIVERGENT | Creality/Creality SPARKX i7 0.8 nozzle | first difference at byte 954 (line 46, column 7; expected 88493 bytes, actual 88503 bytes) context:   line 45: "G1 X115 E.3742  F1600"   line 46: expected "M73 P15 R3"; actual "M73 P14 R3"   line 47: "G1 X110 E.3742  F6400" |
| DIVERGENT | Creality/Creality Sermoon V1 0.4 nozzle | first difference at byte 2015 (line 53, column 9; expected 162464 bytes, actual 167626 bytes) context:   line 52: "G1 Z2.0 F3000                          ; Move Z Axis up"   line 53: expected "M73 P3 R6"; actual "M73 P3 R7"   line 54: "G92 E0" |
| ORCA_ERROR | Cubicon/Cubicon xCeler-I 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-iGF9At") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Mini 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-Jq1Jsx") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Plus 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-EhbyNO") |
| PASS | Custom/MyKlipper 0.2 nozzle |  |
| PASS | Custom/MyKlipper 0.4 nozzle |  |
| PASS | Custom/MyKlipper 0.6 nozzle |  |
| PASS | Custom/MyKlipper 0.8 nozzle |  |
| DIVERGENT | Custom/MyMarlin 0.4 nozzle | first difference at byte 5597 (line 250, column 1; expected 93620 bytes, actual 93854 bytes) context:   line 249: "G1 X123.739 Y121.319 E.09959"   line 250: expected "M73 P15 R4"; actual "G1 X123.234 Y121.319 E.01469"   line 251: expected "G1 X123.234 Y121.319 E.01469"; actual "M73 P15 R4" |
| DIVERGENT | Custom/MyRRF 0.4 nozzle | first difference at byte 3390 (line 149, column 1; expected 102569 bytes, actual 103099 bytes) context:   line 148: "G1 X121.55 Y127.904 E.02578"   line 149: expected "M73 P54 R3"; actual "G1 X122.302 Y128.656 E.04098"   line 150: expected "G1 X122.302 Y128.656 E.04098"; actual "M73 P54 R3" |
| PASS | Custom/MyRepetier 0.4 nozzle |  |
| DIVERGENT | Custom/MyToolChanger 0.2 nozzle | first difference at byte 209688 (line 8001, column 1; expected 462943 bytes, actual 462943 bytes) context:   line 8000: "G1 X170.662 Y177.272 E.0618"   line 8001: expected "G1 X170.662 Y176.461 E.00536"; actual "M73 P50 R5"   line 8002: expected "M73 P50 R5"; actual "G1 X170.662 Y176.461 E.00536" |
| ORCA_ERROR | Custom/MyToolChanger 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-RhI87h") |
| DIVERGENT | Custom/MyToolChanger 0.6 nozzle | first difference at byte 4071 (line 184, column 7; expected 117410 bytes, actual 118518 bytes) context:   line 183: ";WIDTH:0.66"   line 184: expected "G1 F2462"; actual "G1 F2479"   line 185: "G1 X171.547 Y178.453 E.34725" |
| DIVERGENT | Custom/MyToolChanger 0.8 nozzle | first difference at byte 3743 (line 172, column 7; expected 125825 bytes, actual 127095 bytes) context:   line 171: ";WIDTH:0.88"   line 172: expected "G1 F1941"; actual "G1 F1954"   line 173: "G1 X172.077 Y177.923 E.39875" |
| PASS | DeltaMaker/DeltaMaker 2 0.35 nozzle |  |
| DIVERGENT | DeltaMaker/DeltaMaker 2T 0.5 nozzle | first difference at byte 4505 (line 201, column 7; expected 134349 bytes, actual 135171 bytes) context:   line 200: ";WIDTH:0.55"   line 201: expected "G1 F1516"; actual "G1 F1524"   line 202: "G1 X-3.714 Y63.714 E.28054" |
| PASS | DeltaMaker/DeltaMaker 2XT 0.5 nozzle |  |
| PASS | Dremel/Dremel 3D20 0.4 nozzle |  |
| PASS | Dremel/Dremel 3D40 0.4 nozzle |  |
| PASS | Dremel/Dremel 3D45 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri 2 0.2 nozzle | first difference at byte 4011 (line 175, column 1; expected 249434 bytes, actual 250208 bytes) context:   line 174: "G1 X124.358 Y125.569 E.00629"   line 175: expected "M73 P8 R8"; actual "G1 X128.931 Y130.142 E.105"   line 176: expected "G1 X128.931 Y130.142 E.105"; actual "G1 X128.544 Y130.142 E.00629" |
| DIVERGENT | Elegoo/Elegoo Centauri 2 0.4 nozzle | first difference at byte 2514 (line 126, column 1; expected 105424 bytes, actual 106814 bytes) context:   line 125: "G1 X131.759 Y127.933 E.2727"   line 126: expected "M73 P14 R3"; actual "G1 X131.759 Y128.597 E.02468"   line 127: expected "G1 X131.759 Y128.597 E.02468"; actual "M73 P14 R3" |
| PASS | Elegoo/Elegoo Centauri 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 0.2 nozzle | first difference at byte 39060 (line 1608, column 1; expected 239263 bytes, actual 239469 bytes) context:   line 1607: "G1 X123.507 Y123.507 E.07051"   line 1608: expected "M73 P18 R7"; actual "G1 X132.493 Y123.507 E.07051"   line 1609: expected "G1 X132.493 Y123.507 E.07051"; actual "M73 P18 R7" |
| PASS | Elegoo/Elegoo Centauri Carbon 0.4 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 0.6 nozzle | first difference at byte 20818 (line 986, column 1; expected 70635 bytes, actual 70846 bytes) context:   line 985: "G1 X123.31 Y123.31 E.61792"   line 986: expected "M73 P51 R1"; actual "G1 X132.69 Y123.31 E.61792"   line 987: expected "G1 X132.69 Y123.31 E.61792"; actual "M73 P51 R1" |
| PASS | Elegoo/Elegoo Centauri Carbon 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.2 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 2 0.4 nozzle | first difference at byte 3132 (line 148, column 1; expected 105974 bytes, actual 106845 bytes) context:   line 147: "G1 X127.439 Y130.259 E.16797"   line 148: expected "M73 P15 R3"; actual "G1 X126.775 Y130.259 E.02468"   line 149: expected "G1 X126.775 Y130.259 E.02468"; actual "M73 P15 R3" |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 2 0.6 nozzle | first difference at byte 5875 (line 275, column 8; expected 69395 bytes, actual 69789 bytes) context:   line 274: ";WIDTH:0.62"   line 275: expected "G1 F2880"; actual "G1 F2888"   line 276: "G1 X123.866 Y130.634 E.54472" |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 2 0.8 nozzle | first difference at byte 6669 (line 308, column 1; expected 56924 bytes, actual 57019 bytes) context:   line 307: "G1 X124.768 Y125.794 E.64635"   line 308: expected "M73 P32 R1"; actual "G1 X124.768 Y126.832 E.1205"   line 309: expected "G1 X124.768 Y126.832 E.1205"; actual "M73 P32 R1" |
| DIVERGENT | Elegoo/Elegoo Neptune 0.4 nozzle | first difference at byte 9742 (line 443, column 8; expected 105328 bytes, actual 105522 bytes) context:   line 442: ";WIDTH:0.45"   line 443: expected "G1 F2164"; actual "G1 F2167"   line 444: "G1 X101.009 Y108.991 E.25682" |
| PASS | Elegoo/Elegoo Neptune 0.6 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 0.8 nozzle | first difference at byte 3160 (line 160, column 8; expected 56065 bytes, actual 56233 bytes) context:   line 159: ";WIDTH:0.82"   line 160: expected "G1 F1143"; actual "G1 F1146"   line 161: "G1 X101.144 Y108.856 E.89502" |
| PASS | Elegoo/Elegoo Neptune 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 2S 0.4 nozzle | first difference at byte 6734 (line 310, column 7; expected 105289 bytes, actual 105865 bytes) context:   line 309: ";WIDTH:0.45"   line 310: expected "G1 F2163"; actual "G1 F2172"   line 311: "G1 X113.509 Y121.491 E.25682" |
| DIVERGENT | Elegoo/Elegoo Neptune 2S 0.6 nozzle | first difference at byte 1827 (line 92, column 1; expected 65893 bytes, actual 66463 bytes) context:   line 91: "G1 E-4.46 F3600"   line 92: expected "M73 P7 R4"; actual ";WIPE_START"   line 93: expected ";WIPE_START"; actual "G1 F12000" |
| DIVERGENT | Elegoo/Elegoo Neptune 2S 0.8 nozzle | first difference at byte 4347 (line 212, column 1; expected 55906 bytes, actual 56250 bytes) context:   line 211: "G1 X118.206 Y114.268 E.64635"   line 212: expected "M73 P15 R3"; actual "G1 X117.168 Y114.268 E.1205"   line 213: expected "G1 X117.168 Y114.268 E.1205"; actual "M73 P15 R3" |
| DIVERGENT | Elegoo/Elegoo Neptune 3 0.4 nozzle | first difference at byte 1920 (line 95, column 1; expected 105115 bytes, actual 105580 bytes) context:   line 94: "G1 X122.15 Y112.85 E.336"   line 95: expected "M73 P4 R7"; actual "G1 X122.15 Y122.11 E.33455"   line 96: expected "G1 X122.15 Y122.11 E.33455"; actual "M73 P4 R7" |
| PASS | Elegoo/Elegoo Neptune 3 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Max 0.2 nozzle | first difference at byte 5416 (line 244, column 7; expected 224757 bytes, actual 224957 bytes) context:   line 243: ";WIPE_END"   line 244: expected "G1 X214.555 Y216.596 Z.65"; actual "G1 X216.011 Y215.976 Z.65"   line 245: "G1 Z.25" |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Max 0.4 nozzle | first difference at byte 6302 (line 294, column 8; expected 90027 bytes, actual 90136 bytes) context:   line 293: ";WIDTH:0.45"   line 294: expected "G1 F2113"; actual "G1 F2118"   line 295: "G1 X208.102 Y216.898 E.28302" |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.2 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Plus 0.4 nozzle | first difference at byte 3625 (line 174, column 7; expected 89917 bytes, actual 90249 bytes) context:   line 173: ";WIDTH:0.45"   line 174: expected "G1 F2078"; actual "G1 F2085"   line 175: "G1 X158.102 Y166.898 E.28302" |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.2 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Pro 0.4 nozzle | first difference at byte 3606 (line 173, column 8; expected 90074 bytes, actual 90218 bytes) context:   line 172: ";WIDTH:0.45"   line 173: expected "G1 F2081"; actual "G1 F2085"   line 174: "G1 X110.602 Y119.398 E.28302" |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.6 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 3 Pro 0.8 nozzle | first difference at byte 2971 (line 153, column 8; expected 50848 bytes, actual 51290 bytes) context:   line 152: ";WIDTH:0.82"   line 153: expected "G1 F1070"; actual "G1 F1078"   line 154: "G1 X111.144 Y118.856 E.89502" |
| PASS | Elegoo/Elegoo Neptune 3 Pro 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.4 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Max 0.6 nozzle | first difference at byte 5381 (line 250, column 8; expected 64073 bytes, actual 64167 bytes) context:   line 249: ";WIDTH:0.62"   line 250: expected "G1 F1396"; actual "G1 F1398"   line 251: "G1 X210.366 Y215.134 E.54472" |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.4 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Plus 0.6 nozzle | first difference at byte 7713 (line 354, column 8; expected 63991 bytes, actual 64202 bytes) context:   line 353: ";WIDTH:0.62"   line 354: expected "G1 F1391"; actual "G1 F1394"   line 355: "G1 X160.366 Y166.634 E.54472" |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.4 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Pro 0.6 nozzle | first difference at byte 5344 (line 253, column 8; expected 63612 bytes, actual 63706 bytes) context:   line 252: ";WIDTH:0.62"   line 253: expected "G1 F1396"; actual "G1 F1398"   line 254: "G1 X113.366 Y122.634 E.54472" |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Pro 0.8 nozzle | first difference at byte 3404 (line 165, column 1; expected 52939 bytes, actual 54046 bytes) context:   line 164: "G1 E-.96 F3600"   line 165: expected "M73 P20 R2"; actual ";WIPE_START"   line 166: expected ";WIPE_START"; actual "G1 F30000" |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Pro 1.0 nozzle | first difference at byte 1441 (line 70, column 1; expected 45385 bytes, actual 45708 bytes) context:   line 69: "G1 Z.4"   line 70: expected "M73 P17 R1"; actual "G1 E1.5 F1800"   line 71: expected "G1 E1.5 F1800"; actual "M73 P17 R1" |
| PASS | Elegoo/Elegoo Neptune X 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune X 0.6 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune X 0.8 nozzle | first difference at byte 2342 (line 116, column 1; expected 56100 bytes, actual 56236 bytes) context:   line 115: "G1 X119.47 Y120.508 E.12058"   line 116: expected "M73 P11 R3"; actual "G1 X114.492 Y115.53 E.81735"   line 117: expected "G1 X114.492 Y115.53 E.81735"; actual "M73 P11 R3" |
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
| PASS | Eryone/Thinker X400 0.2 nozzle |  |
| PASS | Eryone/Thinker X400 0.4 nozzle |  |
| PASS | FLSun/FLSun Q5 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun QQ-S Pro 0.4 nozzle | first difference at byte 8404 (line 376, column 1; expected 103596 bytes, actual 103931 bytes) context:   line 375: "G1 X3.918 Y3.918 Z1"   line 376: expected "M73 P10 R7"; actual "G1 Z.6"   line 377: expected "G1 Z.6"; actual "M73 P10 R7" |
| PASS | FLSun/FLSun S1 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun Super Racer 0.4 nozzle | first difference at byte 2574 (line 105, column 1; expected 104847 bytes, actual 104847 bytes) context:   line 104: "G1 X5 Y10.611 E.08056"   line 105: expected "M73 P5 R7"; actual "G1 X-5 Y10.611 E.33172"   line 106: expected "G1 X-5 Y10.611 E.33172"; actual "G1 X-7.072 Y10.214 E.06996" |
| PASS | FLSun/FLSun T1 0.4 nozzle |  |
| PASS | FLSun/FLSun V400 0.4 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.25 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.4 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.6 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.8 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.4 Nozzle | first difference at byte 5552 (line 276, column 15; expected 106787 bytes, actual 107888 bytes) context:   line 275: "G1 X4.464 Y4.464 F4800"   line 276: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 277: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.6 Nozzle | first difference at byte 4936 (line 253, column 9; expected 71444 bytes, actual 72396 bytes) context:   line 252: "G1 X4.197 Y4.197 F6000"   line 253: expected "G1 X4.196 Y4.164"; actual "G1 X4.197 Y4.197"   line 254: expected "G1 X4.164 Y4.164"; actual "G1 X4.196 Y4.164" |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.3 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series 0.4 Nozzle | first difference at byte 5126 (line 242, column 15; expected 92980 bytes, actual 94238 bytes) context:   line 241: "G1 X4.464 Y4.464 F4800"   line 242: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 243: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.6 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series HS Nozzle | first difference at byte 5404 (line 256, column 15; expected 103767 bytes, actual 105025 bytes) context:   line 255: "G1 X4.464 Y4.464 F9000"   line 256: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 257: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| PASS | Flashforge/Flashforge Adventurer 5M 0.25 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 5M 0.4 Nozzle | first difference at byte 6412 (line 306, column 8; expected 101878 bytes, actual 102200 bytes) context:   line 305: ";WIDTH:0.45"   line 306: expected "G1 F2763"; actual "G1 F2766"   line 307: "G1 X-4.355 Y4.355 E.28893" |
| PASS | Flashforge/Flashforge Adventurer 5M 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.25 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 5M Pro 0.4 Nozzle | first difference at byte 36813 (line 1861, column 1; expected 102048 bytes, actual 102209 bytes) context:   line 1860: "G1 X4.79 Y4.75 E.29314"   line 1861: expected "M73 P51 R2"; actual "SET_VELOCITY_LIMIT ACCEL=10000"   line 1862: expected "SET_VELOCITY_LIMIT ACCEL=10000"; actual "G1 E-.8 F2100" |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.8 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Artemis 0.4 Nozzle | first difference at byte 5282 (line 252, column 15; expected 110781 bytes, actual 112430 bytes) context:   line 251: "G1 X4.458 Y4.458 F6000"   line 252: expected "G1 X4.458 Y4.439"; actual "G1 X4.458 Y4.458"   line 253: expected "G1 X4.439 Y4.439"; actual "G1 X4.458 Y4.439" |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-yCQgC7") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-Kr1w34") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-gq2sfd") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-Vjd8gL") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-h4O2xU") |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.8 nozzle |  |
| PASS | Flashforge/Flashforge Guider 2s 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.25 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 0.4 HF nozzle | first difference at byte 7528 (line 327, column 7; expected 121969 bytes, actual 122285 bytes) context:   line 326: ";WIDTH:0.45"   line 327: expected "G1 F4389"; actual "G1 F4395"   line 328: "G1 X145.602 Y154.398 E.29177" |
| DIVERGENT | Flashforge/Flashforge Guider4 0.4 nozzle | first difference at byte 6779 (line 290, column 1; expected 115806 bytes, actual 116029 bytes) context:   line 289: "G1 E-2.5 F2700"   line 290: expected ";WIPE_START"; actual "M73 P25 R3"   line 291: expected "G1 F15000"; actual ";WIPE_START" |
| DIVERGENT | Flashforge/Flashforge Guider4 0.6 HF nozzle | first difference at byte 4315 (line 196, column 1; expected 81790 bytes, actual 82170 bytes) context:   line 195: "G1 X150.997 Y153.617 E.04526"   line 196: expected "G1 X153.617 Y150.997 E.20935"; actual "M73 P23 R4"   line 197: expected "G1 X153.617 Y150.196 E.04526"; actual "G1 X153.617 Y150.997 E.20935" |
| PASS | Flashforge/Flashforge Guider4 0.6 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 0.8 HF nozzle | first difference at byte 1237 (line 61, column 1; expected 61365 bytes, actual 61771 bytes) context:   line 60: "G1 E1.5 F1800"   line 61: expected "SET_VELOCITY_LIMIT ACCEL=800"; actual "M73 P26 R2"   line 62: expected ";TYPE:Inner wall"; actual "SET_VELOCITY_LIMIT ACCEL=800" |
| PASS | Flashforge/Flashforge Guider4 Pro 0.25 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.4 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.6 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.6 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 Pro 0.8 HF nozzle | first difference at byte 6635 (line 292, column 1; expected 61525 bytes, actual 61785 bytes) context:   line 291: "G1 X147.055 Y153.057 E.00452"   line 292: expected ";LAYER_CHANGE"; actual "SET_VELOCITY_LIMIT ACCEL=20000"   line 293: expected ";Z:1.26"; actual "G1 X152.969 Y146.986 F33000" |
| PASS | FlyingBear/FlyingBear Ghost 6 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear Ghost7 0.4 nozzle |  |
| DIVERGENT | FlyingBear/FlyingBear Reborn3 0.4 nozzle | first difference at byte 3725 (line 169, column 8; expected 113340 bytes, actual 113446 bytes) context:   line 168: ";WIDTH:0.45"   line 169: expected "G1 F1997"; actual "G1 F1999"   line 170: "G1 X145.602 Y154.398 E.27987" |
| DIVERGENT | FlyingBear/FlyingBear S1 0.4 nozzle | first difference at byte 7667 (line 333, column 8; expected 105893 bytes, actual 106574 bytes) context:   line 332: ";WIDTH:0.45"   line 333: expected "G1 F2001"; actual "G1 F2006"   line 334: "G1 X105.602 Y114.398 E.27987" |
| PASS | Folgertech/Folgertech FT-5 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-5 0.6 nozzle |  |
| PASS | Folgertech/Folgertech FT-6 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-6 0.6 nozzle |  |
| DIVERGENT | Folgertech/Folgertech i3 0.4 nozzle | first difference at byte 6896 (line 317, column 5; expected 93773 bytes, actual 94049 bytes) context:   line 316: ";WIDTH:0.45"   line 317: expected "G1 F5856"; actual "G1 F6000"   line 318: "G1 X95.625 Y104.375 E.29025" |
| PASS | Folgertech/Folgertech i3 0.6 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A10 M 0.4 nozzle | first difference at byte 4709 (line 199, column 1; expected 111325 bytes, actual 112135 bytes) context:   line 198: "G1 X106.032 Y106.032 E.26325"   line 199: expected "M73 P6 R5"; actual "G1 X113.968 Y106.032 E.26325"   line 200: expected "G1 X113.968 Y106.032 E.26325"; actual "M73 P6 R5" |
| DIVERGENT | Geeetech/Geeetech A10 Pro 0.2 nozzle | first difference at byte 14924 (line 616, column 7; expected 315782 bytes, actual 315905 bytes) context:   line 615: "M204 S700"   line 616: expected "G1 X112.087 Y105.926 F9000"; actual "G1 X113.503 Y106.489 F9000"   line 617: "M204 S500" |
| DIVERGENT | Geeetech/Geeetech A10 Pro 0.4 nozzle | first difference at byte 6983 (line 295, column 1; expected 111119 bytes, actual 111455 bytes) context:   line 294: "G1 X107.392 Y106.166 E.04387"   line 295: expected "; stop printing object cube10.stl id:0 copy 0"; actual "M204 S700"   line 296: expected ";LAYER_CHANGE"; actual "G1 X106.427 Y106.369 F9000" |
| PASS | Geeetech/Geeetech A10 Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A10 T 0.4 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A20 0.2 nozzle | first difference at byte 16405 (line 686, column 1; expected 315861 bytes, actual 316102 bytes) context:   line 685: "G1 X129.67 Y120.33 E.07555"   line 686: expected "M73 P7 R15"; actual "G1 X129.67 Y129.65 E.07539"   line 687: expected "G1 X129.67 Y129.65 E.07539"; actual "M73 P7 R15" |
| DIVERGENT | Geeetech/Geeetech A20 0.4 nozzle | first difference at byte 7031 (line 296, column 1; expected 111349 bytes, actual 111663 bytes) context:   line 295: "G1 X122.392 Y121.166 E.04387"   line 296: expected "; stop printing object cube10.stl id:0 copy 0"; actual "M204 S700"   line 297: expected ";LAYER_CHANGE"; actual "G1 X121.427 Y121.369 F9000" |
| PASS | Geeetech/Geeetech A20 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A20 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 T 0.4 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A30 M 0.4 nozzle | first difference at byte 2621 (line 109, column 1; expected 111108 bytes, actual 111715 bytes) context:   line 108: "G1 F900"   line 109: expected "M73 P4 R6"; actual "G1 X155.9 Y164.1 E.30542"   line 110: expected "G1 X155.9 Y164.1 E.30542"; actual "G1 X155.9 Y155.9 E.30542" |
| PASS | Geeetech/Geeetech A30 Pro 0.2 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A30 Pro 0.4 nozzle | first difference at byte 5331 (line 232, column 7; expected 111161 bytes, actual 111453 bytes) context:   line 231: "M204 S700"   line 232: expected "G1 X161.843 Y163.834 F9000"; actual "G1 X163.245 Y163.182 F9000"   line 233: "M204 S500" |
| PASS | Geeetech/Geeetech A30 Pro 0.6 nozzle |  |
| DIVERGENT | Geeetech/Geeetech A30 Pro 0.8 nozzle | first difference at byte 3024 (line 127, column 1; expected 57958 bytes, actual 58322 bytes) context:   line 126: "G1 E-5.66667 F1200"   line 127: expected "M73 P9 R3"; actual ";WIPE_START"   line 128: expected ";WIPE_START"; actual "G1 F900" |
| PASS | Geeetech/Geeetech A30 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.2 nozzle |  |
| DIVERGENT | Geeetech/Geeetech M1 0.4 nozzle | first difference at byte 4460 (line 185, column 1; expected 105839 bytes, actual 106300 bytes) context:   line 184: "G1 X49.461 Y56.482 E.01559"   line 185: expected "M73 P7 R3"; actual "G1 X48.518 Y55.539 E.03997"   line 186: expected "G1 X48.518 Y55.539 E.03997"; actual "G1 X48.518 Y56.059 E.01559" |
| DIVERGENT | Geeetech/Geeetech M1 0.6 nozzle | first difference at byte 7721 (line 343, column 1; expected 70808 bytes, actual 71073 bytes) context:   line 342: "G1 X49.413 Y52.606 E.28624"   line 343: expected "M73 P16 R2"; actual "G1 X49.413 Y53.392 E.05336"   line 344: expected "G1 X49.413 Y53.392 E.05336"; actual "M73 P16 R2" |
| PASS | Geeetech/Geeetech M1 0.8 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar 0.2 nozzle | first difference at byte 33068 (line 1416, column 1; expected 316016 bytes, actual 316128 bytes) context:   line 1415: "G1 X114.89 Y105.11 E.07911"   line 1416: expected "M73 P14 R14"; actual "G1 X114.89 Y114.87 E.07895"   line 1417: expected "G1 X114.89 Y114.87 E.07895"; actual "M73 P14 R14" |
| DIVERGENT | Geeetech/Geeetech Mizar 0.4 nozzle | first difference at byte 5661 (line 248, column 1; expected 111354 bytes, actual 111657 bytes) context:   line 247: "G1 X113.674 Y110.489 E.01469"   line 248: expected "M73 P8 R6"; actual "G1 X110.489 Y113.674 E.13106"   line 249: expected "G1 X110.489 Y113.674 E.13106"; actual "M73 P8 R6" |
| PASS | Geeetech/Geeetech Mizar 0.6 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar 0.8 nozzle | first difference at byte 4026 (line 171, column 2; expected 57857 bytes, actual 58514 bytes) context:   line 170: "G1 E7 F1200"   line 171: expected "M73 P10 R3"; actual "M204 S500"   line 172: expected "M204 S500"; actual ";TYPE:Inner wall" |
| DIVERGENT | Geeetech/Geeetech Mizar M 0.4 nozzle | first difference at byte 5323 (line 231, column 6; expected 110780 bytes, actual 111689 bytes) context:   line 230: "M204 S700"   line 231: expected "G1 X129.343 Y131.334 F9000"; actual "G1 X130.745 Y130.682 F9000"   line 232: "M204 S500" |
| DIVERGENT | Geeetech/Geeetech Mizar Max 0.2 nozzle | first difference at byte 5460 (line 218, column 1; expected 315505 bytes, actual 315981 bytes) context:   line 217: "G1 X156.153 Y161.327 E.00301"   line 218: expected "M73 P3 R16"; actual "G1 X158.673 Y163.847 E.03319"   line 219: expected "G1 X158.673 Y163.847 E.03319"; actual "G1 X158.35 Y163.847 E.00301" |
| PASS | Geeetech/Geeetech Mizar Max 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.2 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar Pro 0.4 nozzle | first difference at byte 3743 (line 157, column 1; expected 110962 bytes, actual 111725 bytes) context:   line 156: "G1 X111.956 Y113.3 E.28786"   line 157: expected "M73 P6 R6"; actual "G1 X111.284 Y113.3 E.02603"   line 158: expected "G1 X111.284 Y113.3 E.02603"; actual "M73 P6 R6" |
| PASS | Geeetech/Geeetech Mizar Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.8 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar S 0.2 nozzle | first difference at byte 54428 (line 2284, column 1; expected 316240 bytes, actual 316240 bytes) context:   line 2283: "G1 X125.256 Y128.005 E.00324"   line 2284: expected "M73 P19 R13"; actual "G1 X125.7 Y126.995 E.00893"   line 2285: expected "G1 X125.7 Y126.995 E.00893"; actual "M73 P19 R13" |
| PASS | Geeetech/Geeetech Mizar S 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.8 nozzle |  |
| DIVERGENT | Ginger Additive/Ginger G1 1.2 nozzle | first difference at byte 596 (line 19, column 9; expected 39732 bytes, actual 39065 bytes) context:   line 18: "EXCLUDE_OBJECT_DEFINE NAME=cube10.stl_id_0_copy_0 CENTER=500,500 POLYGON=[[495,495],[505,495],[505,505],[495,505],[495,495]]"   line 19: expected "M73 P0 R10"; actual "M73 P0 R5"   line 20: ";TYPE:Custom" |
| PASS | Ginger Additive/Ginger G1 3.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 5.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 8.0 nozzle |  |
| DIVERGENT | InfiMech/InfiMech EX 0.4 nozzle | first difference at byte 27098 (line 1272, column 1; expected 102092 bytes, actual 102399 bytes) context:   line 1271: "G1 X129.398 Y120.602 E.29177"   line 1272: expected "M73 P35 R3"; actual "G1 X129.398 Y129.358 E.29045"   line 1273: expected "G1 X129.398 Y129.358 E.29045"; actual "M73 P35 R3" |
| PASS | InfiMech/InfiMech EX+APS 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX 0.4 nozzle |  |
| DIVERGENT | InfiMech/InfiMech TX HSN 0.4 nozzle | first difference at byte 4688 (line 208, column 8; expected 106453 bytes, actual 106650 bytes) context:   line 207: ";WIDTH:0.45"   line 208: expected "G1 F1996"; actual "G1 F1999"   line 209: "G1 X105.602 Y114.398 E.27987" |
| DIVERGENT | Kingroon/Kingroon KLP1 0.4 nozzle | first difference at byte 4153 (line 169, column 1; expected 111929 bytes, actual 112310 bytes) context:   line 168: "G1 X112.801 Y111.507 E.30018"   line 169: expected "M73 P7 R3"; actual "G1 X112.154 Y111.507 E.02414"   line 170: expected "G1 X112.154 Y111.507 E.02414"; actual "G1 X118.493 Y117.846 E.33432" |
| DIVERGENT | Kingroon/Kingroon KP3S 3.0 0.4 nozzle | first difference at byte 513 (line 21, column 9; expected 92286 bytes, actual 90491 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R3"; actual "M73 P0 R2"   line 22: "M201 X4000 Y4000 Z1100 E10000" |
| PASS | Kingroon/Kingroon KP3S PRO S1 0.4 nozzle |  |
| DIVERGENT | Kingroon/Kingroon KP3S PRO V2 0.4 nozzle | first difference at byte 940 (line 39, column 3; expected 103623 bytes, actual 103327 bytes) context:   line 38: ""   line 39: expected "G10 ; retract"; actual "G1 E-.8 F2700"   line 40: ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Kingroon/Kingroon KP3S V1 0.4 nozzle | first difference at byte 5236 (line 216, column 7; expected 107514 bytes, actual 107939 bytes) context:   line 215: ";WIDTH:0.45"   line 216: expected "G1 F4056"; actual "G1 F4075"   line 217: expected "G1 X94.398 Y85.602 E.29177"; actual "G1 X85.602 Y94.398 E.29177" |
| DIVERGENT | LH/LH Stinger 0.4 nozzle | first difference at byte 3855 (line 159, column 8; expected 134563 bytes, actual 135127 bytes) context:   line 158: ";WIDTH:0.4"   line 159: expected "G1 F2133"; actual "G1 F2136"   line 160: "G1 X113.2 Y142.8 E.27078" |
| DIVERGENT | LH/LH Stinger MMU 0.4 nozzle | first difference at byte 627 (line 23, column 1; expected 135321 bytes, actual 135322 bytes) context:   line 22: ";TYPE:Custom"   line 23: expected "_SP_PRINT_START LANE=0 TEMP=230"; actual " _SP_PRINT_START LANE=0 TEMP=230"   line 24: "" |
| PASS | LONGER/LONGER LK10 (0.2 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 (0.4 nozzle) | first difference at byte 5925 (line 228, column 7; expected 124292 bytes, actual 124559 bytes) context:   line 227: ";WIDTH:0.45"   line 228: expected "G1 F5311"; actual "G1 F5322"   line 229: "G1 X108.532 Y116.468 E.26862" |
| DIVERGENT | LONGER/LONGER LK10 (0.6 nozzle) | first difference at byte 12651 (line 504, column 1; expected 109076 bytes, actual 109525 bytes) context:   line 503: "G1 E1 F2100"   line 504: expected "M73 P11 R3"; actual ";TYPE:Sparse infill"   line 505: expected ";TYPE:Sparse infill"; actual ";WIDTH:0.65" |
| DIVERGENT | LONGER/LONGER LK10 (0.8 nozzle) | first difference at byte 51264 (line 2164, column 29; expected 103095 bytes, actual 103095 bytes) context:   line 2163: "G1 X110.137 Y110.137 E-.05789"   line 2164: expected "G1 X110.944 Y110.137 E-.24211"; actual "G1 X110.944 Y110.137 E-.24212"   line 2165: ";WIPE_END" |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.2 nozzle) | first difference at byte 9517 (line 363, column 1; expected 191824 bytes, actual 191918 bytes) context:   line 362: "G1 X161.426 Y164.288 E.00504"   line 363: expected "M73 P7 R3"; actual "G1 X164.288 Y161.426 E.06968"   line 364: expected "G1 X164.288 Y161.426 E.06968"; actual "G1 X164.288 Y161.133 E.00504" |
| PASS | LONGER/LONGER LK10 Plus (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.8 nozzle) | first difference at byte 16056 (line 659, column 1; expected 102665 bytes, actual 103059 bytes) context:   line 658: "G1 X157.637 Y157.637 E.44845"   line 659: expected "M73 P16 R2"; actual "G1 X160.866 Y157.637 E.21665"   line 660: expected "G1 X160.866 Y157.637 E.21665"; actual "M73 P16 R2" |
| PASS | Lulzbot/Lulzbot Taz 4 or 5 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz 6 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz Pro Dual 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz Pro S 0.5 nozzle |  |
| PASS | M3D/M3D Enabler D8500 MM |  |
| PASS | MagicMaker/MM BoneKing 0.4 nozzle |  |
| DIVERGENT | MagicMaker/MM hj SK 0.4 nozzle | first difference at byte 8940 (line 375, column 6; expected 200723 bytes, actual 203085 bytes) context:   line 374: ";WIDTH:0.399999"   line 375: expected "G1 F5191"; actual "G1 F5257"   line 376: "G1 X111.685 Y101.316 E.04733" |
| DIVERGENT | MagicMaker/MM hqs SF 0.4 nozzle | first difference at byte 1076 (line 45, column 9; expected 178583 bytes, actual 180583 bytes) context:   line 44: "G1 F9000"   line 45: expected "M73 P3 R7"; actual "M73 P3 R8"   line 46: "M117 Printing..." |
| PASS | MagicMaker/MM hqs hj 0.4 nozzle |  |
| PASS | MagicMaker/MM slb 0.4 nozzle |  |
| DIVERGENT | Mellow/M1 0.2 nozzle | first difference at byte 375170 (line 15639, column 62; expected 395090 bytes, actual 395090 bytes) context:   line 15638: "; estimated printing time (normal mode) = 10m 11s"   line 15639: expected "; estimated first layer printing time (normal mode) = 0.438319s"; actual "; estimated first layer printing time (normal mode) = 0.438318s"   line 15640: "" |
| PASS | Mellow/M1 0.4 nozzle |  |
| DIVERGENT | Mellow/M1 0.6 nozzle | first difference at byte 3403 (line 152, column 7; expected 100607 bytes, actual 101062 bytes) context:   line 151: ";WIDTH:0.66"   line 152: expected "G1 F2440"; actual "G1 F2456"   line 153: "G1 X49.547 Y56.453 E.34725" |
| PASS | Mellow/M1 0.8 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.2 nozzle |  |
| DIVERGENT | OpenEYE/OpenEYE Peacock V2 0.4 nozzle | first difference at byte 8255 (line 298, column 8; expected 126744 bytes, actual 127362 bytes) context:   line 297: ";WIDTH:0.45"   line 298: expected "G1 F4023"; actual "G1 F4029"   line 299: "G1 X113.895 Y125.355 E.28893" |
| PASS | OpenEYE/OpenEYE Peacock V2 0.6 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.8 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.2 nozzle |  |
| DIVERGENT | OrcaArena/Orca Arena X1 Carbon 0.4 nozzle | first difference at byte 6621 (line 279, column 7; expected 112319 bytes, actual 112429 bytes) context:   line 278: ";WIDTH:0.45"   line 279: expected "G1 F3988"; actual "G1 F3994"   line 280: "G1 X124.052 Y131.948 E.26192" |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.6 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.8 nozzle |  |
| DIVERGENT | Peopoly/Peopoly Magneto X 0.4 nozzle | first difference at byte 185662 (line 6677, column 62; expected 206009 bytes, actual 206009 bytes) context:   line 6676: "; estimated printing time (normal mode) = 7m 15s"   line 6677: expected "; estimated first layer printing time (normal mode) = 0.874670s"; actual "; estimated first layer printing time (normal mode) = 0.874671s"   line 6678: "" |
| PASS | Peopoly/Peopoly Magneto X 0.6 nozzle |  |
| DIVERGENT | Peopoly/Peopoly Magneto X 0.8 nozzle | first difference at byte 12169 (line 442, column 7; expected 204482 bytes, actual 204677 bytes) context:   line 441: ";WIDTH:0.22"   line 442: expected "G1 F4359"; actual "G1 F4362"   line 443: "G1 X145.507 Y204.493 E.12172" |
| PASS | Phrozen/Phrozen Arco 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.2 nozzle |  |
| PASS | Positron3D/The Positron 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.6 nozzle |  |
| PASS | Positron3D/The Positron 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.25 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One 0.4 nozzle | first difference at byte 6667 (line 300, column 1; expected 96233 bytes, actual 96233 bytes) context:   line 299: "G1 X121.021 Y111.083 E.24231"   line 300: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 301: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa CORE One 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One HF 0.4 nozzle | first difference at byte 6631 (line 297, column 1; expected 95737 bytes, actual 95737 bytes) context:   line 296: "G1 X121.021 Y111.083 E.24231"   line 297: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 298: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| DIVERGENT | Prusa/Prusa CORE One HF 0.5 nozzle | first difference at byte 7781 (line 358, column 2; expected 95669 bytes, actual 96309 bytes) context:   line 357: "G1 X129.725 Y114.675 E.39634"   line 358: expected "M73 P86 R4"; actual "M204 T7000"   line 359: expected "M204 T7000"; actual "G1 E-.56 F2700" |
| DIVERGENT | Prusa/Prusa CORE One HF 0.6 nozzle | first difference at byte 4003 (line 186, column 1; expected 70313 bytes, actual 71781 bytes) context:   line 185: "G1 X121.698 Y110.771 E.04817"   line 186: expected "M73 P88 R4"; actual "G1 X124.229 Y113.302 E.1905"   line 187: expected "G1 X124.229 Y113.302 E.1905"; actual "G1 X123.324 Y113.302 E.04817" |
| DIVERGENT | Prusa/Prusa CORE One HF 0.8 nozzle | first difference at byte 6709 (line 323, column 1; expected 54079 bytes, actual 54810 bytes) context:   line 322: "G1 X125.695 Y107.042 E.17816"   line 323: expected "M73 P93 R2"; actual "G1 X127.958 Y109.305 E.46325"   line 324: expected "G1 X127.958 Y109.305 E.46325"; actual "G1 X127.958 Y110.536 E.17816" |
| DIVERGENT | Prusa/Prusa CORE One L 0.4 nozzle | first difference at byte 7475 (line 343, column 6; expected 96077 bytes, actual 96077 bytes) context:   line 342: ";WIDTH:0.45"   line 343: expected "G1 F1912"; actual "G1 F1899"   line 344: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L 0.5 nozzle | first difference at byte 514 (line 21, column 9; expected 90052 bytes, actual 90702 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L 0.6 nozzle | first difference at byte 2106 (line 106, column 7; expected 75897 bytes, actual 75886 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P89 R3"; actual "M73 P88 R3"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L 0.8 nozzle | first difference at byte 2106 (line 106, column 7; expected 51843 bytes, actual 51842 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.4 nozzle | first difference at byte 7439 (line 340, column 6; expected 95609 bytes, actual 95609 bytes) context:   line 339: ";WIDTH:0.45"   line 340: expected "G1 F1912"; actual "G1 F1899"   line 341: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.5 nozzle | first difference at byte 514 (line 21, column 9; expected 89688 bytes, actual 89697 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.6 nozzle | first difference at byte 2107 (line 106, column 7; expected 66982 bytes, actual 67235 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P91 R2"; actual "M73 P90 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.8 nozzle | first difference at byte 2106 (line 106, column 7; expected 51902 bytes, actual 51901 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa MINI 0.25 nozzle | first difference at byte 5535 (line 261, column 1; expected 142049 bytes, actual 142190 bytes) context:   line 260: "G1 X85.125 Y85.125 E.16453"   line 261: expected "M73 P39 R7"; actual "G1 X94.875 Y85.125 E.16453"   line 262: expected "G1 X94.875 Y85.125 E.16453"; actual "M73 P39 R7" |
| PASS | Prusa/Prusa MINI 0.4 nozzle |  |
| DIVERGENT | Prusa/Prusa MINI 0.6 nozzle | first difference at byte 2623 (line 127, column 1; expected 89647 bytes, actual 89826 bytes) context:   line 126: "G1 X92.735 Y92.223 E.06048"   line 127: expected "M73 P40 R6"; actual "G1 X87.777 Y87.265 E.41393"   line 128: expected "G1 X87.777 Y87.265 E.41393"; actual "M73 P40 R6" |
| PASS | Prusa/Prusa MINI 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MINIIS 0.25 nozzle | first difference at byte 21777 (line 975, column 1; expected 215474 bytes, actual 215474 bytes) context:   line 974: "G1 X89.28 Y94.131 E.00421"   line 975: expected "M73 P29 R13"; actual "G1 X94.131 Y89.28 E.0836"   line 976: expected "G1 X94.131 Y89.28 E.0836"; actual "M73 P29 R13" |
| DIVERGENT | Prusa/Prusa MINIIS 0.4 nozzle | first difference at byte 2782 (line 133, column 1; expected 107084 bytes, actual 107307 bytes) context:   line 132: "G1 X92.051 Y86.75 E.06609"   line 133: expected "M73 P35 R8"; actual "G1 X91.388 Y86.75 E.02581"   line 134: expected "G1 X91.388 Y86.75 E.02581"; actual "M73 P35 R8" |
| PASS | Prusa/Prusa MINIIS 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MINIIS 0.8 nozzle | first difference at byte 2661 (line 128, column 1; expected 55005 bytes, actual 55310 bytes) context:   line 127: "G1 X88.764 Y87.402 E.65163"   line 128: expected "M73 P53 R3"; actual "G1 X87.402 Y87.402 E.16377"   line 129: expected "G1 X87.402 Y87.402 E.16377"; actual "M73 P53 R3" |
| PASS | Prusa/Prusa MK3.5 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3.5 0.4 nozzle | first difference at byte 4896 (line 228, column 1; expected 111095 bytes, actual 111095 bytes) context:   line 227: "G1 X129.775 Y100.225 E.32326"   line 228: expected "M73 P72 R8"; actual "G1 X129.775 Y109.735 E.3219"   line 229: expected "G1 X129.775 Y109.735 E.3219"; actual "M73 P72 R8" |
| PASS | Prusa/Prusa MK3.5 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3.5 0.8 nozzle | first difference at byte 2392 (line 111, column 2; expected 55761 bytes, actual 56714 bytes) context:   line 110: "G1 X129.3 Y109.3 F18000"   line 111: expected "M73 P84 R3"; actual "M205 X7 Y7"   line 112: expected "M205 X7 Y7"; actual ";TYPE:Outer wall" |
| PASS | Prusa/Prusa MK3S 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.4 nozzle | first difference at byte 55392 (line 2695, column 1; expected 95415 bytes, actual 95415 bytes) context:   line 2694: "G1 X129.368 Y109.328 E.28846"   line 2695: expected "M73 P85 R1"; actual "; stop printing object cube10.stl id:0 copy 0"   line 2696: expected "; stop printing object cube10.stl id:0 copy 0"; actual ";LAYER_CHANGE" |
| PASS | Prusa/Prusa MK3S 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.8 nozzle | first difference at byte 3732 (line 178, column 1; expected 64861 bytes, actual 64861 bytes) context:   line 177: "G1 X120 Y97.582 E.10208"   line 178: expected "G1 X130 Y97.582 E1.02139"; actual "M73 P6 R4"   line 179: expected "M73 P6 R4"; actual "G1 X130 Y97.582 E1.02139" |
| PASS | Prusa/Prusa MK4 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4 0.4 nozzle |  |
| PASS | Prusa/Prusa MK4 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4 0.8 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S 0.4 nozzle | first difference at byte 4002 (line 188, column 1; expected 92409 bytes, actual 92830 bytes) context:   line 187: "G1 X124.443 Y108.707 E.17184"   line 188: expected "M73 P84 R5"; actual "G1 X123.787 Y108.707 E.0253"   line 189: expected "G1 X123.787 Y108.707 E.0253"; actual "M73 P84 R5" |
| PASS | Prusa/Prusa MK4S 0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S HF0.4 nozzle | first difference at byte 7240 (line 326, column 7; expected 92234 bytes, actual 92234 bytes) context:   line 325: ";WIDTH:0.45"   line 326: expected "G1 F2560"; actual "G1 F2544"   line 327: "G1 X120.675 Y109.325 E.29279" |
| PASS | Prusa/Prusa MK4S HF0.5 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S HF0.6 nozzle | first difference at byte 2804 (line 140, column 1; expected 68251 bytes, actual 69103 bytes) context:   line 139: "G1 X129.46 Y100.54 E.47252"   line 140: expected "M73 P86 R4"; actual "G1 X129.46 Y109.4 E.46934"   line 141: expected "G1 X129.46 Y109.4 E.46934"; actual "M73 P86 R4" |
| DIVERGENT | Prusa/Prusa MK4S HF0.8 nozzle | first difference at byte 3276 (line 161, column 1; expected 51314 bytes, actual 51419 bytes) context:   line 160: "G1 X124.861 Y107.582 E.27641"   line 161: expected "G1 X123.501 Y107.582 E.1088"; actual "M73 P91 R2"   line 162: expected "M73 P91 R2"; actual "G1 X123.501 Y107.582 E.1088" |
| PASS | Prusa/Prusa XL 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.3 nozzle | first difference at byte 2216 (line 81, column 1; expected 125488 bytes, actual 125536 bytes) context:   line 80: "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"   line 81: expected "M73 P77 R9"; actual "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"   line 82: expected "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"; actual "G92 E0 ; reset extruder position" |
| DIVERGENT | Prusa/Prusa XL 0.4 nozzle | first difference at byte 3176 (line 135, column 1; expected 93107 bytes, actual 93155 bytes) context:   line 134: "G1 E.8 F1800"   line 135: expected "M73 P83 R6"; actual ";TYPE:Bottom surface"   line 136: expected ";TYPE:Bottom surface"; actual ";WIDTH:0.50675" |
| PASS | Prusa/Prusa XL 0.5 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.6 nozzle | first difference at byte 2273 (line 82, column 1; expected 70474 bytes, actual 70759 bytes) context:   line 81: "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"   line 82: expected "M73 P87 R4"; actual "G92 E0 ; reset extruder position"   line 83: expected "G92 E0 ; reset extruder position"; actual "G90" |
| DIVERGENT | Prusa/Prusa XL 0.8 nozzle | first difference at byte 4422 (line 198, column 22; expected 53386 bytes, actual 53824 bytes) context:   line 197: ";WIPE_END"   line 198: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 199: expected "G1 E.6 F1800"; actual "G1 Z.6" |
| PASS | Prusa/Prusa XL 5T 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 5T 0.3 nozzle | first difference at byte 7601 (line 353, column 1; expected 136093 bytes, actual 136141 bytes) context:   line 352: "G1 X183.94 Y176.494 E.01038"   line 353: expected "M73 P78 R8"; actual "G1 X176.494 Y183.94 E.26012"   line 354: expected "G1 X176.494 Y183.94 E.26012"; actual "G1 X176.074 Y183.94 E.01038" |
| DIVERGENT | Prusa/Prusa XL 5T 0.4 nozzle | first difference at byte 3333 (line 168, column 1; expected 103099 bytes, actual 103749 bytes) context:   line 167: "G1 X184.55 Y184.55 F24000"   line 168: expected "M73 P83 R6"; actual ";TYPE:Outer wall"   line 169: expected ";TYPE:Outer wall"; actual "G1 F1238" |
| PASS | Prusa/Prusa XL 5T 0.5 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 5T 0.6 nozzle | first difference at byte 2568 (line 125, column 1; expected 81138 bytes, actual 81336 bytes) context:   line 124: "G0 X70 E9 F800 ; continue purging and wipe the nozzle"   line 125: expected "M73 P87 R4"; actual "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"   line 126: expected "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"; actual "M73 P87 R4" |
| DIVERGENT | Prusa/Prusa XL 5T 0.8 nozzle | first difference at byte 4856 (line 248, column 22; expected 64354 bytes, actual 64400 bytes) context:   line 247: ";WIPE_END"   line 248: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 249: expected "G1 E.8 F1800"; actual "G1 Z.6" |
| PASS | Qidi/Qidi Q1 Pro 0.2 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.4 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.6 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.8 nozzle |  |
| DIVERGENT | Qidi/Qidi Q2 0.2 nozzle | first difference at byte 2927 (line 124, column 1; expected 489615 bytes, actual 489604 bytes) context:   line 123: "G1 Z.1"   line 124: expected "G1 E.4 F1800"; actual "M73 P5 R23"   line 125: expected "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250"; actual "G1 E.4 F1800" |
| DIVERGENT | Qidi/Qidi Q2 0.4 nozzle | first difference at byte 4564 (line 196, column 7; expected 127082 bytes, actual 128393 bytes) context:   line 195: ";WIDTH:0.45"   line 196: expected "G1 F2047"; actual "G1 F2070"   line 197: "G1 X130.602 Y139.398 E.29177" |
| PASS | Qidi/Qidi Q2 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2 0.8 nozzle |  |
| DIVERGENT | Qidi/Qidi Q2C 0.2 nozzle | first difference at byte 13144 (line 515, column 1; expected 480168 bytes, actual 480272 bytes) context:   line 514: "G1 X130.907 Y133.295 E.00148"   line 515: expected "G1 X136.705 Y139.093 E.04151"; actual "M73 P5 R24"   line 516: expected "M73 P5 R24"; actual "G1 X136.705 Y139.093 E.04151" |
| PASS | Qidi/Qidi Q2C 0.4 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.8 nozzle |  |
| PASS | Qidi/Qidi X-CF Pro 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 0.4 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Max 3 0.2 nozzle | first difference at byte 6387 (line 252, column 7; expected 299302 bytes, actual 299357 bytes) context:   line 251: ";WIDTH:0.22"   line 252: expected "G1 F3843"; actual "G1 F3824"   line 253: "G1 X158.206 Y166.794 E.06948" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.4 nozzle | first difference at byte 15161 (line 656, column 1; expected 121136 bytes, actual 121138 bytes) context:   line 655: "G1 X167.29 Y157.71 E.29437"   line 656: expected "M73 P52 R5"; actual "G1 X167.29 Y167.23 E.29252"   line 657: expected "G1 X167.29 Y167.23 E.29252"; actual "M73 P52 R5" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.6 nozzle | first difference at byte 3973 (line 173, column 1; expected 75688 bytes, actual 75690 bytes) context:   line 172: "G1 Z.6"   line 173: expected "M73 P58 R3"; actual "G1 E1.4 F1800"   line 174: expected "G1 E1.4 F1800"; actual "M73 P58 R3" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.8 nozzle | first difference at byte 2982 (line 134, column 1; expected 69152 bytes, actual 69154 bytes) context:   line 133: "G1 X161.32 Y165.655 E.34889"   line 134: expected "M73 P63 R2"; actual "G1 X160.236 Y165.655 E.13536"   line 135: expected "G1 X160.236 Y165.655 E.13536"; actual "M73 P63 R2" |
| PASS | Qidi/Qidi X-Max 4 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 0.4 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Plus 3 0.2 nozzle | first difference at byte 6345 (line 252, column 7; expected 299269 bytes, actual 299323 bytes) context:   line 251: ";WIDTH:0.22"   line 252: expected "G1 F3843"; actual "G1 F3824"   line 253: "G1 X135.706 Y144.294 E.06948" |
| PASS | Qidi/Qidi X-Plus 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.6 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Plus 4 0.8 nozzle | first difference at byte 6483 (line 291, column 1; expected 77699 bytes, actual 77877 bytes) context:   line 290: "G1 F30000"   line 291: expected "M73 P15 R2"; actual "G1 X149.791 Y150.318 Z1.25714"   line 292: expected "G1 X149.791 Y150.318 Z1.25714"; actual "G1 X149.221 Y151.207 Z1.31429" |
| DIVERGENT | Qidi/Qidi X-Smart 3 0.2 nozzle | first difference at byte 33858 (line 1401, column 1; expected 287762 bytes, actual 287721 bytes) context:   line 1400: "G1 X87.882 Y85.874 E.05157"   line 1401: expected "G1 X87.118 Y85.874 E.00618"; actual "M73 P32 R13"   line 1402: expected "M73 P32 R13"; actual "G1 X87.118 Y85.874 E.00618" |
| PASS | Qidi/Qidi X-Smart 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.8 nozzle |  |
| DIVERGENT | RH3D/E3NG v1.2S - 0.2 nozzle | first difference at byte 13777 (line 523, column 11; expected 290579 bytes, actual 291096 bytes) context:   line 522: "G1 X121 Y105.338 E.10775"   line 523: expected "G1 X123.081 Y105.734 E.02283"; actual "G1 X123.082 Y105.734 E.02283"   line 524: "G1 X124.872 Y106.868 E.02283" |
| PASS | RH3D/E3NG v1.2S - 0.3 nozzle |  |
| PASS | RH3D/E3NG v1.2S - 0.4 nozzle |  |
| DIVERGENT | RH3D/E3NG v1.2S - 0.5 nozzle | first difference at byte 6247 (line 241, column 1; expected 111046 bytes, actual 111310 bytes) context:   line 240: "G1 X107.113 Y109.203 E.07137"   line 241: expected "M73 P4 R5"; actual "G1 X107.911 Y108.153 E.05116"   line 242: expected "G1 X107.911 Y108.153 E.05116"; actual "M73 P4 R5" |
| PASS | RH3D/E3NG v1.2S - 0.6 nozzle |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Dual) |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Left) |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Right) |  |
| PASS | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Dual) |  |
| PASS | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Left) |  |
| PASS | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Right) |  |
| DIVERGENT | Ratrig/RatRig V-Cast 0.4 nozzle | first difference at byte 7227 (line 290, column 7; expected 115499 bytes, actual 115897 bytes) context:   line 289: ";WIDTH:0.4"   line 290: expected "G1 F5865"; actual "G1 F5876"   line 291: "G1 X145.957 Y154.043 E.23528" |
| PASS | Ratrig/RatRig V-Cast 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 3 200 0.4 nozzle | first difference at byte 33025 (line 1441, column 1; expected 113829 bytes, actual 113971 bytes) context:   line 1440: "G1 E-.7 F2400"   line 1441: expected "M73 P37 R1"; actual ";WIPE_START"   line 1442: expected ";WIPE_START"; actual "G1 F7200" |
| PASS | Ratrig/RatRig V-Core 3 300 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 3 400 0.4 nozzle | first difference at byte 4201 (line 165, column 6; expected 115755 bytes, actual 115902 bytes) context:   line 164: ";WIDTH:0.4"   line 165: expected "G1 F5790"; actual "G1 F5800"   line 166: "G1 X195.957 Y204.043 E.23528" |
| DIVERGENT | Ratrig/RatRig V-Core 3 500 0.4 nozzle | first difference at byte 3027 (line 117, column 1; expected 115221 bytes, actual 115891 bytes) context:   line 116: "G1 X252.145 Y253.613 E.01986"   line 117: expected "M73 P4 R2"; actual "G1 X246.387 Y247.855 E.27544"   line 118: expected "G1 X246.387 Y247.855 E.27544"; actual "M73 P4 R2" |
| DIVERGENT | Ratrig/RatRig V-Core 4 300 0.4 nozzle | first difference at byte 6003 (line 230, column 7; expected 113537 bytes, actual 114206 bytes) context:   line 229: ";WIDTH:0.45"   line 230: expected "G1 F4957"; actual "G1 F4971"   line 231: "G1 X146.082 Y153.918 E.25993" |
| PASS | Ratrig/RatRig V-Core 4 300 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 300 0.8 nozzle | Ratrig/RatRig V-Core 4 300 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 400 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 400 0.8 nozzle | Ratrig/RatRig V-Core 4 400 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 500 0.8 nozzle | Ratrig/RatRig V-Core 4 500 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 300 0.5 nozzle | first difference at byte 5586 (line 217, column 7; expected 109357 bytes, actual 109617 bytes) context:   line 216: ";WIDTH:0.55"   line 217: expected "G1 F4045"; actual "G1 F4055"   line 218: "G1 X146.302 Y153.698 E.3056" |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 300 0.6 nozzle | first difference at byte 82618 (line 3381, column 61; expected 102863 bytes, actual 102863 bytes) context:   line 3380: "; estimated printing time (normal mode) = 2m 45s"   line 3381: expected "; estimated first layer printing time (normal mode) = 0.455069s"; actual "; estimated first layer printing time (normal mode) = 0.455037s"   line 3382: "" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.8 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 400 0.4 nozzle | first difference at byte 6043 (line 231, column 7; expected 115191 bytes, actual 115437 bytes) context:   line 230: ";WIDTH:0.45"   line 231: expected "G1 F4958"; actual "G1 F4967"   line 232: "G1 X196.082 Y203.918 E.25993" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 500 0.8 nozzle | first difference at byte 62871 (line 2488, column 61; expected 83126 bytes, actual 83126 bytes) context:   line 2487: "; estimated printing time (normal mode) = 2m 48s"   line 2488: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2489: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.5 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 0.6 nozzle | first difference at byte 10899 (line 435, column 1; expected 104484 bytes, actual 104653 bytes) context:   line 434: "G1 X154.12 Y154.12 F30000"   line 435: expected "M73 P10 R2"; actual "SET_VELOCITY_LIMIT ACCEL=8000 ACCEL_TO_DECEL=4000"   line 436: expected "SET_VELOCITY_LIMIT ACCEL=8000 ACCEL_TO_DECEL=4000"; actual "G1 F2673" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.5 nozzle | first difference at byte 5355 (line 209, column 7; expected 105981 bytes, actual 106149 bytes) context:   line 208: ";WIDTH:0.55"   line 209: expected "G1 F3028"; actual "G1 F3036"   line 210: "G1 X146.302 Y153.698 E.31184" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.5 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.6 nozzle | first difference at byte 5330 (line 209, column 7; expected 98925 bytes, actual 99094 bytes) context:   line 208: ";WIDTH:0.6"   line 209: expected "G1 F2749"; actual "G1 F2757"   line 210: "G1 X131.437 Y153.563 E.33008" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.8 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.4 nozzle | first difference at byte 8855 (line 336, column 7; expected 113293 bytes, actual 113835 bytes) context:   line 335: ";WIDTH:0.45"   line 336: expected "G1 F3730"; actual "G1 F3741"   line 337: "G1 X181.082 Y203.918 E.26523" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.8 nozzle | first difference at byte 3141 (line 129, column 7; expected 83952 bytes, actual 84819 bytes) context:   line 128: ";WIDTH:0.75"   line 129: expected "G1 F2209"; actual "G1 F2226"   line 130: "G1 X181.771 Y203.229 E.46736" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.4 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.5 nozzle | first difference at byte 5674 (line 220, column 8; expected 110800 bytes, actual 111488 bytes) context:   line 219: ";WIDTH:0.55"   line 220: expected "G1 F4072"; actual "G1 F4073"   line 221: "G1 X246.302 Y253.698 E.3056" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.6 nozzle | first difference at byte 5646 (line 221, column 8; expected 104217 bytes, actual 104653 bytes) context:   line 220: ";WIDTH:0.6"   line 221: expected "G1 F3691"; actual "G1 F3698"   line 222: "G1 X246.437 Y253.563 E.32348" |
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
| DIVERGENT | RolohaunDesign/Rolohaun Delta Flyer Refit 0.4 nozzle | first difference at byte 86654 (line 3704, column 16; expected 126094 bytes, actual 126094 bytes) context:   line 3703: "G1 X-1.055 Y6.015 E.00391"   line 3704: expected "G1 X5.877 Y6.014 E.19257"; actual "G1 X5.877 Y6.013 E.19257"   line 3705: "G1 X5.986 Y5.986 E.00312" |
| PASS | RolohaunDesign/Rook MK1 LDO 0.2 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.4 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.6 nozzle |  |
| DIVERGENT | RolohaunDesign/Rook MK1 LDO 0.8 nozzle | first difference at byte 5671 (line 248, column 7; expected 111593 bytes, actual 112205 bytes) context:   line 247: ";WIDTH:0.88"   line 248: expected "G1 F1918"; actual "G1 F1929"   line 249: "G1 X52.077 Y57.923 E.39875" |
| DIVERGENT | SecKit/SecKit Go3 0.4 nozzle | first difference at byte 3921 (line 159, column 8; expected 110460 bytes, actual 111003 bytes) context:   line 158: ";WIDTH:0.4"   line 159: expected "G1 F2130"; actual "G1 F2134"   line 160: "G1 X145.957 Y154.043 E.22567" |
| PASS | SecKit/SecKit SK-Tank 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.4 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.5 nozzle | first difference at byte 1504 (line 47, column 9; expected 112151 bytes, actual 112684 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R5"; actual "M73 P2 R6"   line 48: "G3 X50 Y-135.0 R144.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.7 nozzle | first difference at byte 3053 (line 133, column 1; expected 76815 bytes, actual 77500 bytes) context:   line 132: "G1 E-3.5 F2700"   line 133: expected "M73 P11 R3"; actual ";WIPE_START"   line 134: expected ";WIPE_START"; actual "G1 F1200" |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.7 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 1.0 nozzle | first difference at byte 4204 (line 204, column 2; expected 60207 bytes, actual 60488 bytes) context:   line 203: "G1 X3.5 Y3.4 E1.28044"   line 204: expected "M73 P21 R2"; actual "M204 T400"   line 205: expected "M204 T400"; actual "G1 X4 Y3.634 F6000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.4 nozzle | first difference at byte 8526 (line 424, column 1; expected 150375 bytes, actual 150720 bytes) context:   line 423: "G1 F1200"   line 424: expected "M73 P11 R7"; actual "G1 X4.8 Y4.8"   line 425: expected "G1 X4.8 Y4.8"; actual "G1 X-4.8 Y4.8 E.28504" |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.5 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.7 nozzle | first difference at byte 5085 (line 261, column 1; expected 76720 bytes, actual 77540 bytes) context:   line 260: "G1 X-4.65 Y-4.65 E.84565"   line 261: expected "M73 P15 R3"; actual "G1 X4.65 Y-4.65 E.84565"   line 262: expected "G1 X4.65 Y-4.65 E.84565"; actual "M73 P15 R3" |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.4 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.5 nozzle | first difference at byte 5699 (line 284, column 1; expected 112548 bytes, actual 112725 bytes) context:   line 283: "G1 X-1.184 Y3.46 E.32175"   line 284: expected "M73 P11 R5"; actual "G1 X-1.851 Y3.46 E.03266"   line 285: expected "G1 X-1.851 Y3.46 E.03266"; actual "M73 P11 R5" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.7 nozzle | first difference at byte 6136 (line 315, column 1; expected 77365 bytes, actual 77540 bytes) context:   line 314: "G1 F1200"   line 315: expected "M73 P17 R3"; actual "G1 X-2.736 Y-2.765 E.00726"   line 316: expected "G1 X-2.736 Y-2.765 E.00726"; actual "G1 X-2.776 Y-2.697 E.00726" |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.7 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v3.2 1.0 nozzle | first difference at byte 4080 (line 197, column 2; expected 60297 bytes, actual 60470 bytes) context:   line 196: "G1 E5 F2100"   line 197: expected "M73 P19 R2"; actual "M205 X3 Y3"   line 198: expected "M205 X3 Y3"; actual "G1 F750" |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.5 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v4 0.7 nozzle | first difference at byte 1504 (line 47, column 9; expected 75522 bytes, actual 77499 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P3 R3"; actual "M73 P3 R4"   line 48: "G3 X50 Y-124.8 R134.4 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 1.0 nozzle |  |
| DIVERGENT | Snapmaker/Snapmaker A250 (0.2 nozzle) | first difference at byte 15559 (line 672, column 1; expected 186120 bytes, actual 186245 bytes) context:   line 671: "G1 E1.2 F2700"   line 672: expected "M73 P11 R12"; actual ";TYPE:Inner wall"   line 673: expected ";TYPE:Inner wall"; actual ";WIDTH:0.22" |
| DIVERGENT | Snapmaker/Snapmaker A250 (0.4 nozzle) | first difference at byte 3860 (line 208, column 8; expected 117389 bytes, actual 117732 bytes) context:   line 207: ";WIDTH:0.45"   line 208: expected "G1 F1940"; actual "G1 F1943"   line 209: "G1 X111.061 Y128.939 E.21349" |
| PASS | Snapmaker/Snapmaker A250 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 BKit (0.4 nozzle) | first difference at byte 14531 (line 685, column 1; expected 117681 bytes, actual 117789 bytes) context:   line 684: "G1 E1.2 F2700"   line 685: expected "M73 P15 R8"; actual ";TYPE:Inner wall"   line 686: expected ";TYPE:Inner wall"; actual "G1 F1056" |
| PASS | Snapmaker/Snapmaker A250 BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 Dual (0.6 nozzle) | first difference at byte 1343 (line 81, column 9; expected 99036 bytes, actual 99335 bytes) context:   line 80: "G0 Y0 F3420.0"   line 81: expected "M73 P4 R7"; actual "M73 P4 R8"   line 82: "" |
| PASS | Snapmaker/Snapmaker A250 Dual (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.6 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.8 nozzle) | first difference at byte 2184 (line 146, column 1; expected 79640 bytes, actual 80368 bytes) context:   line 145: "G1 Z.7 F5760"   line 146: expected "M73 P13 R5"; actual "G1 X119.49 Y129.49 Z.7"   line 147: expected "G1 X119.49 Y129.49 Z.7"; actual "M73 P13 R5" |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.6 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 Dual QSKit (0.8 nozzle) | first difference at byte 3556 (line 212, column 5; expected 79129 bytes, actual 80297 bytes) context:   line 211: ";WIDTH:0.82"   line 212: expected "G1 F991"; actual "G1 F1001"   line 213: "G1 X111.23 Y128.77 E.56661" |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 QS+B Kit (0.6 nozzle) | first difference at byte 7167 (line 362, column 1; expected 92724 bytes, actual 92891 bytes) context:   line 361: "G1 X117.009 Y128.576 E.33676"   line 362: expected "M73 P14 R6"; actual "G1 X116.187 Y128.576 E.03506"   line 363: expected "G1 X116.187 Y128.576 E.03506"; actual "M73 P14 R6" |
| DIVERGENT | Snapmaker/Snapmaker A250 QS+B Kit (0.8 nozzle) | first difference at byte 5443 (line 288, column 7; expected 72524 bytes, actual 73683 bytes) context:   line 287: ";WIDTH:0.82"   line 288: expected "G1 F1017"; actual "G1 F1030"   line 289: "G1 X111.23 Y128.77 E.56661" |
| DIVERGENT | Snapmaker/Snapmaker A250 QSKit (0.2 nozzle) | first difference at byte 15565 (line 672, column 1; expected 185931 bytes, actual 186306 bytes) context:   line 671: "G1 E1.2 F2700"   line 672: expected "M73 P11 R12"; actual ";TYPE:Inner wall"   line 673: expected ";TYPE:Inner wall"; actual ";WIDTH:0.22" |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A250 QSKit (0.6 nozzle) | first difference at byte 3642 (line 197, column 7; expected 92552 bytes, actual 92829 bytes) context:   line 196: ";WIDTH:0.62"   line 197: expected "G1 F1339"; actual "G1 F1344"   line 198: "G1 X110.93 Y129.07 E.34707" |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 BKit (0.2 nozzle) | first difference at byte 29675 (line 1285, column 1; expected 186148 bytes, actual 186273 bytes) context:   line 1284: "G1 X164.129 Y172.576 E.02612"   line 1285: expected "M73 P18 R11"; actual "G1 X164.129 Y174.758 E.02364"   line 1286: expected "G1 X164.129 Y174.758 E.02364"; actual "M73 P18 R11" |
| PASS | Snapmaker/Snapmaker A350 BKit (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 BKit (0.6 nozzle) | first difference at byte 3645 (line 197, column 7; expected 92487 bytes, actual 92764 bytes) context:   line 196: ";WIDTH:0.62"   line 197: expected "G1 F1339"; actual "G1 F1344"   line 198: "G1 X155.93 Y179.07 E.34707" |
| DIVERGENT | Snapmaker/Snapmaker A350 BKit (0.8 nozzle) | first difference at byte 2682 (line 160, column 1; expected 72297 bytes, actual 73575 bytes) context:   line 159: "G1 X156.897 Y171.897 E.11898"   line 160: expected "M73 P15 R5"; actual "G1 X163.103 Y178.103 E.97509"   line 161: expected "G1 X163.103 Y178.103 E.97509"; actual "M73 P15 R5" |
| PASS | Snapmaker/Snapmaker A350 Dual (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual (0.6 nozzle) | first difference at byte 1529 (line 103, column 1; expected 99020 bytes, actual 99287 bytes) context:   line 102: "G1 E3 F200"   line 103: expected "M73 P9 R7"; actual "G92 E0"   line 104: expected "G92 E0"; actual "G1 X0 E6.23628 F3420.0" |
| PASS | Snapmaker/Snapmaker A350 Dual (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual BKit (0.4 nozzle) | first difference at byte 6713 (line 344, column 8; expected 124325 bytes, actual 124433 bytes) context:   line 343: ";WIDTH:0.45"   line 344: expected "G1 F1960"; actual "G1 F1965"   line 345: "G1 X156.061 Y178.939 E.21349" |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.6 nozzle) | first difference at byte 6049 (line 320, column 7; expected 98802 bytes, actual 99413 bytes) context:   line 319: ";WIDTH:0.62"   line 320: expected "G1 F1348"; actual "G1 F1354"   line 321: "G1 X155.93 Y179.07 E.34707" |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual QSKit (0.4 nozzle) | first difference at byte 4020 (line 228, column 8; expected 124101 bytes, actual 124445 bytes) context:   line 227: ";WIDTH:0.45"   line 228: expected "G1 F1933"; actual "G1 F1936"   line 229: "G1 X156.061 Y178.939 E.21349" |
| DIVERGENT | Snapmaker/Snapmaker A350 Dual QSKit (0.6 nozzle) | first difference at byte 1535 (line 103, column 1; expected 99086 bytes, actual 99353 bytes) context:   line 102: "G1 E3 F200"   line 103: expected "M73 P9 R7"; actual "G92 E0"   line 104: expected "G92 E0"; actual "G1 X0 E6.23628 F3420.0" |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.2 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 QS+B Kit (0.4 nozzle) | first difference at byte 6590 (line 326, column 7; expected 117583 bytes, actual 117818 bytes) context:   line 325: ";WIDTH:0.45"   line 326: expected "G1 F1968"; actual "G1 F1972"   line 327: "G1 X156.061 Y178.939 E.21349" |
| DIVERGENT | Snapmaker/Snapmaker A350 QS+B Kit (0.6 nozzle) | first difference at byte 4355 (line 237, column 1; expected 92364 bytes, actual 92819 bytes) context:   line 236: "G1 X163.527 Y177.664 E.03198"   line 237: expected "M73 P13 R7"; actual "G1 X162.727 Y178.463 E.04189"   line 238: expected "G1 X162.727 Y178.463 E.04189"; actual "M73 P13 R7" |
| DIVERGENT | Snapmaker/Snapmaker A350 QS+B Kit (0.8 nozzle) | first difference at byte 7865 (line 404, column 1; expected 73351 bytes, actual 73630 bytes) context:   line 403: "G1 X164.59 Y170.41 E.68985"   line 404: expected "M73 P21 R5"; actual "G1 X164.59 Y179.47 E.68083"   line 405: expected "G1 X164.59 Y179.47 E.68083"; actual "M73 P21 R5" |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker A350 QSKit (0.6 nozzle) | first difference at byte 1355 (line 83, column 1; expected 92344 bytes, actual 92778 bytes) context:   line 82: "G1 E3 F200"   line 83: expected "M73 P8 R7"; actual "G92 E0"   line 84: expected "G92 E0"; actual "G1 X0 E6.23628 F3420.0" |
| DIVERGENT | Snapmaker/Snapmaker A350 QSKit (0.8 nozzle) | first difference at byte 3430 (line 194, column 7; expected 73410 bytes, actual 73578 bytes) context:   line 193: ";WIDTH:0.82"   line 194: expected "G1 F1008"; actual "G1 F1011"   line 195: "G1 X156.23 Y178.77 E.56661" |
| DIVERGENT | Snapmaker/Snapmaker Artisan (0.2 nozzle) | first difference at byte 13345 (line 589, column 1; expected 188207 bytes, actual 188207 bytes) context:   line 588: "G1 X204.129 Y204.114 E.00282"   line 589: expected "G1 X195.871 Y195.871 E.12255"; actual "M73 P12 R11"   line 590: expected "M73 P12 R11"; actual "G1 X195.871 Y195.871 E.12255" |
| PASS | Snapmaker/Snapmaker Artisan (0.4 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker Artisan (0.6 nozzle) | first difference at byte 12517 (line 653, column 1; expected 99070 bytes, actual 99369 bytes) context:   line 652: "G1 X203.576 Y199.979 E.09879"   line 653: expected "M73 P22 R7"; actual "G1 X196.424 Y201.895 E.31568"   line 654: expected "G1 X196.424 Y201.895 E.31568"; actual "M73 P22 R7" |
| PASS | Snapmaker/Snapmaker Artisan (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker J1 (0.2 nozzle) | first difference at byte 2465 (line 164, column 1; expected 188148 bytes, actual 188274 bytes) context:   line 163: "G1 X157.475 Y95.475 E.15271"   line 164: expected "M73 P8 R11"; actual "G1 X166.525 Y95.475 E.15271"   line 165: expected "G1 X166.525 Y95.475 E.15271"; actual "M73 P8 R11" |
| PASS | Snapmaker/Snapmaker J1 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker U1 (0.2 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-dAEaJ4") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4+0.6 nozzle) | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-kPJn6p") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.6 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-rga63h") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.8 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-f5pN5n") |
| PASS | Sovol/Sovol SV01 0.4 nozzle |  |
| PASS | Sovol/Sovol SV01 Pro 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV02 0.4 nozzle | first difference at byte 5178 (line 227, column 1; expected 98306 bytes, actual 98966 bytes) context:   line 226: "G1 X148.158 Y126.627 E.05144"   line 227: expected "M73 P8 R5"; actual "G1 X147.063 Y127.893 E.05144"   line 228: expected "G1 X147.063 Y127.893 E.05144"; actual "G1 X145 Y128.553 E.06655" |
| PASS | Sovol/Sovol SV05 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV06 0.4 High-Speed nozzle | first difference at byte 1666 (line 90, column 19; expected 101958 bytes, actual 101958 bytes) context:   line 89: "G1 X117.495 Y97.735 E.05224"   line 90: expected "G1 X118.551 Y98.188 E.05224"; actual "G1 X118.551 Y98.189 E.05224"   line 91: "G1 X119.527 Y98.794 E.05224" |
| PASS | Sovol/Sovol SV06 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.2 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.6 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.8 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus ACE 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV07 0.4 nozzle | first difference at byte 4721 (line 204, column 1; expected 93196 bytes, actual 93542 bytes) context:   line 203: "G1 X115.78 Y118.616 E.02925"   line 204: expected "M73 P5 R5"; actual "G1 X114.973 Y118.702 E.04322"   line 205: expected "G1 X114.973 Y118.702 E.04322"; actual "M73 P5 R5" |
| DIVERGENT | Sovol/Sovol SV07 Plus 0.4 nozzle | first difference at byte 4032 (line 174, column 1; expected 106781 bytes, actual 107311 bytes) context:   line 173: "G1 X146.375 Y150.578 E5.51963"   line 174: expected "G1 X149.422 Y153.625 E5.67617"; actual "M106 S255"   line 175: expected "G1 X148.896 Y153.625 E5.69525"; actual "G1 X149.422 Y153.625 E5.67617" |
| PASS | Sovol/Sovol SV08 0.2 nozzle |  |
| PASS | Sovol/Sovol SV08 0.4 nozzle |  |
| PASS | Sovol/Sovol SV08 0.6 nozzle |  |
| PASS | Sovol/Sovol SV08 0.8 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.4 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.6 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.8 nozzle |  |
| PASS | Sovol/Sovol Zero 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP300 HS 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP310 Pro 0.4 nozzle |  |
| DIVERGENT | Tiertime/Tiertime UP400 Pro 0.4 nozzle | first difference at byte 4672 (line 195, column 1; expected 106858 bytes, actual 107087 bytes) context:   line 194: "G1 X201.027 Y179.065 E.01639"   line 195: expected "M73 P4 R4"; actual "G1 X204.065 Y176.027 E.13199"   line 196: expected "G1 X204.065 Y176.027 E.13199"; actual "G1 X204.065 Y175.494 E.01639" |
| PASS | Tiertime/Tiertime UP400 Pro 0.6 nozzle |  |
| DIVERGENT | Tiertime/Tiertime UP400 Pro 0.8 nozzle | first difference at byte 6760 (line 289, column 1; expected 83427 bytes, actual 84445 bytes) context:   line 288: "G1 X199.126 Y178.514 E.24382"   line 289: expected "; stop printing object cube10.stl id:0 copy 0"; actual "G1 X197.024 Y177.939 F30000"   line 290: expected ";LAYER_CHANGE"; actual ";WIDTH:0.871416" |
| PASS | Tiertime/Tiertime UP600 HS 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.6 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.8 nozzle |  |
| DIVERGENT | Tronxy/Tronxy X5SA 400 0.4 nozzle | first difference at byte 3485 (line 159, column 7; expected 94584 bytes, actual 94961 bytes) context:   line 158: ";WIDTH:0.45"   line 159: expected "G1 F4227"; actual "G1 F4238"   line 160: "G1 X195.625 Y204.375 E.29025" |
| DIVERGENT | TwoTrees/TwoTrees SK1 0.4 nozzle | first difference at byte 1414 (line 60, column 1; expected 101783 bytes, actual 102276 bytes) context:   line 59: "G1 X190 Y12 F6000 ;Wipe"   line 60: expected "M73 P66 R2"; actual "G1 X180 Y8 F6000 ;Wipe"   line 61: expected "G1 X180 Y8 F6000 ;Wipe"; actual "G1 X170 Y12 F6000 ;Wipe" |
| DIVERGENT | TwoTrees/TwoTrees SP-5 Klipper 0.4 nozzle | first difference at byte 3504 (line 145, column 8; expected 108689 bytes, actual 108915 bytes) context:   line 144: ";WIDTH:0.45"   line 145: expected "G1 F4164"; actual "G1 F4167"   line 146: "G1 X151.032 Y158.968 E.26325" |
| PASS | UltiMaker/UltiMaker 2 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 Klipper 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 RRF 0.4 nozzle |  |
| DIVERGENT | Volumic/EXO42 (0.4 nozzle) | first difference at byte 40140 (line 1760, column 1; expected 120293 bytes, actual 120292 bytes) context:   line 1759: "G1 X214.328 Y205.672"   line 1760: expected "M73 P38 R3"; actual "G1 X208.497 Y205.672"   line 1761: expected "G1 X208.497 Y205.672"; actual "M73 P38 R3" |
| PASS | Volumic/EXO42 IDRE (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO42 IDRE COPY MODE (0.4 nozzle) | first difference at byte 10430 (line 439, column 1; expected 102422 bytes, actual 102946 bytes) context:   line 438: "G1 X117.007 Y208.851 E.02246"   line 439: expected "M73 P11 R3"; actual "G1 X119.351 Y206.507 E.1205"   line 440: expected "G1 X119.351 Y206.507 E.1205"; actual "G1 X118.733 Y206.507 E.02246" |
| PASS | Volumic/EXO42 IDRE MIRROR MODE (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO42 Performance (0.4 nozzle) | first difference at byte 5558 (line 237, column 1; expected 102231 bytes, actual 102672 bytes) context:   line 236: "G1 X208.233 Y206.507 E.02246"   line 237: expected "M73 P7 R3"; actual "G1 X206.507 Y208.233 E.08873"   line 238: expected "G1 X206.507 Y208.233 E.08873"; actual "G1 X206.507 Y207.615 E.02246" |
| PASS | Volumic/EXO42 Stage 2 (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO65 (0.6 nozzle) | first difference at byte 39837 (line 1917, column 1; expected 115024 bytes, actual 115034 bytes) context:   line 1916: "G1 X328.968 Y328.968"   line 1917: expected "M73 P41 R2"; actual "G1 X329.64 Y329.64"   line 1918: expected "G1 X329.64 Y329.64"; actual "M73 P41 R2" |
| PASS | Volumic/EXO65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE MIRROR MODE (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO65 Performance (0.4 nozzle) | first difference at byte 1500 (line 70, column 1; expected 102033 bytes, actual 102651 bytes) context:   line 69: "G1 X328.843 Y321.157 E.27933"   line 70: expected "M73 P4 R3"; actual "G1 X328.843 Y328.803 E.27788"   line 71: expected "G1 X328.843 Y328.803 E.27788"; actual "M73 P4 R3" |
| PASS | Volumic/EXO65 Performance (0.6 nozzle) |  |
| DIVERGENT | Volumic/EXO65 Performance (0.8 nozzle) | first difference at byte 3048 (line 147, column 1; expected 83958 bytes, actual 85185 bytes) context:   line 146: "G1 E-1.9 F1800"   line 147: expected "M73 P6 R3"; actual ";WIPE_START"   line 148: expected ";WIPE_START"; actual "G1 F3600" |
| PASS | Volumic/EXO65 Stage 2 (0.6 nozzle) |  |
| DIVERGENT | Volumic/SH65 (0.4 nozzle) | first difference at byte 4836 (line 220, column 1; expected 120290 bytes, actual 120290 bytes) context:   line 219: "G1 X328.474 Y148.946 E.01769"   line 220: expected "M73 P4 R4"; actual "G1 X323.946 Y153.474 E.17881"   line 221: expected "G1 X323.946 Y153.474 E.17881"; actual "M73 P4 R4" |
| PASS | Volumic/SH65 IDRE (0.4 nozzle) |  |
| DIVERGENT | Volumic/SH65 IDRE COPY MODE (0.4 nozzle) | first difference at byte 10565 (line 444, column 1; expected 102627 bytes, actual 102936 bytes) context:   line 443: "G1 X175.812 Y146.31 E.06707"   line 444: expected "; stop printing object cube10.stl id:0 copy 0"; actual "G1 X174.624 Y146.548"   line 445: expected ";LAYER_CHANGE"; actual ";WIDTH:0.48364" |
| PASS | Volumic/SH65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 Performance (0.4 nozzle) |  |
| PASS | Volumic/SH65 Stage 2 (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS20MK2 (0.4 nozzle) | first difference at byte 4736 (line 220, column 1; expected 115964 bytes, actual 116726 bytes) context:   line 219: "G1 X98.313 Y103.474 E.01769"   line 220: expected "M73 P4 R4"; actual "G1 X103.474 Y98.313 E.20382"   line 221: expected "G1 X103.474 Y98.313 E.20382"; actual "G1 X103.474 Y97.68 E.01769" |
| PASS | Volumic/VS30MK2 (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30MK3 (0.4 nozzle) | first difference at byte 3712 (line 172, column 7; expected 117996 bytes, actual 119020 bytes) context:   line 171: ";WIDTH:0.48"   line 172: expected "G1 F3626"; actual "G1 F3640"   line 173: "G1 X146.168 Y103.832 E.21404" |
| DIVERGENT | Volumic/VS30MK3 Stage 2 (0.4 nozzle) | first difference at byte 18421 (line 801, column 1; expected 101231 bytes, actual 101231 bytes) context:   line 800: "G1 X146.507 Y98.548 E.10493"   line 801: expected "G1 X146.507 Y96.507 E.0742"; actual "M73 P18 R3"   line 802: expected "M73 P18 R3"; actual "G1 X146.507 Y96.507 E.0742" |
| DIVERGENT | Volumic/VS30SC (0.4 nozzle) | first difference at byte 3028 (line 142, column 1; expected 118004 bytes, actual 119018 bytes) context:   line 141: "G1 X150.261 Y103.496 E.01846"   line 142: expected "M73 P2 R4"; actual "G1 X146.504 Y99.739 E.15161"   line 143: expected "G1 X146.504 Y99.739 E.15161"; actual "G1 X146.504 Y100.386 E.01846" |
| PASS | Volumic/VS30SC2 (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 Performance (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30SC2 Stage 2 (0.4 nozzle) | first difference at byte 18421 (line 801, column 1; expected 101231 bytes, actual 101231 bytes) context:   line 800: "G1 X146.507 Y98.548 E.10493"   line 801: expected "G1 X146.507 Y96.507 E.0742"; actual "M73 P18 R3"   line 802: expected "M73 P18 R3"; actual "G1 X146.507 Y96.507 E.0742" |
| PASS | Volumic/VS30ULTRA (0.4 nozzle) |  |
| PASS | Voron/Voron 0.1 0.15 nozzle |  |
| DIVERGENT | Voron/Voron 0.1 0.2 nozzle | first difference at byte 333887 (line 12739, column 62; expected 354370 bytes, actual 354370 bytes) context:   line 12738: "; estimated printing time (normal mode) = 8m 24s"   line 12739: expected "; estimated first layer printing time (normal mode) = 0.486746s"; actual "; estimated first layer printing time (normal mode) = 0.486747s"   line 12740: "" |
| DIVERGENT | Voron/Voron 0.1 0.25 nozzle | first difference at byte 19637 (line 747, column 8; expected 303777 bytes, actual 303982 bytes) context:   line 746: ";WIDTH:0.27"   line 747: expected "G1 F6234"; actual "G1 F6238"   line 748: "G1 X55.654 Y64.346 E.08803" |
| DIVERGENT | Voron/Voron 0.1 0.4 nozzle | first difference at byte 4252 (line 164, column 8; expected 127411 bytes, actual 128081 bytes) context:   line 163: ";WIDTH:0.4"   line 164: expected "G1 F4300"; actual "G1 F4308"   line 165: "G1 X55.957 Y64.043 E.23528" |
| PASS | Voron/Voron 0.1 0.5 nozzle |  |
| PASS | Voron/Voron 0.1 0.6 nozzle |  |
| PASS | Voron/Voron 0.1 0.8 nozzle |  |
| DIVERGENT | Voron/Voron 0.1 1.0 nozzle | first difference at byte 2837 (line 114, column 1; expected 65589 bytes, actual 65852 bytes) context:   line 113: "G1 X61.873 Y60.542 E.65471"   line 114: expected "M73 P5 R1"; actual "G1 X61.873 Y61.873 E.25515"   line 115: expected "G1 X61.873 Y61.873 E.25515"; actual "M73 P5 R1" |
| DIVERGENT | Voron/Voron 2.4 250 0.15 nozzle | first difference at byte 7226 (line 261, column 9; expected 540711 bytes, actual 542244 bytes) context:   line 260: ";WIDTH:0.17"   line 261: expected "G1 F10432"; actual "G1 F10436"   line 262: "G1 X120.406 Y129.594 E.05077" |
| PASS | Voron/Voron 2.4 250 0.2 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.25 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.5 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 250 0.6 nozzle | first difference at byte 4122 (line 154, column 7; expected 89088 bytes, actual 89714 bytes) context:   line 153: ";WIDTH:0.62"   line 154: expected "G1 F2771"; actual "G1 F2780"   line 155: "G1 X121.486 Y128.514 E.47735" |
| PASS | Voron/Voron 2.4 250 0.8 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 250 1.0 nozzle | first difference at byte 8074 (line 326, column 7; expected 66436 bytes, actual 66982 bytes) context:   line 325: "SET_VELOCITY_LIMIT ACCEL=7000 ACCEL_TO_DECEL=3500"   line 326: expected "G1 X125.128 Y123.346 F21000"; actual "G1 X123.387 Y126.574 F21000"   line 327: "SET_VELOCITY_LIMIT ACCEL=5000 ACCEL_TO_DECEL=2500" |
| PASS | Voron/Voron 2.4 300 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.2 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 300 0.25 nozzle | first difference at byte 259893 (line 9736, column 8; expected 315819 bytes, actual 316030 bytes) context:   line 9735: ";WIDTH:0.27"   line 9736: expected "G1 F6234"; actual "G1 F6238"   line 9737: "G1 X145.654 Y154.346 E.08803" |
| PASS | Voron/Voron 2.4 300 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.6 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.8 nozzle |  |
| PASS | Voron/Voron 2.4 300 1.0 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 350 0.15 nozzle | first difference at byte 449540 (line 16162, column 9; expected 541815 bytes, actual 542255 bytes) context:   line 16161: ";WIDTH:0.17"   line 16162: expected "G1 F10432"; actual "G1 F10436"   line 16163: "M106 S255" |
| PASS | Voron/Voron 2.4 350 0.2 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 350 0.25 nozzle | first difference at byte 3879 (line 138, column 1; expected 315144 bytes, actual 316030 bytes) context:   line 137: "G1 X170.854 Y170.854 E.00274"   line 138: expected "M73 P1 R7"; actual "G1 X170.854 Y171.032 E.00274"   line 139: expected "G1 X170.854 Y171.032 E.00274"; actual "G1 X178.968 Y179.146 E.17648" |
| PASS | Voron/Voron 2.4 350 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.5 nozzle |  |
| DIVERGENT | Voron/Voron 2.4 350 0.6 nozzle | first difference at byte 7050 (line 266, column 6; expected 89436 bytes, actual 89703 bytes) context:   line 265: ";WIDTH:0.62"   line 266: expected "G1 F2791"; actual "G1 F2800"   line 267: "G1 X171.486 Y178.514 E.47735" |
| DIVERGENT | Voron/Voron 2.4 350 0.8 nozzle | first difference at byte 3990 (line 159, column 7; expected 78651 bytes, actual 79008 bytes) context:   line 158: ";WIDTH:0.82"   line 159: expected "G1 F2180"; actual "G1 F2195"   line 160: "G1 X171.964 Y178.036 E.72647" |
| PASS | Voron/Voron 2.4 350 1.0 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.15 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.2 nozzle |  |
| DIVERGENT | Voron/Voron Switchwire 250 0.25 nozzle | first difference at byte 3878 (line 138, column 1; expected 314734 bytes, actual 316042 bytes) context:   line 137: "G1 X120.854 Y100.854 E.00274"   line 138: expected "M73 P1 R7"; actual "G1 X120.854 Y101.032 E.00274"   line 139: expected "G1 X120.854 Y101.032 E.00274"; actual "G1 X128.968 Y109.146 E.17648" |
| PASS | Voron/Voron Switchwire 250 0.4 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.5 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.6 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.8 nozzle |  |
| PASS | Voron/Voron Switchwire 250 1.0 nozzle |  |
| DIVERGENT | Voron/Voron Trident 250 0.15 nozzle | first difference at byte 472198 (line 16951, column 8; expected 542026 bytes, actual 542253 bytes) context:   line 16950: ";WIDTH:0.17"   line 16951: expected "G1 F10570"; actual "G1 F10557"   line 16952: "G1 X120.406 Y129.594 E.05077" |
| PASS | Voron/Voron Trident 250 0.2 nozzle |  |
| PASS | Voron/Voron Trident 250 0.25 nozzle |  |
| DIVERGENT | Voron/Voron Trident 250 0.4 nozzle | first difference at byte 17669 (line 679, column 1; expected 131580 bytes, actual 131806 bytes) context:   line 678: "G1 E-.56 F1800"   line 679: expected "M73 P15 R3"; actual ";WIPE_START"   line 680: expected ";WIPE_START"; actual "G1 F7200" |
| PASS | Voron/Voron Trident 250 0.5 nozzle |  |
| PASS | Voron/Voron Trident 250 0.6 nozzle |  |
| PASS | Voron/Voron Trident 250 0.8 nozzle |  |
| DIVERGENT | Voron/Voron Trident 250 1.0 nozzle | first difference at byte 5545 (line 223, column 1; expected 66626 bytes, actual 66990 bytes) context:   line 222: "G1 X125.852 Y122.717 E.7165"   line 223: expected "; stop printing object cube10.stl id:0 copy 0"; actual "SET_VELOCITY_LIMIT ACCEL=7000 ACCEL_TO_DECEL=3500"   line 224: expected ";LAYER_CHANGE"; actual "G1 X123.387 Y123.239 F21000" |
| PASS | Voron/Voron Trident 300 0.15 nozzle |  |
| PASS | Voron/Voron Trident 300 0.2 nozzle |  |
| DIVERGENT | Voron/Voron Trident 300 0.25 nozzle | first difference at byte 3879 (line 138, column 1; expected 315151 bytes, actual 316037 bytes) context:   line 137: "G1 X145.854 Y145.854 E.00274"   line 138: expected "M73 P1 R7"; actual "G1 X145.854 Y146.032 E.00274"   line 139: expected "G1 X145.854 Y146.032 E.00274"; actual "G1 X153.968 Y154.146 E.17648" |
| PASS | Voron/Voron Trident 300 0.4 nozzle |  |
| PASS | Voron/Voron Trident 300 0.5 nozzle |  |
| DIVERGENT | Voron/Voron Trident 300 0.6 nozzle | first difference at byte 39829 (line 1575, column 1; expected 89363 bytes, actual 89722 bytes) context:   line 1574: "G1 E-.56 F1800"   line 1575: expected "M73 P60 R0"; actual ";WIPE_START"   line 1576: expected ";WIPE_START"; actual "G1 F4407.656" |
| PASS | Voron/Voron Trident 300 0.8 nozzle |  |
| PASS | Voron/Voron Trident 300 1.0 nozzle |  |
| DIVERGENT | Voron/Voron Trident 350 0.15 nozzle | first difference at byte 7226 (line 261, column 9; expected 541611 bytes, actual 542264 bytes) context:   line 260: ";WIDTH:0.17"   line 261: expected "G1 F10432"; actual "G1 F10436"   line 262: "G1 X170.406 Y179.594 E.05077" |
| PASS | Voron/Voron Trident 350 0.2 nozzle |  |
| PASS | Voron/Voron Trident 350 0.25 nozzle |  |
| PASS | Voron/Voron Trident 350 0.4 nozzle |  |
| PASS | Voron/Voron Trident 350 0.5 nozzle |  |
| PASS | Voron/Voron Trident 350 0.6 nozzle |  |
| DIVERGENT | Voron/Voron Trident 350 0.8 nozzle | first difference at byte 3990 (line 159, column 7; expected 78038 bytes, actual 79017 bytes) context:   line 158: ";WIDTH:0.82"   line 159: expected "G1 F2173"; actual "G1 F2195"   line 160: "G1 X171.964 Y178.036 E.72647" |
| DIVERGENT | Voron/Voron Trident 350 1.0 nozzle | first difference at byte 3024 (line 119, column 1; expected 66717 bytes, actual 66990 bytes) context:   line 118: "G1 X175.542 Y176.873 E.65467"   line 119: expected "M73 P6 R1"; actual "G1 X174.211 Y176.873 E.25517"   line 120: expected "G1 X174.211 Y176.873 E.25517"; actual "M73 P6 R1" |
| PASS | Voxelab/Voxelab Aquila X2 0.4 nozzle |  |
| DIVERGENT | Vzbot/Vzbot 235 AWD 0.4 nozzle | first difference at byte 4910 (line 192, column 7; expected 124736 bytes, actual 125240 bytes) context:   line 191: ";WIDTH:0.45"   line 192: expected "G1 F5288"; actual "G1 F5295"   line 193: "G1 X113.552 Y113.552 E.24749" |
| PASS | Vzbot/Vzbot 235 AWD 0.5 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.6 nozzle |  |
| DIVERGENT | Vzbot/Vzbot 330 AWD 0.4 nozzle | first difference at byte 7241 (line 277, column 1; expected 124627 bytes, actual 125192 bytes) context:   line 276: "G1 X164.435 Y161.346 E.01548"   line 277: expected "M73 P4 R5"; actual "G1 X161.346 Y164.435 E.12684"   line 278: expected "G1 X161.346 Y164.435 E.12684"; actual "M73 P4 R5" |
| PASS | Vzbot/Vzbot 330 AWD 0.5 nozzle |  |
| DIVERGENT | Vzbot/Vzbot 330 AWD 0.6 nozzle | first difference at byte 4360 (line 172, column 7; expected 109776 bytes, actual 110200 bytes) context:   line 171: ";WIDTH:0.62"   line 172: expected "G1 F3448"; actual "G1 F3456"   line 173: "G1 X161.507 Y161.507 E.31041" |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.2mm nozzle | first difference at byte 3985 (line 171, column 7; expected 213124 bytes, actual 213150 bytes) context:   line 170: "G1 E-1.2 F7200"   line 171: expected "G1 X148.896 Y148.972 Z.56 F15000"; actual "G1 X145.864 Y150.863 Z.56 F15000"   line 172: expected "G1 X153.319 Y153.319 Z.56"; actual "G1 X146.681 Y153.319 Z.56" |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.3mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.4mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.6mm nozzle | first difference at byte 1690 (line 83, column 1; expected 109754 bytes, actual 109765 bytes) context:   line 82: "G1 X157.576 Y156.477 E.10062"   line 83: expected "M73 P44 R6"; actual "G1 X156.501 Y157.562 E.10062"   line 84: expected "G1 X156.501 Y157.562 E.10062"; actual "M73 P44 R6" |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.2mm nozzle | first difference at byte 8286 (line 354, column 8; expected 138981 bytes, actual 139401 bytes) context:   line 353: ";WIDTH:0.4"   line 354: expected "G1 F4404"; actual "G1 F4407"   line 355: "G1 X100.868 Y100.868 E.1512" |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.3mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.4mm nozzle | first difference at byte 1985 (line 95, column 1; expected 132279 bytes, actual 132656 bytes) context:   line 94: "G1 X100 Y97.784 E.04516"   line 95: expected "M73 P43 R6"; actual "G1 X110 Y97.784 E.56236"   line 96: expected "G1 X110 Y97.784 E.56236"; actual "G1 X111.257 Y98.175 E.07402" |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.6mm nozzle |  |
| DIVERGENT | Wanhao/Wanhao D12-300 0.4 nozzle | first difference at byte 3457 (line 110, column 2; expected 129360 bytes, actual 129750 bytes) context:   line 109: "G1 E5 F4200"   line 110: expected "M73 P46 R6"; actual "M204 S500"   line 111: expected "M204 S500"; actual ";TYPE:Inner wall" |
| DIVERGENT | Wanhao France/D12 230 PRO M2 DIRECT 0.4 nozzle | first difference at byte 4794 (line 182, column 7; expected 93527 bytes, actual 93771 bytes) context:   line 181: ";WIDTH:0.45"   line 182: expected "G1 F2084"; actual "G1 F2053"   line 183: "G1 X110.645 Y110.645 E.274" |
| PASS | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle PoopTool |  |
| DIVERGENT | Wanhao France/D12 230 PRO SMARTPAD DIRECT 0.4 nozzle | first difference at byte 5416 (line 178, column 8; expected 114419 bytes, actual 115402 bytes) context:   line 177: ";WIDTH:0.45"   line 178: expected "G1 F2051"; actual "G1 F2053"   line 179: "G1 X110.645 Y110.645 E.274" |
| PASS | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle PoopTool |  |
| PASS | Wanhao France/D12 300 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| DIVERGENT | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL 0.4 nozzle | first difference at byte 9069 (line 306, column 6; expected 115769 bytes, actual 116726 bytes) context:   line 305: ";WIDTH:0.45"   line 306: expected "G1 F2434"; actual "G1 F2215"   line 307: "G1 X154.355 Y145.645 E.274" |
| DIVERGENT | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle | first difference at byte 3104 (line 94, column 1; expected 117647 bytes, actual 119049 bytes) context:   line 93: "G1 X159.65 Y159.65 E.34639"   line 94: expected "G1 X150.35 Y159.65 E.34639"; actual "M73 P3 R5"   line 95: expected "M73 P3 R5"; actual "G1 X150.35 Y159.65 E.34639" |
| PASS | Wanhao France/D12 500 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 MONO DUAL PoopTool 0.4 nozzle |  |
| DIVERGENT | Wanhao France/D12 500 PRO SMARTPAD DIRECT 0.4 nozzle | first difference at byte 12691 (line 430, column 7; expected 116933 bytes, actual 117647 bytes) context:   line 429: ";WIDTH:0.45"   line 430: expected "G1 F2059"; actual "G1 F2063"   line 431: "G1 X245.645 Y245.645 E.274" |
| PASS | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| DIVERGENT | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle | first difference at byte 4385 (line 142, column 1; expected 117451 bytes, actual 118892 bytes) context:   line 141: "G1 X251.193 Y253.547 E.28831"   line 142: expected "G1 X251.193 Y254.22 E.02607"; actual "M73 P5 R5"   line 143: expected "M73 P5 R5"; actual "G1 X251.193 Y254.22 E.02607" |
| PASS | WonderMaker/WonderMaker ZR 0.2 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.4 nozzle |  |
| DIVERGENT | WonderMaker/WonderMaker ZR 0.6 nozzle | first difference at byte 6068 (line 251, column 6; expected 73688 bytes, actual 74106 bytes) context:   line 250: ";WIDTH:0.62"   line 251: expected "G1 F2799"; actual "G1 F2807"   line 252: "G1 X145.93 Y154.07 E.55282" |
| PASS | WonderMaker/WonderMaker ZR 0.8 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.2 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.6 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-F44iIz") |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.2 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra S 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-xDG0ec/command-z947xO") |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.4 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S1000 0.6 nozzle | first difference at byte 4298 (line 179, column 1; expected 107962 bytes, actual 108284 bytes) context:   line 178: "G1 X497.254 Y503.579 E.03838"   line 179: expected "M73 P5 R3"; actual "G1 X503.579 Y497.254 E.42069"   line 180: expected "G1 X503.579 Y497.254 E.42069"; actual "M73 P5 R3" |
| PASS | Z-Bolt/Z-Bolt S1000 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.4 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S300 Dual 0.6 nozzle | first difference at byte 2561 (line 102, column 1; expected 86087 bytes, actual 86329 bytes) context:   line 101: "G1 X146.497 Y148.885 E.05477"   line 102: expected "M73 P4 R2"; actual "G1 X151.115 Y153.503 E.44937"   line 103: expected "G1 X151.115 Y153.503 E.44937"; actual "M73 P4 R2" |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.6 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S400 0.8 nozzle | first difference at byte 5035 (line 210, column 7; expected 59835 bytes, actual 60337 bytes) context:   line 209: ";WIDTH:0.82"   line 210: expected "G1 F2205"; actual "G1 F2225"   line 211: "G1 X196.23 Y203.77 E.90216" |
| DIVERGENT | Z-Bolt/Z-Bolt S400 Dual 0.4 nozzle | first difference at byte 6831 (line 277, column 7; expected 150260 bytes, actual 150563 bytes) context:   line 276: ";WIDTH:0.45"   line 277: expected "G1 F4044"; actual "G1 F4052"   line 278: "G1 X195.645 Y204.355 E.28893" |
| DIVERGENT | Z-Bolt/Z-Bolt S400 Dual 0.6 nozzle | first difference at byte 5904 (line 244, column 7; expected 86169 bytes, actual 86336 bytes) context:   line 243: ";WIDTH:0.62"   line 244: expected "G1 F2815"; actual "G1 F2826"   line 245: "G1 X195.93 Y204.07 E.55282" |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.4 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S600 0.6 nozzle | first difference at byte 5913 (line 248, column 1; expected 82327 bytes, actual 82580 bytes) context:   line 247: "G1 X304.69 Y304.69 F18000"   line 248: expected "M73 P9 R2"; actual ";TYPE:Outer wall"   line 249: expected ";TYPE:Outer wall"; actual "G1 F2826" |
| PASS | Z-Bolt/Z-Bolt S600 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.4 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S600 Dual 0.6 nozzle | first difference at byte 5936 (line 248, column 1; expected 82779 bytes, actual 83032 bytes) context:   line 247: "G1 X304.69 Y304.69 F18000"   line 248: expected "M73 P9 R2"; actual ";TYPE:Outer wall"   line 249: expected ";TYPE:Outer wall"; actual "G1 F2826" |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.6 nozzle |  |
| DIVERGENT | Z-Bolt/Z-Bolt S800 Dual 0.8 nozzle | first difference at byte 3113 (line 129, column 7; expected 57938 bytes, actual 58244 bytes) context:   line 128: ";WIDTH:0.82"   line 129: expected "G1 F2002"; actual "G1 F2013"   line 130: "G1 X397.23 Y404.77 E.90216" |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.25 Nozzle | iQ/iQ TiQ2 0.25 Nozzle process: no compatible preset |
| PASS | iQ/iQ TiQ2 0.4 Nozzle |  |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.6 Nozzle | iQ/iQ TiQ2 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.8 Nozzle | iQ/iQ TiQ2 0.8 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.25 Nozzle | iQ/iQ TiQ8 0.25 Nozzle process: no compatible preset |
| DIVERGENT | iQ/iQ TiQ8 0.4 Nozzle | first difference at byte 22135 (line 885, column 6; expected 130171 bytes, actual 130358 bytes) context:   line 884: "G1 Z1 F18000"   line 885: expected "G1 X246.061 Y187.924 Z1"; actual "G1 X237.93 Y195.977 Z1"   line 886: "G1 Z.6" |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.6 Nozzle | iQ/iQ TiQ8 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.8 Nozzle | iQ/iQ TiQ8 0.8 Nozzle process: no compatible preset |
| PASS | re3D/re3D Gigabot 4 0.4 nozzle |  |
| PASS | re3D/re3D Gigabot 4 0.8 nozzle |  |
| DIVERGENT | re3D/re3D Gigabot 4 XLT 0.4 nozzle | first difference at byte 3963 (line 167, column 1; expected 85851 bytes, actual 86535 bytes) context:   line 166: "G1 X299.229 Y377.55"   line 167: expected "M73 P4 R5"; actual "G1 X298.517 Y377.55"   line 168: expected "G1 X298.517 Y377.55"; actual "M73 P4 R5" |
| PASS | re3D/re3D Gigabot 4 XLT 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 1.75 nozzle |  |
| PASS | re3D/re3D GigabotX 2 XLT 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 XLT 1.75 nozzle |  |
| PASS | re3D/re3D Terabot 4 0.4 nozzle |  |
| PASS | re3D/re3D Terabot 4 0.8 nozzle |  |
| PASS | re3D/re3D TerabotX 2 0.8 nozzle |  |
| PASS | re3D/re3D TerabotX 2 1.75 nozzle |  |
