# Multi-agent workflow and recovery

Handoff document for the coordinator (COORD). It describes how COORD and the workers run, and how
to bring everything back after a machine crash or reboot. It covers the process only. Current
assignments are kept in the coordinator ledger and the inbox, not here. Update this file whenever
a workflow rule changes.

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
| COORD | Claude Code | (session root) | Assigns, reviews, decides, merges, pushes main. Sole reviewer. |
| W1-W6 | pi, `magpie/zero-ai-coding-gpt/gpt-6.1-sol`, 256k | `worker1` … `worker6` | Fixed workers. W5 is the batch integrator. |
| WEB | Claude Code, `claude-opus-5-5`, effort high | `web` | Fixed web/UI worker with its own long-lived worktree. |
| W7+ | pi, same model | `worker7` … | Dynamic workers. Launched when work queues up, killed when their task is merged. |

Fixed workers (W0-W6, WEB) are never killed by the patrol. Only COORD signs `[COORD]`; workers
write `Wn -> COORD: …`.

## Locations

- The main checkout is the only worktree outside `$WORKTREES`. Never put worktrees in `/tmp`, which is
  wiped on reboot, or at the top level of the home directory.
- `$COORD_DIR` holds the coordinator state:
  - `inbox.log`, which workers append to with `bin/notify.sh <name> "<line>"`, and `mark`, the last line read.
  - The decision ledger, `b42-candidates.txt`. It records every RR, verdict, batch composition and user decision.
  - `TODO.md`, the work the user queued for after the current tasks. The patrol dispatches it when a worker frees up.
  - `briefs/`, the task briefs. `briefs/dynamic-worker-rules.md` is the onboarding brief for new workers.
  - `bin/`:
    - `notify.sh`;
    - `pin-runner.sh`, which pins each test to one CPU (`ARES_PIN_CPUS` / `ARES_PIN_OFFSET`);
    - `parity-control-set.sh`;
    - the Orca launchers.
  - `parity-250.env` and `parity-seeded-250.env`, the official and seeded Native comparison environments.
  - `gcroots/`, the Nix GC roots for every Native oracle build. A nightly GC deletes unrooted store paths.
  - `orca-cache/`, the shared persistent Native output cache.
  - `fulls/` and `evidence/`, the batch run roots.
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
   - Run `CronList`. If there is no patrol job, recreate it (see Patrol).
   - Arm the inbox Monitor (see Patrol).
   - Read the new inbox lines: `tail -n +$(( $(cat $COORD_DIR/mark) + 1 )) $COORD_DIR/inbox.log`.
   - Read the last ~50 lines of the ledger.
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
   - `/tmp` is gone, so any run that wrote there must be redone.
   - Native oracles: every store path in the env files must exist and be rooted under
     `$COORD_DIR/gcroots/`.
     - A Native launch that fails with exit 255 and empty output means a store path is missing.
     - Rebuild it with `nix build -o $COORD_DIR/gcroots/<name>`.
   - Git, in the main checkout:
     - Run `git worktree prune` and `git fetch origin`.
     - Check that local main equals origin/main.
   - Interrupted batch runs are not resumed partially. W5 reruns the batch FULL.
6. **Report** to the user what was restored and what has to be rerun.

## Patrol

There are two mechanisms. Both are session-only: they die with the coordinator session, so
recreate them after every restart or context reset.

- **Inbox Monitor.** It expires every 30 minutes; re-arm it each time.
  ```sh
  f=$COORD_DIR/inbox.log; n=$(wc -l < $f); while true; do m=$(wc -l < $f); if [ "$m" -gt "$n" ]; then tail -n +$((n+1)) $f | head -n $((m-n)) | awk '{print substr($0,1,600); fflush()}'; n=$m; elif [ "$m" -lt "$n" ]; then echo "INBOX SHRANK $n -> $m"; n=$m; fi; sleep 2; done
  ```
- **Patrol cron.** It runs about every 15 minutes, at off-minutes (e.g. `4,19,34,49 * * * *`). Each run:
  1. Handle every new inbox line (RRs, questions), then update `mark`. Re-arm the Monitor if it expired.
  2. Run `hc ls` and read each worker's pane.
     - Answer open dialogs.
     - Look for stalls, errors and rule violations.
     - Give an idle fixed worker its next task (check `TODO.md`), or tell the user.
  3. Kill dynamic workers that are idle with no open task, and record each one in the ledger.
  4. Move the current batch forward once its inputs are ready.

## Assigning work

- Every slice is a source-cited rewrite of an upstream `libslic3r` / `libvgcode` boundary (see
  `AGENTS.md`).
- Before dispatching:
  - Verify the premise against main.
  - Dedupe: grep the origin branches and the ledger for the same upstream lines.
- Write larger tasks as a brief in `$COORD_DIR/briefs/`. A brief covers:
  - the goal;
  - the upstream files, functions and lines;
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

1. Create a new worktree under `$WORKTREES` from the base COORD names, on the worker's own branch.
   - Link `OrcaSlicer` in the worktree root to the upstream checkout; the Orca probe builds read it there.
   - Give each worktree its own `CARGO_TARGET_DIR`, so concurrent builds do not lock or overwrite each other.
2. Commit in this order:
   1. the spec in `docs/architecture/`;
   2. a failing test;
   3. the fix.

   The upstream reference is the pinned Orca release.
3. Run these checks:
   - only the affected cases, in both modes. The env files alone omit some bindings (GDB, the
     `*_REFERENCE` flags); the complete set is the `environment` object that the last baseline FULL
     recorded for each mode, minus its target dir, home, PATH and CPU bindings (`ARES_PIN_CPUS` and the
     seeded `ORCA_REPLAY_CPU` both move to the worker's own CPUs). A skipped Native run or a
     missing producer never counts as a result;
   - `cargo fmt`;
   - `cargo clippy --workspace --all-targets -- -D warnings`;
   - wasm32 and web checks, when those parts are touched.
4. Push the branch. Send one RR line through `notify.sh` with:
   - branch, tip and base;
   - the commits;
   - the results;
   - the RR file path.

   Then wait.

## Review (COORD)

For each RR, check:
- **History.**
  - The three-dot diff `git diff main...tip` and every commit message.
  - Base ancestry: `git merge-base --is-ancestor main tip`, or the agreed base (a pending integration tip). A
    branch still based on the unsquashed commits of a landed batch is rebased first (step 6 of the batch rules).
- **Author and trailer.** The author is `5aaee9 <7685264+5aaee9@users.noreply.github.com>`, and each commit has exactly one trailer,
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Hygiene.** Grep the messages and the diff, docs included. Each commit must be clean on its own. Flag any of:
  - worker and batch labels, COORD, coordinator;
  - RR, RED, GREEN, FULL, STOP, gate, queued;
  - approved, review, pending, frozen, qualify/qualification, integrated, evidence;
  - CPU numbers, local paths;
  - `#[allow]` / `#[expect]`.
- **Code.** Compare it against the cited upstream lines. Reject any of:
  - comparator weakening, masks or fitted expectations;
  - Native goldens;
  - assertions that Ares differs from Native;
  - files over 400 lines.
- **Clippy.** It was run with `--all-targets`.

Record the verdict in the ledger, and tell the worker which batch the chain goes into.

## Batches and merging

1. A batch contains the chains accepted before it starts. Later ones go to the next batch.
2. W5:
   - stacks the chains linearly on main (the range-diff must be identical);
   - stacks a release commit last: `chore: release 0.1.<N>` for batch N. It sets `version` in `[workspace.package]`
     of the root `Cargo.toml` and in `web/app/package.json` (plus the `app` entry of `web/package-lock.json`), and
     updates `Cargo.lock` with `cargo update -w --offline`;
   - runs fmt, clippy --all-targets and the wasm32 check on the stacked tip, again on every restacked tip;
   - runs `cargo check --workspace --all-targets` at every chain's `<end>`, because each one becomes a commit on main.
     An `<end>` that does not build is reported to COORD, who reorders the chains or merges it with the chains it
     depends on into one commit;
   - pushes an integration ref.

   Before the FULL, W5 runs the non-parity gate once: the workspace tests outside the parity suite, plus the web
   static checks. Any failure there stops the batch.

   After COORD's go, W5 runs the dual FULL: official and seeded, pinned by `pin-runner.sh`, on the batch CPUs given in the ledger.
   The web full runs against the pinned Orca release resources, with the seeded replay CPU (`ORCA_REPLAY_CPU`) inside
   the browser phase's CPU range.
3. COORD compares the results with the previous baseline:
   - zero PASS→FAIL is required;
   - new tests must pass;
   - no rerun to change an outcome.

   Failures are attributed by chain (bisect).
   - A case whose Native output differs between repeated Native runs on the same input is Native nondeterminism. It
     is not counted as an Ares PASS→FAIL, but only with the repeated Native outputs as proof, and only by COORD's
     decision.
4. COORD verifies the exact tip over the whole range: fast-forward, trailers, hygiene and clippy. A hygiene fix to
   an accepted commit is a rewrite that needs the user's approval; the rewritten tip may differ from the tested tip
   only in the cleaned files, and the range-diff must show only the rewritten commits.
5. COORD lands the batch with one squash commit per chain, in stack order, plus the release commit:
   - No tag keeps the unsquashed commits; record the tested tip and each chain's `<end>` in the ledger.
   - A chain is one accepted RR line; a follow-up stacked right after its chain (such as a test fix) belongs to that
     chain's commit.
   - For each chain, take the stacked commit that ends it (`<end>`) and create
     `git commit-tree <end>^{tree} -p <previous squash commit>`, starting from main. Each tree equals the stacked
     tree at that point, so the last one equals the tested tip's tree (`git rev-parse <tip>^{tree}`); nothing is
     re-tested because the bytes are identical.
   - Message: a title summarising the chain's change, then one `- <subject>` line per original commit
     (`git log --reverse --format='- %s' <previous end>..<end>`), then the single trailer. No batch numbers or other
     internal labels; the author is the usual one.
   - Fast-forward main to the last commit and push main to origin (never force).
   - Pushing main deploys the web app to Cloudflare Pages.
   - Other branches only build and test.
6. Branches built on commits of a squashed batch move onto main with `git rebase --onto main <tested tip>`; the
   trees are identical, so this applies only the branch's own commits. A branch built on an earlier point of the
   stack uses that chain's squash commit and `<end>` instead.

## Batch script entry points

The batch scripts are installed outside the repository. Set `$BATCH_SCRIPTS` to their
installation directory and use one JSON configuration to name the repository, base and tip,
ordered chains, resource checkout and revision, environments, baselines, CPU allocations,
ports and artifact directories. Paths in that configuration must be absolute. Use complete
commit SHAs and fresh worktree paths under `$WORKTREES`.

Every entry point requires a distinct attempt name:

```sh
"$BATCH_SCRIPTS/restack" ranges.json --attempt source-plan --plan
"$BATCH_SCRIPTS/restack" ranges.json --attempt source-stack
```

`--plan` records intended work without compiling or executing tests. It does not prove that
ranges apply without conflicts. Remove it only when that stage is authorized. `collect`
reads existing results and does not take `--plan`.

| Entry point | Input and action |
|---|---|
| `restack` | Read ordered `{id, range: "START..END", title}` entries. Rebase each range onto the preceding result in a detached worktree with recorded-resolution reuse disabled. On conflict, save hunks, index stages and the stopped patch, abort and HOLD. Never skip or resolve a chain automatically. After every chain succeeds, write `LINEAR-CONFIG.json`. |
| `assemble` | Consume a linear configuration. Prove adjacent ancestry intervals cover every source commit exactly once, then replay them in order. Apply an optional final release commit last. |
| `check-ends` | Run `cargo check --workspace --all-targets` at every chain endpoint and the optional release endpoint. Retain every exit, including compiler failures. |
| `static` | Run formatting, workspace/all-target Clippy with warnings denied, and the core/WASM target check on the exact tip. |
| `incremental` | Select tests from actual inventories and run both base and candidate once in both parity modes. Count only entries whose filter status is `matches`. Never reuse an earlier base execution. |
| `prepare` | Build Native binaries and browser artifacts in separate worktrees. Retain dependency order within each lane, both parity inventories and artifact hashes. |
| `full` | Require exact-tip endpoint, static and paired incremental results plus the prepared manifest hash. Consume that manifest once and run official, seeded and browser lanes on disjoint CPU allocations without retries. |
| `collect` | Compare inventories and results with the baselines. Write summary JSON, complete nonPASS frames and a report. Partial coverage cannot qualify. |
| `land` | Require external approval bound to the tested tip. Create one local commit per chain with `git commit-tree`, followed by the optional release commit. Verify the final tree equals the tested tree. Print the push command without executing it. |

After `restack`, use its generated linear configuration for `assemble`. Change only destination
worktree and output paths when allocating the next stage. If `assemble` changes commit IDs,
use its exact resulting tip and endpoints for subsequent stages. Any source restack invalidates
previous endpoint, static, incremental and prepared results.

```sh
"$BATCH_SCRIPTS/assemble" linear.json --attempt assembly
"$BATCH_SCRIPTS/check-ends" execution.json --attempt endpoints
"$BATCH_SCRIPTS/static" execution.json --attempt static
"$BATCH_SCRIPTS/incremental" execution.json --attempt paired
"$BATCH_SCRIPTS/prepare" execution.json --attempt artifacts
"$BATCH_SCRIPTS/full" execution.json --attempt verification
"$BATCH_SCRIPTS/collect" execution.json --attempt report
"$BATCH_SCRIPTS/land" execution.json --attempt local-landing
```

These commands show the order, not blanket authorization to run every stage. Freeze the
prepared manifest path/hash and prerequisite result hashes in the execution configuration
before `full`. Keep canonical environments and SHA-contract suppliers byte-identical. Do not
wrap suppliers to set affinity. The outer runner supplies the replay CPU inside its lane.

### Preparation requirements

- Every worktree must contain an `OrcaSlicer` resource symlink to the configured pinned upstream
  checkout. Verify the link target and resource revision before building.
- Set `CARGO_TARGET_DIR` to `<worktree>/target` for every Cargo command in that worktree.
  Native and browser lanes have separate worktrees, not separate target directories elsewhere.
  Fixtures and browser producers discover sibling binaries and generated files under this root.
- Build Native binaries alongside the ordered browser preparation lane. The browser lane runs
  dependency installation, the release CLI build, Rust fixtures, the print-config-definition
  producer, WASM, app assets, the fixture binary and test inventory listing. Do not copy prior
  bindings or distribution files to make preparation pass.
- Read the expected version from `[workspace.package].version` in the prepared worktree's
  `Cargo.toml`. Require its `Ares <version>` marker in the target release WASM, the package
  `dist/ares_wasm_bg.wasm`, and every app `dist/assets/ares_wasm_bg*.wasm`. Scan all files in
  `web/packages/ares-wasm-js/dist` and `web/app/dist/assets` for other Ares generator versions.
  Missing required files, missing current markers or mixed versions HOLD before manifest creation.
  This scan covers generated outputs, not arbitrary historical strings throughout the repository.
- Set `web_environment.ARES_E2E_NIXPKGS` to the pinned nixpkgs source. Run Playwright through
  `web/e2e/webkit-env.sh`, including mobile WebKit projects, so the bundled browser receives its
  runtime libraries. Also provide the explicit SHA-bound Native suppliers in the web environment.
- Use `web/playwright.config.ts` from the tested repository. Override workers, retries, output
  directories and reporters through Playwright CLI flags. Do not generate an out-of-tree config.
  Keep the configured port consistent with the repository's baseURL and preview server URL.
- Before launching the browser lane and after its owned processes exit, run
  `ss -H -ltn 'sport = :<port>'`. The check passes only when the command succeeds and its output
  is empty. TIME_WAIT sockets are not listeners. Never kill unrelated processes to free a port.

### Outcomes and closeout

Supply the required Native probe environment variables. A skipped reference comparison is not a PASS.

Each attempt directory is exclusive. Keep failed attempts, commands, raw logs and actual exits.
A preparation failure is not a test result. Do not retry an executed test to replace its outcome.

Stop owned processes on an unapproved baseline PASS becoming nonPASS, a failing unpaired new
test, an abnormal terminal outcome, an unapproved Native signal or a CPU escape. Preserve the
stopping frame, signals, interrupted terminals and missing tests. A stopped run remains incomplete.

Report actual PASS-to-FAIL changes, new tests, removals, approved predecessor/successor pairs and
inventory counts. Keep complete browser errors, configuration streams and artifacts. An explicit
exception permits proceeding but does not turn an actual FAIL into PASS. Baseline carry-forward
or rename approval establishes neither byte equality nor causal attribution. Never weaken a
comparator or silently omit a test.

## Standing rules

- **Git.**
  - Never force-push main. `--force-with-lease` only on a worker's own branch.
  - Never `--no-verify`.
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
- **Approval.** Destructive or outward-facing actions need the user's approval.
- **Archives.** A helper that writes into shared archives or caches prints its argv in a dry run first; check the
  order of the output and input arguments before the real run.
- **Reporting.** Report to the user briefly, in Chinese.
