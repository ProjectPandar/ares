# JPEG baseline rewrite

This software is based in part on the work of the Independent JPEG Group.
See the unchanged upstream [README.ijg](README.ijg) for its license.

These files are a Rust rewrite of the libjpeg-turbo3.2.0 subset selected by
OrcaSlicer v2.5.0 `GCode/Thumbnails.cpp::compress_thumbnail_jpg`, not an
unmodified libjpeg distribution or a general JPEG library. Changes replace C
pointers and destination management with safe Rust buffers, retaining the
fixed8-bit quality85 sequential4:2:0 algorithm. Source copyrights are retained
in the corresponding modules. No TurboJPEG API or SIMD assembly is included.
