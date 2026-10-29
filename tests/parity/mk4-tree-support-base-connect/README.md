# mk4-tree-support-base-connect RED fixture

Genuine stock Prusa MK4 project ("Prusa MK4 0.4 nozzle" +
"0.20mm Standard @MK4", single `overhang.obj` whose mesh and
`model_settings.config` are byte-identical to the
`support-interface-top-layers-base-connect` K1 witness family) with the
K1 witness's 58-key support/tree/raft family verbatim, `enable_support=1`,
`support_type=tree(auto)`, and `support_interface_top_layers=2`.

- `witness.3mf` (SHA-256
  `90650d0fdb5c17a2d84ad6f2c87e36ad0f3fc6a3f90645068c243e8738c10834`) —
  the stock MK4 unmodified base slices complete-stream equal today; enabling
  the witness support family diverges. Dual-side stage probes proved every
  `Fill::connect_base_support` stage equal except the cost-classified arch
  selection, and `%a` term dumps proved all cost inputs, `l2`, `dot` and `t`
  bit-equal while only the projection residual differs: the ported
  `evaluate_support_arch_cost` computes `p - (a + t*v)` and clamps `t`
  instead of the upstream `Line.hpp:43-71` `distance_to_squared` branches
  (`t*v - va`, endpoint returns). No Native output is committed; the case
  slices Native at test time in both official and seeded replay modes
  through `native_stream_case::check`.
