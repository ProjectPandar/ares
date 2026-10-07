# Native wipe-tower per-id counterexample

Source: OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`GCodeProcessor.cpp:3163–3165` (`WIPE_TOWER_END` flushes the tower cache),
`process_filaments:5935ff` (ToolChange flushes model/support/total).
These distinct flush boundaries need not attribute each split to the same id.

`native.json` is the same-input `--dump-color-change` receipt from
`/nix/store/1bq61dyqxb8yjdsaaq697ii6hbhb5cxm-orca-slicer-procgcode-2.4.2/bin/orca-slicer`.
The committed overhang fixture and Kobra 3 Max dual-filament statistics test
produce the input. Only the producer label is renamed to OrcaSlicer to admit
CONFIG_BLOCK; all commands/settings remain identical.

- Ares stream SHA256: `8d7935579a36c3b6be90ccd31516a554f23329f0ef9c90186bc15c39ef2e15ab`
- Admitted stream SHA256: `a6efb459b13dac9098871c5822d10c1dddd34253f78dc936184ed26e461d9068`
- Native JSON SHA256: `63d98cbbcbec463cac0c83cad01605833887987c2f1e37bb1f8782465374cb4e`
- Original stream/logs: `/tmp/ares-vg1-port-wip/wipe-tower/`.

Native id0: split sum `2580.3454413658474`, total `2460.0919237297494`,
difference `+120.25351763609797`. Native id1: split sum `1499.796153246658`,
total `1620.049670882756`, difference `-120.25351763609797`.
Global conservation holds; per-id conservation does not.

The live test compares every entry of all five maps by exact f64 bits and
checks global split conservation. Existing footer assertions and whole-byte
comparators remain unchanged. Receipts are retained in
`filament-volumes-oracle-*` directories for each live comparison.
