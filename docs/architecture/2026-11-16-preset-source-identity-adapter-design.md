# Preset source identity adapter design

状态：实施前设计说明。此文只定义 source identity 的数据边界，不实现准入行为。

## 目标

将 `PresetBundle.cpp:556–650::resolve_preset_config` 的同文件判断带入
`ares-core::profiles::load_settings`，同时保持 core 无文件 IO、路径访问和
操作系统依赖。准入只接受 host 已证明与 loaded collection 记录等价的来源；
不按 preset name、parent name、basename、内容 hash 或路径文本回退。

## 数据边界

在 `ares-core` 增加不透明、可比较的 `SourceIdentity` 值。它只支持 equality
和 clone，不暴露路径、编码或平台文件结构。`LoadSettingsFile` 携带可选的
`source_identity`；loaded catalog 的每个真实 collection record 也携带可选
identity。core 只在同一 profile kind 且两侧 identity 都存在时比较它们；空
identity 不互相匹配。

`LoadSettingsSliceRequest` 增加 loaded-catalog/source-record 上下文。现有
catalog JSON 仍承载 preset 内容和 metadata；source identity 作为同一调用的
host-provided companion 数据传入，避免把 host 路径或文件 stat 写进 JSON、
`ProjectSettingsRaw`、动态 option registry 或 G-code。

## Host adapter

`ares-cli` 在读取 `--load-settings`/`--load-filaments` 文件时取得每个文件的
source identity，并为实际 loaded user/system collection 建立相同的 identity
记录。Unix 使用稳定的 device/inode 对；Windows 使用 volume serial/file
index；浏览器调用由资源/目录会话提供稳定的不透明 token。adapter 负责符号
链接/硬链接的同文件等价，core 不 canonicalize、扫描目录或读取环境。

CLI 读取文件字节与 identity 必须是同一次输入准备；缺 identity 的 external
user source 不得被名字或内容放行。system source-manifest 的目录发现和
vendor loading 仍由 host producer 提供，之后只把已加载 canonical records
和它们的 identities 传给 core。

## Resolver 行为

`resolve_preset_config` 的 Rust 重写先按 kind 和 source identity 查找 loaded
records：零匹配返回 source-not-found，多个匹配返回 ambiguous，唯一匹配
替换为 canonical config。`from: user` 不启用 source manifest；`from: system`
仅使用 host 明确提供的 manifest-backed records。typed 且没有 `inherits` 的
文件保留 `OrcaSlicer.cpp:2118–2123` early return，不触发 identity lookup。

准入失败使用专门的 core `SliceError` variant；CLI 只将该 variant 交给已有
`ares-cli/src/failure.rs::CliFailure::config_file_error`，其它
`SliceError::InvalidInput` 继续保持原 Legacy 映射。测试先覆盖 identity
唯一匹配、未加载同名/同字节文件、歧义 identity、typed standalone early
return，以及两个现有 inherited rejection witnesses。

## 实施顺序

1. 增加 source identity 和 loaded-record companion 数据类型，保持现有调用点
   可编译并明确空 identity 不匹配。
2. 在 CLI adapter 读取 preset 文件和 loaded collection producer 处填充 identity。
3. 在 core resolver 中删除 name-based admission，并保留 standalone early return。
4. 增加专门错误 variant 与 `config_file_error` 映射。
5. 按 source-cited E2E 顺序运行 Native/API witnesses，再运行全部相关
   `load_settings*`、`project_preset*`、CLI exit-channel inventory；官方和
   seeded 均要求 P→F=0。
