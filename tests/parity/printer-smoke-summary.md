# OrcaSlicer printer smoke summary

853 of 1001 printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; classic wall generator baseline; cube model).

Statuses: `PASS` (normalized byte equality), `DIVERGENT` (first byte difference), `ORCA_ERROR` (the upstream OrcaSlicer 2.4.2 reference binary itself failed, so no reference stream exists), `VENDOR_INCOMPLETE` (the vendor profile tree does not ship the machine's referenced default process preset), `ARES_ERROR` (Ares failed to load or slice the case).

| status | printer | first divergence |
|---|---|---|
| PASS | Afinia/Afinia H+1(HS) 0.4 nozzle |  |
| PASS | Afinia/Afinia H+1(HS) 0.6 nozzle |  |
| PASS | Anker/Anker M5 0.2 nozzle |  |
| PASS | Anker/Anker M5 0.25 nozzle |  |
| PASS | Anker/Anker M5 0.4 nozzle |  |
| PASS | Anker/Anker M5 0.6 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.2 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.25 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.4 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.6 nozzle |  |
| PASS | Anker/Anker M5C 0.2 nozzle |  |
| PASS | Anker/Anker M5C 0.25 nozzle |  |
| PASS | Anker/Anker M5C 0.4 nozzle |  |
| PASS | Anker/Anker M5C 0.6 nozzle |  |
| PASS | Anycubic/Anycubic 4Max Pro 0.4 nozzle |  |
| PASS | Anycubic/Anycubic 4Max Pro 2 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Chiron 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 6533 (line 284, column 1; expected 114261 bytes, actual 114602 bytes) context:   line 283: "G1 Z.8 F10800"   line 284: expected "M73 P10 R7"; actual "G1 X114.24 Y114.24 Z.8"   line 285: expected "G1 X114.24 Y114.24 Z.8"; actual "M73 P10 R7" |
| PASS | Anycubic/Anycubic Kobra 2 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Neo 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Plus 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Pro 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra 3 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 18994 (line 661, column 29; expected 454235 bytes, actual 454235 bytes) context:   line 660: ";WIPE_START"   line 661: expected "G1 X121.761 Y120.566 E-.70072"; actual "G1 X121.761 Y120.566 E-.70071"   line 662: expected "G1 X121.88 Y120.53 E-.09928"; actual "G1 X121.88 Y120.53 E-.09929" |
| PASS | Anycubic/Anycubic Kobra 3 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra Max 0.4 nozzle |  |
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
| PASS | Artillery/Artillery Hornet 0.4 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.2 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.4 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.6 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.8 nozzle |  |
| DIVERGENT | Artillery/Artillery Sidewinder X1 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 4008 (line 167, column 1; expected 104051 bytes, actual 104247 bytes) context:   line 166: "G1 X154.75 Y145.25 E.43192"   line 167: expected "M73 P4 R8"; actual "G1 X154.75 Y154.71 E.4301"   line 168: expected "G1 X154.75 Y154.71 E.4301"; actual "M73 P4 R8" |
| DIVERGENT | Artillery/Artillery Sidewinder X2 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 2278 (line 96, column 1; expected 104148 bytes, actual 104344 bytes) context:   line 95: "G1 X157.722 Y156.516 E.0712"   line 96: expected "M73 P36 R8"; actual "G1 X156.641 Y157.649 E.0712"   line 97: expected "G1 X156.641 Y157.649 E.0712"; actual "G1 X155 Y158.116 E.07757" |
| PASS | Artillery/Artillery Sidewinder X3 Plus 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X3 Pro 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X4 Plus 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X4 Pro 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.2 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.6 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.8 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.2 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.6 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab H2D 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 366197 (line 15151, column 1; expected 403760 bytes, actual 403749 bytes) context:   line 15150: "G1 X177.857 Y155.896 E.07964"   line 15151: expected "M73 P94 R1"; actual "G1 X177.576 Y155.896 E.00227"   line 15152: expected "G1 X177.576 Y155.896 E.00227"; actual "M73 P94 R0" |
| PASS | BBL/Bambu Lab H2D 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab H2D Pro 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 366970 (line 15152, column 1; expected 404635 bytes, actual 404624 bytes) context:   line 15151: "G1 X177.857 Y155.896 E.07964"   line 15152: expected "M73 P94 R1"; actual "G1 X177.576 Y155.896 E.00227"   line 15153: expected "G1 X177.576 Y155.896 E.00227"; actual "M73 P94 R0" |
| PASS | BBL/Bambu Lab H2D Pro 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.8 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab P1P 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 371083 bytes, actual 371106 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 15m 25s"; actual "; model printing time: 8m 39s; total estimated time: 15m 28s"   line 4: "; estimated first layer printing time (normal mode) = 6m 48s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 143519 bytes, actual 143848 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 37s; total estimated time: 11m 52s"; actual "; model printing time: 5m 39s; total estimated time: 11m 54s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 101993 bytes, actual 101993 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 42s; total estimated time: 9m 56s"; actual "; model printing time: 3m 44s; total estimated time: 9m 57s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 86804 bytes, actual 87104 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 43s; total estimated time: 9m 0s"; actual "; model printing time: 2m 44s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 371249 bytes, actual 371272 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 15m 25s"; actual "; model printing time: 8m 39s; total estimated time: 15m 28s"   line 4: "; estimated first layer printing time (normal mode) = 6m 48s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 143988 bytes, actual 143988 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 5s; total estimated time: 10m 20s"; actual "; model printing time: 4m 8s; total estimated time: 10m 23s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 102134 bytes, actual 102145 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 46s; total estimated time: 9m 0s"; actual "; model printing time: 2m 48s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 87316 bytes, actual 87327 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 7s; total estimated time: 8m 25s"; actual "; model printing time: 2m 9s; total estimated time: 8m 27s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| PASS | BBL/Bambu Lab P2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab X1 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 396001 bytes, actual 395885 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 37s; total estimated time: 18m 36s"; actual "; model printing time: 8m 40s; total estimated time: 18m 39s"   line 4: "; estimated first layer printing time (normal mode) = 9m 58s" |
| DIVERGENT | BBL/Bambu Lab X1 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 169245 bytes, actual 169164 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 38s; total estimated time: 12m 54s"; actual "; model printing time: 5m 40s; total estimated time: 12m 57s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 127211 bytes, actual 127139 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 43s; total estimated time: 11m 1s"; actual "; model printing time: 3m 45s; total estimated time: 11m 2s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
| DIVERGENT | BBL/Bambu Lab X1 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 112013 bytes, actual 111945 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 44s; total estimated time: 10m 7s"; actual "; model printing time: 2m 45s; total estimated time: 10m 9s"   line 4: "; estimated first layer printing time (normal mode) = 7m 23s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 396062 bytes, actual 395946 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 37s; total estimated time: 18m 36s"; actual "; model printing time: 8m 40s; total estimated time: 18m 39s"   line 4: "; estimated first layer printing time (normal mode) = 9m 58s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 169029 bytes, actual 169180 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 7s; total estimated time: 11m 23s"; actual "; model printing time: 4m 9s; total estimated time: 11m 25s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 127323 bytes, actual 127220 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 47s; total estimated time: 10m 4s"; actual "; model printing time: 2m 49s; total estimated time: 10m 6s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 112166 bytes, actual 112077 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 9s; total estimated time: 9m 32s"; actual "; model printing time: 2m 10s; total estimated time: 9m 34s"   line 4: "; estimated first layer printing time (normal mode) = 7m 23s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 394389 bytes, actual 394309 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 19m 11s"; actual "; model printing time: 8m 39s; total estimated time: 19m 15s"   line 4: "; estimated first layer printing time (normal mode) = 10m 35s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 167733 bytes, actual 167663 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 6s; total estimated time: 11m 33s"; actual "; model printing time: 4m 8s; total estimated time: 11m 35s"   line 4: "; estimated first layer printing time (normal mode) = 7m 27s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 93 (line 3, column 28; expected 125678 bytes, actual 125595 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 46s; total estimated time: 10m 17s"; actual "; model printing time: 2m 48s; total estimated time: 10m 19s"   line 4: expected "; estimated first layer printing time (normal mode) = 7m 30s"; actual "; estimated first layer printing time (normal mode) = 7m 31s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 92 (line 3, column 27; expected 110611 bytes, actual 110521 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 8s; total estimated time: 9m 45s"; actual "; model printing time: 2m 9s; total estimated time: 9m 47s"   line 4: "; estimated first layer printing time (normal mode) = 7m 37s" |
| PASS | BBL/Bambu Lab X2D 0.2 nozzle |  |
| PASS | BBL/Bambu Lab X2D 0.4 nozzle |  |
| PASS | BBL/Bambu Lab X2D 0.6 nozzle |  |
| PASS | BBL/Bambu Lab X2D 0.8 nozzle |  |
| PASS | BIQU/BIQU B1 (0.4 nozzle) |  |
| PASS | BIQU/BIQU BX (0.4 nozzle) |  |
| PASS | BIQU/BIQU Hurakan (0.4 nozzle) |  |
| PASS | Blocks/BLOCKS Pro S100 0.4 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 0.6 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 0.8 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 1.0 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 1.2 nozzle |  |
| PASS | Blocks/BLOCKS RD50 V2 0.4 nozzle |  |
| PASS | Blocks/BLOCKS RD50 V2 0.6 nozzle |  |
| PASS | Blocks/BLOCKS RD50 V2 0.8 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.4 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.6 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.8 nozzle |  |
| PASS | CONSTRUCT3D/Construct 1 0.4 nozzle |  |
| PASS | CONSTRUCT3D/Construct 1 XL 0.6 nozzle |  |
| PASS | Chuanying/Chuanying X1 0.25 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.4 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.6 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.8 Nozzle |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 Plus |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle fast |  |
| PASS | CoLiDo/CoLiDo 160 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo SR1 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo X16 0.4 nozzle |  |
| PASS | Comgrow/Comgrow T300 0.4 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.4 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.6 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.8 nozzle |  |
| PASS | Creality/Creality CR-10 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 SE 0.2 nozzle |  |
| PASS | Creality/Creality CR-10 SE 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 SE 0.6 nozzle |  |
| PASS | Creality/Creality CR-10 SE 0.8 nozzle |  |
| PASS | Creality/Creality CR-10 V2 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 V3 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 V3 0.6 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 Max 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 7041 (line 291, column 29; expected 394066 bytes, actual 394066 bytes) context:   line 290: "G1 X195.605 Y204.196 E-.58892"   line 291: expected "G1 X195.605 Y203.947 E-.37351"; actual "G1 X195.605 Y203.947 E-.37352"   line 292: "G1 X195.858 Y204.2 E-.53756" |
| PASS | Creality/Creality CR-6 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.8 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 SE 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 3772 (line 171, column 1; expected 393672 bytes, actual 394163 bytes) context:   line 170: "G1 X117.463 Y113.105 E.00214"   line 171: expected "G1 X121.895 Y117.537 E.05396"; actual "M73 P4 R22"   line 172: expected "G1 X121.895 Y117.786 E.00214"; actual "G1 X121.895 Y117.537 E.05396" |
| PASS | Creality/Creality CR-6 SE 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 SE 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 SE 0.8 nozzle |  |
| PASS | Creality/Creality CR-M4 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 0.4 nozzle |  |
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
| PASS | Creality/Creality Ender-3 S1 Pro 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V2 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V2 Neo 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 7328 (line 293, column 8; expected 119460 bytes, actual 119685 bytes) context:   line 292: ";WIDTH:0.45"   line 293: expected "G1 F1955"; actual "G1 F1958"   line 294: "G1 X106.052 Y113.948 E.26192" |
| PASS | Creality/Creality Ender-3 V3 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 Plus 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 22406 (line 893, column 1; expected 123624 bytes, actual 123624 bytes) context:   line 892: "G1 X146.052 Y146.052 E.26192"   line 893: expected "M73 P21 R4"; actual "G1 X153.948 Y146.052 E.26192"   line 894: expected "G1 X153.948 Y146.052 E.26192"; actual "M73 P21 R4" |
| PASS | Creality/Creality Ender-3 V3 Plus 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 V4 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Max 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Max 0.6 nozzle |  |
| PASS | Creality/Creality Ender-5 Max 0.8 nozzle |  |
| PASS | Creality/Creality Ender-5 Plus 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.2 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.25 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.3 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.4 nozzle |  |
| DIVERGENT | Creality/Creality Ender-5 Pro (2019) 0.5 nozzle | (no match across 3 oracle runs) first difference at byte 4261 (line 201, column 6; expected 98013 bytes, actual 98573 bytes) context:   line 200: ";WIDTH:0.55"   line 201: expected "G1 F1699"; actual "G1 F1705"   line 202: "G1 X106.302 Y113.698 E.3056" |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.6 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.8 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 1.0 nozzle |  |
| PASS | Creality/Creality Ender-5 S1 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5S 0.4 nozzle |  |
| PASS | Creality/Creality Ender-6 0.4 nozzle |  |
| PASS | Creality/Creality Hi 0.2 nozzle |  |
| PASS | Creality/Creality Hi 0.4 nozzle |  |
| PASS | Creality/Creality Hi 0.6 nozzle |  |
| PASS | Creality/Creality Hi 0.8 nozzle |  |
| PASS | Creality/Creality K1 (0.4 nozzle) |  |
| PASS | Creality/Creality K1 (0.6 nozzle) |  |
| PASS | Creality/Creality K1 (0.8 nozzle) |  |
| PASS | Creality/Creality K1 Max (0.4 nozzle) |  |
| PASS | Creality/Creality K1 Max (0.6 nozzle) |  |
| PASS | Creality/Creality K1 Max (0.8 nozzle) |  |
| PASS | Creality/Creality K1 Max_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K1 SE 0.4 nozzle |  |
| PASS | Creality/Creality K1 SE 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Creality/Creality K1 SE 0.8 nozzle | Creality/Creality K1 SE 0.8 nozzle process: default preset "0.40mm Standard @Creality K1 SE 0.8 nozzle" not found |
| DIVERGENT | Creality/Creality K1 SE_CFS-C 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 4005 (line 173, column 8; expected 115293 bytes, actual 116018 bytes) context:   line 172: ";WIDTH:0.45"   line 173: expected "G1 F1361"; actual "G1 F1364"   line 174: "G1 X105.602 Y114.398 E.26837" |
| PASS | Creality/Creality K1C 0.4 nozzle |  |
| PASS | Creality/Creality K1C 0.6 nozzle |  |
| PASS | Creality/Creality K1C 0.8 nozzle |  |
| PASS | Creality/Creality K1C_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K1_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K2 0.2 nozzle |  |
| PASS | Creality/Creality K2 0.4 nozzle |  |
| PASS | Creality/Creality K2 0.6 nozzle |  |
| PASS | Creality/Creality K2 0.8 nozzle |  |
| DIVERGENT | Creality/Creality K2 Plus 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 26719 (line 972, column 1; expected 228107 bytes, actual 228539 bytes) context:   line 971: "G1 X178.364 Y179.102 E.00282"   line 972: expected "M73 P17 R5"; actual "G1 X170.898 Y171.636 E.1109"   line 973: expected "G1 X170.898 Y171.636 E.1109"; actual "M73 P17 R5" |
| PASS | Creality/Creality K2 Plus 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K2 Plus 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 3372 (line 142, column 1; expected 95769 bytes, actual 95769 bytes) context:   line 141: "G1 X174.52 Y178.568 E.29616"   line 142: expected "G1 X173.71 Y178.568 E.05491"; actual "M73 P16 R3"   line 143: expected "M73 P16 R3"; actual "G1 X173.71 Y178.568 E.05491" |
| DIVERGENT | Creality/Creality K2 Plus 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 5106 (line 218, column 1; expected 79002 bytes, actual 79002 bytes) context:   line 217: "G1 X175.467 Y178.313 E.38216"   line 218: expected "M73 P20 R2"; actual "G1 X174.404 Y178.313 E.1009"   line 219: expected "G1 X174.404 Y178.313 E.1009"; actual "M73 P20 R2" |
| PASS | Creality/Creality K2 Pro 0.2 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.4 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.6 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.8 nozzle |  |
| PASS | Creality/Creality K2 SE 0.4 nozzle |  |
| DIVERGENT | Creality/Creality SPARKX i7 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 22156 (line 808, column 1; expected 361674 bytes, actual 361673 bytes) context:   line 807: "G1 X134.126 Y129.358 E.00227"   line 808: expected "M73 P8 R13"; actual "G1 X129.358 Y134.126 E.05455"   line 809: expected "G1 X129.358 Y134.126 E.05455"; actual "G1 X129.077 Y134.126 E.00227" |
| DIVERGENT | Creality/Creality SPARKX i7 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 2827 (line 132, column 1; expected 137363 bytes, actual 137362 bytes) context:   line 131: "G1 X129.221 Y126.2 E.26305"   line 132: expected "M73 P9 R6"; actual "G1 X128.55 Y126.2 E.02728"   line 133: expected "G1 X128.55 Y126.2 E.02728"; actual "M73 P9 R6" |
| DIVERGENT | Creality/Creality SPARKX i7 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 2217 (line 107, column 1; expected 104151 bytes, actual 104148 bytes) context:   line 106: "G1 X134.123 Y127.322 E.1484"   line 107: expected "M73 P13 R4"; actual "G1 X134.123 Y128.122 E.05811"   line 108: expected "G1 X134.123 Y128.122 E.05811"; actual "M73 P13 R4" |
| DIVERGENT | Creality/Creality SPARKX i7 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 954 (line 46, column 7; expected 87352 bytes, actual 88503 bytes) context:   line 45: "G1 X115 E.3742  F1600"   line 46: expected "M73 P15 R3"; actual "M73 P14 R3"   line 47: "G1 X110 E.3742  F6400" |
| DIVERGENT | Creality/Creality Sermoon V1 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 2015 (line 53, column 9; expected 162827 bytes, actual 167626 bytes) context:   line 52: "G1 Z2.0 F3000                          ; Move Z Axis up"   line 53: expected "M73 P3 R6"; actual "M73 P3 R7"   line 54: "G92 E0" |
| ORCA_ERROR | Cubicon/Cubicon xCeler-I 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-6pBYm5") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Mini 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-ziXXi8") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Plus 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-Jgz1jv") |
| PASS | Custom/MyKlipper 0.2 nozzle |  |
| PASS | Custom/MyKlipper 0.4 nozzle |  |
| PASS | Custom/MyKlipper 0.6 nozzle |  |
| PASS | Custom/MyKlipper 0.8 nozzle |  |
| PASS | Custom/MyMarlin 0.4 nozzle |  |
| PASS | Custom/MyRRF 0.4 nozzle |  |
| PASS | Custom/MyRepetier 0.4 nozzle |  |
| ORCA_ERROR | Custom/MyToolChanger 0.2 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-Xn9Jvl") |
| ORCA_ERROR | Custom/MyToolChanger 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-aKsRdB") |
| PASS | Custom/MyToolChanger 0.6 nozzle |  |
| PASS | Custom/MyToolChanger 0.8 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2 0.35 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2T 0.5 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2XT 0.5 nozzle |  |
| PASS | Dremel/Dremel 3D20 0.4 nozzle |  |
| DIVERGENT | Dremel/Dremel 3D40 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 4654 (line 205, column 1; expected 103874 bytes, actual 104124 bytes) context:   line 204: "G1 X-11.587 Y-.761 E3.23024"   line 205: expected "M73 P4 R9"; actual "G1 X-11.587 Y-.065 E3.24418"   line 206: expected "G1 X-11.587 Y-.065 E3.24418"; actual "M73 P4 R9" |
| PASS | Dremel/Dremel 3D45 0.4 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 25199 (line 1045, column 7; expected 238296 bytes, actual 238613 bytes) context:   line 1044: ";WIPE_END"   line 1045: expected "G1 X130.055 Y132.096 Z1.05"; actual "G1 X131.511 Y131.476 Z1.05"   line 1046: "G1 Z.65" |
| PASS | Elegoo/Elegoo Centauri 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.4 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri 2 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 3426 (line 165, column 7; expected 69547 bytes, actual 69758 bytes) context:   line 164: ";WIDTH:0.62"   line 165: expected "G1 F2858"; actual "G1 F2865"   line 166: "G1 X123.866 Y130.634 E.54472" |
| PASS | Elegoo/Elegoo Centauri 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.6 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Centauri Carbon 2 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 6669 (line 308, column 1; expected 56672 bytes, actual 57019 bytes) context:   line 307: "G1 X124.768 Y125.794 E.64635"   line 308: expected "M73 P32 R1"; actual "G1 X124.768 Y126.832 E.1205"   line 309: expected "G1 X124.768 Y126.832 E.1205"; actual "M73 P32 R1" |
| PASS | Elegoo/Elegoo Neptune 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.2 nozzle |  |
| DIVERGENT | Elegoo/Elegoo Neptune 4 Max 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 3718 (line 168, column 8; expected 96236 bytes, actual 96472 bytes) context:   line 167: ";WIDTH:0.45"   line 168: expected "G1 F2002"; actual "G1 F2006"   line 169: "G1 X210.102 Y215.398 E.28302" |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.4 nozzle |  |
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
| DIVERGENT | Eryone/Eryone ER20 0.5 nozzle | (no match across 3 oracle runs) first difference at byte 2090 (line 90, column 7; expected 112154 bytes, actual 113351 bytes) context:   line 89: "G1 X125.739 Y114.239"   line 90: expected "G1 X126.225 Y114.725"; actual "G1 X125.739 Y114.239"   line 91: expected "M205 X9 Y9"; actual "G1 X126.225 Y114.725" |
| DIVERGENT | Eryone/Eryone ER20 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 2079 (line 90, column 7; expected 93467 bytes, actual 94160 bytes) context:   line 89: "G1 X125.667 Y114.167"   line 90: expected "G1 X126.19 Y114.69"; actual "G1 X125.667 Y114.167"   line 91: expected "M205 X9 Y9"; actual "G1 X126.19 Y114.69" |
| DIVERGENT | Eryone/Eryone ER20 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 2110 (line 92, column 7; expected 67425 bytes, actual 67929 bytes) context:   line 91: "G1 X125.399 Y113.899"   line 92: expected "G1 X126.09 Y114.59"; actual "G1 X125.399 Y113.899"   line 93: expected "G1 Z.4"; actual "G1 X126.09 Y114.59" |
| PASS | Eryone/Eryone ER20 Klipper 0.2 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.4 nozzle |  |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.5 nozzle | (no match across 3 oracle runs) first difference at byte 1564 (line 60, column 7; expected 144405 bytes, actual 145287 bytes) context:   line 59: "G1 X125.784 Y114.284"   line 60: expected "G1 X126.24 Y114.74"; actual "G1 X125.784 Y114.284"   line 61: expected "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 SQUARE_CORNER_VELOCITY=9"; actual "G1 X126.24 Y114.74" |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 1571 (line 61, column 7; expected 114496 bytes, actual 115504 bytes) context:   line 60: "G1 X125.667 Y114.167"   line 61: expected "G1 X126.19 Y114.69"; actual "G1 X125.667 Y114.167"   line 62: expected "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 SQUARE_CORNER_VELOCITY=9"; actual "G1 X126.19 Y114.69" |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 1602 (line 63, column 7; expected 80922 bytes, actual 81741 bytes) context:   line 62: "G1 X125.399 Y113.899"   line 63: expected "G1 X126.09 Y114.59"; actual "G1 X125.399 Y113.899"   line 64: expected "G1 Z.4"; actual "G1 X126.09 Y114.59" |
| PASS | Eryone/Thinker X400 0.2 nozzle |  |
| PASS | Eryone/Thinker X400 0.4 nozzle |  |
| PASS | FLSun/FLSun Q5 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun QQ-S Pro 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 6013 (line 262, column 7; expected 103511 bytes, actual 103931 bytes) context:   line 261: ";WIDTH:0.45"   line 262: expected "G1 F2026"; actual "G1 F2035"   line 263: "G1 X-3.918 Y3.918 E.25993" |
| PASS | FLSun/FLSun S1 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun Super Racer 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 2574 (line 105, column 1; expected 104847 bytes, actual 104847 bytes) context:   line 104: "G1 X5 Y10.611 E.08056"   line 105: expected "M73 P5 R7"; actual "G1 X-5 Y10.611 E.33172"   line 106: expected "G1 X-5 Y10.611 E.33172"; actual "G1 X-7.072 Y10.214 E.06996" |
| PASS | FLSun/FLSun T1 0.4 nozzle |  |
| PASS | FLSun/FLSun V400 0.4 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.25 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.4 nozzle |  |
| DIVERGENT | Flashforge/Flashforge AD5X 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 52060 (line 2409, column 10; expected 107692 bytes, actual 108034 bytes) context:   line 2408: "G1 X105.93 Y105.93 E.34707"   line 2409: expected "M73 P64 R1"; actual "M73 P64 R2"   line 2410: "G1 X114.07 Y105.93 E.34707" |
| PASS | Flashforge/Flashforge AD5X 0.8 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.4 Nozzle | (no match across 3 oracle runs) first difference at byte 5552 (line 276, column 15; expected 106902 bytes, actual 107888 bytes) context:   line 275: "G1 X4.464 Y4.464 F4800"   line 276: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 277: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.6 Nozzle | (no match across 3 oracle runs) first difference at byte 4936 (line 253, column 9; expected 71444 bytes, actual 72396 bytes) context:   line 252: "G1 X4.197 Y4.197 F6000"   line 253: expected "G1 X4.196 Y4.164"; actual "G1 X4.197 Y4.197"   line 254: expected "G1 X4.164 Y4.164"; actual "G1 X4.196 Y4.164" |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.3 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series 0.4 Nozzle | (no match across 3 oracle runs) first difference at byte 5126 (line 242, column 15; expected 92980 bytes, actual 94238 bytes) context:   line 241: "G1 X4.464 Y4.464 F4800"   line 242: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 243: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.6 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series HS Nozzle | (no match across 3 oracle runs) first difference at byte 5404 (line 256, column 15; expected 103767 bytes, actual 105025 bytes) context:   line 255: "G1 X4.464 Y4.464 F9000"   line 256: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 257: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| PASS | Flashforge/Flashforge Adventurer 5M 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.8 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Artemis 0.4 Nozzle | (no match across 3 oracle runs) first difference at byte 5282 (line 252, column 15; expected 110781 bytes, actual 112430 bytes) context:   line 251: "G1 X4.458 Y4.458 F6000"   line 252: expected "G1 X4.458 Y4.439"; actual "G1 X4.458 Y4.458"   line 253: expected "G1 X4.439 Y4.439"; actual "G1 X4.458 Y4.439" |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-KPHW0f") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-F55Zo5") |
| PASS | Flashforge/Flashforge Creator 5 0.8 nozzle |  |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-4vksdA") |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.6 nozzle |  |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.8 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider 2s 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 2946 (line 136, column 1; expected 82960 bytes, actual 83421 bytes) context:   line 135: "G1 X24.1 Y4.06 E2.62563"   line 136: expected "M73 P9 R5"; actual "G1 X24.6 Y4.6 F4200"   line 137: expected "G1 X24.6 Y4.6 F4200"; actual "M73 P9 R5" |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.25 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.4 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.6 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.6 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.8 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.25 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.4 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.6 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.6 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.8 HF nozzle |  |
| PASS | FlyingBear/FlyingBear Ghost 6 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear Ghost7 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear Reborn3 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear S1 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-5 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-5 0.6 nozzle |  |
| PASS | Folgertech/Folgertech FT-6 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-6 0.6 nozzle |  |
| PASS | Folgertech/Folgertech i3 0.4 nozzle |  |
| PASS | Folgertech/Folgertech i3 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A10 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A10 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A20 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A30 T 0.4 nozzle |  |
| DIVERGENT | Geeetech/Geeetech M1 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 17168 (line 755, column 1; expected 296994 bytes, actual 297109 bytes) context:   line 754: "G1 X56.62 Y50.751 E.00227"   line 755: expected "M73 P8 R8"; actual "G1 X50.751 Y56.62 E.06714"   line 756: expected "G1 X50.751 Y56.62 E.06714"; actual "G1 X50.47 Y56.62 E.00227" |
| PASS | Geeetech/Geeetech M1 0.4 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.6 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.8 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar M 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 5323 (line 231, column 6; expected 111050 bytes, actual 111689 bytes) context:   line 230: "M204 S700"   line 231: expected "G1 X129.343 Y131.334 F9000"; actual "G1 X130.745 Y130.682 F9000"   line 232: "M204 S500" |
| PASS | Geeetech/Geeetech Mizar Max 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.8 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar Pro 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 4345 (line 175, column 1; expected 315344 bytes, actual 316196 bytes) context:   line 174: "G1 X113.847 Y112.23 E.00301"   line 175: expected "M73 P2 R16"; actual "G1 X107.77 Y106.153 E.08005"   line 176: expected "G1 X107.77 Y106.153 E.08005"; actual "G1 X107.447 Y106.153 E.00301" |
| DIVERGENT | Geeetech/Geeetech Mizar Pro 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 3743 (line 157, column 1; expected 110166 bytes, actual 111725 bytes) context:   line 156: "G1 X111.956 Y113.3 E.28786"   line 157: expected "M73 P6 R6"; actual "G1 X111.284 Y113.3 E.02603"   line 158: expected "G1 X111.284 Y113.3 E.02603"; actual "M73 P6 R6" |
| PASS | Geeetech/Geeetech Mizar Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.8 nozzle |  |
| DIVERGENT | Ginger Additive/Ginger G1 1.2 nozzle | (no match across 3 oracle runs) first difference at byte 596 (line 19, column 9; expected 39732 bytes, actual 39065 bytes) context:   line 18: "EXCLUDE_OBJECT_DEFINE NAME=cube10.stl_id_0_copy_0 CENTER=500,500 POLYGON=[[495,495],[505,495],[505,505],[495,505],[495,495]]"   line 19: expected "M73 P0 R10"; actual "M73 P0 R5"   line 20: ";TYPE:Custom" |
| PASS | Ginger Additive/Ginger G1 3.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 5.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 8.0 nozzle |  |
| PASS | InfiMech/InfiMech EX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech EX+APS 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX HSN 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KLP1 0.4 nozzle |  |
| DIVERGENT | Kingroon/Kingroon KP3S 3.0 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 513 (line 21, column 9; expected 92286 bytes, actual 90491 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R3"; actual "M73 P0 R2"   line 22: "M201 X4000 Y4000 Z1100 E10000" |
| PASS | Kingroon/Kingroon KP3S PRO S1 0.4 nozzle |  |
| DIVERGENT | Kingroon/Kingroon KP3S PRO V2 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 940 (line 39, column 3; expected 103623 bytes, actual 103327 bytes) context:   line 38: ""   line 39: expected "G10 ; retract"; actual "G1 E-.8 F2700"   line 40: ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Kingroon/Kingroon KP3S V1 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 5243 (line 217, column 5; expected 107950 bytes, actual 107938 bytes) context:   line 216: "G1 F4075"   line 217: expected "G1 X94.398 Y85.602 E.29177"; actual "G1 X85.602 Y94.398 E.29177"   line 218: "G1 X85.602 Y85.602 E.29177" |
| PASS | LH/LH Stinger 0.4 nozzle |  |
| DIVERGENT | LH/LH Stinger MMU 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 627 (line 23, column 1; expected 135321 bytes, actual 135322 bytes) context:   line 22: ";TYPE:Custom"   line 23: expected "_SP_PRINT_START LANE=0 TEMP=230"; actual " _SP_PRINT_START LANE=0 TEMP=230"   line 24: "" |
| PASS | LONGER/LONGER LK10 (0.2 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 (0.8 nozzle) | (no match across 3 oracle runs) first difference at byte 51264 (line 2164, column 29; expected 103095 bytes, actual 103095 bytes) context:   line 2163: "G1 X110.137 Y110.137 E-.05789"   line 2164: expected "G1 X110.944 Y110.137 E-.24211"; actual "G1 X110.944 Y110.137 E-.24212"   line 2165: ";WIPE_END" |
| PASS | LONGER/LONGER LK10 Plus (0.2 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.8 nozzle) | (no match across 3 oracle runs) first difference at byte 51208 (line 2164, column 29; expected 103059 bytes, actual 103059 bytes) context:   line 2163: "G1 X157.637 Y157.637 E-.05789"   line 2164: expected "G1 X158.444 Y157.637 E-.24211"; actual "G1 X158.444 Y157.637 E-.24212"   line 2165: ";WIPE_END" |
| PASS | Lulzbot/Lulzbot Taz 4 or 5 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz 6 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz Pro Dual 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz Pro S 0.5 nozzle |  |
| PASS | M3D/M3D Enabler D8500 MM |  |
| PASS | MagicMaker/MM BoneKing 0.4 nozzle |  |
| PASS | MagicMaker/MM hj SK 0.4 nozzle |  |
| PASS | MagicMaker/MM hqs SF 0.4 nozzle |  |
| PASS | MagicMaker/MM hqs hj 0.4 nozzle |  |
| PASS | MagicMaker/MM slb 0.4 nozzle |  |
| DIVERGENT | Mellow/M1 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 375170 (line 15639, column 62; expected 395090 bytes, actual 395090 bytes) context:   line 15638: "; estimated printing time (normal mode) = 10m 11s"   line 15639: expected "; estimated first layer printing time (normal mode) = 0.438319s"; actual "; estimated first layer printing time (normal mode) = 0.438318s"   line 15640: "" |
| PASS | Mellow/M1 0.4 nozzle |  |
| PASS | Mellow/M1 0.6 nozzle |  |
| PASS | Mellow/M1 0.8 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.2 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.4 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.6 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.8 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.2 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.4 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.6 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.8 nozzle |  |
| DIVERGENT | Peopoly/Peopoly Magneto X 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 185662 (line 6677, column 62; expected 206009 bytes, actual 206009 bytes) context:   line 6676: "; estimated printing time (normal mode) = 7m 15s"   line 6677: expected "; estimated first layer printing time (normal mode) = 0.874670s"; actual "; estimated first layer printing time (normal mode) = 0.874671s"   line 6678: "" |
| PASS | Peopoly/Peopoly Magneto X 0.6 nozzle |  |
| PASS | Peopoly/Peopoly Magneto X 0.8 nozzle |  |
| PASS | Phrozen/Phrozen Arco 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.2 nozzle |  |
| PASS | Positron3D/The Positron 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.6 nozzle |  |
| PASS | Positron3D/The Positron 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.25 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 6638 (line 299, column 1; expected 95925 bytes, actual 96233 bytes) context:   line 298: "G1 X126.083 Y106.021 E.01949"   line 299: expected "M73 P86 R4"; actual "G1 X121.021 Y111.083 E.24231"   line 300: expected "G1 X121.021 Y111.083 E.24231"; actual "G1 X121.021 Y110.507 E.01949" |
| PASS | Prusa/Prusa CORE One 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One HF 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 6631 (line 297, column 1; expected 95737 bytes, actual 95737 bytes) context:   line 296: "G1 X121.021 Y111.083 E.24231"   line 297: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 298: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa CORE One HF 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One L 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 7475 (line 343, column 6; expected 95769 bytes, actual 96077 bytes) context:   line 342: ";WIDTH:0.45"   line 343: expected "G1 F1912"; actual "G1 F1899"   line 344: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L 0.5 nozzle | (no match across 3 oracle runs) first difference at byte 514 (line 21, column 9; expected 90693 bytes, actual 90702 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 2106 (line 106, column 7; expected 75646 bytes, actual 75886 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P89 R3"; actual "M73 P88 R3"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 2106 (line 106, column 7; expected 51843 bytes, actual 51842 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 7439 (line 340, column 6; expected 95609 bytes, actual 95609 bytes) context:   line 339: ";WIDTH:0.45"   line 340: expected "G1 F1912"; actual "G1 F1899"   line 341: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.5 nozzle | (no match across 3 oracle runs) first difference at byte 514 (line 21, column 9; expected 89437 bytes, actual 89697 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 2107 (line 106, column 7; expected 67236 bytes, actual 67235 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P91 R2"; actual "M73 P90 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 2106 (line 106, column 7; expected 51902 bytes, actual 51901 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| PASS | Prusa/Prusa MINI 0.25 nozzle |  |
| PASS | Prusa/Prusa MINI 0.4 nozzle |  |
| DIVERGENT | Prusa/Prusa MINI 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 2623 (line 127, column 1; expected 89154 bytes, actual 89826 bytes) context:   line 126: "G1 X92.735 Y92.223 E.06048"   line 127: expected "M73 P40 R6"; actual "G1 X87.777 Y87.265 E.41393"   line 128: expected "G1 X87.777 Y87.265 E.41393"; actual "M73 P40 R6" |
| PASS | Prusa/Prusa MINI 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MINIIS 0.25 nozzle | (no match across 3 oracle runs) first difference at byte 3298 (line 151, column 1; expected 214079 bytes, actual 215474 bytes) context:   line 150: "G1 X88.893 Y86.184 E.00766"   line 151: expected "M73 P24 R14"; actual "G1 X93.816 Y91.107 E.12799"   line 152: expected "G1 X93.816 Y91.107 E.12799"; actual "G1 X93.816 Y91.524 E.00766" |
| DIVERGENT | Prusa/Prusa MINIIS 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 2756 (line 132, column 1; expected 106565 bytes, actual 107307 bytes) context:   line 131: "G1 X93.25 Y87.949 E.02581"   line 132: expected "M73 P35 R8"; actual "G1 X92.051 Y86.75 E.06609"   line 133: expected "G1 X92.051 Y86.75 E.06609"; actual "G1 X91.388 Y86.75 E.02581" |
| PASS | Prusa/Prusa MINIIS 0.6 nozzle |  |
| PASS | Prusa/Prusa MINIIS 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3.5 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 4896 (line 228, column 1; expected 111095 bytes, actual 111095 bytes) context:   line 227: "G1 X129.775 Y100.225 E.32326"   line 228: expected "M73 P72 R8"; actual "G1 X129.775 Y109.735 E.3219"   line 229: expected "G1 X129.775 Y109.735 E.3219"; actual "M73 P72 R8" |
| PASS | Prusa/Prusa MK3.5 0.6 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 4896 (line 217, column 1; expected 94564 bytes, actual 95415 bytes) context:   line 216: "G1 X131.248 Y98.202 E.04019"   line 217: expected "M73 P4 R6"; actual "G1 X132.051 Y99.235 E.04019"   line 218: expected "G1 X132.051 Y99.235 E.04019"; actual "G1 X132.189 Y100 E.02388" |
| PASS | Prusa/Prusa MK3S 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 1810 (line 83, column 1; expected 63607 bytes, actual 64861 bytes) context:   line 82: "G1 X120 Y112.418 E1.02139"   line 83: expected "M73 P3 R4"; actual "G1 X118.681 Y112.026 E.14058"   line 84: expected "G1 X118.681 Y112.026 E.14058"; actual "M73 P3 R4" |
| PASS | Prusa/Prusa MK4 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4 0.4 nozzle |  |
| PASS | Prusa/Prusa MK4 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4 0.8 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 4519 (line 213, column 2; expected 92830 bytes, actual 92830 bytes) context:   line 212: "G1 E-.7 F2100"   line 213: expected "M73 P84 R4"; actual "M486 S0"   line 214: expected "M486 S0"; actual "G1 X129.325 Y109.325 Z.6 F18000" |
| PASS | Prusa/Prusa MK4S 0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S HF0.4 nozzle | (no match across 3 oracle runs) first difference at byte 7240 (line 326, column 7; expected 92234 bytes, actual 92234 bytes) context:   line 325: ";WIDTH:0.45"   line 326: expected "G1 F2560"; actual "G1 F2544"   line 327: "G1 X120.675 Y109.325 E.29279" |
| PASS | Prusa/Prusa MK4S HF0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.8 nozzle |  |
| PASS | Prusa/Prusa XL 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.3 nozzle | (no match across 3 oracle runs) first difference at byte 2216 (line 81, column 1; expected 125488 bytes, actual 125536 bytes) context:   line 80: "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"   line 81: expected "M73 P77 R9"; actual "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"   line 82: expected "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"; actual "G92 E0 ; reset extruder position" |
| DIVERGENT | Prusa/Prusa XL 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 3163 (line 134, column 1; expected 92853 bytes, actual 93155 bytes) context:   line 133: "G1 Z.2"   line 134: expected "M73 P83 R6"; actual "G1 E.8 F1800"   line 135: expected "G1 E.8 F1800"; actual ";TYPE:Bottom surface" |
| PASS | Prusa/Prusa XL 0.5 nozzle |  |
| PASS | Prusa/Prusa XL 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 4422 (line 198, column 22; expected 53778 bytes, actual 53824 bytes) context:   line 197: ";WIPE_END"   line 198: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 199: expected "G1 E.6 F1800"; actual "G1 Z.6" |
| PASS | Prusa/Prusa XL 5T 0.25 nozzle |  |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.3 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-MbZdFa") |
| DIVERGENT | Prusa/Prusa XL 5T 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 3333 (line 168, column 1; expected 103701 bytes, actual 103749 bytes) context:   line 167: "G1 X184.55 Y184.55 F24000"   line 168: expected "M73 P83 R6"; actual ";TYPE:Outer wall"   line 169: expected ";TYPE:Outer wall"; actual "G1 F1238" |
| PASS | Prusa/Prusa XL 5T 0.5 nozzle |  |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-jSLmYD") |
| DIVERGENT | Prusa/Prusa XL 5T 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 4856 (line 248, column 22; expected 63840 bytes, actual 64400 bytes) context:   line 247: ";WIPE_END"   line 248: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 249: expected "G1 E.8 F1800"; actual "G1 Z.6" |
| PASS | Qidi/Qidi Q1 Pro 0.2 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.4 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.6 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.8 nozzle |  |
| PASS | Qidi/Qidi Q2 0.2 nozzle |  |
| PASS | Qidi/Qidi Q2 0.4 nozzle |  |
| PASS | Qidi/Qidi Q2 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2 0.8 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.2 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.4 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.8 nozzle |  |
| PASS | Qidi/Qidi X-CF Pro 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 0.4 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Max 3 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 6982 (line 272, column 1; expected 299371 bytes, actual 299357 bytes) context:   line 271: "G1 X167.191 Y157.809 E.0759"   line 272: expected "G1 X167.191 Y167.161 E.07566"; actual "M73 P26 R14"   line 273: expected "M73 P26 R14"; actual "G1 X167.191 Y167.161 E.07566" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 15161 (line 656, column 1; expected 121136 bytes, actual 121138 bytes) context:   line 655: "G1 X167.29 Y157.71 E.29437"   line 656: expected "M73 P52 R5"; actual "G1 X167.29 Y167.23 E.29252"   line 657: expected "G1 X167.29 Y167.23 E.29252"; actual "M73 P52 R5" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 2412 (line 110, column 1; expected 75167 bytes, actual 75690 bytes) context:   line 109: "G1 X166.068 Y161.21 E.05665"   line 110: expected "M73 P57 R3"; actual "G1 X163.79 Y158.932 E.22542"   line 111: expected "G1 X163.79 Y158.932 E.22542"; actual "M73 P57 R3" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 1025 (line 48, column 1; expected 68790 bytes, actual 69154 bytes) context:   line 47: "G1 X152.5 E0.32 F3000"   line 48: expected "M73 P61 R3"; actual "G1 Y153.5 E13.12 F3000"   line 49: expected "G1 Y153.5 E13.12 F3000"; actual "G1 X162.5 E-1.6 F3000" |
| PASS | Qidi/Qidi X-Max 4 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.8 nozzle |  |
| DIVERGENT | RH3D/E3NG v1.2S - 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 13777 (line 523, column 11; expected 291096 bytes, actual 291096 bytes) context:   line 522: "G1 X121 Y105.338 E.10775"   line 523: expected "G1 X123.081 Y105.734 E.02283"; actual "G1 X123.082 Y105.734 E.02283"   line 524: "G1 X124.872 Y106.868 E.02283" |
| PASS | RH3D/E3NG v1.2S - 0.3 nozzle |  |
| PASS | RH3D/E3NG v1.2S - 0.4 nozzle |  |
| PASS | RH3D/E3NG v1.2S - 0.5 nozzle |  |
| PASS | RH3D/E3NG v1.2S - 0.6 nozzle |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Dual) |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Left) |  |
| PASS | Raise3D/Raise3D Pro3 0.4 nozzle (Right) |  |
| PASS | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Dual) |  |
| PASS | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Left) |  |
| PASS | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Right) |  |
| PASS | Ratrig/RatRig V-Cast 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Cast 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 200 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 300 0.8 nozzle | Ratrig/RatRig V-Core 4 300 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 400 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 400 0.8 nozzle | Ratrig/RatRig V-Core 4 400 0.8 nozzle process: no compatible preset |
| DIVERGENT | Ratrig/RatRig V-Core 4 500 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 5175 (line 196, column 1; expected 113883 bytes, actual 114206 bytes) context:   line 195: "G1 X246.65 Y250.879 E.13101"   line 196: expected "M73 P6 R2"; actual "G1 X246.65 Y251.53 E.02439"   line 197: expected "G1 X246.65 Y251.53 E.02439"; actual "M73 P6 R2" |
| PASS | Ratrig/RatRig V-Core 4 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 500 0.8 nozzle | Ratrig/RatRig V-Core 4 500 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.5 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 300 0.6 nozzle | (no match across 3 oracle runs) first difference at byte 82618 (line 3381, column 61; expected 102863 bytes, actual 102863 bytes) context:   line 3380: "; estimated printing time (normal mode) = 2m 45s"   line 3381: expected "; estimated first layer printing time (normal mode) = 0.455069s"; actual "; estimated first layer printing time (normal mode) = 0.455037s"   line 3382: "" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 500 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 62871 (line 2488, column 61; expected 83126 bytes, actual 83126 bytes) context:   line 2487: "; estimated printing time (normal mode) = 2m 48s"   line 2488: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2489: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.8 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 5936 (line 223, column 7; expected 113394 bytes, actual 113834 bytes) context:   line 222: ";WIDTH:0.45"   line 223: expected "G1 F3715"; actual "G1 F3721"   line 224: "G1 X146.082 Y153.918 E.26523" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 63890 (line 2567, column 61; expected 84764 bytes, actual 84764 bytes) context:   line 2566: "; estimated printing time (normal mode) = 2m 48s"   line 2567: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2568: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.8 nozzle | (no match across 3 oracle runs) first difference at byte 63904 (line 2569, column 61; expected 84822 bytes, actual 84822 bytes) context:   line 2568: "; estimated printing time (normal mode) = 2m 48s"   line 2569: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2570: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Minion 0.4 nozzle |  |
| DIVERGENT | RolohaunDesign/Rolohaun Delta Flyer Refit 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 13371 (line 532, column 1; expected 125851 bytes, actual 126094 bytes) context:   line 531: "G1 E-1.4 F3600"   line 532: expected "M73 P10 R3"; actual ";WIPE_START"   line 533: expected ";WIPE_START"; actual "G1 F9000" |
| PASS | RolohaunDesign/Rook MK1 LDO 0.2 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.4 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.6 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.8 nozzle |  |
| PASS | SecKit/SecKit Go3 0.4 nozzle |  |
| PASS | SecKit/SecKit SK-Tank 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC Artemis 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 300 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v3.2 1.0 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.4 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.5 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 0.7 nozzle |  |
| PASS | SeeMeCNC/SeeMeCNC RostockMAX v4 1.0 nozzle |  |
| PASS | Snapmaker/Snapmaker A250 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker Artisan (0.2 nozzle) | (no match across 3 oracle runs) first difference at byte 10242 (line 457, column 7; expected 187817 bytes, actual 188207 bytes) context:   line 456: "G1 X199.125 Y195.786 E.04833"   line 457: expected "G1 X197.232 Y196.47 F11400"; actual "G1 X196.709 Y196.665 F11400"   line 458: expected "G1 F3799.871"; actual ";WIDTH:0.286708" |
| PASS | Snapmaker/Snapmaker Artisan (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker U1 (0.2 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-bI5Knv") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4+0.6 nozzle) | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-xoHSOF") |
| PASS | Snapmaker/Snapmaker U1 (0.6 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.8 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-l5z2kw") |
| PASS | Sovol/Sovol SV01 0.4 nozzle |  |
| PASS | Sovol/Sovol SV01 Pro 0.4 nozzle |  |
| PASS | Sovol/Sovol SV02 0.4 nozzle |  |
| PASS | Sovol/Sovol SV05 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV06 0.4 High-Speed nozzle | (no match across 3 oracle runs) first difference at byte 1666 (line 90, column 19; expected 101958 bytes, actual 101958 bytes) context:   line 89: "G1 X117.495 Y97.735 E.05224"   line 90: expected "G1 X118.551 Y98.188 E.05224"; actual "G1 X118.551 Y98.189 E.05224"   line 91: "G1 X119.527 Y98.794 E.05224" |
| PASS | Sovol/Sovol SV06 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.2 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.6 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.8 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV07 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV07 Plus 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 4032 (line 174, column 1; expected 107311 bytes, actual 107311 bytes) context:   line 173: "G1 X146.375 Y150.578 E5.51963"   line 174: expected "G1 X149.422 Y153.625 E5.67617"; actual "M106 S255"   line 175: expected "G1 X148.896 Y153.625 E5.69525"; actual "G1 X149.422 Y153.625 E5.67617" |
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
| PASS | Tiertime/Tiertime UP400 Pro 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.6 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.8 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.6 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.8 nozzle |  |
| PASS | Tronxy/Tronxy X5SA 400 0.4 nozzle |  |
| DIVERGENT | TwoTrees/TwoTrees SK1 0.4 nozzle | (no match across 3 oracle runs) first difference at byte 1414 (line 60, column 1; expected 101783 bytes, actual 102276 bytes) context:   line 59: "G1 X190 Y12 F6000 ;Wipe"   line 60: expected "M73 P66 R2"; actual "G1 X180 Y8 F6000 ;Wipe"   line 61: expected "G1 X180 Y8 F6000 ;Wipe"; actual "G1 X170 Y12 F6000 ;Wipe" |
| PASS | TwoTrees/TwoTrees SP-5 Klipper 0.4 nozzle |  |
| PASS | UltiMaker/UltiMaker 2 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 Klipper 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 RRF 0.4 nozzle |  |
| DIVERGENT | Volumic/EXO42 (0.4 nozzle) | (no match across 3 oracle runs) first difference at byte 4023 (line 184, column 1; expected 119642 bytes, actual 120292 bytes) context:   line 183: "G1 X214.28 Y214.24 E.23794"   line 184: expected "M73 P4 R4"; actual "G1 X214.328 Y214.24 F8100"   line 185: expected "G1 X214.328 Y214.24 F8100"; actual "M73 P4 R4" |
| PASS | Volumic/EXO42 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Stage 2 (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO65 (0.6 nozzle) | (no match across 3 oracle runs) first difference at byte 745 (line 43, column 9; expected 111569 bytes, actual 115034 bytes) context:   line 42: "G1 E-2.4 F1800"   line 43: expected "M73 P2 R4"; actual "M73 P2 R5"   line 44: ";_SET_FAN_SPEED_CHANGING_LAYER" |
| PASS | Volumic/EXO65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.6 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.8 nozzle) |  |
| PASS | Volumic/EXO65 Stage 2 (0.6 nozzle) |  |
| DIVERGENT | Volumic/SH65 (0.4 nozzle) | (no match across 3 oracle runs) first difference at byte 1950 (line 98, column 1; expected 119886 bytes, actual 120290 bytes) context:   line 97: "G1 X329.76 Y145.24 E.26586"   line 98: expected "M73 P3 R5"; actual "G1 X329.76 Y154.72 E.26474"   line 99: expected "G1 X329.76 Y154.72 E.26474"; actual "M73 P3 R5" |
| PASS | Volumic/SH65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 Performance (0.4 nozzle) |  |
| PASS | Volumic/SH65 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/VS20MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK3 (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30MK3 Stage 2 (0.4 nozzle) | (no match across 3 oracle runs) first difference at byte 18421 (line 801, column 1; expected 101231 bytes, actual 101231 bytes) context:   line 800: "G1 X146.507 Y98.548 E.10493"   line 801: expected "G1 X146.507 Y96.507 E.0742"; actual "M73 P18 R3"   line 802: expected "M73 P18 R3"; actual "G1 X146.507 Y96.507 E.0742" |
| PASS | Volumic/VS30SC (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 Performance (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30SC2 Stage 2 (0.4 nozzle) | (no match across 3 oracle runs) first difference at byte 20655 (line 901, column 1; expected 100924 bytes, actual 101231 bytes) context:   line 900: "G1 X145.72 Y95.72 E.3111"   line 901: expected "M73 P21 R3"; actual "G1 X154.28 Y95.72 E.3111"   line 902: expected "G1 X154.28 Y95.72 E.3111"; actual "M73 P21 R3" |
| PASS | Volumic/VS30ULTRA (0.4 nozzle) |  |
| PASS | Voron/Voron 0.1 0.15 nozzle |  |
| DIVERGENT | Voron/Voron 0.1 0.2 nozzle | (no match across 3 oracle runs) first difference at byte 333887 (line 12739, column 62; expected 354370 bytes, actual 354370 bytes) context:   line 12738: "; estimated printing time (normal mode) = 8m 24s"   line 12739: expected "; estimated first layer printing time (normal mode) = 0.486746s"; actual "; estimated first layer printing time (normal mode) = 0.486747s"   line 12740: "" |
| PASS | Voron/Voron 0.1 0.25 nozzle |  |
| PASS | Voron/Voron 0.1 0.4 nozzle |  |
| PASS | Voron/Voron 0.1 0.5 nozzle |  |
| PASS | Voron/Voron 0.1 0.6 nozzle |  |
| PASS | Voron/Voron 0.1 0.8 nozzle |  |
| PASS | Voron/Voron 0.1 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.2 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.25 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.6 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.8 nozzle |  |
| PASS | Voron/Voron 2.4 250 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.2 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.25 nozzle |  |
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
| PASS | Voron/Voron Switchwire 250 0.4 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.5 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.6 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.8 nozzle |  |
| PASS | Voron/Voron Switchwire 250 1.0 nozzle |  |
| PASS | Voron/Voron Trident 250 0.15 nozzle |  |
| PASS | Voron/Voron Trident 250 0.2 nozzle |  |
| PASS | Voron/Voron Trident 250 0.25 nozzle |  |
| PASS | Voron/Voron Trident 250 0.4 nozzle |  |
| PASS | Voron/Voron Trident 250 0.5 nozzle |  |
| PASS | Voron/Voron Trident 250 0.6 nozzle |  |
| PASS | Voron/Voron Trident 250 0.8 nozzle |  |
| PASS | Voron/Voron Trident 250 1.0 nozzle |  |
| PASS | Voron/Voron Trident 300 0.15 nozzle |  |
| PASS | Voron/Voron Trident 300 0.2 nozzle |  |
| PASS | Voron/Voron Trident 300 0.25 nozzle |  |
| PASS | Voron/Voron Trident 300 0.4 nozzle |  |
| PASS | Voron/Voron Trident 300 0.5 nozzle |  |
| PASS | Voron/Voron Trident 300 0.6 nozzle |  |
| PASS | Voron/Voron Trident 300 0.8 nozzle |  |
| PASS | Voron/Voron Trident 300 1.0 nozzle |  |
| PASS | Voron/Voron Trident 350 0.15 nozzle |  |
| PASS | Voron/Voron Trident 350 0.2 nozzle |  |
| PASS | Voron/Voron Trident 350 0.25 nozzle |  |
| PASS | Voron/Voron Trident 350 0.4 nozzle |  |
| PASS | Voron/Voron Trident 350 0.5 nozzle |  |
| PASS | Voron/Voron Trident 350 0.6 nozzle |  |
| PASS | Voron/Voron Trident 350 0.8 nozzle |  |
| PASS | Voron/Voron Trident 350 1.0 nozzle |  |
| PASS | Voxelab/Voxelab Aquila X2 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.5 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.6 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.5 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.6 nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.2mm nozzle | (no match across 3 oracle runs) first difference at byte 3985 (line 171, column 7; expected 213124 bytes, actual 213150 bytes) context:   line 170: "G1 E-1.2 F7200"   line 171: expected "G1 X148.896 Y148.972 Z.56 F15000"; actual "G1 X145.864 Y150.863 Z.56 F15000"   line 172: expected "G1 X153.319 Y153.319 Z.56"; actual "G1 X146.681 Y153.319 Z.56" |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.3mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.4mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.6mm nozzle | (no match across 3 oracle runs) first difference at byte 1690 (line 83, column 1; expected 109578 bytes, actual 109765 bytes) context:   line 82: "G1 X157.576 Y156.477 E.10062"   line 83: expected "M73 P44 R6"; actual "G1 X156.501 Y157.562 E.10062"   line 84: expected "G1 X156.501 Y157.562 E.10062"; actual "M73 P44 R6" |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.2mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.3mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.4mm nozzle | (no match across 3 oracle runs) first difference at byte 2009 (line 96, column 1; expected 132655 bytes, actual 132656 bytes) context:   line 95: "G1 X110 Y97.784 E.56236"   line 96: expected "M73 P43 R6"; actual "G1 X111.257 Y98.175 E.07402"   line 97: expected "G1 X111.257 Y98.175 E.07402"; actual "M73 P43 R6" |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.6mm nozzle |  |
| PASS | Wanhao/Wanhao D12-300 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle PoopTool |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle PoopTool |  |
| PASS | Wanhao France/D12 300 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.2 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.8 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.2 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-LQRbdX") |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.6 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-oBmb7Q") |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.2 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra S 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-vORT6j/command-P3v005") |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.8 nozzle |  |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.25 Nozzle | iQ/iQ TiQ2 0.25 Nozzle process: no compatible preset |
| PASS | iQ/iQ TiQ2 0.4 Nozzle |  |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.6 Nozzle | iQ/iQ TiQ2 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.8 Nozzle | iQ/iQ TiQ2 0.8 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.25 Nozzle | iQ/iQ TiQ8 0.25 Nozzle process: no compatible preset |
| DIVERGENT | iQ/iQ TiQ8 0.4 Nozzle | (no match across 3 oracle runs) first difference at byte 22135 (line 885, column 6; expected 130340 bytes, actual 130358 bytes) context:   line 884: "G1 Z1 F18000"   line 885: expected "G1 X246.061 Y187.924 Z1"; actual "G1 X237.93 Y195.977 Z1"   line 886: "G1 Z.6" |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.6 Nozzle | iQ/iQ TiQ8 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.8 Nozzle | iQ/iQ TiQ8 0.8 Nozzle process: no compatible preset |
| PASS | re3D/re3D Gigabot 4 0.4 nozzle |  |
| PASS | re3D/re3D Gigabot 4 0.8 nozzle |  |
| PASS | re3D/re3D Gigabot 4 XLT 0.4 nozzle |  |
| PASS | re3D/re3D Gigabot 4 XLT 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 1.75 nozzle |  |
| PASS | re3D/re3D GigabotX 2 XLT 0.8 nozzle |  |
| PASS | re3D/re3D GigabotX 2 XLT 1.75 nozzle |  |
| PASS | re3D/re3D Terabot 4 0.4 nozzle |  |
| PASS | re3D/re3D Terabot 4 0.8 nozzle |  |
| PASS | re3D/re3D TerabotX 2 0.8 nozzle |  |
| PASS | re3D/re3D TerabotX 2 1.75 nozzle |  |
