# Plan: KSR FDM Test V4 task173 Arachne transition test-support partition

1. Move shared strategy, configuration, and central-chain fixtures into `transitions/test_support.rs`.
2. Update parent, endpoint, and application tests to use the shared test-support module without changing assertions.
3. Run all Arachne transition tests, line-count checks, formatting, and workspace Clippy.
4. Record the partition in `docs/roadmap.md`, commit, and push independently.
