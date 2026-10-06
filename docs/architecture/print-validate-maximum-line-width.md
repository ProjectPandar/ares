# Print::validate：线宽上界 rewrite slice

## 固定上游与 Rust owner

OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`。本 slice 移植 `src/libslic3r/Print.cpp:1524-1556` 的 used-extruder min/max nozzle 与 `validate_extrusion_width` lambda 上界，以及 `:1654-1671` 的对象/region 调用顺序。`libslic3r.h:68` 的 `MAX_LINE_WIDTH_MULTIPLIER` 是 5。

Rust destination 是现有 `crates/ares-core/src/print/validate/extrusion_width.rs`；沿用 O4 的 borrowed resolved object/regions、共同 usage/filament_map 及 flat settings/STL adapter。当前 lower-only shell 是这个上游 Print 边界的临时兼容表示，不是自研 Ares pipeline。只有 native 公共输入证实 Print 上界可达才实现；若全部候选在加载期被挡住，停止并报告，不制造隔离异常路径。

## 两个不同函数：不得合并或混称

1. **加载期 config 校验**：`PrintConfig.cpp:10337::Slic3r::validate(const FullPrintConfig&, bool)`，`:10487-10517` 从**整个 cfg.nozzle_diameter** 取 min/max，对列出的宽度执行 5×global-max（bridge 为 global-min）检查，汇总 keyed diagnostics。调用入口 `DynamicPrintConfig::validate` (`:8866-8877`) 先用 defaults 组装 FullPrintConfig。`OrcaSlicer.cpp:7198-7210` CLI 参数检查、`:3647-3655` 完整 3MF/config 检查先于 Print apply/validate；完整加载失败输出 `Param values in 3mf/config error:` 与 keyed 文本并退出。
2. **Print 阶段**：`Print.cpp:1543-1556::Print::validate` 内 lambda 使用**实际 extruders** 的 min/max nozzle，逐对象及 region 查宽度，返回首错，精确无 key 的 `Too large line width`。`OrcaSlicer.cpp:6038` apply 后 `:6063-6094` 调用 Print::validate，未通过不 process、不产 G-code；该普通错误为 CLI_VALIDATE_ERROR，Unix exit205。

两函数各自检查上游定义的数值规则，不是 Rust 重复规则需要合并。加载期先拒绝的输入不算 Print RED。`options/validation/extrusion_width.rs` 的 Slic3r::validate rewrite 属于 pi-tower O3，**本片不碰该文件，不改其诊断/聚合/桥宽规则/阶段接线**。

## 上界与原 O4 的不可协商顺序

每次原 lambda 调用依次：

- `width_min = config.get_abs_value(key, min_used_nozzle)`，为 0 则 auto，整个 lambda 成功。
- 否则 `width_min <= object.layer_height` 返回 O4 的 `Too small line width`。
- 否则 `width_max = config.get_abs_value(key, max_used_nozzle)`，**width_max > max_used_nozzle * 5** 返回 `SliceError::InvalidInput("Too large line width")`。等号合法，无 tolerance、clamp、profile 修复或新防御。

Float 是原 absolute value；Percent 分别以 min/max nozzle 求绝对值。沿用 object line_width → conditional support_line_width → key-before-all_regions(inner/outer/sparse/internal-solid/top/skin/skeleton) 顺序。不得先跑所有 lower 再跑所有 upper：早期 key 上界与后续 key 下界之间的首错会改变。现有 O4 首错及后续 O1/O7 等先后位置按同源边界保持。

将现有 nozzle minimum owner 改成一次映射/遍历同时返回 minimum/maximum；applied 和 flat 都用它，不再建 maximum 遍历、复制 usage 或扫描未使用 nozzle。已有映射边界校验及 support0/wipe/selector 语义不变。仍跳过没有 print_objects 的对象。

## 公共可达性：先验证，不能仅靠公式假定

- **优先候选：general line_width**。加载期宽度列表 `PrintConfig.cpp:10497-10506` 没有 general line_width；Print lambda `Print.cpp:1654-1656` 则先检查它。选择合法原项目，仅用公开配置输入将 general line_width 设为大于 5×used max；原生必须实际越过 loading 并返回精确 `Too large line width\n` / exit205 / 无 G-code。Ares 独立 Print validation 或 validated composition 当前接受、或返回后续旧 lower/O1 错误，才构成真实差异。原项目/hash、唯一配置变体及所有调用/输出分别留证，不修补其他字段消掉首错。
- **第二候选：absolute width + unused larger nozzle**。加载期允许 width <=5×global max，但 Print 的 used max 较小时可拒绝；须公开输入具备合法 cardinality/mapping、实际不用大 nozzle，并实测两阶段，不虚构 usage 或直接调 private helper。
- **百分比不是自动的绕过路径**。同一 raw percent P 在两个正 nozzle 基准 d 上，正常数值的 `P*d/100 > 5*d` 都归约为 P>500；单纯 global-max 与 used-max 不同不能据此声称 percentage 上界公开可达。仍测试 500% 等号/auto 及 O4 min-baseline 首错；若靠 config scope/object overrides 才有差异，必须单独 source-cite 合并顺序并实测，不借浮点病态值或输入修复造 RED。

原生 CLI 的 config keyed `too large line width <value>`、bridge 错误、O4 Too small、模板拒绝、参数/setup 错误都不能当作本 Print 上界 RED。原生只调用固定版本未改算法的 CLI，不新建 harness、不改抛出或校验点。不可达时不移植额外错误处理。

## 包含、延后与测试 seam

包含 general/conditional-support/逐 region 上界、used min/max 共享计算、absolute/percent/zero/严格 >、现有独立 Print 与 validated project/profile/settings/STL 组合。GUI settings/placement 保持可检查；纯 processing 不新增 Print gate；无部分 G-code。延后 initial-layer width（本 lambda 不检查）、bridge_line_width 的独立 Print 条件、tree/layer/nozzle 其他校验、geometry/flow/template/加载期 options 规则及完整 Print::validate parity。

测试 seam 沿用 O4：公开 validate_project_plate / slice_project_plate_validated，原生完整 CLI；flat validate_print / validate_stl 及 profile validated 只补同 owner 的必要控制。不写 private nozzle/width 单测，不新增不存在的输入守卫。顺序：先 spec，再原生实测的真实 RED，再预写控制，再产品。

控制覆盖 auto/5×等号/刚越上界、加载期先拒绝、百分比不可误称 bypass、unused nozzle、support inactive/active、首个 upper 对后续 lower 的优先级；原 O4 的 8 个 fixture/hash/精确诊断及 float/geometry/G-code 断言不弱化。测试有 artifact root 用 caller root，否则保留 test tempdir，不依赖环境变量、不跳过验证。门禁为 plain workspace nextest、strict all-target/all-feature Clippy、fmt、WASM，加当前 base 独立 official/seeded 状态与共同 Ares 字节对照。改动模块 <=400 LOC；分析执行日志和统计在外部证据及 commit message，不写入 ARD。
