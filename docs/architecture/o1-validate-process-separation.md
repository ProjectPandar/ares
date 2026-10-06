# ARD — Print 校验与处理的独立生命周期

## 上游依据

- OrcaSlicer v2.4.2 `src/libslic3r/Print.hpp:911,935` 将 validate 与 process 声明为独立公开步骤；`Print.cpp:1262` 实现 validate，`:2175-2644` 的 process 不调用 validate。
- CLI `src/OrcaSlicer.cpp:6040-6063` 在 apply 后设置 BBL 状态并 validate，`:6146,6156,6161` 才调用 process。
- GUI `src/slic3r/GUI/Plater.cpp:8036,17716` 调 BackgroundSlicingProcess::validate；`BackgroundSlicingProcess.cpp:678-684` 设置 preset BBL 标识并委托 Print 校验，`:230,281,445` 的线程处理独立于校验。

## 不可协商决策

1. ares-core 提供独立公开的 Print 校验与纯处理 API。纯 slice、project/profile plain/processed、run_slicing_pipeline 不承担 O1 业务门控；已有配置解析、类型验证及算法必要前置条件保持不变。
2. CLI/WASM 外部切片边界负责 validate-before-process，且只调用一个 core validated 组合入口。组合入口在同一已加载、已应用配置的 Model/Project 上校验后处理，失败立即返回，不输出 G-code；不得先后调用两个 byte-in API 重载输入。
3. STL 组合入口共用 pipeline 的唯一 model-load seam，借用已应用的 options 校验后继续处理同一 Model。3MF/profile 在 state 的 load、plate-select、resolve seam 借用 Project 和 resolved config 校验，随后沿原 stages 移动同一状态，不再序列化、加载归档或解析有效配置。
4. profile 在同一 compose/load 路径应用选定预设、placement、overrides；standalone validate、组合切片与 settings-view 共享该路径。settings-view 不切片、不承担 Print 校验。调用方匹配 Some(profile) 后显式传入，不以 expect 推断选定 profile 存在。
5. validate 与 process 使用相同的 plate、overrides、profile 与有效配置。模板保持原始文本，不渲染、不插入或删除 G92、不修改默认值、stock preset 或原始模型输入。
6. O1 仅重写 `Print.cpp:1695-1749`，统一返回 Result<(), SliceError>，以 InvalidInput(String) 适配 `PrintBase.hpp:30-39::StringObjectException` 的精确文本。object/opt_key、其他 Print 校验及 O2/O4 延期，规则不复制到 adapters。

## Rust 边界

- `ares-core/src/print/validate.rs` 拥有 O1 规则与已应用配置的校验；identity 子模块对齐上游 BBL 身份判断。
- 公开 standalone 入口为 validate_print、validate_stl、validate_project_plate、validate_with_profiles；validate_print 借用 ProjectSettings 与调用方确定的 BBL 状态。
- 公开组合入口为 slice_stl_validated、project plate 的 plain/processed validated、profile 的 plain/processed validated。内部 source 合同位仅选择本次调用是否先校验，不进入用户配置或 G-code，不是 feature toggle、默认豁免或 fallback。
- `project_slice/state.rs`、`project_slice/profile_entry.rs` 与 `pipeline.rs` 复用原处理链。现有 Print/STL scaffold 是上游 Print 生命周期的临时兼容壳，不设计独立 Ares pipeline。
- `ares-cli` 只承担文件/终端适配，`ares-wasm` 只承担字节/绑定适配；四个 Tier 1 平台共用 core 规则与组合生命周期。

O1 的正则、优先级、精确错误与 E2E 验收合同见 [规则 spec](o1-print-validate-extruder-reset.md)。
