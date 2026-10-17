# SPEC(提案): tool-change 层边界 retract+wipe 时序（换刀层头顺序）

状态：SPEC-DRAFT（docs-only）。承接：wiping-pass residual discovery——
R1 基 + `flush_into_infill=1` 的 combined control 中，ares 完整切片但
归一化 diff=152（38 处 × 4 行块）：wipe 块（`G1 E-3` +
`;WIPE_START/;WIPE_END`）位置错误。wiping pass 本体已移植（双方
`; PURGING FINISHED` 各 76 处、内容一致），分歧仅为**层头顺序**。
首分歧 @ 归一化 N:4933。

## 1. 上游边界（源：`OrcaSlicer/src/libslic3r/GCode.cpp` process_layer
层头序列）

- `:4638-4642`：`before_layer_change_gcode`（R1 基中含
  `;BEFORE_LAYER_CHANGE / ;[layer_z] / G92 E0`）；
- `:4662`：`change_layer(print_z)`（`GCode.cpp:5685-5718`）——
  **`:5692-5697`**：`retract_when_changing_layer && will_move_z` 时
  `this->retract(false, false, lift_type)` **在此处**发出 retract+wipe
  （`GCode::retract :7652-7670`：`FILAMENT_CONFIG(wipe) && 有 wipe 路径`
  时先 `retract_for_toolchange`/`retract` 再 `m_wipe.wipe`）；
  `:5699` object change labels；Z 提升经 `m_need_change_layer_lift_z`
  **延迟**到换刀处理后（故 G-code 中 `G1 Z1.8` 在 PURGING 边界之后）；
- `:4673-4681`：`layer_change_gcode`；`:4688`：
  `;_SET_FAN_SPEED_CHANGING_LAYER`；
- 净序（native 实证，N.gcode:4925-4948）：`G92 E0` → **retract+wipe** →
  `;_SET_FAN_SPEED_CHANGING_LAYER` → 对象标签 → `; PURGING FINISHED`
  → `G1 Z1.8 …`。ares（A.gcode 同窗口）把同一 wipe 块推迟到
  `; PURGING FINISHED` 之后、`G1 Z1.8` 之前。

## 2. Rust 目标边界

`crates/ares-core/src/project_slice/gcode_emit/layers/tool_entry.rs`（及
其调用链中的层头/retract 排序）：把层界 retract+wipe（等价
`change_layer :5692-5697`）移回层头序列中 `before_layer_change` 之后、
`;_SET_FAN_SPEED_CHANGING_LAYER`/对象标签/purge 边界之前；Z 提升保持
现有延迟位置不变（native `G1 Z1.8` 在边界后，ares 此点已一致）。

## 3. 包含行为

- 仅换刀层边界（tool-change + `retract_when_changing_layer`）在
  flushing override（`flush_into_infill/objects/support` 产生 PURGING
  边界标签）存在时的 **wipe 块位置**；块内容（E 值、wipe 路径、F）
  不变——两侧已逐字节一致，纯排序修复；
- RED target：`fi-ctl` 输入（R1 基 combined skirt + `flush_into_infill=1`，
  即 discovery 用的 fi-ctl.3mf）native/ares 严格相等——先 FAIL；
  control：R1 基 combined 无 `flush_into_*`（即既有 r1c2 输入，已全片
  PASS）——证明分歧只来自 wipe override 的排序路径；
- failure inventory：①wipe 块晚于 purge 边界（当前态，N:4933）②修序
  后无 flush 路径回归（control 腿）③Z 提升位置被误动 ④wiping pass
  本体内容被误改（应只动排序）。

## 4. 延后行为

- 非 `retract_when_changing_layer` 触发的 retract 时序；
- spiral vase / timelapse 层头插入物；
- Wipe 路径计算本体（已移植，内容一致）；
- per-object skirt + tower + flush 的 wiping-pass 残余守卫（回原链，
  本片 GREEN 后重评）。

## 5. 测试先行（TDD）

- RED：fixture=fi-ctl.3mf（钉 sha）+ control=r1c2 输入（复用已批
  skirt ctl fixture 或钉副本）双腿 test-only commit；首分歧必须与
  N:4933 分析一致（wipe 块序，非内容差）；
- GREEN：§2 排序修复，target 转绿 + control 与 per_object_skirt 族
  无回归。
