# OrcaSlicer printer smoke summary

894 of 1001 printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; classic wall generator baseline; cube model).

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
| PASS | Anycubic/Anycubic Kobra 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Neo 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Plus 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Pro 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra 3 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 18994 (line 661, column 29; expected 454235 bytes, actual 454235 bytes) context:   line 660: ";WIPE_START"   line 661: expected "G1 X121.761 Y120.566 E-.70072"; actual "G1 X121.761 Y120.566 E-.70071"   line 662: expected "G1 X121.88 Y120.53 E-.09928"; actual "G1 X121.88 Y120.53 E-.09929" |
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
| PASS | Artillery/Artillery Sidewinder X1 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X2 0.4 nozzle |  |
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
| PASS | BBL/Bambu Lab H2D 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2D 0.8 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2D Pro 0.8 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.8 nozzle |  |
| PASS | BBL/Bambu Lab P1P 0.2 nozzle |  |
| PASS | BBL/Bambu Lab P1P 0.4 nozzle |  |
| PASS | BBL/Bambu Lab P1P 0.6 nozzle |  |
| PASS | BBL/Bambu Lab P1P 0.8 nozzle |  |
| PASS | BBL/Bambu Lab P1S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab P1S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab P1S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab P1S 0.8 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab X1 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 70518 (line 1510, column 2; expected 396001 bytes, actual 395920 bytes) context:   line 1509: ";_SET_FAN_SPEED_CHANGING_LAYER"   line 1510: expected "M976 S1 P1 ; scan model before printing 2nd layer"; actual "M204 S10000"   line 1511: expected "M400 P100"; actual "G17" |
| DIVERGENT | BBL/Bambu Lab X1 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 54793 (line 800, column 1; expected 168931 bytes, actual 169164 bytes) context:   line 799: "G1 X100 F5000"   line 800: expected "G1 X70 F15000"; actual "M73 P43 R7"   line 801: expected "M73 P43 R7"; actual "G1 X70 F15000" |
| DIVERGENT | BBL/Bambu Lab X1 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 123 (line 3, column 58; expected 127211 bytes, actual 127128 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 43s; total estimated time: 11m 1s"; actual "; model printing time: 3m 43s; total estimated time: 11m 0s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
| DIVERGENT | BBL/Bambu Lab X1 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 54139 (line 768, column 1; expected 112013 bytes, actual 111934 bytes) context:   line 767: "G1 X76 F15000"   line 768: expected "G1 X65 F15000; shake to put down garbage"; actual "M73 P54 R4"   line 769: expected "G1 X80 F6000"; actual "G1 X65 F15000; shake to put down garbage" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 70568 (line 1514, column 2; expected 396062 bytes, actual 395981 bytes) context:   line 1513: ";_SET_FAN_SPEED_CHANGING_LAYER"   line 1514: expected "M976 S1 P1 ; scan model before printing 2nd layer"; actual "M204 S10000"   line 1515: expected "M400 P100"; actual "G17" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 92 (line 3, column 27; expected 169057 bytes, actual 169169 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 7s; total estimated time: 11m 23s"; actual "; model printing time: 4m 6s; total estimated time: 11m 23s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 55769 (line 862, column 1; expected 127323 bytes, actual 127240 bytes) context:   line 861: "G2 I0.5 J0 F3000"   line 862: expected "G2 I0.5 J0 F3000"; actual "M73 P60 R3"   line 863: "G2 I0.5 J0 F3000" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 92 (line 3, column 27; expected 112166 bytes, actual 112087 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 9s; total estimated time: 9m 32s"; actual "; model printing time: 2m 8s; total estimated time: 9m 32s"   line 4: "; estimated first layer printing time (normal mode) = 7m 23s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 52759 (line 730, column 1; expected 394389 bytes, actual 394308 bytes) context:   line 729: "    G1 X120 F12000"   line 730: expected ""; actual "M73 P1 R18"   line 731: expected "    G1 X20 Y50 F12000"; actual "" |
| DIVERGENT | BBL/Bambu Lab X1E 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 92 (line 3, column 27; expected 167733 bytes, actual 167652 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 6s; total estimated time: 11m 33s"; actual "; model printing time: 4m 5s; total estimated time: 11m 33s"   line 4: "; estimated first layer printing time (normal mode) = 7m 27s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 54906 (line 862, column 1; expected 125678 bytes, actual 125595 bytes) context:   line 861: "G2 I0.5 J0 F3000"   line 862: expected "G2 I0.5 J0 F3000"; actual "M73 P61 R4"   line 863: expected ""; actual "G2 I0.5 J0 F3000" |
| DIVERGENT | BBL/Bambu Lab X1E 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 92 (line 3, column 27; expected 110322 bytes, actual 110532 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 8s; total estimated time: 9m 45s"; actual "; model printing time: 2m 7s; total estimated time: 9m 44s"   line 4: "; estimated first layer printing time (normal mode) = 7m 37s" |
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
| DIVERGENT | Creality/Creality CR-6 Max 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 7041 (line 291, column 29; expected 394066 bytes, actual 394066 bytes) context:   line 290: "G1 X195.605 Y204.196 E-.58892"   line 291: expected "G1 X195.605 Y203.947 E-.37351"; actual "G1 X195.605 Y203.947 E-.37352"   line 292: "G1 X195.858 Y204.2 E-.53756" |
| PASS | Creality/Creality CR-6 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.8 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 SE 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 7068 (line 292, column 29; expected 394163 bytes, actual 394163 bytes) context:   line 291: "G1 X113.105 Y121.696 E-.58892"   line 292: expected "G1 X113.105 Y121.447 E-.37351"; actual "G1 X113.105 Y121.447 E-.37352"   line 293: "G1 X113.358 Y121.7 E-.53756" |
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
| PASS | Creality/Creality Ender-3 V3 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 Plus 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 22406 (line 893, column 1; expected 123624 bytes, actual 123624 bytes) context:   line 892: "G1 X146.052 Y146.052 E.26192"   line 893: expected "M73 P21 R4"; actual "G1 X153.948 Y146.052 E.26192"   line 894: expected "G1 X153.948 Y146.052 E.26192"; actual "M73 P21 R4" |
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
| PASS | Creality/Creality Ender-5 Pro (2019) 0.5 nozzle |  |
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
| PASS | Creality/Creality K1 SE_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K1C 0.4 nozzle |  |
| PASS | Creality/Creality K1C 0.6 nozzle |  |
| PASS | Creality/Creality K1C 0.8 nozzle |  |
| PASS | Creality/Creality K1C_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K1_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K2 0.2 nozzle |  |
| PASS | Creality/Creality K2 0.4 nozzle |  |
| PASS | Creality/Creality K2 0.6 nozzle |  |
| PASS | Creality/Creality K2 0.8 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.2 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K2 Plus 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 3372 (line 142, column 1; expected 95401 bytes, actual 95769 bytes) context:   line 141: "G1 X174.52 Y178.568 E.29616"   line 142: expected "G1 X173.71 Y178.568 E.05491"; actual "M73 P16 R3"   line 143: expected "M73 P16 R3"; actual "G1 X173.71 Y178.568 E.05491" |
| DIVERGENT | Creality/Creality K2 Plus 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 5106 (line 218, column 1; expected 79002 bytes, actual 79002 bytes) context:   line 217: "G1 X175.467 Y178.313 E.38216"   line 218: expected "M73 P20 R2"; actual "G1 X174.404 Y178.313 E.1009"   line 219: expected "G1 X174.404 Y178.313 E.1009"; actual "M73 P20 R2" |
| PASS | Creality/Creality K2 Pro 0.2 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.4 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.6 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.8 nozzle |  |
| PASS | Creality/Creality K2 SE 0.4 nozzle |  |
| DIVERGENT | Creality/Creality SPARKX i7 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 22156 (line 808, column 1; expected 361301 bytes, actual 361673 bytes) context:   line 807: "G1 X134.126 Y129.358 E.00227"   line 808: expected "M73 P8 R13"; actual "G1 X129.358 Y134.126 E.05455"   line 809: expected "G1 X129.358 Y134.126 E.05455"; actual "G1 X129.077 Y134.126 E.00227" |
| DIVERGENT | Creality/Creality SPARKX i7 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 2827 (line 132, column 1; expected 137363 bytes, actual 137362 bytes) context:   line 131: "G1 X129.221 Y126.2 E.26305"   line 132: expected "M73 P9 R6"; actual "G1 X128.55 Y126.2 E.02728"   line 133: expected "G1 X128.55 Y126.2 E.02728"; actual "M73 P9 R6" |
| DIVERGENT | Creality/Creality SPARKX i7 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 2217 (line 107, column 1; expected 104151 bytes, actual 104148 bytes) context:   line 106: "G1 X134.123 Y127.322 E.1484"   line 107: expected "M73 P13 R4"; actual "G1 X134.123 Y128.122 E.05811"   line 108: expected "G1 X134.123 Y128.122 E.05811"; actual "M73 P13 R4" |
| DIVERGENT | Creality/Creality SPARKX i7 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 954 (line 46, column 7; expected 88493 bytes, actual 88503 bytes) context:   line 45: "G1 X115 E.3742  F1600"   line 46: expected "M73 P15 R3"; actual "M73 P14 R3"   line 47: "G1 X110 E.3742  F6400" |
| DIVERGENT | Creality/Creality Sermoon V1 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 2015 (line 53, column 9; expected 162827 bytes, actual 167626 bytes) context:   line 52: "G1 Z2.0 F3000                          ; Move Z Axis up"   line 53: expected "M73 P3 R6"; actual "M73 P3 R7"   line 54: "G92 E0" |
| ORCA_ERROR | Cubicon/Cubicon xCeler-I 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-Cryraw") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Mini 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-3HHxHQ") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Plus 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-LAuO1r") |
| PASS | Custom/MyKlipper 0.2 nozzle |  |
| PASS | Custom/MyKlipper 0.4 nozzle |  |
| PASS | Custom/MyKlipper 0.6 nozzle |  |
| PASS | Custom/MyKlipper 0.8 nozzle |  |
| PASS | Custom/MyMarlin 0.4 nozzle |  |
| PASS | Custom/MyRRF 0.4 nozzle |  |
| PASS | Custom/MyRepetier 0.4 nozzle |  |
| ORCA_ERROR | Custom/MyToolChanger 0.2 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-JXpsGz") |
| PASS | Custom/MyToolChanger 0.4 nozzle |  |
| ORCA_ERROR | Custom/MyToolChanger 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-5Mn7J4") |
| ORCA_ERROR | Custom/MyToolChanger 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-d4dm0n") |
| PASS | DeltaMaker/DeltaMaker 2 0.35 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2T 0.5 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2XT 0.5 nozzle |  |
| PASS | Dremel/Dremel 3D20 0.4 nozzle |  |
| PASS | Dremel/Dremel 3D40 0.4 nozzle |  |
| PASS | Dremel/Dremel 3D45 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.8 nozzle |  |
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
| PASS | Elegoo/Elegoo Neptune 4 Max 0.4 nozzle |  |
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
| PASS | Eryone/Eryone ER20 0.5 nozzle |  |
| PASS | Eryone/Eryone ER20 0.6 nozzle |  |
| DIVERGENT | Eryone/Eryone ER20 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 2253 (line 100, column 1; expected 67425 bytes, actual 67425 bytes) context:   line 99: "G1 X126.09 Y105.41 E8.36239"   line 100: expected "M73 P62 R2"; actual "G1 X126.09 Y114.558 E9.47928"   line 101: expected "G1 X126.09 Y114.558 E9.47928"; actual "M73 P62 R2" |
| PASS | Eryone/Eryone ER20 Klipper 0.2 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.4 nozzle |  |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.5 nozzle | (no match across 6 oracle runs) first difference at byte 4471 (line 138, column 9; expected 144405 bytes, actual 144344 bytes) context:   line 137: "G1 X125.235 Y114.077 F15000"   line 138: expected "M73 P5 R2"; actual "M73 P5 R1"   line 139: "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 SQUARE_CORNER_VELOCITY=9" |
| PASS | Eryone/Eryone ER20 Klipper 0.6 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.8 nozzle |  |
| PASS | Eryone/Thinker X400 0.2 nozzle |  |
| PASS | Eryone/Thinker X400 0.4 nozzle |  |
| PASS | FLSun/FLSun Q5 0.4 nozzle |  |
| PASS | FLSun/FLSun QQ-S Pro 0.4 nozzle |  |
| PASS | FLSun/FLSun S1 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun Super Racer 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 2574 (line 105, column 1; expected 104847 bytes, actual 104847 bytes) context:   line 104: "G1 X5 Y10.611 E.08056"   line 105: expected "M73 P5 R7"; actual "G1 X-5 Y10.611 E.33172"   line 106: expected "G1 X-5 Y10.611 E.33172"; actual "G1 X-7.072 Y10.214 E.06996" |
| PASS | FLSun/FLSun T1 0.4 nozzle |  |
| PASS | FLSun/FLSun V400 0.4 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.25 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.4 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.6 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.8 nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 3 Series 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 3 Series 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.3 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 4 Series HS Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Artemis 0.4 Nozzle |  |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-UcVfkg") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-p8ycpW") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-q7X8CH") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-QbNe0X") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-FDuz8S") |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.8 nozzle |  |
| PASS | Flashforge/Flashforge Guider 2s 0.4 nozzle |  |
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
| PASS | Geeetech/Geeetech M1 0.2 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.4 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.6 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.4 nozzle |  |
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
| DIVERGENT | Ginger Additive/Ginger G1 1.2 nozzle | (no match across 6 oracle runs) first difference at byte 596 (line 19, column 9; expected 39732 bytes, actual 39065 bytes) context:   line 18: "EXCLUDE_OBJECT_DEFINE NAME=cube10.stl_id_0_copy_0 CENTER=500,500 POLYGON=[[495,495],[505,495],[505,505],[495,505],[495,495]]"   line 19: expected "M73 P0 R10"; actual "M73 P0 R5"   line 20: ";TYPE:Custom" |
| PASS | Ginger Additive/Ginger G1 3.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 5.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 8.0 nozzle |  |
| PASS | InfiMech/InfiMech EX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech EX+APS 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX HSN 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KLP1 0.4 nozzle |  |
| DIVERGENT | Kingroon/Kingroon KP3S 3.0 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 513 (line 21, column 9; expected 92286 bytes, actual 90491 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R3"; actual "M73 P0 R2"   line 22: "M201 X4000 Y4000 Z1100 E10000" |
| PASS | Kingroon/Kingroon KP3S PRO S1 0.4 nozzle |  |
| DIVERGENT | Kingroon/Kingroon KP3S PRO V2 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 940 (line 39, column 3; expected 103623 bytes, actual 103327 bytes) context:   line 38: ""   line 39: expected "G10 ; retract"; actual "G1 E-.8 F2700"   line 40: ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Kingroon/Kingroon KP3S V1 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 8346 (line 347, column 6; expected 107950 bytes, actual 107938 bytes) context:   line 346: ";WIDTH:0.45"   line 347: expected "G1 F1722"; actual "G1 F1679"   line 348: expected "G1 X85.602 Y94.398 E.29178"; actual "G1 X85.602 Y94.398 E.29177" |
| PASS | LH/LH Stinger 0.4 nozzle |  |
| DIVERGENT | LH/LH Stinger MMU 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 627 (line 23, column 1; expected 135321 bytes, actual 135322 bytes) context:   line 22: ";TYPE:Custom"   line 23: expected "_SP_PRINT_START LANE=0 TEMP=230"; actual " _SP_PRINT_START LANE=0 TEMP=230"   line 24: "" |
| PASS | LONGER/LONGER LK10 (0.2 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 (0.8 nozzle) | (no match across 6 oracle runs) first difference at byte 4081 (line 159, column 7; expected 102769 bytes, actual 103095 bytes) context:   line 158: ";WIDTH:0.82"   line 159: expected "G1 F2786"; actual "G1 F2797"   line 160: "G1 X106.111 Y107.5 E.4294" |
| PASS | LONGER/LONGER LK10 Plus (0.2 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.8 nozzle) | (no match across 6 oracle runs) first difference at byte 51208 (line 2164, column 29; expected 103059 bytes, actual 103059 bytes) context:   line 2163: "G1 X157.637 Y157.637 E-.05789"   line 2164: expected "G1 X158.444 Y157.637 E-.24211"; actual "G1 X158.444 Y157.637 E-.24212"   line 2165: ";WIPE_END" |
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
| DIVERGENT | Mellow/M1 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 375170 (line 15639, column 62; expected 395090 bytes, actual 395090 bytes) context:   line 15638: "; estimated printing time (normal mode) = 10m 11s"   line 15639: expected "; estimated first layer printing time (normal mode) = 0.438319s"; actual "; estimated first layer printing time (normal mode) = 0.438318s"   line 15640: "" |
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
| DIVERGENT | Peopoly/Peopoly Magneto X 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 185662 (line 6677, column 62; expected 206009 bytes, actual 206009 bytes) context:   line 6676: "; estimated printing time (normal mode) = 7m 15s"   line 6677: expected "; estimated first layer printing time (normal mode) = 0.874670s"; actual "; estimated first layer printing time (normal mode) = 0.874671s"   line 6678: "" |
| PASS | Peopoly/Peopoly Magneto X 0.6 nozzle |  |
| PASS | Peopoly/Peopoly Magneto X 0.8 nozzle |  |
| PASS | Phrozen/Phrozen Arco 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.2 nozzle |  |
| PASS | Positron3D/The Positron 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.6 nozzle |  |
| PASS | Positron3D/The Positron 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.25 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 6667 (line 300, column 1; expected 96233 bytes, actual 96233 bytes) context:   line 299: "G1 X121.021 Y111.083 E.24231"   line 300: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 301: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa CORE One 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One HF 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 6631 (line 297, column 1; expected 95737 bytes, actual 95737 bytes) context:   line 296: "G1 X121.021 Y111.083 E.24231"   line 297: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 298: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa CORE One HF 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One L 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 4811 (line 230, column 8; expected 95966 bytes, actual 96077 bytes) context:   line 229: ";WIDTH:0.45"   line 230: expected "G1 F1892"; actual "G1 F1893"   line 231: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L 0.5 nozzle | (no match across 6 oracle runs) first difference at byte 514 (line 21, column 9; expected 90693 bytes, actual 90702 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 2106 (line 106, column 7; expected 75897 bytes, actual 75886 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P89 R3"; actual "M73 P88 R3"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 2106 (line 106, column 7; expected 51590 bytes, actual 51842 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 4775 (line 227, column 8; expected 95276 bytes, actual 95609 bytes) context:   line 226: ";WIDTH:0.45"   line 227: expected "G1 F1892"; actual "G1 F1893"   line 228: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.5 nozzle | (no match across 6 oracle runs) first difference at byte 514 (line 21, column 9; expected 89688 bytes, actual 89697 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 2107 (line 106, column 7; expected 67236 bytes, actual 67235 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P91 R2"; actual "M73 P90 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 2106 (line 106, column 7; expected 51902 bytes, actual 51901 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| PASS | Prusa/Prusa MINI 0.25 nozzle |  |
| PASS | Prusa/Prusa MINI 0.4 nozzle |  |
| PASS | Prusa/Prusa MINI 0.6 nozzle |  |
| PASS | Prusa/Prusa MINI 0.8 nozzle |  |
| PASS | Prusa/Prusa MINIIS 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MINIIS 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 2782 (line 133, column 1; expected 107296 bytes, actual 107307 bytes) context:   line 132: "G1 X92.051 Y86.75 E.06609"   line 133: expected "M73 P35 R8"; actual "G1 X91.388 Y86.75 E.02581"   line 134: expected "G1 X91.388 Y86.75 E.02581"; actual "M73 P35 R8" |
| PASS | Prusa/Prusa MINIIS 0.6 nozzle |  |
| PASS | Prusa/Prusa MINIIS 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3.5 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 4896 (line 228, column 1; expected 110835 bytes, actual 111095 bytes) context:   line 227: "G1 X129.775 Y100.225 E.32326"   line 228: expected "M73 P72 R8"; actual "G1 X129.775 Y109.735 E.3219"   line 229: expected "G1 X129.775 Y109.735 E.3219"; actual "M73 P72 R8" |
| PASS | Prusa/Prusa MK3.5 0.6 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.25 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.4 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 6769 (line 318, column 12; expected 64861 bytes, actual 64861 bytes) context:   line 317: ";WIPE_START"   line 318: expected "G1 F2930.735"; actual "G1 F2930.736"   line 319: "G1 X126.794 Y112.418 E-.24" |
| PASS | Prusa/Prusa MK4 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4 0.4 nozzle |  |
| PASS | Prusa/Prusa MK4 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4 0.8 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 4519 (line 213, column 2; expected 92830 bytes, actual 92830 bytes) context:   line 212: "G1 E-.7 F2100"   line 213: expected "M73 P84 R4"; actual "M486 S0"   line 214: expected "M486 S0"; actual "G1 X129.325 Y109.325 Z.6 F18000" |
| PASS | Prusa/Prusa MK4S 0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S HF0.4 nozzle | (no match across 6 oracle runs) first difference at byte 7240 (line 326, column 7; expected 92234 bytes, actual 92234 bytes) context:   line 325: ";WIDTH:0.45"   line 326: expected "G1 F2560"; actual "G1 F2544"   line 327: "G1 X120.675 Y109.325 E.29279" |
| PASS | Prusa/Prusa MK4S HF0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.8 nozzle |  |
| PASS | Prusa/Prusa XL 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.3 nozzle | (no match across 6 oracle runs) first difference at byte 2216 (line 81, column 1; expected 125488 bytes, actual 125536 bytes) context:   line 80: "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"   line 81: expected "M73 P77 R9"; actual "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"   line 82: expected "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"; actual "G92 E0 ; reset extruder position" |
| DIVERGENT | Prusa/Prusa XL 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 3176 (line 135, column 1; expected 93107 bytes, actual 93155 bytes) context:   line 134: "G1 E.8 F1800"   line 135: expected "M73 P83 R6"; actual ";TYPE:Bottom surface"   line 136: expected ";TYPE:Bottom surface"; actual ";WIDTH:0.50675" |
| PASS | Prusa/Prusa XL 0.5 nozzle |  |
| PASS | Prusa/Prusa XL 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 4422 (line 198, column 22; expected 53778 bytes, actual 53824 bytes) context:   line 197: ";WIPE_END"   line 198: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 199: expected "G1 E.6 F1800"; actual "G1 Z.6" |
| PASS | Prusa/Prusa XL 5T 0.25 nozzle |  |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.3 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-Wy3Ia4") |
| DIVERGENT | Prusa/Prusa XL 5T 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 3333 (line 168, column 1; expected 103701 bytes, actual 103749 bytes) context:   line 167: "G1 X184.55 Y184.55 F24000"   line 168: expected "M73 P83 R6"; actual ";TYPE:Outer wall"   line 169: expected ";TYPE:Outer wall"; actual "G1 F1238" |
| PASS | Prusa/Prusa XL 5T 0.5 nozzle |  |
| PASS | Prusa/Prusa XL 5T 0.6 nozzle |  |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-yXcUww") |
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
| DIVERGENT | Qidi/Qidi X-Max 3 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 23738 (line 930, column 1; expected 299355 bytes, actual 299357 bytes) context:   line 929: "G1 X166.626 Y163.985 E.00227"   line 930: expected "M73 P28 R13"; actual "G1 X161.015 Y158.374 E.06418"   line 931: expected "G1 X161.015 Y158.374 E.06418"; actual "G1 X160.734 Y158.374 E.00227" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 2563 (line 116, column 1; expected 120049 bytes, actual 121138 bytes) context:   line 115: "G1 X162.393 Y158.7 E.02598"   line 116: expected "M73 P46 R5"; actual "G1 X166.3 Y162.607 E.21379"   line 117: expected "G1 X166.3 Y162.607 E.21379"; actual "G1 X166.3 Y163.279 E.02598" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 3973 (line 173, column 1; expected 75688 bytes, actual 75690 bytes) context:   line 172: "G1 Z.6"   line 173: expected "M73 P58 R3"; actual "G1 E1.4 F1800"   line 174: expected "G1 E1.4 F1800"; actual "M73 P58 R3" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 1025 (line 48, column 1; expected 68790 bytes, actual 69154 bytes) context:   line 47: "G1 X152.5 E0.32 F3000"   line 48: expected "M73 P61 R3"; actual "G1 Y153.5 E13.12 F3000"   line 49: expected "G1 Y153.5 E13.12 F3000"; actual "G1 X162.5 E-1.6 F3000" |
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
| DIVERGENT | RH3D/E3NG v1.2S - 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 13777 (line 523, column 11; expected 291096 bytes, actual 291096 bytes) context:   line 522: "G1 X121 Y105.338 E.10775"   line 523: expected "G1 X123.081 Y105.734 E.02283"; actual "G1 X123.082 Y105.734 E.02283"   line 524: "G1 X124.872 Y106.868 E.02283" |
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
| PASS | Ratrig/RatRig V-Core 4 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 500 0.8 nozzle | Ratrig/RatRig V-Core 4 500 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.5 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 300 0.6 nozzle | (no match across 6 oracle runs) first difference at byte 5566 (line 218, column 7; expected 102599 bytes, actual 102863 bytes) context:   line 217: ";WIDTH:0.6"   line 218: expected "G1 F3671"; actual "G1 F3683"   line 219: "G1 X146.437 Y153.563 E.32348" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 500 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 62871 (line 2488, column 61; expected 83126 bytes, actual 83126 bytes) context:   line 2487: "; estimated printing time (normal mode) = 2m 48s"   line 2488: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2489: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.4 nozzle |  |
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
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 28767 (line 1163, column 1; expected 84466 bytes, actual 84764 bytes) context:   line 1162: "G1 X246.075 Y246.075 E.56816"   line 1163: expected "M73 P46 R1"; actual "G1 X253.925 Y246.075 E.56816"   line 1164: expected "G1 X253.925 Y246.075 E.56816"; actual "M73 P46 R1" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.8 nozzle | (no match across 6 oracle runs) first difference at byte 63904 (line 2569, column 61; expected 84822 bytes, actual 84822 bytes) context:   line 2568: "; estimated printing time (normal mode) = 2m 48s"   line 2569: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2570: "" |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Minion 0.4 nozzle |  |
| DIVERGENT | RolohaunDesign/Rolohaun Delta Flyer Refit 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 86654 (line 3704, column 16; expected 126094 bytes, actual 126094 bytes) context:   line 3703: "G1 X-1.055 Y6.015 E.00391"   line 3704: expected "G1 X5.877 Y6.014 E.19257"; actual "G1 X5.877 Y6.013 E.19257"   line 3705: "G1 X5.986 Y5.986 E.00312" |
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
| DIVERGENT | Snapmaker/Snapmaker Artisan (0.2 nozzle) | (no match across 6 oracle runs) first difference at byte 10242 (line 457, column 7; expected 187557 bytes, actual 188207 bytes) context:   line 456: "G1 X199.125 Y195.786 E.04833"   line 457: expected "G1 X197.232 Y196.47 F11400"; actual "G1 X196.709 Y196.665 F11400"   line 458: expected "G1 F3799.871"; actual ";WIDTH:0.286708" |
| PASS | Snapmaker/Snapmaker Artisan (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.8 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.2 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-hRQxXU") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-LUwcFQ") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4+0.6 nozzle) | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-QgBucV") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.6 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-1LVIvp") |
| PASS | Snapmaker/Snapmaker U1 (0.8 nozzle) |  |
| PASS | Sovol/Sovol SV01 0.4 nozzle |  |
| PASS | Sovol/Sovol SV01 Pro 0.4 nozzle |  |
| PASS | Sovol/Sovol SV02 0.4 nozzle |  |
| PASS | Sovol/Sovol SV05 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV06 0.4 High-Speed nozzle | (no match across 6 oracle runs) first difference at byte 1666 (line 90, column 19; expected 101958 bytes, actual 101958 bytes) context:   line 89: "G1 X117.495 Y97.735 E.05224"   line 90: expected "G1 X118.551 Y98.188 E.05224"; actual "G1 X118.551 Y98.189 E.05224"   line 91: "G1 X119.527 Y98.794 E.05224" |
| PASS | Sovol/Sovol SV06 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.2 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.6 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.8 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV07 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV07 Plus 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 4032 (line 174, column 1; expected 107043 bytes, actual 107311 bytes) context:   line 173: "G1 X146.375 Y150.578 E5.51963"   line 174: expected "G1 X149.422 Y153.625 E5.67617"; actual "M106 S255"   line 175: expected "G1 X148.896 Y153.625 E5.69525"; actual "G1 X149.422 Y153.625 E5.67617" |
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
| DIVERGENT | TwoTrees/TwoTrees SK1 0.4 nozzle | (no match across 6 oracle runs) first difference at byte 1414 (line 60, column 1; expected 101783 bytes, actual 102276 bytes) context:   line 59: "G1 X190 Y12 F6000 ;Wipe"   line 60: expected "M73 P66 R2"; actual "G1 X180 Y8 F6000 ;Wipe"   line 61: expected "G1 X180 Y8 F6000 ;Wipe"; actual "G1 X170 Y12 F6000 ;Wipe" |
| PASS | TwoTrees/TwoTrees SP-5 Klipper 0.4 nozzle |  |
| PASS | UltiMaker/UltiMaker 2 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 Klipper 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 RRF 0.4 nozzle |  |
| DIVERGENT | Volumic/EXO42 (0.4 nozzle) | (no match across 6 oracle runs) first difference at byte 40140 (line 1760, column 1; expected 120293 bytes, actual 120292 bytes) context:   line 1759: "G1 X214.328 Y205.672"   line 1760: expected "M73 P38 R3"; actual "G1 X208.497 Y205.672"   line 1761: expected "G1 X208.497 Y205.672"; actual "M73 P38 R3" |
| PASS | Volumic/EXO42 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Stage 2 (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO65 (0.6 nozzle) | (no match across 6 oracle runs) first difference at byte 39837 (line 1917, column 1; expected 115024 bytes, actual 115034 bytes) context:   line 1916: "G1 X328.968 Y328.968"   line 1917: expected "M73 P41 R2"; actual "G1 X329.64 Y329.64"   line 1918: expected "G1 X329.64 Y329.64"; actual "M73 P41 R2" |
| PASS | Volumic/EXO65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.6 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.8 nozzle) |  |
| PASS | Volumic/EXO65 Stage 2 (0.6 nozzle) |  |
| DIVERGENT | Volumic/SH65 (0.4 nozzle) | (no match across 6 oracle runs) first difference at byte 4836 (line 220, column 1; expected 120290 bytes, actual 120290 bytes) context:   line 219: "G1 X328.474 Y148.946 E.01769"   line 220: expected "M73 P4 R4"; actual "G1 X323.946 Y153.474 E.17881"   line 221: expected "G1 X323.946 Y153.474 E.17881"; actual "M73 P4 R4" |
| PASS | Volumic/SH65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 Performance (0.4 nozzle) |  |
| PASS | Volumic/SH65 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/VS20MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK3 (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30MK3 Stage 2 (0.4 nozzle) | (no match across 6 oracle runs) first difference at byte 18421 (line 801, column 1; expected 101231 bytes, actual 101231 bytes) context:   line 800: "G1 X146.507 Y98.548 E.10493"   line 801: expected "G1 X146.507 Y96.507 E.0742"; actual "M73 P18 R3"   line 802: expected "M73 P18 R3"; actual "G1 X146.507 Y96.507 E.0742" |
| PASS | Volumic/VS30SC (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 Performance (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30SC2 Stage 2 (0.4 nozzle) | (no match across 6 oracle runs) first difference at byte 18421 (line 801, column 1; expected 101231 bytes, actual 101231 bytes) context:   line 800: "G1 X146.507 Y98.548 E.10493"   line 801: expected "G1 X146.507 Y96.507 E.0742"; actual "M73 P18 R3"   line 802: expected "M73 P18 R3"; actual "G1 X146.507 Y96.507 E.0742" |
| PASS | Volumic/VS30ULTRA (0.4 nozzle) |  |
| PASS | Voron/Voron 0.1 0.15 nozzle |  |
| DIVERGENT | Voron/Voron 0.1 0.2 nozzle | (no match across 6 oracle runs) first difference at byte 333887 (line 12739, column 62; expected 354370 bytes, actual 354370 bytes) context:   line 12738: "; estimated printing time (normal mode) = 8m 24s"   line 12739: expected "; estimated first layer printing time (normal mode) = 0.486746s"; actual "; estimated first layer printing time (normal mode) = 0.486747s"   line 12740: "" |
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
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.2mm nozzle | (no match across 6 oracle runs) first difference at byte 3985 (line 171, column 7; expected 213124 bytes, actual 213150 bytes) context:   line 170: "G1 E-1.2 F7200"   line 171: expected "G1 X148.896 Y148.972 Z.56 F15000"; actual "G1 X145.864 Y150.863 Z.56 F15000"   line 172: expected "G1 X153.319 Y153.319 Z.56"; actual "G1 X146.681 Y153.319 Z.56" |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.3mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.4mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.6mm nozzle | (no match across 6 oracle runs) first difference at byte 1613 (line 80, column 1; expected 108772 bytes, actual 109765 bytes) context:   line 79: "G1 X157.562 Y143.499 E.10062"   line 80: expected "M73 P44 R6"; actual "G1 X157.97 Y145 E.10237"   line 81: expected "G1 X157.97 Y145 E.10237"; actual "G1 X157.97 Y155 E.65837" |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.2mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.3mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.4mm nozzle | (no match across 6 oracle runs) first difference at byte 2009 (line 96, column 1; expected 132655 bytes, actual 132656 bytes) context:   line 95: "G1 X110 Y97.784 E.56236"   line 96: expected "M73 P43 R6"; actual "G1 X111.257 Y98.175 E.07402"   line 97: expected "G1 X111.257 Y98.175 E.07402"; actual "M73 P43 R6" |
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
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-pR9PIJ") |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.6 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-nRAZIj/command-uEXpAN") |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.2 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.4 nozzle |  |
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
| DIVERGENT | iQ/iQ TiQ8 0.4 Nozzle | (no match across 6 oracle runs) first difference at byte 21659 (line 859, column 1; expected 129927 bytes, actual 130358 bytes) context:   line 858: "G1 X245.736 Y197.238 E.0621"   line 859: expected ";LAYER_CHANGE"; actual "G1 X246.055 Y195.983 F18000"   line 860: expected ";Z:0.6"; actual ";WIDTH:0.463894" |
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
