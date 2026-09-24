let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'void GCodeProcessor::finalize(bool post_process)
{
    m_result.z_offset = m_z_offset;' 'void GCodeProcessor::finalize(bool post_process)
{
    if (getenv("ORCA_DUMP_G1CACHE") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_G1CACHE"), "a"); if (f) { for (size_t mi = 0; mi < m_time_processor.machines.size(); ++mi) { if (!m_time_processor.machines[mi].enabled) continue; for (const auto& it : m_time_processor.machines[mi].g1_times_cache) fprintf(f, "%u %.6f\n", it.id, (double)it.elapsed_time); } fclose(f); } }
    m_result.z_offset = m_z_offset;'
    '';
  })
