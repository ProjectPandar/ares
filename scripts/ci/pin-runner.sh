#!/usr/bin/env bash
# nextest target runner: pins each test process (and the Orca it spawns) to one
# CPU, so available_parallelism() is 1 for both Ares and Orca, while different
# tests run on different CPUs. Used as
# CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER in parity jobs.
n=$(nproc --all)
cpu=$(( (${ARES_PIN_OFFSET:-0} + ${NEXTEST_TEST_GLOBAL_SLOT:-0}) % n ))
# Seeded reports record the replay CPU; it is the one this test is pinned to.
export ORCA_REPLAY_CPU=$cpu
exec taskset -c "$cpu" "$@"
