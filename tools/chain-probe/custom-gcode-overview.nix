# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Check the extraction with extract-custom-gcode-overview.mjs --check.
import ./orca-release {
  pname = "orca-slicer-custom-gcode-overview";
  patches = [ ./procgcode.patch ./custom-gcode-overview.patch ];
  postPatch = ''
    cp ${./custom-gcode-overview-oracle.hpp} src/custom-gcode-overview-oracle.hpp
    mkdir -p src/custom-gcode-overview
    cp ${./custom-gcode-overview/kernel.hpp} src/custom-gcode-overview/kernel.hpp
  '';
}
