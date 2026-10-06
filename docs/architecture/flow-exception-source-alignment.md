# Flow exception：libslic3r Flow 的条件、类型与文本映射

## 上游 rewrite 边界

固定源为 OrcaSlicer v2.4.2，commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`。本 slice 的 owner 是 `src/libslic3r/Flow.cpp` / `Flow.hpp`，不是 `Print::validate`，也不是 Ares 自研 pipeline。包含 `Flow::with_spacing`、`rounded_rectangle_extrusion_spacing`、`mm3_per_mm` 的失败分支，以及现有 Rust Flow 对应调用中的错误传播。`Flow.hpp:27-44` 的类型关系为 `FlowError : InvalidArgument`，`FlowErrorNegativeSpacing` 和 `FlowErrorNegativeFlow` 分别继承 `FlowError`；不能将 `with_spacing` 抛出的普通 `InvalidArgument` 冒称为这两个派生类。

## 不可协商的条件与诊断

| 上游 owner | 精确失败条件 | 原生异常类型 | Rust 公共结果及精确文本 |
| --- | --- | --- | --- |
| `Flow.cpp:147-164`, `Flow::with_spacing` 的非 bridge 分支 | 按 float 运算 `new_width = old_width + (new_spacing - old_spacing)`；**new_width < old_height** | `Slic3r::InvalidArgument` | `SliceError::InvalidInput("Invalid spacing supplied to Flow::with_spacing(), check your layer height and extrusion width")` |
| `Flow.cpp:201-206`, `rounded_rectangle_extrusion_spacing` | float `spacing = width - height * float(1 - 0.25 * PI)`，**spacing <= 0** | `FlowErrorNegativeSpacing` | `SliceError::InvalidInput("Flow::spacing() produced negative spacing. Did you set some extrusion width too small?")`；文本来源 `Flow.cpp:14-15` |
| `Flow.cpp:219-229`, `Flow::mm3_per_mm` | 按上游 float 舍入计算截面积，**res <= 0**；bridge 为圆面积，非 bridge 为圆角矩形面积 | `FlowErrorNegativeFlow` | `SliceError::InvalidInput("Flow::mm3_per_mm() produced negative flow. Did you set some extrusion width too small?")`；文本来源 `Flow.cpp:17-18` |

三种失败复用现有 `SliceError::InvalidInput` 外部输入诊断承载，不新增公共错误枚举层或兼容 fallback。来源类型、条件和文本一一对应；同属 InvalidArgument 家族不等于同一失败谓词。保留原始英文消息；本 core slice 不引入 I18N/UI 行为，也不附加 option key。

`with_spacing` 原生 assert 检查的是 **旧** `m_width >= m_height`，随后的 throw 检查的是 **新** width。本次替换 Rust 对新 width 的 debug assertion 为可传播的失败，不能用 catch-unwind、release 关闭断言或 Print 前置门控代替。旧状态的原生内部 invariant 仍是内部 invariant，不泛化为公共防御校验。等号 `new_width == height` 合法，不照搬 O4 的 `<=`。

bridge 分支保留 `gap = old_spacing - old_width`、`new_diameter = new_spacing - gap` 和 width/height 同步更新，不能套用非 bridge 的新 width 下界。`with_spacing` 本身不调用原生 `mm3_per_mm()`；不要提前把 NegativeFlow 挪到该方法入口、检查旧缓存或改成通用正 spacing 门控。NegativeFlow 在对应的体积消费位置发生。零 spacing/零体积包含在各自的 `<= 0` 中。NaN/Infinity 不属于这三个上游比较新增的失败条件；已有真正外部输入边界检查保持独立，不将其重命名为这三种异常。

## Rust destination 与传播边界

- owner：`crates/ares-core/src/project_slice/perimeters/flow.rs`；按需要将共享 spacing/volume 算术与错误方法拆到 `flow/spacing.rs`、`flow/volume.rs`，每个修改文件保持不超过 400 LOC。保留既有 float 运算顺序和舍入，不以错误修复为名改公式。
- `perimeters/types.rs::Flow` 是上游 `Flow.hpp::Flow` 的临时兼容 shell，含 eager `mm3_per_mm` 缓存；它不是另一套流量模型。`with_width` / `with_height` 对应 `Flow.hpp:92-99` 构造路径，spacing 失败应复用同一 NegativeSpacing owner，不继续用 `invalid support interface flow` / `invalid smaller external perimeter flow` 合并替代来源诊断。缓存的生成与原生体积 accessor 的失败时机必须区分，不能为了复用而对所有纯算术 helper 全局提前验证。
- `with_spacing` 改为 `Result<Flow, SliceError>`；先逐调用点记录实际失败可达性，只在几何/配置输入确实可能触发失败的路径传播 `?`。若 spacing 来自已保证合法的内部不变量，注明对应不可达条件，保留简单签名，不增加逐层防御或新门控。`fill_entities/flow.rs::materialized_flow` 与 `fill_entities/concentric.rs` 的具体传播范围以该审计为准；不吞错、不跳过区域、不返回旧 flow、不输出部分 G-code。对应原生调用边界为 `Fill/FillBase.cpp:133-184::Fill::fill_surface_extrusion`：有输出路径时先消费原 `params.flow.mm3_per_mm()`，非 internal flow 才调用 `with_spacing` 并消费新 flow 的体积，Arachne 分支另有 `with_spacing`；保留这些条件与首错次序，不在空路径或旧 flow 尚未被消费时新增提前失败。其他 materialized fill 调用者只做必要的签名传播，不改填充路径、间距或 role 规则。
- `build_nonbridging_from_width` 与对应 Flow 体积消费处使用上表的独立 spacing / volume 失败；`require_positive_volume` 中来源属于 `mm3_per_mm` 的非正体积不再使用角色专属泛化文本。与 option 定义域验证不同的错误保持不同来源。
- 从纯处理入口 `slice_project_plate_processed` / 对应纯 profile processing 返回 `SliceError`；该 slice 不调用 `validate_project_plate`、不修改 O4/O1/config loading 的顺序、不门控 settings/placement API。validated composition 对原始 O4 无效输入仍首先返回 `Too small line width`，不能要求它继续执行到 Flow。

## 包含与延后

包含上述三个失败分支、现有对应 Flow 构造/消费路径的必要一致诊断和 Result 传播。合法流量、bridge/non-bridge 算术、既有 bit-exact float 测试和 G-code 字节必须不变。原 `with_spacing` 新 width debug assertion、泛化 spacing/volume 错误承载仅作为被上述上游 owner 替换的 scaffold。

延后 `FlowErrorMissingVariable` / placeholder extrusion-width 解析、`new_from_config_width(height <= 0)` 的独立 `Invalid flow height supplied to new_from_config_width()` InvalidArgument、其余 cross-section 算法与未移植的角色解析/bridge 特殊算法、完整 Flow 类 parity，以及 O4 上界和其他 Print validation。若已有 cross-section/ratio 调用链为传播这三种错误而需要 Result，只改必要的传播与来源 accessor，不扩展算法。本 slice 不声称涵盖所有 InvalidArgument 或所有 FlowError。

## Test-first 计划与验收

先批准 spec，再提交 RED/正反控制测试，最后写产品代码。首先用字节未改的 `tests/parity/validation/o4-ender3s1plus-0.2.3mf`（SHA256 `edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110`）记录 **纯处理** 路径的完整结果。基线 panic 或 release 错误/接受的实际结果均如实保留，不预设二者相同；GREEN 要求精确 with_spacing InvalidArgument 文本且无 G-code。独立 Print 和 validated 路径仍须 O4 精确拒绝，编辑/placement 不新增 gate。

原生 Flow 方法 oracle **仅用于隔离边界**：先确认三种条件能否由公开输入到达；若三者均可达，不建立该 harness。确需隔离时，其最小 harness 源码、构建命令和输出放在 `tools/flow-oracle/`，链接固定版本的真实 `Flow.cpp` 即可，不引入 OrcaSlicer 构建系统。再提交 RED，随后预写控制测试，最后实现；本 slice 不新增 catch_unwind（测试或产品），panic 的 RED 由测试进程失败与调用前保存的输入/参数证据记录。原生 CLI 对此 fixture 在 Print 阶段 exit205，不能把其 `Too small line width` 当成 Flow 异常 oracle。Flow 条件/类型/文本需用调用真实固定版本 `Flow.cpp` / `Flow.hpp` 的原生方法 oracle 验证并保存源码版本、编译/调用命令、参数 float bits、动态异常类型、what 文本及结果；不以重写公式或改 upstream throw/assert 的桩冒充原生 oracle。不能经 CLI 到达的私有数值边界在隔离测试之前明确失败方式：旧/新 width 混淆、`<`/`<=` 混淆、bridge 错套 non-bridge 谓词、非正 spacing 与 volume 混类、错误提前消费、float 顺序漂移、错误被吞以及 debug/release 差异。原生方法 oracle 的构建可行性与真正 RED 需先验证；无法构建时报告 blocker，不制造成功证据。

预写原生/ Rust 正反控制：非 bridge 新 width 小于/等于/大于 height；正/零/负 rounded spacing；bridge 零体积与正体积，以及合法 bridge spacing 变换；不改变现有 float-bits 断言。用 public E2E 验证传播，无法从公开输入到达的源方法边界才做上述隔离对照。每例持久保留 input/参数哈希、命令、native 类型/文本、Rust 结果及无输出证据；未设置 artifact 环境变量也可保留测试自己的 tempdir。已有 GREEN 不能称为 RED，断言不能放宽。

退出条件：全部预写控制通过，纯处理 panic 转精确 SliceError、validated 的 O4 首错保持；workspace nextest（不用 cargo test）、严格 all-target/all-feature Clippy、fmt、WASM 全绿；最新 base 独立 official/seeded 双侧对照与共同 Ares G-code 字节一致，参考残留原样保留。Core 不增加文件系统、终端、UI/OpenGL 或平台特有行为，保持 WASM/Windows/macOS/Linux 边界。执行日志和统计放外部证据与 commit message，不放本 ARD。
