# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Check the extraction with extract-segment-colors.mjs --check.
import ./orca-release {
  pname = "orca-slicer-segment-colors";
  patches = [ ./procgcode.patch ./segment-colors.patch ];
  postPatch = ''
    cp ${./segment-colors-oracle.hpp} src/segment-colors-oracle.hpp
    mkdir -p src/segment-colors
    cp ${./segment-colors/kernel.hpp} src/segment-colors/kernel.hpp
  '';
}
