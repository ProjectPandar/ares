# Rear seam visibility initialization

Frozen original Native-exported qualified Prusa MK4 stock projects:

| Input | SHA256 |
| --- | --- |
| aligned-control.3mf | dc37585d4483eae1f5eec7331d395c99132b3bd5959ec96d486e1c901c1f0d89 |
| rear-target.3mf | 747a5ed4ac32c8d9d070ffd126f8da559a5d0c72554db84de203300a6dba2a47 |

They are the original non-BBL stock baseline mk4/project.3mf and
rear/project.3mf,respectively. Not regenerated.
Raw upstream overhang.obj was exported with actual flattened MK4 machine,
process and filament. Native export/slice0 and effective stock IDs verified.
SOLE effective config delta:seam_position aligned→back (rear).
No archived G-code or static Native reference output is embedded.

SeamPlacer.hpp58-68 initializes visibility0;SeamPlacer.cpp1427-1457 omits rear
from visibility computation while preserving rear selection/alignment.
The public cases use unchanged NativeCase:stable dynamic Native2FIRST,validated
byteAPI and actualCLI,strict complete ordered generator comparisons/raw equality.
Control must pass,target is actual RED before the production change.
No expected rejection/panic catch,mask/tolerance or hardcoded expected seam index.
See docs/architecture/rear-seam-visibility-initialization.md for source boundary,
causal trace and HOST-only replay limitations. No production correction here.
