# Release diagnostics use the move remapping implemented by OrcaSlicer itself.
import ./orca-release {
  pname = "orca-slicer-merged-oracle";
  patches = [
    ./orca-release/seed.patch
    ./procgcode.patch
    ./procgcode-dynamics.patch
    ./marker-time.patch
    ./print-statistics.patch
    ./merged-oracle-extra.patch
  ];
  postPatch = ''
    cp ${./marker-time-oracle.hpp} src/marker-time-oracle.hpp
    mkdir -p src/marker-time
    cp ${./marker-time/kernel.hpp} src/marker-time/kernel.hpp
    cp ${./actual-speed-profile-oracle.hpp} src/actual-speed-profile-oracle.hpp
    mkdir -p src/actual-speed-profile
    cp ${./actual-speed-profile/kernel.hpp} src/actual-speed-profile/kernel.hpp
    cp ${./custom-gcode-overview-oracle.hpp} src/custom-gcode-overview-oracle.hpp
    mkdir -p src/custom-gcode-overview
    cp ${./custom-gcode-overview/kernel.hpp} src/custom-gcode-overview/kernel.hpp
    cp ${./role-times-oracle.hpp} src/role-times-oracle.hpp
    mkdir -p src/role-times
    cp ${./role-times/kernel.hpp} src/role-times/kernel.hpp
    cp ${./segment-hw-oracle.hpp} src/segment-hw-oracle.hpp
    mkdir -p src/segment-hw
    cp ${./segment-hw/kernel.hpp} src/segment-hw/kernel.hpp
  '';
}
