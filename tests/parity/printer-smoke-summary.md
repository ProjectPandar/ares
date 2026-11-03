# OrcaSlicer printer smoke summary

971 of 1001 printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; cube model).

Snapshot producer is Ares revision `8a52b2d2e21b38af727250327a497af5c9a3a914`; complete artifacts remain at `${TMP_ROOT}/ares-printers-8a52b2d2-initialized`, with terminal failure recorded in the sibling `.log`. There are 20 `ORCA_ERROR` and 10 `VENDOR_INCOMPLETE` results, no recorded Ares slicing error or byte divergence. This run uses the separately seeded/initialized Orca reference, not the unmodified official binary. Cached-reference manifests retain unknown producer identity rather than inferring it from the requested wrapper. This snapshot predates ARD-0083 and is not a current-revision or untouched-default acceptance claim.

This matrix pins a MODIFIED baseline (`smoke_overrides`: `wall_generator=classic`, `detect_thin_wall=0`, bed-exclude and post-process cleared, one filament) — it verifies parity of the ported classic-wall slice across every printer's machine/process parameterization, not untouched default profiles. Arachne, thin-wall detection, multi-extruder, and vendor post-process paths are upstream boundaries pending port; an untouched-profile acceptance matrix is deferred to those milestones.

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
| PASS | Anycubic/Anycubic Kobra 3 0.2 nozzle |  |
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
| PASS | BBL/Bambu Lab X1 0.2 nozzle |  |
| PASS | BBL/Bambu Lab X1 0.4 nozzle |  |
| PASS | BBL/Bambu Lab X1 0.6 nozzle |  |
| PASS | BBL/Bambu Lab X1 0.8 nozzle |  |
| PASS | BBL/Bambu Lab X1 Carbon 0.2 nozzle |  |
| PASS | BBL/Bambu Lab X1 Carbon 0.4 nozzle |  |
| PASS | BBL/Bambu Lab X1 Carbon 0.6 nozzle |  |
| PASS | BBL/Bambu Lab X1 Carbon 0.8 nozzle |  |
| PASS | BBL/Bambu Lab X1E 0.2 nozzle |  |
| PASS | BBL/Bambu Lab X1E 0.4 nozzle |  |
| PASS | BBL/Bambu Lab X1E 0.6 nozzle |  |
| PASS | BBL/Bambu Lab X1E 0.8 nozzle |  |
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
| PASS | Creality/Creality CR-6 Max 0.2 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.8 nozzle |  |
| PASS | Creality/Creality CR-6 SE 0.2 nozzle |  |
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
| PASS | Creality/Creality Ender-3 V3 Plus 0.4 nozzle |  |
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
| PASS | Creality/Creality K2 Plus 0.6 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.8 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.2 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.4 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.6 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.8 nozzle |  |
| PASS | Creality/Creality K2 SE 0.4 nozzle |  |
| PASS | Creality/Creality SPARKX i7 0.2 nozzle |  |
| PASS | Creality/Creality SPARKX i7 0.4 nozzle |  |
| PASS | Creality/Creality SPARKX i7 0.6 nozzle |  |
| PASS | Creality/Creality SPARKX i7 0.8 nozzle |  |
| PASS | Creality/Creality Sermoon V1 0.4 nozzle |  |
| ORCA_ERROR | Cubicon/Cubicon xCeler-I 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-38HbvX") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Mini 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-Dcljgt") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Plus 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-wy97aR") |
| PASS | Custom/MyKlipper 0.2 nozzle |  |
| PASS | Custom/MyKlipper 0.4 nozzle |  |
| PASS | Custom/MyKlipper 0.6 nozzle |  |
| PASS | Custom/MyKlipper 0.8 nozzle |  |
| PASS | Custom/MyMarlin 0.4 nozzle |  |
| PASS | Custom/MyRRF 0.4 nozzle |  |
| PASS | Custom/MyRepetier 0.4 nozzle |  |
| ORCA_ERROR | Custom/MyToolChanger 0.2 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-bEr9p1") |
| ORCA_ERROR | Custom/MyToolChanger 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-re3SOh") |
| ORCA_ERROR | Custom/MyToolChanger 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-F8HsxB") |
| ORCA_ERROR | Custom/MyToolChanger 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-CfJ5ZW") |
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
| PASS | Eryone/Eryone ER20 0.8 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.2 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.4 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.5 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.6 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.8 nozzle |  |
| PASS | Eryone/Thinker X400 0.2 nozzle |  |
| PASS | Eryone/Thinker X400 0.4 nozzle |  |
| PASS | FLSun/FLSun Q5 0.4 nozzle |  |
| PASS | FLSun/FLSun QQ-S Pro 0.4 nozzle |  |
| PASS | FLSun/FLSun S1 0.4 nozzle |  |
| PASS | FLSun/FLSun Super Racer 0.4 nozzle |  |
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
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-8XGfIY") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-Wicsr9") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-oZSAAP") |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.4 nozzle |  |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-KVbOoU") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 Pro 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-jdK5dv") |
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
| PASS | Ginger Additive/Ginger G1 1.2 nozzle |  |
| PASS | Ginger Additive/Ginger G1 3.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 5.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 8.0 nozzle |  |
| PASS | InfiMech/InfiMech EX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech EX+APS 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX HSN 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KLP1 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KP3S 3.0 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KP3S PRO S1 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KP3S PRO V2 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KP3S V1 0.4 nozzle |  |
| PASS | LH/LH Stinger 0.4 nozzle |  |
| PASS | LH/LH Stinger MMU 0.4 nozzle |  |
| PASS | LONGER/LONGER LK10 (0.2 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.6 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.8 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.2 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.6 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.8 nozzle) |  |
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
| PASS | Mellow/M1 0.2 nozzle |  |
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
| PASS | Peopoly/Peopoly Magneto X 0.4 nozzle |  |
| PASS | Peopoly/Peopoly Magneto X 0.6 nozzle |  |
| PASS | Peopoly/Peopoly Magneto X 0.8 nozzle |  |
| PASS | Phrozen/Phrozen Arco 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.2 nozzle |  |
| PASS | Positron3D/The Positron 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.6 nozzle |  |
| PASS | Positron3D/The Positron 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.25 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.3 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.4 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.4 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One L 0.4 nozzle |  |
| PASS | Prusa/Prusa CORE One L 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One L 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One L 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One L HF 0.4 nozzle |  |
| PASS | Prusa/Prusa CORE One L HF 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One L HF 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One L HF 0.8 nozzle |  |
| PASS | Prusa/Prusa MINI 0.25 nozzle |  |
| PASS | Prusa/Prusa MINI 0.4 nozzle |  |
| PASS | Prusa/Prusa MINI 0.6 nozzle |  |
| PASS | Prusa/Prusa MINI 0.8 nozzle |  |
| PASS | Prusa/Prusa MINIIS 0.25 nozzle |  |
| PASS | Prusa/Prusa MINIIS 0.4 nozzle |  |
| PASS | Prusa/Prusa MINIIS 0.6 nozzle |  |
| PASS | Prusa/Prusa MINIIS 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.25 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.4 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.6 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.25 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.4 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.6 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.8 nozzle |  |
| PASS | Prusa/Prusa MK4 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4 0.4 nozzle |  |
| PASS | Prusa/Prusa MK4 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4 0.8 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.3 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.4 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.8 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.4 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.8 nozzle |  |
| PASS | Prusa/Prusa XL 0.25 nozzle |  |
| PASS | Prusa/Prusa XL 0.3 nozzle |  |
| PASS | Prusa/Prusa XL 0.4 nozzle |  |
| PASS | Prusa/Prusa XL 0.5 nozzle |  |
| PASS | Prusa/Prusa XL 0.6 nozzle |  |
| PASS | Prusa/Prusa XL 0.8 nozzle |  |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.25 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-xCH9aT") |
| PASS | Prusa/Prusa XL 5T 0.3 nozzle |  |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-9Yh6W0") |
| PASS | Prusa/Prusa XL 5T 0.5 nozzle |  |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-gKOwVZ") |
| PASS | Prusa/Prusa XL 5T 0.8 nozzle |  |
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
| PASS | Qidi/Qidi X-Max 3 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Max 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Max 3 0.8 nozzle |  |
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
| PASS | RH3D/E3NG v1.2S - 0.2 nozzle |  |
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
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.8 nozzle |  |
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
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Minion 0.4 nozzle |  |
| PASS | RolohaunDesign/Rolohaun Delta Flyer Refit 0.4 nozzle |  |
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
| PASS | Snapmaker/Snapmaker Artisan (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.8 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.2 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-w1ioCR") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-nSWsmq") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4+0.6 nozzle) | Export/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-x9g9Rp") |
| PASS | Snapmaker/Snapmaker U1 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker U1 (0.8 nozzle) |  |
| PASS | Sovol/Sovol SV01 0.4 nozzle |  |
| PASS | Sovol/Sovol SV01 Pro 0.4 nozzle |  |
| PASS | Sovol/Sovol SV02 0.4 nozzle |  |
| PASS | Sovol/Sovol SV05 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 0.4 High-Speed nozzle |  |
| PASS | Sovol/Sovol SV06 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.2 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.6 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.8 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV07 0.4 nozzle |  |
| PASS | Sovol/Sovol SV07 Plus 0.4 nozzle |  |
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
| PASS | TwoTrees/TwoTrees SK1 0.4 nozzle |  |
| PASS | TwoTrees/TwoTrees SP-5 Klipper 0.4 nozzle |  |
| PASS | UltiMaker/UltiMaker 2 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 Klipper 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 RRF 0.4 nozzle |  |
| PASS | Volumic/EXO42 (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/EXO65 (0.6 nozzle) |  |
| PASS | Volumic/EXO65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.6 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.8 nozzle) |  |
| PASS | Volumic/EXO65 Stage 2 (0.6 nozzle) |  |
| PASS | Volumic/SH65 (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 Performance (0.4 nozzle) |  |
| PASS | Volumic/SH65 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/VS20MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK3 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK3 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/VS30SC (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 Performance (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/VS30ULTRA (0.4 nozzle) |  |
| PASS | Voron/Voron 0.1 0.15 nozzle |  |
| PASS | Voron/Voron 0.1 0.2 nozzle |  |
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
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.2mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.3mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.4mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.6mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.2mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.3mm nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.4mm nozzle |  |
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
| PASS | WonderMaker/WonderMaker ZR Ultra 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.8 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra S 0.2 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-0IPhS9") |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra S 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("${TMP_ROOT}/ares-printers-8a52b2d2-initialized/orca-runner-v2RDil/command-1tmLUC") |
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
| PASS | iQ/iQ TiQ8 0.4 Nozzle |  |
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
