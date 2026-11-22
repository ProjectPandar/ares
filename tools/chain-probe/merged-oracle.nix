# Release diagnostics use the move remapping implemented by OrcaSlicer itself.
import ./orca-release {
  pname = "orca-slicer-merged-oracle";
  patches = [
    ./orca-release/seed.patch
    ./procgcode.patch
    ./procgcode-dynamics.patch
    ./marker-time.patch
    ./print-statistics.patch
  ];
  postPatch = ''
    cp ${./marker-time-oracle.hpp} src/marker-time-oracle.hpp
    mkdir -p src/marker-time
    cp ${./marker-time/kernel.hpp} src/marker-time/kernel.hpp
  '';
}
