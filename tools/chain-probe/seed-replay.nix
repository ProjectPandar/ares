# Seeded OrcaSlicer v2.5.0 replay executable without print-statistics side effects.
import ./orca-release {
  pname = "orca-slicer-seed-replay";
  patches = [
    ./orca-release/seed.patch
  ];
}
