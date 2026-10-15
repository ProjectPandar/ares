# Object Rectilinear ironing regional ZAA direction inputs

Input-only SHA-pinned3MFs; approved SPECcc66cf7ee. Sole approved2.4mm
wide slab mesh/profile provenance from Line/Honeycomb fixtures. Fixed ZIP
metadata. Concentric sparse/top/bottom/internal-solid, density100%, shell
counts2/thickness0, empty rotation templates, topmost Rectilinear ironing,
angle45, solid direction45, fixed angle0, spacing0.1, flow10%.
ZAA min_z0.05/perimeter-height35; enabled flag0/flag1 and disabled flag1.

Native12/12 accepted, repeat/cross-mode stable. Physical layer12/object11
Ironing has real undirected geometry delta (1331 vs1451 segments); Native
non-Ironing XYZ/E geometry is equal. Both controls strict API/CLI PASS.
Ares enabled flag1 Ironing still equals Native flag0 instead of flag1.

Owner: Fill.cpp1631/1705-1723 -> FillBase.cpp311-313. No filler replacement
or multiple-region lifecycle in this fixture. First motion delta at layer12
Ironing entry travel; earlier M73 progress delta retained by strict comparator
(GCodeProcessor.cpp1232-1257). Native outputs remain external.

SHA256:
- flag-false.3mf: c5d72f90b67bb660a9db3e338aa5e2776213dd71cc0e2cbfb35a42504ec84d1d
- flag-true.3mf: a835814e4d5f50ce32e7ddccd36d142ed8da928de51aa579ca492f74c6442274
- disabled-zaa-flag-true.3mf: cdecbac900db4d1982b05168f4cc63266dd83602a6075a12780f1e676984e53f
