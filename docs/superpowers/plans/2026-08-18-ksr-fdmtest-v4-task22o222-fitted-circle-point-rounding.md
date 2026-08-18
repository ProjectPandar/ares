# Plan: KSR FDM Test V4 task222 fitted-circle Point constructor rounding

1. Replace task219's incomplete integer-storage expectation with a failing regression covering a fitted center whose scaled coordinates have fractions above one half.
2. Follow `Point(double, double)` and restore `std::round` conversion while preserving the pre-conversion radius.
3. Run focused arc tests and regenerate the complete KSR fixture output; record normalized first divergence plus line, arc, and wipe counts.
4. Correct task219 spec and roadmap text so no obsolete truncation contract remains.
5. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited correction independently.
