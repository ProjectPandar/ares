# Spec: KSR FDM Test V4 task197 Arachne small odd-line removal

## Observable contract

After stitching, each inset removes only odd, open lines whose source walk length is below its minimum junction width times the configured factor; top/bottom processing instead uses half the minimum width. Even lines, closed odd lines, and lines meeting the threshold remain. Removal may reorder an inset by swapping in its last line, matching the source operation.

A focused test removes a short odd open segment while retaining a longer peer. The wall-toolpath pipeline applies removal immediately after stitching. Files remain below 400 LOC; focused Arachne tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/WallToolPaths.cpp:671-700`, `shorterThan` and `removeSmallLines`, into a wall-toolpath postprocess child module. Inner-contour separation, simplification, concentric-internal conversion, cooling, timing, and remaining exact G-code differences are deferred.
