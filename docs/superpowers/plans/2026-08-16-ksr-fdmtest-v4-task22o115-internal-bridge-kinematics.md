# Plan: Task 22o.115 internal-bridge motion options

1. Add failing typed-option assertions for KSR bridge speed, internal-bridge speed, and bridge acceleration.
2. Add a failing output assertion for the exact internal-bridge acceleration and feedrate sequence.
3. Resolve percent options against their Orca bases and route bridge roles through the dedicated kinematics.
4. Run focused tests, smoke-slice the KSR 3MF, then run rustfmt and clippy before committing and pushing.
