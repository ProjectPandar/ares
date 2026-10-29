# CI Tier 1 修复 / 提速 / skipped 处理方案（worker4，SPEC v2 待批准）

范围：`.github/workflows/`。不改断言语义、不新增 `#[ignore]`、不提交
Native 产物。runner 为 org 级 ephemeral VM
（`shaula-project-pandar-pve-*`，每 job 不同），任何 host 路径（本机
/tmp、/nix/store）均不可用——Native 一律在 CI 内用仓库内 Nix 表达式
现场构建或下载官方 AppImage。

## 当前 artifact 生命周期（2026-10-29 用户裁决）

Tier 1、web E2E 和 Pages 不上传 GitHub artifacts。截图、trace、G-code 和
Native 比较文件只写入 runner workspace；诊断文本写入 job logs。runner
workspace 在 job 结束后不保证可用，不能承诺下载截图或保留产物。

Pages deploy 在同一个 job 中 checkout、setup、`build:pages` 后部署；不从
build job 下载文件，但仍用 `needs: build` 等待检查成功。已移除仅供证据
留存的上传步骤，以及没有上传方之后多余的全局 artifact cleanup job。

未来如果确实需要跨 job 文件：生产方上传，消费方下载并消费，然后必须在
**消费 job 内立即删除** artifact（例如 `geekyeggo/delete-artifact` 或
GitHub REST API）；不能以短 retention 或末尾独立 cleanup job 替代。

## 1. 失败清单与根因（不变）

- run 38145688037：`embedded_script_whole_file` 2 测试读
  `repo_root()/OrcaSlicer/resources/profiles` 失败（os error 2）。
  native job 无 OrcaSlicer checkout → 环境缺资源，非回归；fail-fast
  掩盖其余 7712 测试。
- **第二处环境缺失（step1 run 38147360608 发现）**：wasm job 的
  `project-slice.spec.mjs` 设置 `ARES_REQUIRE_ORCA=1` 并透传
  `ARES_KSR_INITIALIZED_{BIN,NATIVE_BIN,RESOURCES_DIR}`——CI 未提供 →
  `browser_output_matches_initialized_orca_242_ordered_bytes` 红。
  结论：wasm job 同样需要 ksr-initialized Native（复用
  native-oracle-build 输出的 hestia substitute，不需重复构建）。
- 历史 3s 失败（38142910641 等）：ubuntu-latest billing，切
  self-hosted 已解决。

**已执行（step1，批准部分）**：分支 `worker4/ci-tier1-fix` @ c6b36592d，
native job 加 OrcaSlicer sparse checkout（`resources/profiles` +
`src/libslic3r`，pin v2.4.2 `8500fcdcc`）+ `nextest --no-fail-fast`；
CI run 38147360608 验证中。

## 2. Job 拓扑（v2）

```
native   (self-hosted) : checkout + OrcaSlicer sparse + rust-cache
                         + nextest --workspace --no-fail-fast + clippy
format   (self-hosted) : 不变
web      (self-hosted) : 原 wasm job 改名/重构，见 §2a
native-oracle-build (matrix, self-hosted): Nix + hestia，
                  每 job 一个变体，构建上传
parity-seeded   (self-hosted): hestia substitute + seeded env
                  + nextest -p ares-cli orca_parity
parity-official (self-hosted): hestia substitute + AppImage 解包
                  + official env + nextest -p ares-cli orca_parity
```

### 2a. `web` job（原 wasm job 重构，用户指令）

保留的 Tier1 WASM 检查（不变）：`cargo check -p ares-core
--target wasm32`、`cargo check -p ares-wasm --target wasm32`、
`node scripts/gen-configdef.mjs --check` +
`cargo run -p ares-core --release --example print_config_def` +
`scripts/verify-configdef.mjs`、wasm-bindgen 产物构建（`cargo build
-p ares-wasm --target wasm32` + wasm-bindgen，改用
`taiki-e/install-action@wasm-bindgen`）。

移除：旧 `crates/ares-wasm/tests/browser` 全套（npm ci/test、
node --check ×3、export audit 步骤——export audit 逻辑可移入 web
lint/构建脚本，见 §6 待确认）。

替换为 web/ 工作区：
```
npm ci --prefix web           （setup-node cache: web/package-lock.json）
npm run lint                  （oxlint packages app e2e）
npm run typecheck             （tsc）
npm run test:e2e              （cargo nextest ares-wasm 三 test +
                               build:wasm/build + playwright test）
```
Playwright 浏览器缓存：`actions/cache@v4` `~/.cache/ms-playwright`，
key 含 `web/package-lock.json` hash（内含 @playwright/test 版本）。
config 含 chromium(桌面)+webkit(iphone/pixel) 三 project → 安装
`playwright install --with-deps chromium webkit`。

**web job oracle env（对齐 `web/e2e/oracleEnv.ts` + 各 spec 调用，
CI 未设即 skip 的那批，必须全部注入使其真跑）**：

| env | 来源 | 消费的 spec |
|---|---|---|
| `ORCA_APPDIR` | 官方 v2.4.2 AppImage 解包 squashfs-root | colorChange / slice / presetSlice / profileSlice / configRoundTrip |
| `ARES_PROCGCODE_BIN` | `tools/chain-probe/procgcode.nix` | colorChange 等 |
| `ARES_PROCGCODE_DYNAMICS_BIN` | procgcode-dynamics.nix | dynamics 对比 |
| `ARES_MARKER_TIME_BIN` | `tools/chain-probe/marker-time.nix` | markerTimeFormat/Native |
| `ARES_MARKER_TIME_STABLE_BIN` | `marker-time-stable-options.nix` | toolPosition |
| `ARES_PRINT_STATISTICS_BIN` | `print-statistics.nix` | estimation/summary |
| `ARES_KSR_INITIALIZED_*` | ksr-initialized.nix（原 wasm browser 测试需要；web e2e 是否还用待核 —— project-slice.spec 语义并入哪支 spec 需 w2D 确认） | 旧 browser 路线 |
| `ORCA_SLICER_DIR` | checkout 的 `OrcaSlicer/`（web job 已有 sparse checkout） | profiles/presetImport |
| `ORCA_REPLAY_SEED=0` | 固定 | estimation |

矩阵 build job 需新增 `marker-time` + `marker-time-stable-options`
两个变体（合计 8 个）。web job 的 oracle 全走 hestia substitute +
AppImage 解包步骤（与 parity-official 共用同一解包脚本
`scripts/ci/fetch-orca-appimage.sh`，入仓）。

### 2b. 旧 `crates/ares-wasm/tests/browser/` 处置（决策点 F）

覆盖对照：

| 旧 spec（project-slice.spec.mjs） | web/e2e 对应 |
|---|---|
| browser project route emits complete finite KSR G-code（KSR initialized Native 对比） | 无直接对应——web 有 profileSlice/presetSlice（official AppImage oracle），但 **KSR initialized 路线的 browser 对比仅此一处**。删除会丢失该覆盖，除非 w2D 已在 web/ 重写等价 spec |
| bindings expose no internal checkpoint hooks | 无（旧 export audit 的 spec 版） |
| repeated project slicing identical bytes | 无直接对应 |

**裁决（v4）**：旧 `crates/ares-wasm/tests/browser` 不在 CI 中保留；
其独有的 KSR initialized 浏览器对比由 **w2D 移植到 web/e2e**，移植
合入后 w2D 提交删除旧目录（no legacy）。本分支不删该目录，仅将
web job 切到 web/ 工作区；删除与移植是 w2D 提交的依赖项。

- 三个 job 并行；`native` 不注入任何 `ARES_*` parity env（普通 workspace
  标准环境，SKIPPED 仍按 stderr 显式记录）。
- parity jobs `env: ARES_REQUIRE_ORCA=1`：任何 gate env 缺失 → 显式
  panic 红（B2 语义），绝不静默。
- parity job runner label：ephemeral VM 即 self-hosted（同 label 会与
  native 抢 runner？ephemeral 每 job 独立 VM，无争抢问题）。
  **待确认**：self-hosted VM 是否有 Nix daemon/磁盘余量跑 Orca 构建；
  若无，parity jobs 用 `ubuntu-latest`（billing 问题需账号先恢复；
  fallback 记录）。**决策点 E**。

## 3. Native 构建与缓存

### 3.1 Nix 安装 + hestia 缓存（用户指定）

```yaml
permissions:
  contents: read
  actions: read        # hestia 预检 evicted cache entries
steps:
  - uses: NixOS/nix-installer-action@main
  - uses: Mic92/hestia@v3
```

- 另在默认分支新增每日 GC workflow：复制
  `Mic92/hestia` `.github/workflows/gc.yml`，permissions `actions: write`
  （删除过期 cache entries 需要）。
- 约束：
  - **hestia cache token ~6h**：单次 job 内全部构建必须 6h 内完成。
    **裁决（v3）**：`native-oracle-build` 拆为**矩阵 job**，每 job
    一个变体（seed-replay / procgcode / procgcode-stable /
    procgcode-dynamics / print-statistics / ksr-initialized +
    gdb），各自构建并上传 hestia；parity-seeded、parity-official、
    wasm 三 job 只 substitute+test。每 build job 内 `--cores 4
    --max-jobs 1`（7GB 内存防 OOM），job 尾 `nix-collect-garbage`
    回收构建树（矩阵并行下仅防单 job 超盘）。单变体估算
    40–60min（冷）→ 远低于 6h；矩阵并行不受 token 约束。
    gdb 体量小，并入任一 job 或单独小变体。
  - **10GB 配额与 rust-cache 共享**：orca-slicer 单变体输出
    ~350–500MB，5 变体 ≈ 2GB + 其 nixpkgs deps 闭包。启用
    `upstream-cache-filter: true`（hestia input）只存非
    cache.nixos.org 路径——deps 直接从上游 substitute，配额压力
    降到 <1GB。GC workflow 兜底。
  - **PR 语义**：默认分支缓存对 PR 只读；PR 写自己 scope——与本
    workflow 无冲突。

### 3.2 Native 产物清单（对照 `/tmp/ares-parity-control-set-v2.sh`）

| env | 构建来源 | 模式 |
|---|---|---|
| `ARES_ORCA_BIN` + `ORCA_REPLAY_*` | `scripts/orca-replay/build.nix`（seed-only.patch，`ORCA_REPLAY_SOURCE`=上游 checkout、`ORCA_REPLAY_NIXPKGS`=pinned nixpkgs） | seeded |
| `ARES_ORCA_BIN`/`ARES_ORCA_NATIVE_BIN`/`ARES_ORCA_RESOURCES_DIR` | 官方 OrcaSlicer v2.4.2 AppImage 下载 + `--appimage-extract` → `squashfs-root`（等价本机 `/tmp/orca-official-242`） | official |
| `ARES_PROCGCODE_BIN` | `tools/chain-probe/procgcode.nix` + procgcode.patch | 两 job 共用 |
| `ARES_PROCGCODE_STABLE_BIN` | `tools/chain-probe/procgcode-stable-options.nix` | 共用 |
| `ARES_PROCGCODE_DYNAMICS_BIN` | `tools/chain-probe/procgcode-dynamics.nix` | 共用 |
| `ARES_PRINT_STATISTICS_BIN` | `tools/chain-probe/print-statistics.nix`（src 为 seed-replay source store path → CI 内改为 `builtins.path` 包装变量，需小改 nix 或提供 source 目录） | 共用 |
| `ARES_KSR_INITIALIZED_{BIN,NATIVE_BIN,RESOURCES_DIR}` | `tools/orca-probes/ksr-initialized.nix`（历史初始化源，需 `ORCA_KSR_INITIALIZED_SOURCE`——该源的生成方式需确认，见 §6 风险） | 共用 |
| `ARES_GDB_BIN` | `nix build nixpkgs#gdb` 或直接 nix profile | 共用 |

### 3.3 pinned nixpkgs

本机 `.nix` 均用 store path `/nix/store/9qzkr0n…-source`
（`lib.version = 26.11pre-git`）。store path 无 rev 元数据；CI 中改
用 `pkgs.fetchFromGitHub`/`fetchTarball` pin **nixpkgs 具体 commit**。
不同 rev → orca-slicer 依赖版本可能漂移，seeded 模式逐字节 parity 有
漂移风险。**需要确定等价 rev**：从本机 `nixos-unstable` channel
snapshot 反查（spec 批准后我定位 rev 写入 workflow pin）。

### 3.4 env 注入

parity job 步骤末尾把 control-set-v2 refs 全集写入 step env：
26 个 `ARES_PARITY_*_REFERENCE=1` + `ARES_PARITY_{SWEEP,CLUSTER,
OPTIONS}`（后三个按 seeded/official 各取）+ `ARES_PARITY_ARTIFACT_ROOT=
$RUNNER_TEMP/parity-artifacts`（绝对路径、repo 外）+ `ARES_REQUIRE_ORCA=1`
+ `ARES_ROLE_ORACLE_LIVE=1` + seeded 的 `ARES_PARITY_SEEDED_REFERENCE=1`
/`ORCA_REPLAY_SEED=0`/`ORCA_REPLAY_CPU=<n>`。
CPU pin：本机用 `CARGO_TARGET_X86_64_..._RUNNER` + 仓外脚本；CI 改为
仓内脚本 `scripts/ci/pin-runner.sh`（移植 `/tmp/ares-pin-runner.sh`，
5 行 taskset 逻辑），或 parity run 用 `--test-threads N` + 不 pin
（seeded 连续流需单 CPU——**必须 pin**，故提交该脚本入仓）。
AppImage 在 CI 需 `squashfs`/FUSE：ubuntu 镜像自带 fuse，用
`--appimage-extract` 无需挂载。

## 4. 提速（ephemeral runner 版）

| 措施 | 位置 | 效果 |
|---|---|---|
| `Swatinem/rust-cache@v2`（GHA cache，ephemeral 可读写） | native + wasm job | 消除全量冷编译；warm run 预计 native 14min→3–5min |
| hestia | parity jobs | 见 §3 |
| `taiki-e/install-action@wasm-bindgen`（pin 0.2.121）替 `cargo install` | wasm job | 省编译 ~1–3min |
| `actions/cache@v4` 缓存 `~/.cache/ms-playwright`（key 含 lock hash） | wasm job | 省浏览器下载 ~1min |
| `--no-fail-fast` | native | 完整性（非提速） |

估算对比（待 CI 实测回填）：native before 14m11s（截断），warm
cache 预计 3–5min；parity-seeded/official 首次 60–90min（Orca 构建）
→ 缓存命中后仅 Rust 编译+测试 ~5–15min；wasm 预计 16→12min。

## 5. `#[ignore]` 5 测试处置（A：删除或改真断言）

| 测试 | 性质 | 处置 |
|---|---|---|
| `ares-core` `gt_oracle_tests::toolchange_tail_delays_match_gt_oracle` (:60) | 断言 ares 5.8204 vs GT 5.6211 的**已知发散**；且注释自述 oracle SEGFAULT 部分流不可信 | **删除**（发散探针，非 gate；残差事实移入模块注释） |
| `gt_oracle_tests` :243 `prefix probe` | 打印探针 | **删除**（连同 println 探针体） |
| `gt_oracle_tests` :284 `ladder probe` | 打印探针 | **删除** |
| `ares-cli` `ksr_fdmtest_v4::dump_elegoo_depositions` (:123) | ARES_ELEGOO_DUMP 诊断 dump | **删除** |
| `ares-core` `no_unapproved_dynamic_values::print_current_dynamic_value_baseline` (:78) | print-only 迁移辅助，输出已固化在 `scripts/dynamic_value_baseline.txt` | **删除** |

删除后 nextest "skipped" 应为 0（仅 env-gated 的 stderr SKIPPED 在
parity jobs 全 env 下清零；native job 仍记录 SKIPPED 属预期）。

## 6. 风险 / 待确认

- E：parity job runner（self-hosted ephemeral 有 Nix? vs ubuntu-latest
  billing）。
- `ARES_KSR_INITIALIZED_*` 源（`ORCA_KSR_INITIALIZED_SOURCE`）的生成
  方式待查 `tools/orca-probes/README.md`；若是历史独有 patch 源，
  需把其 source 构造脚本一并入仓。
- `print-statistics.nix` 硬编码本机 store path → 需小改为 env 传入
  （同 procgcode.nix 模式）。
- nixpkgs pin rev 待定（§3.3）。
- Orca 冷构建逼近 6h token 上限风险 → build job 拆分缓解（§3.1）。
- 10GB 配额：5 变体 ~2GB + drv closures；`upstream-cache-filter` +
  daily GC 控制。

## 附录 B：合并 oracle 构建方案（后续任务，SPEC 待审）

背景（已调查）：上游最新 release 仍为 v2.4.2（当前 pin
`8500fcdcc`）；main@cab7a7d75 的 GCodeProcessor.cpp:7750-7792 只重
映射 `TimeBlock.move_id`，`OptionsZCorrector::m_move_id`
（GCodeProcessor.hpp:1042-1075）仍未重映射 —— stale-index bug 依
旧，pin 不变。

### B.1 合并目标

一个构建 = 上游 8500fcdcc + 顺序应用：
1. `scripts/orca-replay/seed-only.patch`（FuzzySkin/GCode seeded RNG）
2. `scripts/orca-replay/concentric-initialization.patch`
3. `scripts/orca-replay/object-id-initialization.patch`（Print.hpp m_id）
4. `tools/chain-probe/procgcode.patch`（`--process-gcode`/`--dump-moves`）
5. `tools/chain-probe/procgcode-stable-options.patch`（OptionsZCorrector m_move_id remap 修正）
6. `tools/chain-probe/procgcode-dynamics.patch`（`--dump-move-dynamics` + travel accel flavor 对齐 + m_pressure_advance 初始化）
7. `tools/chain-probe/marker-time.patch`（`--dump-marker-time`，依赖 marker-time.nix 的 postPatch：cp marker-time-oracle.hpp + kernel.hpp）
8. `tools/chain-probe/print-statistics.patch`（`plate_N.print_statistics.json`）

新增 `scripts/ci/merged-oracle.patch`（或 `tools/chain-probe/merged.nix`
串列 patches + postPatch）。patches 全部 apply 于同一 src，文件
级冲突仅 OrcaSlicer.cpp（6213/7533/7566/7587 不同 hunks）与
GCodeProcessor.cpp（2445/2524/6006 不同 hunks），顺序 apply
预期无冲突；若冲突则生成单个 merged patch 提交。

`ksr-initialized` 保持独立（FuzzySkin RNG 取 ORCA_REPLAY_SEED 的
ksr-initialized-source.patch 配方不同）；official = 未改 AppImage。

### B.2 合并后 env 映射（消费方不变，值改为同一路径）

`ORCA_REPLAY_BIN`、`ARES_PROCGCODE_BIN`、`ARES_PROCGCODE_STABLE_BIN`、
`ARES_PROCGCODE_DYNAMICS_BIN`、`ARES_MARKER_TIME_BIN`、
`ARES_MARKER_TIME_STABLE_BIN`、`ARES_PRINT_STATISTICS_BIN` →
`$MERGED/bin/orca-slicer`；`ARES_ORCA_{NATIVE_BIN,RESOURCES_DIR}`（seeded
模式）→ `$MERGED/bin/.orca-slicer-wrapped` / `$MERGED/share/OrcaSlicer`。

矩阵 build job 从 8 变体收敛为：`merged`、`ksr-initialized`、`gdb`。

### B.3 验收（逐字节/逐测试相同）

- 同一 main hash 上用原 8 构建 和 merged oracle 各跑一次 seeded
  四段 full（排队，W5→worker3→worker1→worker2 之后）：FAIL 集合
  必须相等（已知基线 print_estimated_stats×2）。
- SHA 对照表覆盖 SAFC/concentric fixture 与 seed-replay 原切片；
  任何不等即停下报告，不调整测试。
- procgcode / procgcode-stable / dynamics / marker-time /
  marker-time-stable / print-statistics / seed-replay 的所有
  原测试输出 SHA 与原独立构建**逐一相同**（关键：stable-options 的
  m_move_id remap 与 dynamics 的 m_pressure_advance 初始化不得改变
  切片导出 G-code/M73 —— 两者仅在 `--process-gcode` 分支生效，
  理论无影响，但**任何差异即停下报告，不调整测试**）。
- 验证方式：merged 构建 vs 原 8 构建各跑同一 fixture 集合的
  `--slice`（输出 G-code SHA256）与全部 dump flag 输出 SHA256 对照
  表，写回本节。

### B.3a directed 结果（已执行，/tmp/worker4-ps-check/）

`print_estimated_stats` ×2 对三个 producer 各 seeded/official ×3：

| producer | seeded | official |
|---|---|---|
| `v9ydlg…`（控制脚本现状，旧 patch rev） | FAIL×3 | FAIL×3 |
| `v4zhqnbm…`（当前 patch rev） | PASS×3 | PASS×3 |
| `m20g9jgc…`（merged） | PASS×3 | PASS×3 |

首个差异：两测试在 `print_estimated_stats.rs:62` unwrap
`normal_prepare_time` → v9ydlg 的 `plate_1.print_statistics.json`
缺该字段（旧 patch 只有 `normal_time`）。**基线 print_estimated_stats
×2 失败 = 过期 oracle，不是 Ares 问题**。控制脚本 v2 的
`ARES_PRINT_STATISTICS_BIN` 仍指 v9ydlg，未改（待协调者确认换 v4zhqnbm
或 merged）。

### B.3b CI-环境修复（已执行）

首个 cold run (38161090181) 暴露三个 VM 环境问题，全部修复：

1. `nix-collect-garbage` 在 hestia drain 前删掉 oracle 输出 → 消费 job 缓存 miss。已删除该步骤。
2. nix 构建的 orca-slicer 在 VM 上 abort `locale::_S_create_c_locale`：
   `LC_ALL=C.UTF-8` + `LOCALE_ARCHIVE`（nixpkgs glibcLocales，env 前缀传递 NIX_PATH）。
3. `scripts/orca-parity.sh` 向 AppImage 注入 nixpkgs 运行时库，但 Ubuntu 24
   glibc 2.39 < nixpkgs glibc 2.42 → `GLIBC_ABI_GNU2_TLS not found`。修复：
   解析的库集中找到 nix glibc `ld-linux-x86-64.so.2` 时用其作 loader 启动
   AppImage（找不到则回退 host loader，wrapper_tests 语义不变）。
4. web job 的 `wasm-bindgen-cli` 须与 Cargo.lock 的 wasm-bindgen 版本严格
   一致；改为从 Cargo.lock 读版本后 `cargo install`。
5. `ORCA_REPLAY_BIN` 误指 merged-oracle（其 print-statistics patch 产生
   plate_1.print_statistics.json 副作用并改变 G-code）：seeded parity 挂
   25 个（cli_retained_exit、wipe_tags、vendor_profile_compatibility、
   owner_pair、adjacent_color、roles 等）。矩阵加入 seed-replay 变体，
   `ORCA_REPLAY_BIN=$ORACLE_seed_replay/…`。

### B.4 风险

- 补丁语义交叉：dynamics 的 travel-accel 对齐在 `process_file` 路径
  （非切片）；seed-only 的 GCode.cpp 改动与 print-statistics 的
  OrcaSlicer.cpp 无重叠。切片路径新增代码仅为惰性 flag 分支。
- 若 merged 切片 SHA ≠ 原 seed-replay 切片 SHA：停下报告（可能是
  concentric/object-id 初始化改变了默认填充行为 —— 这正是 worker6
  SAFC 调查的 UB 面，差异本身是有价值证据，不得调测试消化）。

## 附录 A：VM 探测结果（ci-probe run 38148271502，绿）

- runner：`shaula-project-pandar-pve-03b34a46`，Ubuntu kernel 6.8，x86_64
- **8 CPU / 7GB RAM + 3GB swap / 30G 盘（23G 可用）**
- `NixOS/nix-installer-action@main` 正常（Nix 2.35.2，KVM enable，
  trusted-users root runner）；`Mic92/hestia@v3`（v3.1.0）daemon 正常，
  substituter 于 127.0.0.1 可用，cache token 捕获成功
- `github:NixOS/nixpkgs/dc56dcbed57fa354b45a3a68a4eedd74663ae5ce`
  → narHash `sha256-dveqXShac4ZH9whSBeFyP8GY4iQU6zpMIkyCgPUdLOY=`，
  store path `/nix/store/9qzkr0n…-source` 与本机完全一致（校验通过）
- `nix build nixpkgs#hello` 成功（cache.nixos.org substitute 正常）
- 资源评估（v3 裁决已采纳）：矩阵 build job 每 job 一个变体，
  `--cores 4 --max-jobs 1` 防 OOM，job 尾 nix-collect-garbage

## 7. 实施顺序

1. ~~step1 sparse checkout + --no-fail-fast~~（已推 c6b36592d 验证中）
2. spec 批准 → `worker4/ci-tier1` 分支：workflow 重构 + hestia + gc.yml
   + `scripts/ci/pin-runner.sh` + print-statistics.nix 参数化 + 5 个
   `#[ignore]` 删除。
3. 推分支触发 CI：记录首跑（冷）与第二跑（缓存）各 job 耗时。
4. REVIEW-REQUEST 附 run 链接 + 前后耗时 + skipped 清零证据。

## Review correction: Native extraction ownership

- Canonical `/tmp/ares-parity-control-set-v2.sh` official mode selects
  `/tmp/orca-wrapper-inherit-cpu.sh`, which delegates to the arachne worktree's
  host-exec wrapper, using shared `/tmp/orca-official-242/squashfs-root`.
  Seeded mode selects `scripts/orca-seeded-parity.sh`, not this AppImage wrapper.
  These are not candidate-wrapper validation evidence.
- The candidate wrapper now has only direct host execution (no Nix loader in
  the resolved library set) or direct execution of a patched **job-local copy**.
  A Nix loader requires both `ORCA_PATCHELF` and `ORCA_JOB_APPDIR`; missing inputs
  fail explicitly. Invoking the Native via `ld.so` is forbidden because upstream
  `program_location()` would resolve the wrong resource directory.
- `ORCA_APPDIR` is read-only reference input. `ORCA_JOB_APPDIR` must be outside
  that extraction, unique to the job, and is populated under a lock before
  patching its binary. CI web/official jobs supply separate runner-temp paths.
  Local directed validation invokes the candidate wrapper with the shared input
  and a fresh external job destination; hashes and interpreters of input/copy
  are retained in the validation artifact root.
- `bash scripts/ci/check-orca-wrapper-isolation.sh` checks missing-patchelf
  rejection, exact argument forwarding to the copied binary, rejection of a
  shared destination, and byte-identical source preservation. It retains its
  synthetic command artifacts under `/tmp/orca-wrapper-isolation-*`.
