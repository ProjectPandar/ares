# SPEC(提案): btPainted（painted ears）——`make_brim_ears` 非 auto 路径 discovery

状态：SPEC-DRAFT（docs-only，未跑 Native）。

## 0. Roadmap 观察记录（按 coord 指示并入本 commit；经纠正修订）

**BrimEars+tree ears-only `no_brim_offset` 项 = 死代码（非 latent）**
（discovery 2026-10-15，6/6 迭代，artifact /tmp/w5-brim2/{n,a}-eart*）。
依据 `Brim.cpp:457-458`：btEar 使 use_auto_brim_ears=true →
has_inner_brim=has_outer_brim=**true**，故 `:621 (!has_outer_brim)`
与 `:623 (!has_inner_brim && !has_outer_brim)` 两项对 ears **永不
可达**——是 ears 路径的死代码，而非可观测性缺口（先前
ba599af1a 的前提有误，NO-RED STOP 因这一更强的理由成立）。
`:631 has_inner||has_outer` 门的 `offset_ex(contour,0)` 对 ears
可达但与 contour 项重复（no-op）。不记入 roadmap latent 清单。

## 1. 上游边界

- `PrintConfig.cpp:428`：`{"painted", btPainted}`；
- `Brim.cpp:456-458`：`use_brim_ears = brim_type == btPainted`，
  has_inner/has_outer 对 painted 皆真（同 ears）；
- `Brim.cpp:354-412 make_brim_ears`：读
  `model_object()->brim_points`（逐点：`z<=0` 过滤、
  `brim_width=floor(scale_(head_front_radius)/flowWidth/2)*flowWidth*2`
  量化、POLY_SIDE_COUNT 多边形、model_trsf 平移、EFC snap 分支）；
  `:537/:551` outer/inner 双调用；`:552-556` 后续同 ears。
- 3MF 载体：`Format/bbs_3mf.cpp:217
  Metadata/brim_ear_points.txt`，`:2962-3010` 解析（v0 格式：
  首行 `brim_points_format_version=0`，随后每对象
  `<objid>=<x> <y> <z> <r> ...` 4 元组）。

## 2. ares 现状

`options/brim.rs:12` 已映射 `painted → BrimType::Painted`；但
`brim.rs` 分派对 Painted 走 ears 臂（auto 检测），**无
`brim_ear_points.txt` 解析、无 painted 点集**——若 fixture 携带
painted 点，ares 要么空 brim 要么走错路径，预期强分歧。

## 3. Discovery 计划（Native-first，需授权，CPU7,8）

fixture：canon base + `brim_type=painted` + zip 追加
`Metadata/brim_ear_points.txt`（v0 格式，对象 1 一点：
底面角外缘坐标 + head_front_radius=5）。判定：
- native 生成 painted ear 且 ares 分歧 → public RED（模式同
  31df0fd92）→ GREEN 分阶段（txt 解析 → BrimPoint 模型 →
  `make_brim_ears` 移植，均需 amendment 逐函数批复）；
- native 不生成（如 z 过滤/半径量化吞掉）→ 调参重试（≤4 迭代）
  → 仍无 → NO-RED STOP。
- EFC snap 分支（`:377-404`）延后（EFC 本身独立链）。

## 4. 成本对比（为何不选 :591 extruder 门）

`:591` 需多喷头 + support_filament≠brim filament 的机器级
fixture（canon 单喷头，改造成本高、噪声大）；btPainted 仅需
zip 追加一个 txt + brim_type 改值，discovery 一次可判。

## 5. 延后

`:591` extruder 门、跨对象 `:687-717`、EFC snap、btPainted 的
inner 调用（`:551`）独立对拍。
