# ARD：3MF 未内嵌 filament 选择时采用加载数量

上游固定为 OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`。
本次只重写 `src/OrcaSlicer.cpp:1893,2528–2529` 的 filament count 选择规则，
不设计新的 Ares 配置或切片流程。

## 上游行为

3MF 导入阶段由 `current_filaments_name.size()` 得到 `filament_count`
（`:1893`）。读取显式 `--load-filaments` 后，如果这个数量为零，则采用
`load_filament_count`（`:2528–2529`）。内嵌数量非零时不执行该替换。

这里的替换是该上游调用点定义的正常行为，不是缺省 preset、旧实现或错误
恢复路径。没有加载 filament 时，数量仍为零；不能凭空制造一个 filament。

## Rust 边界

目标是 `crates/ares-core/src/project/domain/profile_input.rs` 中
`Project::from_3mf_with_profiles` 的 count 选择。当前代码在内嵌数量为零时直接
返回 `project embeds no filament selection`，因此显式加载 filament 不能到达
已有 overlay。用上游 count 规则替换这一不对应源码的拒绝。

沿用已有 selected-profile 输入：选中的 filament 数量代表本调用点的已加载
数量。内嵌数量非零时继续使用原数量和现有超量选择错误。只有内嵌数量为零
才采用所加载的数量。有效数量必须一致传入现有 per-slot overlay、identity/
expression/different-settings group 构造及 CLI-loaded project 初始化，不能
只绕过错误而留下零长度下游布局。

不改写原始 3MF 来制造内嵌选择。使用已有 public API、raw/typed config、
profile overlay 和切片入口；现有 scaffold 仅承接此处的上游 count 状态，
不是独立的 Ares filament 安装器。核心保持纯数据，CLI 适配器负责文件 I/O。
浏览器通过同一 core 路径使用该规则。

## 包含与延后

包含：

- 3MF 的内嵌 `filament_settings_id` 数量为零且显式加载一个或多个 filament；
- 内嵌数量非零时原有数量优先，保留现有超量选择拒绝；
- 内嵌和加载数量均为零时仍拒绝，不创造选择；
- 采用有效数量后，既有 overlay 和配置初始化收到同一个数量。

延后：

- 自动寻找 filament、安装/member/identity 解析、AppConfig、cache 和 vendor
  loader；挂起的 Preset normalization 切片保持独立；
- 上游 OBJ 自动配色引起的加载数量扩展、空 CLI 文件槽与 first-filament
  default 文件策略；
- 内嵌数量非零时，扩展现有多于内嵌数量的选择能力；
- 新的配置成员策略、preset vector normalize、数组到标量字符串处理、
  geometry、motion、time/statistics 和新公开 API。

## 测试先于实现

使用已有 `slice_with_load_settings` 公共 byte API，提交同一 3MF 与同一组
原始 preset 文件给 API 和原版 Orca `--load-settings/--load-filaments`。
从已有可重复的 3MF fixture 只把内嵌 `filament_settings_id` 设置为空数组；
保留其余配置和模型。源 fixture 与派生输入均保留哈希及可复现的生成方法。
缺失字段可能采用非空反序列化默认值，因此不能用删除字段代替零数量见证。

先保留一个 Native 成功、API 在现有零数量拒绝处失败的运行时见证。不能以
编译或进程启动失败代替。Native 和 API 成功时比较完整 G-code，只允许既有
的 generator 行例外；状态码、成功标记或单个 footer 值不替代完整比较。

覆盖以下失效模式：

1. 内嵌零、加载一个：不得早拒绝；应进入对应的一槽 overlay。
2. 内嵌零、加载多个：数量取已加载数量，不硬编码为一；各槽身份和配置按
   已有顺序应用，不截断 group 或 self-index。原单槽 fixture 搭配两个加载
   文件时，Native 因原 flush matrix 尺寸返回 `-100`、不输出 G-code；这组
   作为拒绝控制，API 应到达同一矩阵错误，不手填矩阵制造成功。单加载槽
   仍是完整 G-code 成功比较的主要见证。
3. 内嵌非零：不因加载数量少而缩短项目；原有 whole-output 控制不改变。
4. 内嵌和加载均为零：不隐式选择 default filament。
5. 选择超出非零内嵌数量：保留既有错误，不扩大此切片范围。

失败测试和生产实现分开提交。保留 Native argv、原始通道/返回状态、API
结果、完整输出及比较结果，不用手填预期配置或额外 footer 过滤。

## 验证出口

- 有界 Native official/seeded E2E 从上述目标失败转为完整输出相等，控制不退化。
- 执行 `cargo fmt --check`、
  `cargo clippy --locked --workspace --all-targets -- -D warnings`，Rust 测试
  使用 `cargo nextest run`。生产模块保持不超过 400 行。
- 实际 release-browser WASM 执行变更的公共 API 路径，并保留可核验产物；
  仅编译不是浏览器行为证据。
