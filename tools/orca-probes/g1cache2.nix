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
    if (getenv("ORCA_DUMP_TOTAL") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_TOTAL"), "a"); if (f) { for (size_t mi = 0; mi < m_time_processor.machines.size(); ++mi) fprintf(f, "MACHINE %zu enabled=%d time=%.9f cache_n=%zu last_id=%u\\n", mi, (int)m_time_processor.machines[mi].enabled, m_time_processor.machines[mi].time, m_time_processor.machines[mi].g1_times_cache.size(), m_time_processor.machines[mi].g1_times_cache.empty() ? 0u : m_time_processor.machines[mi].g1_times_cache.back().id); fclose(f); } }
    m_result.z_offset = m_z_offset;'
    '';
  })
