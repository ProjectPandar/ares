# Spec: Chamber-control activation coverage

Follow ARD-0049's PrintConfig/GCode/GCodeWriter source boundaries and
existing core/CLI Rust destinations.

Correct chamber capability classification and activate the five selected
domains. Acceptance: 12 generated cases cover three Boolean pairs and both
0/1500/seeded temperature domains. Each exported requested value remains
effective, all dependencies are verified, and the original's executable
commands/placeholder witness match expectations before full G-code equality.

Capability false does not suppress chamber commands upstream. Activation
false and target zero suppress automatic heating/shutdown; minimum chamber
temperature only reaches the template. Auxiliary fan true brackets waiting
heating. Two further cases with explicit M141/M191 startup commands must
suppress automatic heating without suppressing the final shutdown.

Single-filament coverage does not certify multi-tool aggregation or every
firmware/template interaction. No physical printer is controlled. Existing
scaffolding only validates the cited rewrite, not a new Ares pipeline.
