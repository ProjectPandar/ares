# Plan: Task 22O.157 exact rational scanline ordering

1. Add a failing scanline contract with two geometrically ordered intersections that round to the same integer Y coordinate.
2. Retain source numerator/denominator values on each real intersection and sort with exact cross multiplication before deriving rounded points.
3. Give phony and focused-test intersections denominator-one positions.
4. Remove the compensating next-zigzag region-extension gate and verify region emission remains valid.
5. Run rectilinear contracts and the KSR motion contract, then run rustfmt and Clippy.
6. Record the roadmap milestone, commit, and push independently.
