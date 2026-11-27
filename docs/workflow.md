# Multi-agent workflow and recovery

Handoff document for the coordinator (COORD). It describes how COORD and the workers run, how
changes land on `main` through GitHub Merge Queue, and how to bring everything back after a
machine crash or reboot. It covers the process only. Current assignments are kept in the
coordinator ledger and the inbox, not here. Update this file whenever a workflow rule changes.

Machine-specific locations are written as placeholders:

| Placeholder | Meaning |
|---|---|
| `$COORD_DIR` | Coordinator state directory (outside the repository) |
| `$WORKTREES` | Directory that holds every worktree except the main checkout |
| `$EVIDENCE` | Worker logs and artifacts (never committed) |
| `$PI_SESSIONS` | pi session files, one directory per working directory |

## Roles

| Role | Agent | Terminal name | Notes |
|---|---|---|---|
| COORD | Claude Code | (session root) | Assigns, reviews, decides, opens/enqueues PRs into Merge Queue. Sole reviewer. |
| W1-W6 | pi, `magpie/zero-ai-coding-gpt/gpt-6.1-sol`, 256k | `worker1` … `worker6` | Fixed workers. W5 handles batch/release PR assembly and optional local qualification. |
| WEB | Claude Code, `claude-opus-5-5`, effort high | `web` | Fixed web/UI worker with its own long-lived worktree. |
| W7+ | pi, same model | `worker7` … | Dynamic workers. Launched when work queues up, killed when their task is merged. |

Fixed workers (W0-W6, WEB) are never killed by the patrol. Only COORD signs `[COORD]`; workers
write `Wn -> COORD: …`.

## Locations

- The main checkout is the only worktree outside `$WORKTREES`. Never put worktrees in `/tmp`, which is
  wiped on reboot, or at the top level of the home directory.
- `$COORD_DIR` holds the coordinator state:
  - `inbox.log`, which workers append to with `bin/notify.sh <name> "<line>"`, and `mark`, the last line read.
  - The decision ledger, `b42-candidates.txt`. It records every RR, verdict, PR/Merge Queue landing, and user decision.
  - `TODO.md`, the work the user queued for after the current tasks. The patrol dispatches it when a worker frees up.
  - `briefs/`, the task briefs. `briefs/dynamic-worker-rules.md` is the onboarding brief for new workers.
  - `bin/`:
    - `notify.sh`;
    - `pin-runner.sh`, which pins each test to one CPU (`ARES_PIN_CPUS` / `ARES_PIN_OFFSET`);
    - `parity-control-set.sh`;
    - the Orca launchers.
  - `parity-250.env` and `parity-seeded-250.env`, the official and seeded Native comparison environments.
  - `moonraker-250.env`, including `ARES_MOONRAKER_LEGACY_METADATA_PY` for Moonraker v0.9.3 metadata extraction.
  - `gcroots/`, the Nix GC roots for every local Native oracle build. A nightly GC deletes unrooted store paths.
  - `orca-cache/`, the shared persistent local Native output cache.
  - An upstream Orca history bare repository, used with `git --git-dir=… show <rev>:<path>`.
  - A rooted python3, always run with `-I`.
- `$EVIDENCE/<worker>/<task>/` holds worker logs and artifacts.

## Terminals (`hc`)

Workers run in herdr-core terminals managed by `hc`.

- `hc ls` lists terminals and their status (working / done / blocked).
- `hc read <t>` shows the screen and history. Always read before sending: an open select dialog
  swallows the next Enter and picks option 1.
- `hc send <t> "<text>"` pastes the text and presses Enter.
- `hc keys <t> <key>` answers a dialog: send the option key, then Enter.
- `hc kill <t>` ends a worker.

## Recovery after a crash or reboot

1. **Services.**
   - Run `hc server status` and start the server if needed.
   - Run `hc ls` to see what survived.
   - The terminal host may have restored old worker processes on the same session files. Kill them
     before relaunching, so that no session has two writers.
2. **Coordinator.** Start Claude Code in the main checkout and resume the previous session
   (`claude --resume`). Then:
   - Recreate the patrol cron job if missing (see Patrol).
   - Arm the inbox Monitor (see Patrol).
   - Read the new inbox lines: `tail -n +$(( $(cat $COORD_DIR/mark) + 1 )) $COORD_DIR/inbox.log`.
   - Read the last ~50 lines of the ledger and check open PRs / Merge Queue (`gh pr list`, `gh run list --event merge_group`).
3. **Fixed workers.** Relaunch each one with its previous session so it keeps its context. Use the
   newest session file for that worker under `$PI_SESSIONS/<cwd-key>/`.
   ```sh
   hc run --bg --outlive-parent --name workerN --cwd <worktree> \
     pi --session <session.jsonl> --model magpie/zero-ai-coding-gpt/gpt-6.1-sol --thinking <medium|high> \
     "Machine restarted. Re-check your worktree, branch and last step, then continue; report status to COORD."
   hc run --bg --outlive-parent --name web --cwd <web worktree> \
     claude --resume <id> --model claude-opus-5-5 --dangerously-skip-permissions --effort high \
     "Machine restarted. Re-check your branch and last step, then continue; report status to COORD."
   ```
   If the session's cwd no longer exists, pi shows a Continue dialog: run `hc keys <t> enter`.
4. **Dynamic workers.** Relaunch only those whose task is still open in the ledger, starting from
   their brief.
5. **Lost state checks.**
   - `/tmp` is gone, so any local run that wrote there must be redone.
   - Local Native oracles: every store path in the env files must exist and be rooted under
     `$COORD_DIR/gcroots/`.
     - A Native launch that fails with exit 255 and empty output means a store path is missing.
     - Rebuild it with `nix build -o $COORD_DIR/gcroots/<name>`.
   - Git, in the main checkout:
     - Run `git worktree prune`, `git fetch origin`, and `git pull --ff-only origin main`.
     - Verify local `main` equals `origin/main`.
   - Check active GitHub Merge Queue entries (`gh pr list --json number,state,mergeStateStatus,autoMergeRequest`); if a queued PR failed, diagnose, push the fix, and re-enqueue.
6. **Report** to the user what was restored and what has to be rerun.

## Patrol

There are two mechanisms. Both are session-only: they die with the coordinator session, so
recreate them after every restart or context reset.

- **Inbox Monitor.** Re-arm it whenever it expires.
  ```sh
  f=$COORD_DIR/inbox.log; n=$(wc -l < $f); while true; do m=$(wc -l < $f); if [ "$m" -gt "$n" ]; then tail -n +$((n+1)) $f | head -n $((m-n)) | awk '{print substr($0,1,600); fflush()}'; n=$m; elif [ "$m" -lt "$n" ]; then echo "INBOX SHRANK $n -> $m"; n=$m; fi; sleep 2; done
  ```
- **Patrol cron.** Runs every 15 minutes at off-minutes (e.g. `4,19,34,49 * * * *`). Each run:
  1. Handle every new inbox line (RRs, questions), then update `mark`. Re-arm the Monitor if it expired.
  2. Run `hc ls` and read each worker's pane.
     - Answer open dialogs.
     - Look for stalls, errors, and rule violations.
     - Give an idle fixed worker its next task (check `TODO.md`), or tell the user.
  3. Kill dynamic workers that are idle with no open task, and record each one in the ledger.
  4. Check open PRs and GitHub Merge Queue runs (`gh pr list`, `gh run list --event merge_group`); fast-forward local `main` once a PR merges.

## Assigning work

- Every slice is a source-cited rewrite of an upstream `libslic3r` / `libvgcode` boundary (see
  `AGENTS.md`).
- Before dispatching:
  - Verify the premise against `main`.
  - Dedupe: grep open PRs, origin branches, and the ledger for the same upstream lines.
- Write larger tasks as a brief in `$COORD_DIR/briefs/`. A brief covers:
  - the goal;
  - the upstream files, functions, and lines;
  - the included and deferred behaviour;
  - how it is verified.

  Then `hc send` the worker a short message that points at the brief.
- Give each worker a disjoint CPU range and, for browser tests, its own preview port. Exploration-only
  workers (investigation, no test suites or full builds) get no dedicated CPUs and run unpinned.
- To launch a dynamic worker, write its task brief, then run:
  ```sh
  hc run --bg --outlive-parent --name workerN --cwd <main checkout> pi --model magpie/zero-ai-coding-gpt/gpt-6.1-sol \
    --thinking medium "Read $COORD_DIR/briefs/dynamic-worker-rules.md and then $COORD_DIR/briefs/<task>.md, and do the task. You are workerN."
  ```

## Slice workflow (worker side)

1. Create a new worktree under `$WORKTREES` from the base COORD names (`main` or an agreed
   predecessor), on the worker's own branch.
   - Link `OrcaSlicer` in the worktree root to the pinned upstream checkout; local probes read it there.
   - Give each worktree its own `CARGO_TARGET_DIR`, so concurrent builds do not lock or overwrite each other.
   - Ensure local `hk` git hooks (`hk.pkl`) are active (`hk install` in the repository).
2. Commit in TDD order:
   1. the spec in `docs/architecture/`;
   2. a failing test (RED);
   3. the fix (GREEN).

   The upstream reference is the pinned Orca release (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
3. Run focused local checks before pushing:
   - affected parity cases in both modes (official and seeded), using each mode's canonical
     environment, reference flags, Moonraker env, shared `ARES_ORCA_CACHE_DIR`, and `pin-runner.sh`
     affinity;
   - before focused verification, verify the worktree's `OrcaSlicer` link points to the pinned
     revision; disclose any failed preflight as not executed rather than PASS/FAIL;
   - `cargo fmt --check`;
   - `cargo clippy --workspace --all-targets -- -D warnings`;
   - `cargo check --target wasm32-unknown-unknown -p ares-core -p ares-wasm`;
   - web static checks (`bun run --cwd web fmt:check`, `lint`, `typecheck`, `test:unit`) and
     focused Playwright specs when web or WASM code is touched.
4. Push the branch to `origin` (local `hk` `pre-push` hook runs in `< 9s`). Send one RR line
   through `notify.sh` with branch, tip, base, commits, verification results, and RR file path,
   then wait for COORD review.

## Review (COORD)

For each RR, check:
- **History.** The three-dot diff `git diff main...tip`, every commit message, and base ancestry
  (`git merge-base --is-ancestor main tip`, or the agreed predecessor).
- **Author and trailers.** Author/committer is `5aaee9 <7685264+5aaee9@users.noreply.github.com>`.
  Commit messages MUST NOT contain any `Co-Authored-By:` trailer.
- **Hygiene.** Grep commit messages and diff (docs included). Flag any of:
  - `Co-Authored-By:` trailers;
  - worker and batch labels, COORD, coordinator;
  - RR, RED, GREEN, FULL, STOP, gate, queued;
  - approved, review, pending, frozen, qualify/qualification, integrated, evidence;
  - CPU numbers, local paths;
  - `#[allow]` / `#[expect]`.
- **Code.** Compare against cited upstream lines. Reject comparator weakening, masks, fitted
  expectations, committed Native goldens, assertions that Ares differs from Native, and
  non-generated files over 400 lines.
- **Clippy & static.** Verify `--all-targets` Clippy, wasm32 check, and web static checks.

Record the verdict in the ledger, then advance the accepted chain to a PR for GitHub Merge Queue.

## Merging via GitHub Merge Queue

`main` is protected by a GitHub Ruleset requiring linear history, a Pull Request, and **GitHub
Merge Queue** with the two required status checks `tier1-gate` and `pages-build`. Direct pushes to
`main` are disabled. Full CI architecture, latency budgets, shard completeness proofs, and GHCR
oracle image details live in [docs/ci-merge-queue.md](ci-merge-queue.md).

### Standard PR flow (single accepted chain)

1. **Prepare clean PR commits.** On top of `origin/main`, produce one clean linear commit per
   logical slice (squashing worker TDD iterations when appropriate via `git commit-tree` or rebase,
   preserving exact verified tree, `5aaee9` author/committer, no `Co-Authored-By:` trailer, and
   clean message with zero internal labels). Push the PR branch and open a PR targeting `main`
   (`gh pr create --base main`).
2. **PR fast loop (`pull_request`, ~30–60s).** `Tier 1` runs `fast-checks` (merged `cargo fmt`,
   `bun fmt`, `bun lint`, TS7 `bun check`, `bun test --randomize`, shard proofs with strict ms
   budgets) in parallel with diff-selected `rust-wasm-check`, `rust-clippy`, and `rust-nonparity`
   (`scripts/ci/select-pr-checks.sh`), while `Cloudflare Pages` runs `pages-build`.
   - When an on-demand full parity check is desired before queuing, add the `run-full-parity`
     label to the PR.
3. **Enqueue into Merge Queue.** Once review is complete and PR checks pass, arm Merge Queue:
   ```sh
   gh pr merge <pr-number> --auto --rebase
   ```
   GitHub rebases the PR onto the latest `main` in a speculative `merge_group` ref.
4. **Complete Merge Queue gate (`merge_group`).**
   - **Docs-only fast path (strictly fail-closed):** When every changed file in
     `merge_group.base_sha..HEAD` is a pure Markdown doc (`docs/**/*.md`, `README.md`, `AGENTS.md`,
     `CONTRIBUTING.md`, `CHANGELOG.md`; never `THIRD_PARTY_NOTICES.md` or any code/CI/config file),
     `merge-scope` skips the 25-minute parity/web E2E matrix and completes in ~40 seconds via
     `fast-checks`, `tier1-gate`, and `pages-build`.
   - **All other changes (100% complete suite):** Any non-doc file or unreadable/empty diff fails
     closed to the complete 29-job suite on public `ubuntu-latest` runners:
     - `fast-checks`, `rust-wasm-check`, `rust-clippy`, `rust-nonparity`;
     - `build-parity` (builds `orca_parity` nextest archive + `target/debug/ares` and proves 100%
       8-shard completeness for both official and seeded profiles);
     - 8 `parity-official` shards + 8 `parity-seeded` shards using the pinned, SHA-256-verified GHCR
       `ci-oracles` v2.5.0 image (zero Nix installation) and SHA-verified Moonraker extractors;
     - `web-build` (release CLI, `gen-configdef --check`, `verify-configdef.mjs`, WASM + web app +
       unit tests + desktop shard completeness proof);
     - `web-streaming`, 4 timing-balanced `web-desktop` shards, and `web-packages-mobile`
       (reusing the prebuilt `orca_parity` archive, `--workers 1 --retries 0`);
     - `tier1-gate` and `Cloudflare Pages` (`pages-build`).
5. **Auto-merge & production deploy.** When `tier1-gate` and `pages-build` both pass 100% green,
   GitHub Merge Queue fast-forwards `main` with linear history. On `push: main`, `Tier 1` does not
   re-run tests already gated by `merge_group`; `Cloudflare Pages` only builds/packages
   `web/app/dist` and deploys directly to production.
6. **Handling Merge Queue failures.**
   - Never retry flaky jobs, relax budgets/timeouts, skip tests, or weaken comparators.
   - Inspect failing job logs (`gh run view <run-id> --log-failed` or
     `gh api --allow-escape-sequences repos/ProjectPandar/ares/actions/jobs/<job-id>/logs`).
   - Fix the root cause cleanly on the PR branch. While a PR is still active in the Merge Queue,
     branch pushes are blocked (`GH006`); dequeue first via GraphQL `dequeuePullRequest` if needed,
     push the fix, and re-arm `gh pr merge <pr-number> --auto --rebase`.
7. **Post-merge sync.** After Merge Queue lands a PR:
   - Fast-forward local `main`: `git fetch origin && git pull --ff-only origin main`.
   - Remove finished PR worktrees (`git worktree remove`) and rebase active worker branches onto
     updated `origin/main` (`git rebase origin/main`).

### Batch / release PRs (`chore: release 0.1.<N>`)

When multiple accepted chains should land together as a versioned batch/release (or share
dependencies):

1. W5 stacks the accepted chains linearly onto `origin/main` (one clean commit per accepted chain,
   verifying every commit endpoint compiles with `cargo check --workspace --all-targets`) and
   appends the release commit `chore: release 0.1.<N>` updating:
   - `[workspace.package].version` in root `Cargo.toml`;
   - `version` in `web/app/package.json` (and `web/bun.lock` if updated);
   - `Cargo.lock` via `cargo update -w --offline`.
2. W5 runs static checks (`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `wasm32` check, web `fmt:check`/`lint`/`typecheck`/`test:unit`) on the stacked tip and pushes
   the release PR branch.
3. COORD audits the complete PR commit range (authors, zero `Co-Authored-By:` trailers, hygiene,
   LOC `<= 400`), opens the release PR targeting `main`, and waits for the
   tree-addressed web-build producer check on the PR head to be green before
   enqueueing it via `gh pr merge <pr> --auto --rebase`. An in-progress producer
   is not sufficient; this ordering rule does not change repository rulesets or
   required checks.
4. GitHub Merge Queue runs the full 29-job suite and lands the release atomically onto `main`.
   Optional local machine verification (`$COORD_DIR/bin/batch/` against local Nix oracles) may be
   used for heavy local profiling or bisecting, but never replaces the mandatory Merge Queue gate.

### Oracle and toolchain image updates

- Updating pinned OrcaSlicer v2.5.0 oracle patches/recipes triggers `.github/workflows/oracles-build.yml`
  on `main`, publishing a new content-addressed `ghcr.io/projectpandar/ares/ci-oracles:<tag>`
  digest and pruning stale tags. Update `ORACLE_IMAGE_DIGEST` in `scripts/ci/oracle-image.env`
  via a normal PR once published.
- Updating runner toolchain pins triggers `.github/workflows/toolchain-build.yml` on `main`.

## Standing rules

- **Git.**
  - Never push directly to `main` and never force-push `main`. `--force-with-lease` only on a
    worker's own branch.
  - Never `--no-verify` and never bypass `hk` hooks or Merge Queue required checks.
  - Never include `Co-Authored-By:` trailers in commit messages.
  - Don't amend accepted commits without asking.
  - Never `rm -rf` a variable path.
- **Secrets.**
  - Never read or print tokens or credentials.
  - Never commit user-supplied files or anything derived from them; use synthetic fixtures.
- **Tests.**
  - Public-API E2E with meaningful behaviour names.
  - No committed Native outputs.
  - Output must not depend on core count.
  - Upstream undefined behaviour is not ported.
  - Load canonical reference flags for every local parity run; skipped required Native comparisons
    never count as PASS.
  - Zero test retries, zero relaxed budgets/timeouts, zero change-detector tests (see `AGENTS.md`).
- **Native instrumentation.** Use only `printf` instrumentation compiled into a local upstream build.
  Do not use a debugger or debugger scripts.
- **Approval.** Destructive or outward-facing actions need the user's approval.
- **Reporting.** Report to the user briefly, in Chinese.
