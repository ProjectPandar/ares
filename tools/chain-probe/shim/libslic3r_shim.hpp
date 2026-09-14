#pragma once
// Standalone shim: satisfy PressureEqualizer.hpp includes with minimal
// definitions. We compile the .cpp with -DSTANDALONE_PE below to bypass
// libslic3r.h entirely.
#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>
#include <cmath>
#include <cassert>
#include <limits>
#include <algorithm>
