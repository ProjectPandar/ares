# SPEC: Line/CrossHatch ironing 残余分歧——连接与起点（fallback 记录）

来源：df0114018 的 fallback 条款执行——两臂接入（未提交工作
树）后两腿仍 FAIL，按约定 STOP、还原守卫、记录腿内首差。

## 1. 触发实验（工作树两臂，未提交；守卫已还原）

在 RED 分支 `fbe70486a` 工作树上给 `surface.rs` 加 Line/CrossHatch
分派臂（调用既有 `fill_entities::line`/`crosshatch` 包装器）后，
official 定向 25 腿 = 23 PASS + 2 FAIL（line/crosshatch）——**残余分歧
属实，非守卫语法问题**。实验后 `git checkout --` 还原，分支停在 RED。

## 2. 腿内首差

### Line（ares 697 E vs native 699）
首差 @ 行 35142：
- native：`G1 X115.381 Y104.209 E.00043`（**挤出的链接段**，并入折线）
- ares：`G1 X115.381 Y104.209 F9000` + `G1 F900`（**travel**）

且后续 35152-35156 ares 以多条短挤出（E.00043/E.00066…）拼出 native
单条合并段。定性：**连接语义分歧**——native `FillLine.cpp:78-100` 的
`chain_polylines` + `_can_connect(|dx|,|dy|)`（`_diagonal_distance=
2×spacing`、`_line_oscillation`）+ `offset(min_spacing/2)` 包含性把短链
接并入折线；ares `fill/line.rs` 在 **0.1mm ironing spacing**（远密于
sparse parity 区）下阈值/振荡语义行为不同，两处链接退化为 travel
（-2 E 行）。
候选机制点：`_line_oscillation = line_spacing - min_spacing` 在
density=1 时为 0 的边界行为；`extra=0.3×min_spacing` 端点外延后的
连接判定几何。

### CrossHatch（ares 769 E vs native 779）
首差 @ 行 35330（ironing 段起点）：
- native：`G1 X117.256 Y104.25 Z10.2`
- ares：`G1 X114.286 Y104.25 Z10.2`

定性：**起点/链序分歧**——同一层 native 从另一端开始（X117.256 vs
X114.286，间隔恰呈图案相位），后续行序随之错位（-10 E 行）。候选机制
点：`chain_or_connect_infill`（`FillBase.cpp:1820`）的首折线选择/
最短路径排序 vs ares `fill/cross_hatch.rs` 的排序；或
`generate_infill_layers` 相位在 ironing z 上的取整差。
（注：sparse 路径 parity 在常规 spacing 下成立，分歧疑为 ironing
spacing/单层 z 相位交互。）

## 3. 建议后续切片边界

1. **Line 连接对齐**：逐坐标比对 `fill/line.rs` 与 `FillLine.cpp:78-100`
   在 0.1mm spacing 下的 `_can_connect` 判定序列（以首差点 X115.381
   Y104.209 为锚），修正阈值/包含性语义；oracle `f260b89b16f2`。
2. **CrossHatch 起点对齐**：核对 z 相位取整（`fmod`/int 截断，
   `FillCrossHatch.cpp:170-176` 的 `int phase` 截断语义）与链序；oracle
   `a944fc77a8af`。
两项独立可拆分；RED 腿已在分支 `fbe70486a` 待用，守卫维持
原样直至各自 GREEN 双模式全过。

## 4. 边界声明

- 不在本 SPEC 提交任何代码；`surface.rs` 守卫与 `fbe70486a` RED 不动。
- 两残余修复合入前，Line/CrossHatch ironing 仍守卫
  （UnsupportedProjectFeature）。
