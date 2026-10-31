# Flow exception：两个可达的 libslic3r Flow 失败分支

## 上游 rewrite 边界

固定源：OrcaSlicer v2.4.2，commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`。owner 是 `src/libslic3r/Flow.cpp` / `Flow.hpp`，不是 `Print::validate`，不设计新的 Ares pipeline。本 slice 仅包含 `Flow::with_spacing` 非 bridge 的新 width 失败，以及 `rounded_rectangle_extrusion_spacing` 的非正 spacing 失败和必要传播。

`Flow.hpp:27-44`：`FlowError : InvalidArgument`，`FlowErrorNegativeSpacing` / `FlowErrorNegativeFlow` 分别继承 `FlowError`。with_spacing 抛出的普通 InvalidArgument 不能冒称为这两个派生类。

## 不可协商的条件、类型和文本

| owner | 精确条件 | 原生类型 | Rust 公共结果 |
| --- | --- | --- | --- |
| `Flow.cpp:147-164::Flow::with_spacing` 非 bridge 分支 | float `new_width = old_width + (new_spacing - old_spacing)`，**new_width < old_height** | `Slic3r::InvalidArgument` | `SliceError::InvalidInput("Invalid spacing supplied to Flow::with_spacing(), check your layer height and extrusion width")` |
| `Flow.cpp:201-206::rounded_rectangle_extrusion_spacing` | float `spacing = width - height * float(1 - 0.25 * PI)`，**spacing <= 0** | `FlowErrorNegativeSpacing` | `SliceError::InvalidInput("Flow::spacing() produced negative spacing. Did you set some extrusion width too small?")`，文本源 `Flow.cpp:14-15` |

复用现有 SliceError 承载，不新增公共错误层、fallback 或 I18N/UI；保留英文文本，不追加 option key。`with_spacing` 上游 assert 检查的是**旧** width >= height，throw 检查的是**新** width；不能把新 width 的可恢复失败放在 debug_assert 中。等号合法，不照搬 O4 的 <=。旧状态 invariant 保持内部 invariant，不泛化为公共防御校验。

bridge 分支保留 gap / new_diameter 的源 float 运算与 width/height 同步更新，不套用非 bridge 新 width 下界。不得以数学 no-op 替代 bridge 浮点运算。with_spacing 不消费原生 mm3_per_mm，不提前检查旧缓存或添加通用正 spacing 门控。NaN/Infinity 不属于本次新增的上游比较失败条件；已有外部输入检查保持独立。

## NegativeFlow 不可达：不移植额外防御

`Flow.cpp:219-229::Flow::mm3_per_mm` 的 res <= 0 抛 `FlowErrorNegativeFlow`，文本源 `:17-18`：`Flow::mm3_per_mm() produced negative flow. Did you set some extrusion width too small?`。该分支不纳入本 slice。

尝试通过公开 bridge_flow=0 触发零面积时，Rust `project_slice/perimeters/preflight.rs:120-121` 先返回精确 `invalid Orca option bridge_flow`，到不了 Flow::mm3_per_mm。对应上游拒绝点为 `PrintConfig.cpp:10337::validate(const FullPrintConfig&, bool)` 的 `:10427-10429` bridge_flow <= 0，以及 `:10432-10434` internal_bridge_flow <= 0；`DynamicPrintConfig::validate` (`:8866-8877`) 进入该校验。不能移除前置拒绝来人为制造 Flow 可达性。

按项目不可达场景规则，不为 NegativeFlow 增加错误处理；mm3_per_mm 的签名、纯算术及已有缓存/消费处理保持不变。公开 bridge0 例是 GREEN，锁定 preflight 首错，而不是 Flow RED。同样不改 preflight、ratio、volume accessor 或其他算法，不建立 `tools/flow-oracle/` 或任何原生 harness。

## Rust destination 与调用点审计

- `crates/ares-core/src/project_slice/perimeters/flow.rs` 是 Flow owner；必要时将共享 spacing 算术/错误和 with_spacing 移到 `flow/spacing.rs`，修改文件 <=400 LOC。保留现有 float 操作顺序与舍入。
- `perimeters/types.rs::Flow` 是 Flow.hpp 的临时兼容 shell，不是另一套流量模型。with_width / with_height 对应 `Flow.hpp:92-99` 构造路径；仅其非正 spacing 失败复用同一 NegativeSpacing owner，其他既有 volume 检查/诊断不变，不引入新的提前消费。
- with_spacing 返回 Result。`group_fills/params/projection.rs` 对 solid/bridge 保存 `params.spacing = f64::from(params.flow.spacing)`；固定 materialized 调用的 sparse non-bridge InternalInfill 分支直接用原 flow，其余 solid/bridge cast 回 f32 为原 spacing。即使投影保留原 spacing，也不在生产代码用 expect 提取 Result；materialized_flow 统一返回 Result，已有 Result 调用链直接传播 ?，不新增检查或改变算术。
- `fill_entities/monotonic.rs::append` 的 generated.spacing 可被几何调整，确实可达失败；与固定 spacing 共用 materialized_flow 并传播 ?。`concentric.rs::append` 只接 `group_fills/narrow.rs` 的 InternalSolid → ConcentricInternal，原 spacing 也在已有 Result 返回路径直接用 ? 传播，与源 FillBase 的 with_spacing 调用一致。append_configured 的 full-density sparse Arachne 路径使用 nominal sparse spacing 与实际 flow，不可套用 solid 同-spacing 假定；该直接调用保留真正 fallible 边界，不修改其几何生成算法。
- 对应源 `Fill/FillBase.cpp:133-184::Fill::fill_surface_extrusion`：非空路径才消费旧 volume；非 internal flow 才 with_spacing 并消费新 volume；Arachne 另有 with_spacing。保持条件、首错及既有 volume 逻辑，不在空路径提前制造错误。`with_flow_ratio` 签名保持简单。
- 从纯 `slice_project_plate_processed` / profile processing 返回 SliceError；不 catch_unwind、不吞错、不跳过区域、不返回旧 flow、不产出部分 G-code。不调用 Print validation；validated 对原 S1 仍先返回 O4 `Too small line width`，settings/placement 不新增 gate。

## Test-first 与退出条件

顺序为 spec → 真实 RED → 预写控制 → 实现。原 `tests/parity/validation/minimum-width-ender3s1plus-0.2.3mf` SHA256 `edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110` 无 overrides 的纯处理返回精确 with_spacing 错误且无 G-code；narrow width 的公开输入返回精确 NegativeSpacing；bridge0 保持 preflight 首错；合法 flow 产出完整 G-code，validated S1 保持 O4 首错。不把 native CLI 的 O4 exit205 当成 Flow oracle，也不把前置拒绝或控制测试自身错误当成 Flow RED。

不新增 catch_unwind（测试或产品）；panic 由失败测试进程和调用前保存的 input/invocation 记录。测试自己的 kept tempdir 不依赖 artifact 环境变量。保持既有 float-bits 断言；Result 的必要机械提取不放宽断言。不新增不可能场景的隔离测试/错误处理。

合法 flow 与 G-code 字节必须不变。workspace nextest、严格 all-target/all-feature Clippy、fmt、WASM 全绿；当前 base 独立 official/seeded 双侧对照与共同 Ares 字节一致，参考残留原样保留。Core 不增加文件系统、终端、UI/OpenGL 或平台特有行为。延后 NegativeFlow、MissingVariable、new_from_config_width(height <=0) 的相邻 InvalidArgument、其他 cross-section/role/bridge 算法和完整 Flow 类 parity。执行日志和统计放外部证据/commit message，不放本 ARD。
