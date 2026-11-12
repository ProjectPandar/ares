# Extend the supplied pinned Native derivation with a headless thumbnail observer.
# ORCA_THUMBNAILS_BASE_RECIPE selects the same source/dependencies as the Native supplier.
let
  recipe = builtins.getEnv "ORCA_THUMBNAILS_BASE_RECIPE";
in
assert recipe != "";
(import (builtins.toPath recipe)).overrideAttrs (old: {
  pname = "orca-slicer-gcode-thumbnail-observer";
  postPatch = (old.postPatch or "") + ''
    cp ${./gcode-thumbnails.cpp} src/gcode-thumbnail-observer.cpp
    cat >> CMakeLists.txt <<'CMAKE'
    add_executable(gcode-thumbnail-observer src/gcode-thumbnail-observer.cpp)
    target_include_directories(gcode-thumbnail-observer PRIVATE "''${CMAKE_SOURCE_DIR}/src" "''${CMAKE_BINARY_DIR}/src")
    target_link_libraries(gcode-thumbnail-observer PRIVATE libslic3r)
    install(TARGETS gcode-thumbnail-observer RUNTIME DESTINATION bin COMPONENT thumbnail-observer)
    CMAKE
  '';
  buildFlags = [ "gcode-thumbnail-observer" ];
  installPhase = ''
    cmake --install . --component thumbnail-observer
  '';
  postInstall = "";
  postFixup = "";
  dontWrapGApps = true;
})
