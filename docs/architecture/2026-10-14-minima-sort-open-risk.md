# 已知风险：clipper 局部极小排序的 MSVC/GCC 未决状态

状态：docs-only 风险记录（无代码变更）。

## 背景

Native oracle（official 2.4.2）为 GCC/libstdc++ 构建，`std::sort` 的
等键置换遵循 GCC introsort。ares 引擎中有两个排序站点：

- `intersections.rs::fix_intersection_order`（交点列表，降 Y）
- `minima.rs`（局部极小列表，升 Y）

## 历史

- `1f5e27363`（2026-08-22）：以"oracle 为 GCC 构建"为据，**两处**同时
  切到 `fixed_gcc_sort_by`，并更新钉住期望。
- `2434774f0`（2026-08-24）：标题 "match target sort permutation"，
  **无正文**，把两处回退到 `fixed_msvc_sort_by`，并删除了钉住 GCC 序
  的测试（`task22f_large_minima_match_complete_source_sibling_order`
  等）。回退动机无记录。
- `ce51d9ee1`（本线）：在 canonical 套件内 oracle 覆盖的 fixture
  （line ironing，等 Y 交点 tie 密集）上证明 **intersections 站点的
  MSVC 置换无法达成 byte-parity，GCC 置换精确复现 oracle**
  （`f260b89b16f2`）——即 `2434774f0` 在 intersections 站点被证伪，
  intersections 恢复 GCC。当时（8 月）的回退验证环境无记录，鉴于
  非 canonical 环境（缺 `ARES_PARITY_*_REFERENCE` 系列引用）可产生
  假失败，其原始动机建议复核。

## 未决风险

`minima.rs` 现仍为 MSVC 序：
- 今日 canonical 套件中**没有任何 oracle 用例产生可判别的 minima
  tie**（两种排序均全套通过），故该选择既未被 oracle 证实、也未被
  证伪。
- 按单一工具链原理（native 二进制内所有 `std::sort` 均为 GCC 行为），
  minima 站点的 MSVC 序是**潜在分歧风险**：一旦出现 tie 密集的
  minima 输入（如对称 ±x 同底边几何），输出序可能与 oracle 分叉，
  症状与本次 intersections 案例同构（OutRec 创建序 → 输出序）。
- 处置：**不预先翻转**（无证据不翻）。当首个含 minima tie 的 oracle
  用例出现时，以同法判定（dump 对照 vendored GCC 序 vs MSVC 序的
  oracle 复现），届时一并清理两站点与 `2434774f0` 遗留删除的
  pinning 测试。

## 关联

- 修复与证据：`w5-ironing-line` 链（`5493fbb5e..ce51d9ee1`）
- 残余 SPEC：`2026-10-14-clipper-equal-scanline-intersection-order-spec.md`
