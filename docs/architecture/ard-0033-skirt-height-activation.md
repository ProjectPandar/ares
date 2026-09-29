# ARD-0033: Skirt height tests must preserve height ownership

Use `Print.cpp`'s finite-skirt branch: positive skirt loops, draft shield
disabled. Otherwise an infinite skirt can mask the tested layer count.
Verify these dependencies in the exported project. Do not alter printer
defaults globally; dependencies apply only to `skirt_height` coverage.

The definition declares a ceiling; `PrintConfig.cpp:10422-10424` supplies
the effective minimum zero by rejecting negatives. Test zero shutdown,
negative rejection and one/two-layer cases. Use actual random
interior generation instead of labeling constant 2 as seeded. Whole-G-code
comparison and the incomplete-domain gate remain unchanged.
