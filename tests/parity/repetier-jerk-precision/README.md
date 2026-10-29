# Repetier M207 ostream precision inputs

Source GCodeWriter.cpp262/272-285; SPEC
`docs/architecture/2026-10-24-repetier-jerk-precision-spec.md`, frozenbase0bb.
StockEnder3/toweroff/Repetier, ONLYdefault_jerk varies. GUI Tab2566/coFloat
PrintConfig3289-3296; no UIdecimalprecision/physicaltuningclaim.

| Input | Jerk | SHA256 |
|---|---|---|
| integer-control.3mf | 7 | dd456257226ef4563c4742764fee803e47fc459568f31da39f59413441e6d626 |
| fractional-jerk.3mf | 7.12345 | 1029964bc3ef6b4457d215249a85db6c75692f7dcdc22cb77ff307338f420076 |
| exact-milli-control.3mf | 7.125 | ea68cf109f4abd9ff14966be8eeb9627fc179d258cb11ab4991b00cd1b274201 |

Three directexistingnative_stream_casecalls, fresh Native replay then callback
reads actualsaveddefaultjerk and observes correspondingNativeM207value. No new
helper/hardcodedNativecoordinate/count/valuegoldens. CompleteunmaskedAPI/CLI
comparison, generator-lineonly;noNativeoutputscommitted.
Targetfixed3REDexpected,two controlsPASSbothmodes. Failureclassesdocumentedbefore
production in SPEC:wrongformat/flavor/cache/clamp/globalformatter/accidental
Klipperfix. GREEN onlyRepetierfield,nootherbranchchange/testsinputsfrozen.
Groupedregistrationno squeeze;noFULL.
