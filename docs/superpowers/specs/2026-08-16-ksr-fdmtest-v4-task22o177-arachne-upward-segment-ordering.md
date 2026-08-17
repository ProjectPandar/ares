# Spec: KSR FDM Test V4 task177 Arachne upward segment ordering

## Observable contract

Segment preparation selects only skeletal half-edges that have both quad neighbors and are oriented upward. Candidates are ordered from greater destination boundary radius to smaller radius; source flat-edge tie rules use distance to the next rise so propagation dependencies remain deterministic.

A focused graph test observes selection and descending-radius order. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1430-1467`, the `upward_quad_mids` collection and ordering prefix of `generateSegments`. Upward/downward beading propagation, junction generation and connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
