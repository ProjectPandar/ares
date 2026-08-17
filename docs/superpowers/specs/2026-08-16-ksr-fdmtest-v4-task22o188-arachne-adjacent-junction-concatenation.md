# Spec: KSR FDM Test V4 task188 Arachne adjacent junction concatenation

## Observable contract

When a quad's peak-facing side spans an adjacent skeletal edge, connection appends that edge's retained junctions after removing overlapping perimeter indices from the peak segment. The same rule applies symmetrically after the peak. The resulting side vectors still connect from shared innermost perimeter outward.

A focused multi-edge quad test observes both outer and inner generated perimeter lines. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1982-2011`, the adjacent-side concatenation in `connectJunctions`. Domain traversal, odd-edge classification/deduplication, three-way breaks, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
