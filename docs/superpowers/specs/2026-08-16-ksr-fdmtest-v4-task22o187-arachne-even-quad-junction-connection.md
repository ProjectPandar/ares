# Spec: KSR FDM Test V4 task187 Arachne even-quad junction connection

## Observable contract

For one even-walled quad chain, connection finds the peak edge, uses the following edge's twin as the opposite peak-facing side, initializes absent junction storage as empty, and connects the two retained sides from inner perimeter outward. A domain-boundary flag is forwarded to toolpath assembly.

A focused two-edge quad test observes the generated perimeter line. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports the even-quad path through OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1955-1981,2012-2049`, `connectJunctions`. Adjacent-edge concatenation, domain traversal, odd-edge classification/deduplication, three-way breaks, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
