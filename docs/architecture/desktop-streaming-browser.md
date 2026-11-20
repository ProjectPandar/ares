# Isolate the large streaming browser test

The streaming test exercises slicing a 200 mm cube, the rendered toolpath Preview, matching viewer and exported G-code layer counts, and the WASM memory bound. Keep that coverage intact. In software-rendered Chromium, closing the test's browser context can leave GPU work in flight, delaying context creation for the next test in the same browser.

Run the streaming spec in `desktop-streaming` with the same Chromium settings as `desktop`. Playwright starts a separate test process and browser for that project. With `--workers=1`, running both projects schedules the streaming check first without reordering the other desktop specs. A new browser context alone would not isolate the GPU process.

To run the complete desktop browser coverage, include both `--project=desktop` and `--project=desktop-streaming`, or run the suite without a project filter. Only the streaming spec changes project. Application rendering and test assertions, timeouts, and retry settings are unchanged.
