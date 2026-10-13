# organic merge single-worker：RED/GREEN 证据记录

配套：`2026-10-14-organic-merge-single-worker-spec.md`（SPEC）、RED `65c5d8e8d`
（E2E + fixture）、GREEN `5c9f54b0d`（单 worker 化）。

## 机器相关性证据（scratch 插桩，未提交代码）

committed E2E 依赖运行时继承亲和宽度 ≥16 才能观察到分歧；机器相关性本身由
一次性插桩构建证明（`ARES_ORGANIC_FORCE_WORKERS` 直接改写 `workers()` 返回值，
两次切片均保持单核亲和，即纯粹比较 bucket 布局输入）：

- fixture：E2E 同款输入（30×10mm overhang，organic tree(manual)，
  `tree_support_branch_distance_organic=10`；merge 层 n 分布 8..86，共 45 层）。
- workers=1：`/tmp/isfix/m1/w1.gcode`，generator sha `ebf751d2d424b551`。
- forced workers=64：`/tmp/isfix/m1/w2.gcode`，sha `af4918add7e9b6ef`；
  与 workers=1 逐行 diff **46844 行**。
- 阈值序列（同 fixture）：w=3/4/8 输出与 w=1 逐字节相同
  （`t8.gcode` sha=`ebf751d2…`）；**w=16 起分歧**（`t16.gcode` sha=
  `23f133cd7015480a`），w=32/64 继续扩大差异。
- 机理与 SPEC §1 一致：布局翻转条件 `workers > (n+2)/4`；n≥62 的层在 w=16
  首次翻转；n<6 层的小翻转在全部测试输入上输出中性。

## committed E2E 的复现条件

`crates/ares-cli/tests/organic_merge_affinity.rs`：单核腿（继承集首个 CPU）
vs 继承完整亲和集腿，断言逐字节相等，附 `;TYPE:Support` 防空虚守卫。

- 继承宽度 ≥16 的主机：RED（当前实现两腿不同）→ GREEN（单 worker 化后相同）。
- 继承宽度 <16（含本 directed 分配的 2 核、CI tier1 self-hosted 裸
  `cargo nextest --workspace` job 的 runner 分配）：改动前后均 PASS
  （trivially green）——按裁定不为此改测试迁就 runner；RED 证据以上表
  插桩结果为准。
- parity-* job 使用 pin-runner（测试二进制单核），该 E2E 两腿同为单核，
  恒绿，属预期。

## 控制组（M2/M3，CPU7，official oracle env）

- 改动前基线：`tree_support` 31/31 PASS（`/tmp/isfix/m2-art`）。
- GREEN 后复跑：31/31 PASS，逐例一致（harness 内 Ares 本就单核，
  改动为 no-op，符合 SPEC §4 预测）。
