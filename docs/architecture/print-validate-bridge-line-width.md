# Print::validate：region bridge_line_width 检查

## 上游 source boundary / Rust destination

固定 OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`。按 Print::validate 顺序继续，在已移植的 height/ordinary width (`Print.cpp:1627-1673`) 后、O1 layer templates (`:1695-1749`) 前，移植紧接的 **bridge-width region/role loop** (`:1675-1689`)。这是 libslic3r rewrite，不新增 Ares pipeline 行为。

Rust destination 为 `print/validate/extrusion_width.rs` 的现有 applied/flat per-object traversal，可按400LOC规则放同目录私有 bridge-width 文件；不新建处理入口、mapper、配置解析器或 pipeline。typed carriers 已存在：`ObjectOptions::{thick_bridges,thick_internal_bridges,layer_height}`、`RegionOptions::bridge_line_width` 及四个 filament selector。现有 traversal 仅覆盖 ordinary widths，是该 upstream lambda/loop 的暂不完整兼容 shell；本片补其紧接的 source-owned bridge loop，不能把 bridge 塞进普通宽度 min/max lambda。

`PrintRegion.cpp:7-23::PrintRegion::extruder` 定义角色：frPerimeter→inner_wall_filament_id，frInfill→sparse_infill_filament_id，frSolidInfill→internal_solid_filament_id，frTopSolidInfill→top_surface_filament_id。本 loop **没有** frExternalPerimeter、bottom-surface role。`Config.hpp:624-628::ConfigOptionVector::get_at` 对 out-of-range 返回 front；`PrintRegion.cpp:51-53` 也明确说明 selector0−1 的 unsigned underflow/front。这不是 applied filament_map→physical nozzle lookup，不能复用 ordinary mapper 强行映射，也不能改成 minimum used/max nozzle。

## 不可协商的谓词与顺序

每对象：首层/对象 height → object general/support widths → ordinary region keys-before-regions → **bridge regions-before-roles** →下一对象；全部对象后才 O1。不要把 bridge 检查全局后置，也不要交换各 region/role 的首错。

每个 region 依次四个角色，直接 selector−1 查询 config nozzle vector get_at/front；百分比基于该角色 nozzle，而非 used min/max。absolute保持绝对值。`width<=0` 跳过本角色（不继承 general width、不推导自动 Flow）；然后 `width>nozzle` 返回精确 `Bridge line width must not exceed nozzle diameter`；否则若 **!(thick_bridges && thick_internal_bridges)** 且 `width<=object.layer_height`，返回精确 `Too small line width`。等于 nozzle合法，等于 layer-height在非both-thick时非法。both-thick只豁免 lower，不豁免 upper。

不要求模型实际有 bridge、不走 Flow/materialization/geometric pass。应用后的 resolved object/region（含 object/part/layer scoped overrides）是唯一来源，不另跑 resolution或改 usage。保留已有 applied-map 快照/删除条件、ordinary mapper、raft direct-index、加载/校验/处理独立生命周期；Bridge的 direct角色访问不新增第二个 filament mapper。

## 两个独立 validation owner 与原生公开可达

`PrintConfig.cpp:10487-10517::Slic3r::validate` 在 CLI参数/full config loading (`OrcaSlicer.cpp:7198-7210,:3647-3655`) 前置，以 **max nozzle** 解析 bridge百分比，再以 **min nozzle** 限制 upper；loading error有数值后缀。`Print::validate` 是 apply (`:6038`) 后的 per-region/role source owner (`:6063-6094`)，205错误；不要合并或替换这两个规则，不触碰 pi-tower 的 `options/validation/*`。

1. **原件 lower**：`tests/parity/validation/layer-reset-kobra-plus-relative.3mf` SHA256 `0919c79bc4f84dd0fdaffd237645a0e91124fa12cf07a24da6376d4eeeca9093`，原 layer_height0.2/nozzle0.4，thick_bridges0/thick_internal_bridges1。只公开 override bridge_line_width0.1（25%也可）：Native205，stderr精确 `Too small line width\n`；Ares公开CLI更晚报原 O1 missing-reset。未改原模型/width/profile/template或补 G92；这个原O1输入本身已经通过普通height/width owner，正好观测bridge在O1之前。
2. **scoped upper**：原件的公开3MF派生，只在原 object id2 metadata 加 `bridge_line_width=0.5`，保留所有其他 entry的原解压 bytes，特别是 `Metadata/project_settings.config` 与 geometry/template完全不动。保留原件与原始XML/新增行/ZIP重封装脚本；派生文件 SHA256 `e385145d47ec818770a1b7bb142c6fd6ac5e9f577d89830269f8fcfc79275909`。Native205，stderr精确 `Bridge line width must not exceed nozzle diameter\n`；Ares公开CLI仍报 O1 missing-reset。它验证 object/region scope穿过global loading，不是修坏profile/width来制造reachability；Native与Ares读取同一个派生输入、无额外repair overrides。
3. **不冒充 upper RED**：原件 global override0.5 得 Native254：`Params in command line error: \nbridge_line_width: Bridge line width must not exceed nozzle diameter: 0.500000\nsetup params error\n`；这是 loading，不是 Print。mixed nozzle/percent全局也不能凭ratio假想绕过loading。

真实控制：50%即0.2得到lower205；51%即0.204与0 auto都越过bridge、到原O1；both-thick（原生Bool CLI无独立值，用 `--thick-bridges --thick-internal-bridges`）+0.1也越过bridge到O1，原internal-only仍lower205。错误Bool命令把1当文件导致253、`--no-thick-internal-bridges`不支持导致254，均作为无效诊断尝试保留，不能当 reachability或控制。原件与scoped upper均无Native/Ares G-code；不声称完整slice成功。

## Included / deferred / TDD gate

Included：上述整个 `Print.cpp:1675-1689` loop、四个role、direct nozzle/front、FloatOrPercent、both-thick conjunction、bounds/equality/auto与精确诊断/对象区域顺序；公开 project validation/composition/flat/STL桥接同owner。STL adapter仅透传已存在 typed region/object字段，不改generic DTO。profiles validated入口共用applied owner，不改catalog加载。

Deferred：独立 loading bridge-width聚合规则（当前Rust loading实现未覆盖bridge special max，由其owner另片处理）、Flow/bridge算法、实际bridge生成、tool ordering、warning/bed compatibility、更广pipeline时序。不要修 loading guard、删bridge_flow preflight、改变mapper、schema/defaults/template来让本片GREEN。

Spec批准后先写两个 genuine native-backed **public** RED：原件lower + scoped upper，精确code/text/no G-code、hash、双Aresvalidation/composition结果都落盘；raw/invalid尝试不覆盖。upper派生需验证entry-level差异仅唯一metadata entry新增目标key（ZIP压缩header可不同），固定实际fixture bytes，避免重新封装时间改变hash；原件始终不改。

预写控制：0/百分比0跳过；layer-height下/equal/上、nozzle等值/upper；00/01/10/11的AND、both-thick upper仍失败；region/role先后、默认selector0及out-of-range/front直接访问、不经过filament_map；outer/bottom selector不进入四role loop；普通width/height优先、bridge优先于O1、sourceobject/part/layer范围与STL adapter字段。保留已有height/O4/upper/O1/Flow断言；任何旧诊断期望迁移先逐例报协调者批准，不自动换成generic TooSmall。

使用默认kept per-test tempdir产可重放证据，artifact env可选。plain workspace nextest、strict all-target/all-feature Clippy、fmt、WASM及当前base独立官方/seeded控制、共同G-code逐字节对照，旧参考残留/无效尝试如实保留，comparator不变。未获本 spec 批准前不写Rust测试/实现；ARD记录source/决策，执行日志/计数保存在外部证据和commit message。
