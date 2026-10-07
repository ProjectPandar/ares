#!/bin/sh
# Playwright WebKit executable for e2e/webkit-env.sh: the bundled
# minibrowser-wpe/MiniBrowser wrapper with the Nix libraries appended to its
# LD_LIBRARY_PATH (the wrapper itself replaces LD_LIBRARY_PATH).
dir="$ARES_E2E_WEBKIT_DIR"
export WEBKIT_EXEC_PATH="$dir/bin"
export WEBKIT_INJECTED_BUNDLE_PATH="$dir/lib"
export WEBKIT_INSPECTOR_RESOURCES_PATH="$dir/share"
export LD_LIBRARY_PATH="$dir/lib:$dir/sys/lib:$ARES_E2E_WEBKIT_LIBS"
export GST_PLUGIN_SYSTEM_PATH_1_0="$ARES_E2E_WEBKIT_LIBS/gstreamer-1.0"
WEBKIT_FORCE_COMPLEX_TEXT=1 exec "$dir/bin/MiniBrowser" "$@"
