# XY machine jerk writer cache rounding inputs

Source GCodeWriter.cpp32-56 flavor-gatedlrintcache;consumers289-293.
SPEC `docs/architecture/2026-10-24-machine-jerk-rounding-spec.md`, base09296.
No production changes in RED. StockEnder3/toweroffMarlin2. Activepair common
processdefault_jerk9;ONLYmachine_max_jerk_xvector8vs7.6. Inactivecontroldefault
jerk0/X7.6 distinguishes raw header from admitted body clamp.
GUI Tab5634-5635;Nativeacceptedsavedfloat,nodisplayprecision/physicaltuningclaim.

| Input | SHA256 |
|---|---|
| integer-control.3mf | fc0fba0a17069c48a0ddcd6bd332025db4702f659d71cbd9c6a7f5a087990132 |
| fractional-x-limit.3mf | 4ea82ec44a0d1c7527800e666c7a294c71b717fb90a1c361d09f82dff00241d7 |
| inactive-control.3mf | 6b3c9d972c96f04cc849e3da9d67ff06679146d94b53e39819fb16916be4d152 |

Three directexistingnative_stream_casecalls,nohelper. Nativefreshdouble-replay
then callback derives rawlimitfrominput,checks headerXrawpreserved and active
bodyXrounded(orinactivebodyabsent). NoNativecoordinate/count/valuegoldens;
no outputscommitted. Then fullunmaskedAPI/CLIcomparatorgenerator-lineonly,
includingconfig/timing/header. Diagnosticheader/bodyobservationsarenotfilters.
REDtargetfails,two controlsPASSbothmodes. FailureclassesinSPECbeforecode:
rawcache,wrongtie/flavorgate,headerZEaccelchanges,consumerclampchanges.
Source review separately verifies ties-even and falsegate0;theseMarlin2non-tie
casesalone do not prove allflavors/tiebehavior. GREENinputs/testsfreeze.
NoFULL,no lint suppression,groupregistrationnosqueeze.
