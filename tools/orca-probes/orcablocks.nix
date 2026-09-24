let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'float block_time = block.time();' 'float block_time = block.time();
        if (getenv("ORCA_DUMP_BLOCKS") != nullptr) { static std::mutex mb; mb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_BLOCKS"), "a"); if (f) { fprintf(f, "%u %d %.9f %.9f %.9f %.9f %.9f %.9f %.9f\n", block.g1_line_id, (int)block.move_type, block.distance, block.feedrate_profile.cruise, block.acceleration, block.feedrate_profile.entry, block.feedrate_profile.exit, block.safe_feedrate, double(block_time)); fclose(f); } mb.unlock(); }'
    '';
  })
