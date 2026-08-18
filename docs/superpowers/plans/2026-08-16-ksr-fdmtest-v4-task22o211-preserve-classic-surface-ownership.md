# Plan: KSR FDM Test V4 task211 preserve classic surface ownership

1. Add a failing real-fixture assertion that layer-two thin entities occupy multiple perimeter-owning islands.
2. Carry aligned source-surface indices for classic perimeter and thin-fill outputs through layer-region and fill entity records.
3. Build source-index-to-island ownership from assigned perimeters, then place owned thin fills through that map; retain spatial assignment for unowned entities.
4. Run ownership/island tests and compare fixture feature interleaving/counts.
5. Run line-count checks, formatting, and workspace Clippy; update `docs/roadmap.md`, commit, and push independently.
