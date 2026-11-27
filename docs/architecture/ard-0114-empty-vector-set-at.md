# ARD-0114: Initialize empty filament slot destinations from their defaults

OrcaSlicer v2.5.0 allows a bare empty string to decode as a present, empty string vector (`src/libslic3r/Config.cpp:173–176,982–985`, `src/libslic3r/Config.hpp:1225–1229`). During ordinary filament refresh, `CLI::run` gets that existing option and calls `set_at` (`src/OrcaSlicer.cpp:3743–3771`). `ConfigOptionVector::set_at` asserts that the destination has at least its default member, then copies `front()` when it needs to grow (`src/libslic3r/Config.hpp:451–459`). Taking `front()` of an empty vector is undefined behavior when assertions are disabled. The OrcaSlicer release completed a project slice with blank notes in every filament slot; that observation does not define the empty-list behavior.

**Decision:** At the ordinary filament slot assignment boundary, Ares restores the registered option default when the destination is empty, then uses the existing destination-front growth and source-element-zero assignment. For example, `filament_notes` defaults to a vector with one blank member (`src/libslic3r/PrintConfig.cpp:3069–3076`). The rule applies to every key reaching this assignment path. It does not use the source as the initial fill value or change how explicit empty values decode elsewhere.

See [the implementation and verification spec](empty-vector-set-at.md).
