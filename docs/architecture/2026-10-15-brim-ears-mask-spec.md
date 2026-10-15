# SPEC(提案): BrimEars 臂 support no-brim mask——brim 分臂链收尾（discovery-only）

状态：SPEC-DRAFT（docs-only）。承接：OuterOnly `da3c31320`、
OuterAndInner `3ee8f272c 链`、InnerOnly `b6780dfdb` 三臂均已落
parity；`gcode_emit/brim.rs` 的 `support_mask` 现覆盖除 BrimEars 外
全部臂。

## 1. 上游边界

- `Brim.cpp:outer_inner_brim_area:590-634`：support no-brim 块对
  ears 类型同样进入，但结构不同——
  - `:597-605` stInnerNormal：与类型无关 emplace（同前三臂已移植
    的 fill coverage 分支）；
  - `:607-634` stInnerTree：`has_inner_brim||has_outer_brim` 门内
    的 `offset_ex(contour,0)` 一项对 ears 为假（ears 两者皆
    非），其余 support 轮廓 emplace 不变——即 ears 的 tree mask
    与前三臂**存在一项差**（无 0-offset 环），GREEN 不能简单复用
    `support_mask`，需对照 :607-634 逐项核对；
- `:682-689` 统一 diff 生效；
- ears 几何本体 `:651-679`（`brim_edge_cases`/`stick_out_area`，
  ares `brims/ears.rs` 已移植）不在本片范围。

## 2. ares 现状

`brim.rs` BrimEars 臂（ears::area）`forbidden` 未并入任何 support
mask。

## 3. RED 设计（需 discovery）

ears 触发条件苛刻：需要**尖角/小夹角**对象（
`brim_ears_max_angle`<180° 的凸角）且首层 brim 区与 support 轮廓
相交。fixture 方案：canon base 派生 + 三角/楔形对象（单凸尖角）
+ `brim_type=brim_ears`（或 auto_brim+`brim_ears_detection_length`
路径，须先对拍确认 ares ears 检测链已 parity）+ normal(auto)
support 贴近尖角。
- Native-first：构造后先确认 native 确实生成 ears 且被 support
  裁剪（brimE 差≠0）；若 ares ears 链本身已分歧（非 mask 层），
  STOP→报 RootCause 另立片；
- Ares==Native 则 STOP 报 NO-RED。

## 4. 纳入 / 延后

纳入：BrimEars 臂 support mask（按 §1 tree/stInner 两分支逐项对
照）+ §3 RED。
延后：ears 几何本体、`:591` extruder 门、跨对象 `:687-717`。

## 5. 风险

(1) ears 检测链在 ares 的 parity 未经独立 RED 覆盖，fixture 构造
失败率高（角过钝→无 ears；角过尖→native 也可能拒）；(2) tree
support 分支的 :607-634 单项差意味着 GREEN 需要独立 mask 变体
而非臂匹配扩项——比前三臂多一步上游逐项核对。
