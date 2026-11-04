# PrintObject centering C1 inputs

Source: Model.cpp1586-1599, PrintObject.cpp93-130, PrintApply.cpp138-170.
SPEC49cf8a57a; public validated byte API and real CLI are the seams.
Native official/seeded full ordered generator bytes are the oracle.
The supplemental rotation-exclusion case cites GCode.cpp8084-8118/5348-5372
for per-group and per-copy labels; only exclude_object is activated. Inputs only;
all Native products and launch/comparison receipts go to external artifact roots.
Regenerate from repository root with `generate.py`; provenance records
base/input SHA, transform membership, and checked effective settings.

Failure inventory before production: rejecting multiple groups; using the first
printable instance instead of the original first; treating source order as group
order; recomputing center after scaling; wrong signed rotation/mirror extraction;
quantizing center+translation together; dropping Z shrink; double emitting copies
in skirt/brim, wrong frame for support/seam. Fourteen cases cover these boundaries;
single and XY-copy controls must remain strict. No shared-mesh behavior is included
in C1; C2's first RED will be the previously measured implicit-sharing 3mm mismatch.
