# worker4 handoff — compat17 状态 2026-10-11

## 合入状态（main = ea15e0451）

| 提交 | 内容 | 状态 |
|---|---|---|
| `c6e01dcf2` | -17 branches 1-2 裁决 + machine_switch relaxation | merged |
| `355cbaa49` | spec gap 文档（inherits→worker5、rel-e 冗余说明） | merged |
| `60f6ffd14` | branch1_switch_reject test-only 回归对照 | merged |
| `ea15e0451` | compat17 branches 2-3 六例覆盖 + `process-compatible-including-rel.json` | merged |

## 下一切片：`-16` upward gate（OrcaSlicer.cpp:2624-2647）

**等待依赖**：worker5 `failure.rs` 的 `-16`/`CLI_3MF_NEW_MACHINE_NOT_SUPPORTED`
(240) 族行 + 测试 runner `EarlyExit::NewMachineNotSupported`（spec
`docs/superpowers/specs/2026-10-11-cli-load-settings-compat17-branch3-upward16.md`
已批 `44bba3c04`）。

**语义要点**（spec Correction 节为准）：
- `-16` 仅在 machine+process 列表可达：branch 判定不兼容 && new_printer
  非空 && current(embedded) 非空 && 名异 && process 非空 && upward 向量
  非空且不含 `new_printer_system_name` → 240。
- machine-only：`:2625-2627` 先置 compatible，`-16` 不可达
  （`upward16_machine_only` accept 例已入 spec 计划）。
- upward 向量来源：worker1 `UpwardMachineInput::UpToDateFiles`
  (`project/upward_machine.rs:96-142` = `OrcaSlicer.cpp:2284-2304`)；
  需在 `profiles/entry.rs` 的 composite 路径接 `CliUpwardMachine`
  （core 侧，与 main.rs `FetchOriginal` 接线同形）。
- **此时 correction-wip 可恢复**：`-16`/upward 读 embedded
  (`upward_compatible_machine`, `:2301`) 与 merged 覆盖分叉 ——
  `worker4-compat17-correction-wip` @ `5c7308af0`（embedded raw 快照
  `embedded_settings_raw` + machine_switch 收紧 + branch3）届时
  先 RED 再 GREEN。

## 证据路径

- 四腿 full: `/tmp/worker4-compat17-four-legs/`（seeded/official ×
  base/cand，全绿 931-933/931-933）
- workspace 8306/8306: `/tmp/worker4-compat17-ws-gate3.log`
- test-only 两模式 22/22: `/tmp/worker4-b2b3-gates/{seeded,official}.log`
- 上游探针: `/tmp/compat17-probe/`（unflattened machine→native 239 /
  rel-e 移除仍 239 / switch machine+process→239）
- findings ledger: `/tmp/ares-matrix-findings.md`

## 环境注意事项

- repo `OrcaSlicer` symlink 现指向
  `/nix/store/hxvv0g1qlw9lwimj2km3ghxphrl9p4qf-orca-slicer-2.4.2-seed-replay-source`
  （原 `/tmp/ares-orca-replay-source.u3RYzz` 已被外部删除；
  `/tmp/worker4-compat17-base-e6` worktree 同改）
- base worktree `/tmp/worker4-compat17-base-e6` 停留在 `6e4c6c843`，
  下次 full 前须 `git checkout <base>`
- 重任务一律走 `/tmp/worker4-gated-workspace.sh` 或
  `/tmp/worker4-four-legs.sh`（inbox 行首 marker 让路监视）
- 通知正文绝不写 `EXCLUSIVE-FULL-START/END` 字样（web 误触发教训）
- `machine-different-name.json` = "Creality Ender-3 V3 0.4 nozzle"，
  embedded `printer_settings_id` = "Creality Ender-3 0.4 nozzle"，
  embedded PCP = ["Embedded Printer B","Embedded Printer A","Embedded Printer B"]
