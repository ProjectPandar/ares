# Spec: KSR FDM Test V4 task209 medial Voronoi coordinate conversion

## Observable contract

Finite Boost Voronoi vertices become integer Ares points using C++ integral conversion semantics: truncate positive and negative fractional coordinates toward zero. They are not rounded to the nearest integer. The same converted endpoints drive edge widths, eligibility, chaining, and emitted gap-fill paths.

A focused sign/boundary test pins `7.9 → 7`, `-7.9 → -7`, and sub-unit values to zero. Fixture gap-fill blocks remain 206 versus reference 470; moves change by one to 8,058 versus 5,881, excluding conversion semantics as the count deficit's cause. Files remain below 400 LOC; medial-axis/gap tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports the reached conversions from OrcaSlicer 2.4.2 `Geometry/MedialAxis.cpp:493-537,627-630`, where Voronoi vertex doubles construct integral `Point` coordinates. It changes only conversion semantics; diagram annotation, validation limits, gap domains, timing, and remaining differences are deferred.
