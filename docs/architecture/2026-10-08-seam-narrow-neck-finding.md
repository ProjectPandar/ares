# Narrow-neck first-layer perimeter ordering finding

Status: independent observed Native/Ares path divergence; next-slice diagnosis
follows the C/C1 reachability report. No fix or new
implementation spec is included in this finding. Baseline `016456549`.

All six inputs have `wall_generator=arachne`, neck_half_width0.25mm, one
object/region, one wall, first/regular heights0.2mm. Native and Ares both
label the first extrusion `;TYPE:Outer wall`. Native begins with the open
narrow-neck perimeter; Ares begins with a closed lobe perimeter. This is
a perimeter/Arachne narrow-wall observation, not an identified gap-fill
root cause. Both AlignedBack and Nearest inputs exhibit the divergence.

For M73-first cases the true whole-file first difference occurs in startup,
before layer1. The separately recorded first differing path is always
layer1, Z0.2mm; initial-hop Z0.6 is travel, not a printing layer. No M73
difference is filtered from the whole-file verdict.

## C0-neck0.25-r0

Input: `C0-neck0.25-r0.3mf`.
SHA256: `6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d`.

first difference at byte 847 (line 32, column 7; expected 26891 bytes, actual 26750 bytes).

```text
first difference at byte 847 (line 32, column 7; expected 26891 bytes, actual 26750 bytes)
context:
  line 31: "G1 X2 Y10 F3000"
  line 32: expected "M73 P28 R0"; actual "M73 P29 R0"
  line 33: "M104 S220 ; set final nozzle temp"
```

First differing path: layer1, Z0.2. Native raw line64 `G1 X107.945 Y110`; Ares raw line62 `G1 X100.31 Y113.69`.

Native: open narrow-neck Arachne perimeter, `G1 X107.945 Y110` → `G1 X112.037 Y110 E.13751`.
Ares: closed Arachne lobe perimeter, `G1 X100.31 Y113.69` → `G1 X100.31 Y113.69 E.22678`.

## C0-neck0.25-r17

Input: `C0-neck0.25-r17.3mf`.
SHA256: `98e80f92caedbee7824eb587e026f6e786d9a02b2a0af7c8f88726b688af560d`.

first difference at byte 1398 (line 63, column 7; expected 26808 bytes, actual 26840 bytes).

```text
first difference at byte 1398 (line 63, column 7; expected 26808 bytes, actual 26840 bytes)
context:
  line 62: "G1 Z.6 F9000"
  line 63: expected "G1 X108.051 Y109.404"; actual "G1 X106.713 Y112.854"
  line 64: "G1 Z.6"
```

First differing path: layer1, Z0.2. Native raw line63 `G1 X108.051 Y109.404`; Ares raw line63 `G1 X106.713 Y112.854`.

Native: open narrow-neck Arachne perimeter, `G1 X108.051 Y109.404` → `G1 X111.966 Y110.601 E.13867`.
Ares: closed Arachne lobe perimeter, `G1 X106.713 Y112.854` → `G1 X106.713 Y112.854 E.07727`.

## C0-neck0.25-r43

Input: `C0-neck0.25-r43.3mf`.
SHA256: `87833a359652053d8b8208485deb16db6e135eca625864d33e758da1f9a37eef`.

first difference at byte 1398 (line 63, column 7; expected 26797 bytes, actual 26829 bytes).

```text
first difference at byte 1398 (line 63, column 7; expected 26797 bytes, actual 26829 bytes)
context:
  line 62: "G1 Z.6 F9000"
  line 63: expected "G1 X108.51 Y108.61"; actual "G1 X105.794 Y111.124"
  line 64: "G1 Z.6"
```

First differing path: layer1, Z0.2. Native raw line63 `G1 X108.51 Y108.61`; Ares raw line63 `G1 X105.794 Y111.124`.

Native: open narrow-neck Arachne perimeter, `G1 X108.51 Y108.61` → `G1 X111.503 Y111.402 E.13867`.
Ares: closed Arachne lobe perimeter, `G1 X105.794 Y111.124` → `G1 X105.794 Y111.124 E.07727`.

## nearest-control-neck0.25-r0

Input: `nearest-control-neck0.25-r0.3mf`.
SHA256: `ce5caae069fdf344382fc395f1ec85f576623bd33fbe483706a9b980705bcc41`.

first difference at byte 847 (line 32, column 7; expected 26803 bytes, actual 26739 bytes).

```text
first difference at byte 847 (line 32, column 7; expected 26803 bytes, actual 26739 bytes)
context:
  line 31: "G1 X2 Y10 F3000"
  line 32: expected "M73 P28 R0"; actual "M73 P29 R0"
  line 33: "M104 S220 ; set final nozzle temp"
```

First differing path: layer1, Z0.2. Native raw line63 `G1 X107.945 Y110`; Ares raw line62 `G1 X107.69 Y113.69`.

Native: open narrow-neck Arachne perimeter, `G1 X107.945 Y110` → `G1 X112.037 Y110 E.13751`.
Ares: closed Arachne lobe perimeter, `G1 X107.69 Y113.69` → `G1 X107.69 Y113.69 E.07724`.

## nearest-control-neck0.25-r17

Input: `nearest-control-neck0.25-r17.3mf`.
SHA256: `3e33655889bf7acc578d2a36f4b10611b23119f3a827f9f88bd51cea53612cb9`.

first difference at byte 847 (line 32, column 7; expected 26795 bytes, actual 26838 bytes).

```text
first difference at byte 847 (line 32, column 7; expected 26795 bytes, actual 26838 bytes)
context:
  line 31: "G1 X2 Y10 F3000"
  line 32: expected "M73 P29 R0"; actual "M73 P28 R0"
  line 33: "M104 S220 ; set final nozzle temp"
```

First differing path: layer1, Z0.2. Native raw line62 `G1 X108.051 Y109.404`; Ares raw line63 `G1 X108.87 Y105.796`.

Native: open narrow-neck Arachne perimeter, `G1 X108.051 Y109.404` → `G1 X111.966 Y110.601 E.13867`.
Ares: closed Arachne lobe perimeter, `G1 X108.87 Y105.796` → `G1 X108.87 Y105.796 E.22678`.

## nearest-control-neck0.25-r43

Input: `nearest-control-neck0.25-r43.3mf`.
SHA256: `82b1200bdbedba3ec487dbbf3ab61f75b9fdc4fd1ffe5a06ad33d1f4f3d05601`.

first difference at byte 1386 (line 62, column 6; expected 26785 bytes, actual 26817 bytes).

```text
first difference at byte 1386 (line 62, column 6; expected 26785 bytes, actual 26817 bytes)
context:
  line 61: "G1 Z.6 F9000"
  line 62: expected "G1 X108.51 Y108.61"; actual "G1 X110.827 Y105.726"
  line 63: "G1 Z.6"
```

First differing path: layer1, Z0.2. Native raw line62 `G1 X108.51 Y108.61`; Ares raw line62 `G1 X110.827 Y105.726`.

Native: open narrow-neck Arachne perimeter, `G1 X108.51 Y108.61` → `G1 X111.503 Y111.402 E.13867`.
Ares: closed Arachne lobe perimeter, `G1 X110.827 Y105.726` → `G1 X110.827 Y105.726 E.22678`.

## Upstream boundary for the next diagnosis

Pinned upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`PerimeterGenerator.cpp:525–566` materializes closed Arachne paths as
ExtrusionLoop and open paths as ExtrusionMultiPath. Its
`process_arachne:2320–2330` orders available candidates by is_closed before
selection; `GCode.cpp:6131–6146` emits region perimeters in their stored
order. These are source leads for the next slice, not a completed causal
trace. Ares perimeter materialization/traversal and entity emission need
comparison before any spec or production edit.
