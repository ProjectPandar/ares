# Plan: Remove obsolete Clipper path-order pinning

1. Remove tests that assert internal subject insertion order, exact source-stage checkpoint bytes or hashes, hard-coded geometry checksums or point counts, exact disjoint result order, or unchanged input vertex serialization.
2. Replace serialization-order and exact-count assertions in geometry, compensation, perimeter, bridge, and fill-entity tests with behavior, topology, nonempty-output, structural relationship, and order-invariant assertions while retaining public gates and boundary cases.
3. Run the focused geometry tests, formatting, lint, and workspace nextest suite, then commit and push.
