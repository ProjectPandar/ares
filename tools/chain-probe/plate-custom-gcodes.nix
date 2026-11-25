# Plate custom G-codes oracle (docs/architecture/project-plate-custom-gcodes.md):
# OrcaSlicer v2.5.0 with seeded replay plus plate-custom-gcodes.patch, which with
# ARES_PLATE_CUSTOM_GCODES_DUMP=<path> writes Model::plates_custom_gcodes right
# after Model::read_from_file and exits.
import ./orca-release {
  pname = "orca-slicer-plate-custom-gcodes";
  patches = [
    ./orca-release/seed.patch
    ./plate-custom-gcodes.patch
  ];
}
