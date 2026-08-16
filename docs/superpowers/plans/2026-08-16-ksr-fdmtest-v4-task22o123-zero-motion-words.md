# Plan: Task 22o.123 valid zero-valued motion words

1. Add focused formatter assertions and a KSR output assertion for bare numeric motion words.
2. Split numeric formatting from the oversized motion module and preserve exact zero before leading-zero elision.
3. Run focused formatter and KSR tests, rustfmt, and clippy; commit and push.
