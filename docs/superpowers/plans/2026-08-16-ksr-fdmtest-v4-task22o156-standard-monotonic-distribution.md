# Plan: Task 22O.156 standard monotonic queue distribution

1. Add a failing focused contract for singleton distribution consumption and the next MT19937-64 word.
2. Replace modulo sampling with the standard multiply-and-reject bounded-integer mapping.
3. Remove the obsolete exact branching-order assertion; retain deterministic, complete, precedence-preserving behavioral checks.
4. Run every rectilinear fill contract and the KSR motion contract.
5. Run rustfmt and Clippy, record the roadmap milestone, then commit and push independently.
