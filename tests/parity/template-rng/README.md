# Seeded startup-template fixtures

Exact, unmodified Orca-exported stock 3MF inputs for the PlaceholderParser RNG
rewrite. Native source8500fcdccaa10b5099ac20d252af3a7c560046f1 (Orca2.4.2).
Original frozen producer:9a7b46fc, stock matrix complete:false; copying these six
inputs is not a new complete sweep. No reference output is fabricated or stored
as a passing snapshot: seeded E2E regenerates complete native output.

| Fixture | Frozen case | SHA256 |
|---|---|---|
| flashforge-04.3mf | case-nhVbMr | d17f2d8a325862dbe4cd45a50c9fc1e8b1e218a70b331a305a4730a75b17570d |
| flashforge-06.3mf | case-bKs3EZ | 3c17436c087ab729e1bec03ebedb4f0227370c6b77104523806c0e6d887daff3 |
| flashforge-08.3mf | case-1jESfd | c75b99b351ce0e7a4c2c17452b71054bfbf642ba3a480e450eb1c176169bea36 |
| raise3d-dual.3mf | case-TY2alA | 1b4859c750898c003c20465db87f1ffa06b4d592328d2857ad52d6969b58dfd8 |
| raise3d-left.3mf | case-HoYp3R | af694e57e88e87d1a72ce2c818d4ca0706c552dda2c742d2643b052946dc319f |
| raise3d-right.3mf | case-1Bg0jk | 6a0a06b6e39500c7110cf541a4cabd9df7071bf872066be46cf6cb7afffb1704 |

Run the runner::template_rng tests with ARES_PARITY_SEEDED_REFERENCE=1, an
explicit ORCA_REPLAY_SEED (0 and1), ORCA_REPLAY_CPU=0, the extended replay
executable and ARES_PARITY_ARTIFACT_ROOT. They retain input, both native runs,
Rust output and producer/seed/result JSON, and require complete ordered bytes
with only the established generator-line normalization.
