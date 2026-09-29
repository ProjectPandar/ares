# ARD-0025: Incomplete parity evidence cannot pass acceptance

The option coverage entrypoint is an acceptance boundary for observable
G-code equality. A known reason for missing evidence does not supply that
evidence. Therefore every selected domain must be PASS and the selection
must be nonempty for successful completion.

Reports remain descriptive: absent upstream ports, rejected values,
unbounded sampling, and nondeterministic reference outputs retain their
existing classifications. Their diagnostic text must never determine
whether acceptance succeeds. Preserve reports and full output artifacts
before signaling failure so the next source-cited rewrite can use them.

This decision changes only the CLI parity harness. It adds no core slicing
behavior and does not relax the all-printer, all-option objective.
