# SPEC: HilbertCurve ironing 残余——首差定位与修复边界（诊断片）

前置：HilbertCurve RED `c126da004`（GREEN 未达成，STOP 报告在案）。

## 0. 引注勘误（诊断中发现的既有提交事实错误）

上游 `FillPlanePath.hpp` 真值：**ArchimedeanChords `centered()=true`（:65）、
HilbertCurve `centered()=false`（:72）、OctagramSpiral `centered()=true`（:88）**。
ares `fill/plane_path.rs::centered()` 映射**正确**（代码无 bug）。但
HilbertCurve SPEC（f44f2c076）与 ArchimedeanChords GREEN 提交
（80063f5de）的注释/文档把 centered 归属引反了（Archimedean 实走
center shift，注释写 min-corner）。两处 byte-parity 结果不受影响
（代码正确），但注释需在 HilbertCurve GREEN 或独立 docs 修正提交中更正。

## 1. 诊断方法（可复现）

- ares temp 探针（未提交、已还原，`git diff`=0 证明）在
  `fill/plane_path.rs::fill_component` 分层 dump：
  STAGE1=generate 原始折线、STAGE2=`intersection_open_polylines` 后、
  STAGE3=chain+回转变换后（世界坐标）；dump 共 343 片段 ×2 expolygon。
- native 侧从 canonical oracle gcode
  按 travel 切分提取折线集。
- 同空间（gcode mm）集合对拍：点多重集、折线多重集、计数。

## 2. 事实（fixture: ip-hilbertcurve.3mf, 19118E/632c2d357e5c）

1. **点多重集近乎恒等**：~7000 个折线顶点中仅 **5 个边界点**不同
   （native-only：114.27,104.32 / 114.27,115.68 / 115.68,106.587 等；
   ares-only：115.68,111.009 等）→ **hilbert 顶点生成本体正确**。
2. **片段数不同**：native 353 条 vs ares STAGE2 343 条（**少 10**），
   差异集中于边界微片段（两点、~0.03mm、贴边裁剪）→
   `intersection_pl`（`ClipperUtils`）片段化差异。
3. **顺序/方向不同**：218 条折线内容差经配对分析全部为**同点集、
   不同顺序或反向**（例：native `(100.116,115.728)→(100.093,115.75)`，
   ares `(100.116,115.728)→(100.138,115.75)`——同一起点的两条贴边
   微片段，两侧选择 extrude/travel 的方向相反）→
   `chain_polylines`（`ShortestPath.hpp`）序差。

## 3. 首差判定与候选站点

**首差在 `intersection_pl` 的贴边片段化**（证据 §2.2 的 10 条片段与
§2.1 的 5 个边界点），链序差（§2.3）既可能是其下游放大，也可能是
`chain_polylines` 在等距 tie 上的独立分歧——两者需 vendored 对拍分离：

- **站点 A**（上游 `ClipperUtils.hpp::intersection_pl` → clipper
  开放折线裁剪的交点/切点序）：与已修的 `fix_intersection_order`
  等键序族同源。对拍法：vendored Clipper6 + intersection_pl 喂 ares STAGE1 原始折线 + 收缩后
  expolygon，比 native 片段集（需 gdb 断点或 vendored 全链取得）。
- **站点 B**（上游 `ShortestPath.hpp::chain_polylines`）：贪心最近端
  + reversal 选择。对拍法：同一片段集（ares STAGE2）喂 vendored
  chain_polylines 与 ares `ordering::chain`，比输出序。

## 4. 修复边界（预计）

- 若 A：`intersection_open_polylines` 对应排序器等键行为修正
  （同 ce51d9ee1 模式，逐一钉 native 片段）。
- 若 B：`ordering::chain` 的等距 tie/reversal 选择对齐。
- 无容差、不合并片段、不吞边界点（5 个差异点必须逐字节复现）。

## 5. 不做

- 不改 `centered()`（已正确）；不改 hilbert 生成本体；
- 不动其他 plane-path 模式（Octagram/Archimedean 已 parity）。
