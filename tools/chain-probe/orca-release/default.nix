{ pname, patches, postPatch ? "" }:
let
  nixpkgs = builtins.getEnv "ORCA_PROCGCODE_NIXPKGS";
  pkgs = import nixpkgs {};
  revision = "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19";
  src = pkgs.fetchzip {
    name = revision;
    url = "https://codeload.github.com/SoftFever/OrcaSlicer/tar.gz/${revision}";
    extension = "tar.gz";
    hash = "sha256-WL3tXUkj/XIltcUSSnHGbOXWWLuYZ/hEeZ6L94w72c0=";
  };

  # Satisfy the release's CAD import APIs from deps/OCCT/OCCT.cmake.
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
  wxInspector = pkgs.stdenv.mkDerivation {
    pname = "wxinspector-orca-250b";
    version = "1.0.0";
    src = pkgs.fetchurl {
      url = "https://github.com/Noisyfox/wxInspector/archive/refs/tags/v1.0.0.zip";
      sha256 = "0ba163956f2d468b19a91b96c5aba66ee9610843ea41dda628ea44cdafde7db7";
    };
    nativeBuildInputs = [ pkgs.cmake pkgs.unzip pkgs.pkg-config wx ];
    buildInputs = [ wx pkgs.gtk3 ];
    preConfigure = ''
      cmakeFlagsArray+=("-DCMAKE_CXX_FLAGS=-DwxDEBUG_LEVEL=0 $(pkg-config --cflags gtk+-3.0)")
    '';
    cmakeFlags = [ "-DCMAKE_POSITION_INDEPENDENT_CODE=ON" ];
  };
  slvs = pkgs.stdenv.mkDerivation ({
    pname = "libslvs-orca-250";
    version = "4d870452";
    src = pkgs.fetchurl {
      url = "https://github.com/JacobStoren/SolveSpaceLib/archive/4d8704523e4bf212fadf5189f92484244f670fea.zip";
      hash = "sha256-HEvd6cPG7yDqS1C3NgHeVnafLrExs2kn18ZInxAubDA=";
    };
    nativeBuildInputs = [ pkgs.cmake pkgs.unzip ];
    postPatch = ''cp ${src}/deps/SLVS/CMakeLists.txt.in CMakeLists.txt'';
  } // affinity);

  # CI leaves this unset. Local users of a shared Nix daemon can bind the
  # oracle builder itself, rather than only pinning the nix client.
  cpus = builtins.getEnv "ORCA_BUILD_TASKSET";
  affinity = pkgs.lib.optionalAttrs (cpus != "") {
    realBuilder = "${pkgs.util-linux}/bin/taskset";
    args = [ "-c" cpus "${pkgs.bash}/bin/bash" "-e"
      "${pkgs.path}/pkgs/stdenv/generic/source-stdenv.sh"
      "${pkgs.path}/pkgs/stdenv/generic/default-builder.sh" ];
  };
in
assert nixpkgs != "";
pkgs.orca-slicer.overrideAttrs (old: {
  inherit pname src;
  version = "2.5.0";
  doCheck = false;
  nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.python312 ];
  buildInputs = builtins.filter
    (dep: !(pkgs.lib.isDerivation dep && builtins.elem (pkgs.lib.getName dep) [ "cgal" "opencascade-occt" ]))
    old.buildInputs ++ [ pkgs.cgal occt pkgs.assimp pkgs.libjpeg_turbo pkgs.libdatachannel pkgs.python312 slvs wxInspector ];
  patches = [ (builtins.elemAt old.patches 0) (builtins.elemAt old.patches 3)
    ./opencv-components.patch ./fhs-runtime.patch ] ++ patches;
  prePatch = old.prePatch + ''
    substituteInPlace src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp \
      --replace-fail '<libnoise/module/modulebase.h>' '<noise/module/modulebase.h>'
  '';
  postPatch = (old.postPatch or "") + postPatch;
  cmakeFlags = builtins.filter (flag: !(pkgs.lib.hasInfix "ORCA_VERSION_CHECK_DEFAULT" flag)) old.cmakeFlags
    ++ [ "-DORCA_BUNDLED_UV_EXECUTABLE=${pkgs.uv}/bin/uv" ];
} // affinity)
