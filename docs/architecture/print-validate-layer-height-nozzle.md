# Print::validate：layer-height / used-nozzle rewrite slice

## 上游边界与 destination

固定 OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`。移植 `src/libslic3r/Print.cpp:1627-1652::Print::validate` 的 non-raft first-layer height 与 object layer-height 检查；共享 `:1524-1532` 已用 nozzle minimum，与已合入 O4 / Print upper-width 共用一次 used-extruder traversal。

Rust destination 是 `crates/ares-core/src/print/validate/extrusion_width.rs` 内现有 nozzle-dependent object validation owner：其当前 width-only 命名是上游 nozzle block 的临时兼容 shell。本片增加一个私有层高检查，在每对象原 `object_widths` 之前调用，仍使用同一个 min/max tuple，不另建 usage/nozzle owner、遍历、fallback、pipeline。无需文件重命名或独立 geometry/config crate；模块远小于 400 LOC。

## Included 与 deferred

- **Included first layer**：仅 `!object.has_raft()` 分支，global `m_config.initial_layer_print_height.value > min_used_nozzle` 返回精确 `Layer height cannot exceed nozzle diameter.`。比较原 float，严格 >，等号合法，无百分比基准、EPSILON、clamp 或自动改层高。
- **Included object layer**：`object.config().layer_height.value > min_used_nozzle` 同样返回该精确文本，包括 raft 对象的这个独立 predicate。resolved object layer override 保持 object-local；不是用 global layer_height 覆盖所有对象。
- **Deferred raft first layer**：`Print.cpp:1629-1641` 的 raft_layers==1 选 support_interface_filament，否则选 support_filament，selector0 用 minUsed，否则直接 `m_config.nozzle_diameter.get_at(selector-1)`。其应用期 logical/physical remapping 边界尚未核清，**本片不猜、不替换成 min/maxUsed、也不加新的 mapper/守卫/处理路径**。raft 对象暂不执行新的 first-layer predicate，保持已有 shell；这不是完整 Print::validate 或 raft validation parity。若公开 RED 必须借 raft 输入才可达，停止缩片，不制造 native harness。
- Deferred tree-support flow/tip/branch checks与warnings (`Print.cpp:1557-1625`)、bridge width branches、printer max_layer_height/其他层高约束、slicing parameters与自适应层高算法、generic config validation、输出格式和 profile 数据修复。

## 两个函数与顺序

加载期 `PrintConfig.cpp:10337-10354::Slic3r::validate(FullPrintConfig,bool)` 检查 layer_height 正数/SCALING_FACTOR 条件及 initial_layer_print_height 正数，后续定义范围检查也属于这个函数；并非本 Print used-nozzle 上界。DynamicPrintConfig::validate (`:8866-8877`) 用 defaults 构建 FullPrintConfig。CLI 参数 `OrcaSlicer.cpp:7198-7210` 和完整项目 `:3647-3655` 在 apply/Print::validate (`:6038,:6063-6094`) 之前；加载期拒绝、未知参数/setup error 不算本 RED。不碰 pi-tower 的 `options/validation/*` 及接线、也不改 pi-tip template owner。

上游对象循环顺序是 tree/warnings（尚未移植）→ first-layer predicate → object-layer predicate → 本对象 general/support/region width predicates → 下一个对象 → 后续 O1。在当前 bounded shell 中两层高 predicate 必须在**每个对象**的 width checks 前执行，不能先对所有对象跑高度，再跑所有对象宽度；早对象 width 与后对象 height 的首错会不同。Flat view 同序。不存在 print_objects 的对象沿用 skip。保留已有 Print 独立 validation / validated composition；settings/placement 可检查，纯 processing 不新增 gate。

## adapters 与公开可达性 gate

- Applied：global initial height 从 `resolved.views.full.process.print` 取值，layer height 从 resolved object 取值。使用现有 BoundedProjectUsage / filament_map / nozzle owner，不扫描全部物理 nozzle；未使用小 nozzle 不得导致拒绝。
- Flat：仍构建 ObjectOptions/RegionOptions，沿用 collect_usage + 同 nozzle tuple；initial height 取 global PrintOptions，不从对象 layer_height 派生。
- STL：只在现有 selected-field adapter 加 `initial_layer_print_height`，原 layer_height/nozzle getter/usage字段不变。`options/initial_layer_print_height.rs::SliceOptions::initial_layer_print_height` 的 default 是 DEFAULT_LAYER_HEIGHT（0.2），不是随 object 层高的 fallback；不为此重写 legacy DTO 或 generic JSON parsing。
- Project/profile validated entry points 均复用同 borrowed validation owner，仍只加载/应用一次。没有额外 process-stage 层高修复或第二套 nozzle 比较。

先写 spec，再在原 S1 Plus 0.2 nozzle 项目中仅改变一个公开 layer_height 或 initial_layer_print_height 为 0.3。原生必须实际跨过 loading 并返回 Unix205、精确 `Layer height cannot exceed nozzle diameter.\n`、无 G-code；现 Ares 的后续 O4/upper/O1 错误不等价。保留原低 width / 模板 / nozzle / geometry，禁止为得到 acceptance-vs-rejection 修输入。若被更早配置或 Print 边界拦住，记录真实 blocker，不冒称本 RED。公开 CLI/validate_project_plate/validated slice 的 input hash、唯一 overrides、阶段、真实 stdout/stderr 和无 G-code分别留证；不写隔离 private-helper harness。

## TDD 与退出标准

批准后：先 commit 两个真实 native-backed public API RED（global first-layer / object layer-height分别测），再预写控制，再产品。控制包括 <=minUsed/刚越界、unused smaller nozzle、support/wipe-used nozzle最小值、global first height与object-local层高、两 predicate优先于同对象 width与O1、原 width 首错在 earlier-object 内优先于 later-object height（若合法公开 fixture支持）。raft initial明确 deferred，不制造无来源自动修复。新增高度会合法改变先前 Ares 的 width/O1 首错，若旧断言确实过时须逐条列出原 native/source 证据向协调者申请迁移，不能默认修改断言。

始终保留 per-test tempdir 默认 artifacts，不要求 artifact env，不减弱原 O4 / upper / Flow 测试或比较器。门禁 plain workspace nextest、strict all-target/all-feature Clippy、fmt、WASM；新当前 base 独立官方/seeded complete controls、共同 Ares G-code bytes 与残余 reference 错误诚实保存。执行日志、计数在外部 evidence/commit messages，不写入 ARD。公开不可达则停止，本 spec 不授权产品在可达性未证实时先实现。
