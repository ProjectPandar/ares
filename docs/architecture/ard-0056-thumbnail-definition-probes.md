# ARD-0056: Canonical thumbnail-definition parity probes

The boundary is `PrintConfigDef::handle_legacy_composite`
(`libslic3r/PrintConfig.cpp:8290-8317`) and
`GCodeThumbnails::make_and_check_thumbnail_list`
(`libslic3r/GCode/Thumbnails.cpp:530-577`). Existing Rust destinations are
`options::typed_legacy::thumbnails`, `gcode_thumbnails`, and project settings
normalization. This slice validates those rewrite boundaries through the
native parity adapter; it does not introduce an Ares-owned pipeline.

The current authored `300x300` input is valid but is rewritten to
`300x300/PNG`. Generic byte-sensitive application proof correctly rejects
that difference. Generate explicit canonical definitions for the five
serialized formats, empty input and a multi-definition list. Keep generic
proof strict; test shorthand/default-format conversion separately with
independent literal expectations and complete ordered G-code comparisons.

Include changed dimensions to distinguish normalization from preset
replacement. Verify the effective definition in the official project and
final G-code, then compare the entire first official output against Ares.
No source-text pinning assertions or output-field filtering are permitted.
Live explicit BTT_TFT exposes a missing `GCode::do_export` rewrite branch
(`GCode.cpp:2567-2635`): omit the complete header block when the normalized
thumbnail list contains BTT_TFT. Port it to `project_slice::gcode_emit::header`
and guard the header-only layer-count replacement at its caller. Width blocks,
motion, statistics and config output remain unchanged.

The native artifact adapter selects raw ordered byte equality for BTT_TFT
from the input project's configuration, before comparing outputs. It must
record this exact comparator in evidence; ordinary inputs still require
the existing one-generator-line contract. No missing-header fallback based
on comparison failure is allowed. Add mutation checks at the artifact seam.

These are headless CLI definition/configuration tests, not rendered thumbnail
image tests. GUI image generation, arbitrary string exhaustion and all
printer contexts are deferred; the global parity gate remains incomplete.
In particular this non-BBL slice does not verify the BBL exclude-object state
side effects nested inside the upstream header branch. Comparator-mode
selection for unnormalized raw 3MF shorthand plus BTT default format is also
not covered here; the live comparisons consume canonical official exports.
