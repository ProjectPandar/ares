let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCodeWriter.cpp \
        --replace-fail 'std::string GCodeWriter::set_speed(double F, const std::string &comment, const std::string &cooling_marker)
{
' 'std::string GCodeWriter::set_speed(double F, const std::string &comment, const std::string &cooling_marker)
{
    if (getenv("ORCA_DUMP_SETSPEED") != nullptr) { static std::mutex sq; sq.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SETSPEED"), "a"); if (f) { fprintf(f, "SETSPEED %.12g [%s]\n", F, cooling_marker.c_str()); fclose(f); } sq.unlock(); }
'
    '';
  })
