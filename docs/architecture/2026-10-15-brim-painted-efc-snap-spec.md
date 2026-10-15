# SPEC(提案): btPainted ears EFC snap（Brim.cpp:392-404）——painted ears
补全收官片

状态：SPEC-DRAFT（docs-only）。承接：btPainted outer GREEN
`cbe4ccda3`（stack-19 已并）与 inner GREEN `f0ff95af3f`（stack-20
已并）——二者均把 `:392-404` 的 EFC snap 记为延后项。本片即该
延后项：painted ear 圆心吸附到 EFC 调整后的底层轮廓。

## 1. 上游边界（源：`OrcaSlicer/src/libslic3r/Brim.cpp`）

- `make_brim_ears` `:392-411`（snap 主体）：
  - `:362-367`：`use_efc_outline` 为真时**惰性**计算一次
    `efc_outline = get_print_object_bottom_layer_expolygons(*object)`；
    `raw_outline = object->layers().front()->lslices`（**未** EFC 调整，
    用于岛屿归属匹配）；
  - `:393-404`：对每个 ear，`efc_outline` 非空且
    `closest_point_on_matching_island(raw_outline, efc_outline,
    Point(pt_x, pt_y), closest)` 命中时，`pt_x/pt_y` 被替换为吸附点，
    随后 `:411` `contour.translate(Point(pt_x, pt_y))` 以吸附后圆心
    平移盘——即**仅圆心吸附，盘半径/量化路径（:375-380/:382）不变**；
- `use_brim_efc_outline` `:56-62`：`brim_use_efc_outline` &&
  `elefant_foot_compensation > 0` && `elefant_foot_compensation_layers > 0`
  && `raft_layers == 0`；
- `closest_point_on_expolygons` `:65-80`：遍历每个 ExPolygon 的**所有
  轮廓与孔**（`contour_or_hole(i)`），`Polygon::closest_point(from)`
  顶点级最近点，`int64` 差平方的 `double` 距离比较，**严格小于**才
  更新（先遇者并列优先）；
- `find_containing_expolygon_index` `:84-96`：首个 `contains(from)` 的
  lslices 岛屿索引（`ExPolygon::contains` 含孔排除语义）；
- `closest_point_on_matching_island` `:99-108`：岛屿命中时
  `island_outline = intersection_ex(efc_outline, raw_outline[island])`
  非空则在其上吸附，否则（及未命中）回退到整个 `efc_outline` 上吸附；
- `get_print_object_bottom_layer_expolygons` `:112-118`：底层各 region
  的 `region->slices.surfaces` 做 `closing_ex(SCALED_EPSILON)` 后拼接
  （**EFC 已在 slices 中生效**，closing 只弥 region 边界缝）。

## 2. Rust 目标边界

- `crates/ares-core/src/brims/ears.rs::painted`：新增 snap 前置步骤
  ——在逐点建盘之前，对 `(pt_x, pt_y)`（既有截断 scale_ 后圆心）执行
  吸附替换；盘生成与后续 outer/inner 裁剪链不动；
- `crates/ares-core/src/project_slice/gcode_emit/brim.rs` Painted 臂：
  传入吸附所需两组轮廓：
  - `raw_outline`：现有 lslices 源（`source` 的未 `outline::select`
    原始形，即 `.cloned().unwrap_or_default()` 那份）；
  - `efc_outline`：复用 `outline.rs` 的底层 region
    `closing_ex(SCALED_EPSILON, Miter, 3)` 机制（`brim/outline.rs
    :36-44` 同源 `:112-118`），但 painted 需要**原始拼接结果**而非
    与 ordinary 的 intersection——提取共享 helper 供 `select` 与
    painted 共用；
- gate：`options.brim_use_efc_outline.0 && elefant_foot_compensation>0
  && layers>0 && raft_layers==0`（与 `outline.rs::select :18-26` 同一
  谓词，直接复用 `options::brim::parse_efc_outline_offset` 已解析的
  `Option<f64>` 非空判定）；
- 几何原语：`closest_point_on_expolygons`（含孔全轮廓、i64 差平方）、
  `find_containing_expolygon_index`（点在含孔 ExPolygon 内）、
  `closest_point_on_matching_island`（island intersection 非空则局部
  吸附，否则全局回退）落 `brims/ears.rs` 私有函数（或同文件小节），
  惰性求值 efc_outline（首个需要吸附的点时计算一次）。

## 3. 包含行为

- 仅 btPainted（`make_brim_ears` 唯一调用方是 painted 分臂 `:537/:551`；
  BrimEars auto 臂不经过 `:392-404`——`make_auto_brim_ears` 另源）；
- outer 盘与 inner 盘共用同一吸附后圆心（`:411` 在分支汇合后平移，
  两调用点各自进入 `:392-404`，吸附输入相同故结果相同）；
- 吸附失败（efc 空 / 无最近点）保持原圆心，无 fallback 差异；
- 严格相等断言 RED：fixture 需 brim_use_efc_outline=true +
  EFC>0 + layers>0 + 无 raft + painted 点靠近（但不落在）EFC 收缩
  轮廓——native 盘心吸附、ares 不动 → G1 坐标分歧；control 腿关闭
  `brim_use_efc_outline` 维持现有 byte-parity。

## 4. 延后行为

- EFC 对 painted **面积链**（外环 gap-offset、孔 inset）的影响：上游
  painted 分支的面积源仍是原始 lslices（`:502-513` 的 EFC 替换仅覆盖
  非 painted 路径），本片不改 ares Painted 臂现有 `source` 语义；
- `Polygon::closest_point` 的上游实现细节（投影 vs 顶点级）以 native
  对拍为准，若顶点级足够则不引投影（discovery 定值）；
- BrimEars auto 臂（btEar）、AutoBrim/NoBrim 的 `:468-472` EFC
  base-slices 交换：本片 RED 仅覆盖 Painted，故 gate 保持
  OuterOnly/InnerOnly/OuterAndInner/Painted；btEar/AutoBrim 延后
  （需独立 RED+ctl）；
- 跨对象、combine、EFC outline 与 support mask 的交互组合保持既有
  片边界。

## 5. 测试先行（TDD）

- discovery（Native-first，CPU7,8，≤4 迭代）：pocket/方板类对象 +
  painted 点 + EFC 开启，native vs ares 盘心坐标对拍定值
  （吸附距离 = EFC 补偿量附近的顶点距离）；
- RED：input-only fixture + 双腿（snap on/off）test-only commit；
- GREEN：如上 §2 实现，RED 转绿 + control 与全 brim 族无回归。
