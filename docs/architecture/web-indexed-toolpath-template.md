# Indexed Web Preview segment template

`OrcaSlicer/src/libvgcode/src/SegmentTemplate.cpp:13-37` defines eight vertex IDs and an
ordered list of 24 triangle indices. In `web/app/src/viewport/toolpathMesh.ts`, replace the
expanded stream of IDs with those eight IDs and an element buffer. Preserve the triangle
order and winding, the per-instance segment IDs, and all shader inputs and calculations.
Texture packing, option markers, and viewer-data assembly are outside this change.

`ViewerImpl::load` sets the view and enables the drawable entities before rendering
(`ViewerImpl.cpp:1293-1296`). Web Preview must likewise submit one segment draw when the
initial enabled view arrives, not an extra segment draw while it is still loading.
Verify actual WebGL submissions and compare framebuffer bytes within each browser.
