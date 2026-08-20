# Plan: 3MF project options are archive-owned

1. Update the failing CLI test to assert the public project-option ownership rule rather than a removed core loader detail.
2. Reject `--options` for `.3mf` input before reading or parsing the archive; retain the existing STL route.
3. Run the focused CLI tests, formatting, lint, and workspace nextest suite, then commit and push.
