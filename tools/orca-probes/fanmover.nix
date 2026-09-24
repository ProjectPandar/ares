let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/FanMover.cpp \
        --replace-fail 'void FanMover::_process_gcode_line(GCodeReader& reader, const GCodeReader::GCodeLine& line)
{' 'void FanMover::_process_gcode_line(GCodeReader& reader, const GCodeReader::GCodeLine& line)
{
    if (getenv("ORCA_DUMP_FANMOVER") != nullptr) fprintf(stderr, "FM raw=[%s] custom=%d front=%d back=%d bts=%.3f bufN=%zu\\n", line.raw().c_str(), (int)m_is_custom_gcode, m_front_buffer_fan_speed, m_back_buffer_fan_speed, m_buffer_time_size, m_buffer.size());'
    '';
  })
