# Preheat normal move-duration streaming oracle on OrcaSlicer v2.5.0.
import ./orca-release {
  pname = "orca-slicer-preheat-move-durations";
  patches = [
    ./orca-release/seed.patch
    ./procgcode.patch
    ./marker-time.patch
    ./preheat-duration-oracle.patch
  ];
  postPatch = ''
    cp ${./marker-time-oracle.hpp} src/marker-time-oracle.hpp
    mkdir -p src/marker-time
    cp ${./marker-time/kernel.hpp} src/marker-time/kernel.hpp
  '';
}
