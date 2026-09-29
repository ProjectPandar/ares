# ARD-0035: Do not invent a zero lower bound for hot-plate temperature

`PrintConfig.cpp:1061` omits min; `Config.hpp`'s default lower limit is
`-FLT_MAX`, while `ConfigOptionInts` stores signed integers. Resolve the
representable lower bound with official execution, not physical-temperature
intuition. Keep temperature formulas distinct: highest-temperature selection
starts from zero upstream, while first-filament selection returns its value.
No clamping or rejection may be added merely to make test inputs convenient.
