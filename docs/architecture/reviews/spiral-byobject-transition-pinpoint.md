# PINPOINT — by-object inter-object travel Z 与次对象入场 hop 序列（A 下游子类）

- 分支：`spiral-byobject-transition`（自 W1 冻结 tip `a1a72362a`）
- 基线：SPEC `097db3559` → RED `4f52aa81b` → GREEN-1 `1440528a9`
- 冻结输入：`two-object-by-object-apart.3mf`（sha `2dbf797d…7342`）
- 状态：**只读定位完成，HOLD 待实现授权**（M73 位置类 B 归 W4，本文档不涉及）

## 1. 观测差异（GREEN-1 后残余的 A 下游子类）

Native（官方与 seeded 字节一致；gdb 复现于 seeded，`ORCA_REPLAY_SEED=0`）：

```
3414: G1 Z6.6 F42000
3415: M204 S10000
3416: G1 Z7 F42000
3417: ; stop printing object, unique label id: 45
3418: M625
3419: G1 X50 Y40 Z6.2 F42000
3420: M104 S220 ; set nozzle temperature
3421: M106 S0
3422: ; CHANGE_LAYER  → Z_HEIGHT 0.2 → 3425: G1 Z.2 F42000
...
3440: M204 S6000
3441: G1 Z.6                    ← cube 首路径入场 hop-up（0.2+0.4）
3442: ; start printing object, unique label id: 56
3443: M624 AgAAAAAAAAA=
3445(附近): G1 Z.2             ← 入场 hop-down
```

Ares（`transition.rs` 当前实现）：

```
3414: G1 Z6.6 F42000        ✓
3415: M204 S10000           ✓
3416: ; stop printing object, unique label id: 45   ← 标签先于第二次 lift（序反）
3417: M625
3418: G1 Z7 F42000          （lift 位置在标签后）
3419: G1 X50 Y40 Z7 F42000  ← dest Z 用了提升值 7，应为名义 Z 6.2
...
3440: M204 S6000
（3441 G1 Z.6 / G1 Z.2 整对缺失）                    ← lifted 记账错误所致
```

三个微差异：(i) 第二次 lift 与 stop 标签的顺序；(ii) 原点 travel 的 dest Z（提升值 vs 名义值）；(iii) 次对象首路径入场 hop 对缺失。

## 2. 上游机制（全部 gdb 验证 + 源码引用）

切换块 `_do_export`（`GCode.cpp:3298-3330`，`finished_objects > 0`）按序：

1. **(a) `retract()`**（`GCode.cpp:3319`；`GCode::retract` `:7652+`）：E 已回抽 → 无 E 行；
   lift 部分立即执行 → `G1 Z6.6 F42000`（nominal 6.2 + z_hop 0.4，`m_lifted=0.4`）。
2. **(b) `travel_to_z(m_max_layer_z + z_hop)`**（`:3321`；`GCodeWriter.cpp:817-827`）：
   目标 6.6 == 当前物理 Z → 非力分支 `will_move_z` 假 → **reduce-lift 记账**（nominal 6.2，
   `m_lifted -= 0.4 → 0`），**不发射任何行**。gdb 证实：band (5,8.5) 内唯一非
   change_layer 的 `travel_to_z` 调用即此处（z=6.6，caller `_do_export`）。
3. **(c) `travel_to(Point(0,0), erNone, "move to origin position for next object")`**
   （`:3327` → `GCode::travel_to` `:7350`），`set_origin(shift)` 后 (0,0) = 板坐标
   (50,40)，且 z=DBL_MAX：
   - 加速度选择（`:7396-7431`）→ `M204 S10000`；
   - 回抽判定（`:7432-7462`）：长 travel 触发 `retract()` → E 无行，但
     `needs_lift`（`GCode.cpp:7680+`，与 E 状态无关）→ **第二次立即 lift
     6.6+0.4=7.0 → `G1 Z7 F42000`，`m_lifted=0.4`**。gdb 入口态证实：
     `travel_to_xyz` 入口 `m_pos=(117.647,151.704,7.000)`、`m_lifted=0.4`、
     `m_to_lift=0`、`force_z=0`、dest=(50,40,6.2)；
   - 标签冲刷 `add_object_change_labels`（`:7465-7467`）→ 上一对象末层设置的
     `object_end_str`（`:5483-5486`）= `; stop printing object, unique label id: 45` +
     `M625`；
   - 单段 travel 分支（`:7479-7483`）：dest3d.z = `z == DBL_MAX ? m_nominal_z : z`
     = **6.2**（前一对象名义层 Z，`change_layer` 缓存）→ `travel_to_xyz(dest3d,
     comment, m_need_change_layer_lift_z=false)`（`GCodeWriter.cpp:685+`）：
     `m_to_lift=0` 跳过 branch-1；`will_move_z(6.2)` 真 → **branch-3 实际 XYZ 移动并
     取消 lift** → `G1 X50 Y40 Z6.2 F42000`，**`m_lifted=0`**，`m_pos=(50,40,6.2)`。
4. `printing_by_object_gcode` 模板 + 首层温度（`:3330-3341`）→ `M104 S220` / `M106 S0`。
5. 次对象 `process_layers` → `change_layer(0.2)` → `G1 Z.2`；首路径 `_extrude`
   （`:6378`）→ `travel_to(首点)`：长 travel 回抽判定再次 lift →
   **`G1 Z.6`**（0.2+0.4）→ 标签（`M624` 对）→ travel → unlift **`G1 Z.2`** → 复抽。
   **该入场 hop 对依赖切换 travel 结束后 `m_lifted=0`**。

## 3. Ares 偏差根因（`layers/transition.rs`）

1. `motion::tower::writer_z(lift_z, …)` 对应 (b) ✓（Z6.6）；但 (a) 的首次 lift 与
   (c) 内第二次 lift 在 Ares 由手动 `hop_z = lift_z + z_hop` 的 `writer_z` 近似
   （`transition.rs:85-90`），且发生在 `append_object_start`（标签）**之后** ——
   上游顺序是 lift（retract 判定内）→ 标签 → 移动。
2. 原点 travel 手写 `G1 X{} Y{} Z{hop_z}`（`transition.rs:92-101`），dest Z 用提升值
   7.0；上游是 `travel_to_xyz` branch-3：dest Z = **名义 Z**（前一对象
   `m_nominal_z`），且移动即消 lift。
3. `state.lifted = true; lifted_amount = z_hop`（`transition.rs:110-113`）与上游
   相反：branch-3 后 `m_lifted=0`，次对象首路径 travel 自带 `G1 Z.6`/`G1 Z.2`
   入场 hop 对 —— Ares 因 lifted 预置而吞掉了它。

## 4. GREEN-2 实现边界（待授权）

- 目标文件：`crates/ares-core/src/project_slice/gcode_emit/layers/transition.rs`（及
  若共享 travel 帮助器需要，最小触及 `motion` 模块）。
- 改动：(1) 原点 travel 的 dest Z 改为前一对象名义 Z 并走 branch-3 语义（移动消
  lift、结束态 `lifted=false`）；(2) 第二次 lift（→ `max_layer_z + 2×z_hop` 物理，
  等效 lift 后 `lifted=z_hop` 的中间态）先于标签冲刷发射；(3) 删除
  `lifted=true/lifted_amount` 预置，令次对象首路径恢复自身 hop 对。
- 不变量：单对象 by-layer 场景无此切换块（`finished_objects==0`），零影响；
  回归集 33 测试必须全绿（M73 类 B 三处仍红属预期，witness 2/2 由 W4 的 B 修复后
  才可达）。
- 验收：冻结输入 diff 剩余仅 3 处 M73/E 交换（line 1779/2465/3259）与生成时间戳行。

## 5. 证据

- gdb 追踪：`green-check/spiral-byobject-transition-byobject-apart-dHxWE9/gdb-z7/`
  （`trace.cmd`/`trace.log` = travel_to_z 断点带；`trace2.cmd`/`trace2.log` =
  travel_to_xyz 入口态 + 标签冲刷；seeded 复现 `ORCA_REPLAY_SEED=0`）。
- 双侧字节：同目录 `../native.gcode` vs `../ares.api.gcode`（首差 byte 68897）。
