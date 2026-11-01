# v2.5.0-beta (345efec64): dependencies from upstream CMake, not 2.4.2.
{ pname, source, nixpkgs }:
let
  pkgs = import nixpkgs {};
  src = builtins.path {
    path = builtins.toPath source;
    name = "orca-slicer-2.5.0-beta-oracle-source";
    filter = path: type: builtins.baseNameOf path != ".git";
  };
  # deps/OCCT/OCCT.cmake pins 8.0.1 plus this seam pcurve fix; CAD's
  # Standard_Failure::what() use requires that API, not nixpkgs' 7.9.3.
  occt = pkgs.opencascade-occt.overrideAttrs (old: {
    version = "8.0.1";
    src = pkgs.fetchurl {
      url = "https://github.com/Open-Cascade-SAS/OCCT/archive/refs/tags/V8_0_1.zip";
      sha256 = "7c033d917ee8f040c0512d289dcc5f02c148889d5bac17c3e25639accb44f0da";
    };
    patches = [ ./occt-seam-pcurve.patch ];
    nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.unzip ];
    cmakeFlags = old.cmakeFlags ++ [ "-DBUILD_MODULE_Draw=OFF" "-DBUILD_DOC_Overview=OFF" ];
  });
  wx = builtins.head (builtins.filter
    (dep: pkgs.lib.isDerivation dep && pkgs.lib.hasPrefix "wxwidgets" (pkgs.lib.getName dep))
    pkgs.orca-slicer.nativeBuildInputs);
  # src/CMakeLists.txt:93 and deps/wxInspector/wxInspector.cmake:24-33.
  wxInspector = pkgs.stdenv.mkDerivation {
    pname = "wxinspector-orca-250b";
    version = "1.0.0";
    src = pkgs.fetchurl {
      url = "https://github.com/Noisyfox/wxInspector/archive/refs/tags/v1.0.0.zip";
      sha256 = "0ba163956f2d468b19a91b96c5aba66ee9610843ea41dda628ea44cdafde7db7";
    };
    nativeBuildInputs = [ pkgs.cmake pkgs.unzip pkgs.pkg-config wx ];
    buildInputs = [ wx pkgs.gtk3 ];
    # wxInspector's GTK backend directly includes gtk/gtk.h and gdk/gdk.h.
    preConfigure = ''
      cmakeFlagsArray+=("-DCMAKE_CXX_FLAGS=-DwxDEBUG_LEVEL=0 $(pkg-config --cflags gtk+-3.0)")
    '';
    cmakeFlags = [ "-DCMAKE_POSITION_INDEPENDENT_CODE=ON" ];
  };
  # deps/SLVS/SLVS.cmake pins this archive; its adjacent CMakeLists.txt.in
  # installs the unmodified solver required by libslic3r/CMakeLists.txt:25-26.
  slvs = pkgs.stdenv.mkDerivation {
    pname = "libslvs-orca-250b";
    version = "3.0-4d870452";
    src = pkgs.fetchurl {
      url = "https://github.com/JacobStoren/SolveSpaceLib/archive/4d8704523e4bf212fadf5189f92484244f670fea.zip";
      sha256 = "1c4bdde9c3c6ef20ea4b50b73601de56769f2eb131b36927d7c6489f102e6c30";
    };
    nativeBuildInputs = [ pkgs.cmake pkgs.unzip ];
    postPatch = ''cp ${src}/deps/SLVS/CMakeLists.txt.in CMakeLists.txt'';
  };
in
assert source != "";
assert nixpkgs != "";
pkgs.orca-slicer.overrideAttrs (old: {
  inherit pname src;
  version = "2.5.0-beta";
  # libslic3r/CMakeLists.txt:627,629,685-741 requires CGAL6.2, assimp,
  # OCCT TKDESTEP + ModelingAlgorithms, JPEG and the source-pinned solver;
  # top-level CMakeLists.txt:850 requires LibDataChannel and :968-1030
  # embeds exactly Python3.12.13 (available at the unchanged nixpkgs pin).
  nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.python312 ];
  buildInputs = builtins.filter
    (dep: !(builtins.elem (pkgs.lib.getName dep) [ "cgal" "opencascade-occt" ]))
    old.buildInputs ++ [ pkgs.cgal occt pkgs.assimp pkgs.libjpeg_turbo pkgs.libdatachannel pkgs.python312 slvs wxInspector ];
  # Keep the applicable WebKit/IlmBase packaging fixes. The old OpenCV hunk
  # moved; the GUI-only configurable-update-check patch is outside this oracle.
  patches = [ (builtins.elemAt old.patches 0) (builtins.elemAt old.patches 3) ./opencv-components-beta.patch ./fhs-runtime-beta.patch ];
  # FHS packages use rooted libraries/Python, not AppImage FFmpeg6 or
  # bundled libpython staging in src/CMakeLists.txt:315-326,363-367,
  # or bundled FFmpeg installation in CMakeLists.txt:1325-1339.
  # New FuzzySkin.cpp:1 uses libnoise's upstream include directory spelling;
  # the pinned distro package installs the same header under noise/.
  prePatch = old.prePatch + ''
    substituteInPlace src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp \
      --replace-fail '<libnoise/module/modulebase.h>' '<noise/module/modulebase.h>'
  '';
  cmakeFlags = builtins.filter
    (flag: !(pkgs.lib.hasInfix "ORCA_VERSION_CHECK_DEFAULT" flag))
    old.cmakeFlags;
})
