# SPEC: OuterAndInner 单对象 brim parity——support no-brim mask 推广至 combined 分支

状态：SPEC-REVIEW-REQUEST（docs-only）。承接已批 brim 链
（`da3c31320`，stack-15 排队）；本片为 roadmap OuterOnly/OuterAndInner
延后项的下一片。

## 1. 上游边界（file:function:lines）

`Brim.cpp:outer_inner_brim_area:421-725`：
- OuterAndInner 组装：`:459-460`
  `has_inner_brim = brim_type == btInnerOnly || btOuterAndInner || …`、
  `has_outer_brim = … btOuterOnly || btOuterAndInner …`（两者皆真）；
- outer 环 `:527-544`（同 OuterOnly 片已移植）；
- inner 环 `:546-556`：`innerExpoly = offset_ex(holes_rev,
  -brim_width - brim_offset)`，`outerExpoly = offset_ex(holes_rev,
  -brim_offset)`，`brim_area_object += intersection(diff(outer, inner),
  holes_rev)`；
- **support no-brim `:590-634`（本片核心）**：上游位于逐对象段，
  **与 brim_type 无关**——任意 brim 类型的 `no_brim_area` 都并入
  support 轮廓（stInnerNormal fill coverage / stInnerTree 轮廓），
  经 `:682-689` diff 掉。

## 2. ares 现状与缺口（Discovery 实证，caac760d4..da3c31320）

`gcode_emit/brim.rs` 分派：OuterOnly 臂（:137）在 `da3c31320` 链已
注入 `support_no_brim` mask；**OuterAndInner 臂（:117
`brims::combined::area`）与 InnerOnly 臂（:113）未注入**——上游
`:590-634` 是类型无关的，两臂缺同源行为。

Discovery（`/tmp/w5-brim2/one.3mf`：canon base +
`brim_type=outer_and_inner`、`brim_width=8`、tree(manual)+organic
support，official 2.4.2 vs ares，CPU7/8）：
- 归一化 diff **1496 行**；brim E-move native 571 vs ares 919；
- native brim bbox X[98.14,121.497] Y[97.647,122.353]——Y 与 ares
  方环一致（±12.35），**X 左侧被 support mask 裁去 0.5mm**（ares
  未裁，输出对称方环 X[97.647,122.353]）；
- 对象底面无孔（ares lslices[0][0] holes=0）→ inner 分支贡献为
  空，分歧全部来自 outer 环的 mask 缺失——与已批链同根因、同
  mask、不同分支。

## 3. 纳入 / 延后

纳入（GREEN 全集）：
1. 将 `support_no_brim` mask 注入推广到 OuterAndInner 臂
   （`brim.rs:117` 路径，destination：combined 分支的 forbidden
   ——`gcode_emit/brim.rs` 与 OuterOnly 臂同源同式）；
2. RED：`one.3mf` 变体（outer_and_inner w8）input-only fixture +
   `OrcaRunner::from_env` compare_case 双模式 byte-parity 腿
   （复刻 `brim_outeronly_w5.rs` 模式，无 native 产物入库）。

延后：
- InnerOnly 臂 mask（`:113`）：需独立 RED（有孔对象 fixture），
  本片不夹带；
- inner 环几何对拍（`:546-556`，`combined.rs::hole_area`）——
  本 fixture 无孔，inner 分支未经 RED 覆盖，不改动
  `combined.rs` 内部；
- `:591` extruder 门、跨对象 `:687-717`（维持既有延后）。

## 4. RED 设计与门

- Fixture：`tests/parity/validation/w5-brim/outer-inner-w8.3mf`
  （canon base 仅改 brim_type=outer_and_inner、brim_width=8，
  sha 钉死）；native 参考本地留档不入库。
- 今日必红实证：§2（1496 行 / 571 vs 919 / bbox X 裁剪缺失）。
- GREEN 门：新腿双模式绿 + 既有 207 定向集（brim/support/tower/
  skirt）双模式不回归 + strict clippy 0 / fmt 净 / wasm32 0 @
  exact HEAD。
- 若 GREEN 中 mask 注入后仍残余分歧（inner 分支或其他），按协议
  STOP→探针→amendment 指名唯一函数后再动。

## 5. 风险

OuterAndInner 臂的 forbidden 语义与 OuterOnly 不同
（`brim.rs:120-122`：仅 objectIsland，无 no-inner/no-outer mask，
引 Brim.cpp:560-570）——mask 注入点须保持该差异（support mask
是 `:590-634` 的独立来源，不与 hole-mask 语义混同）。
