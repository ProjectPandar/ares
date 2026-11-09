# ARD / spec：PresetBundle 源身份准入

状态：方案。此次只定义源码重写切片，不包含生产实现。

## 1. 起点与源码归属

上游固定为 OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`。
Rust 起点为 `8ba63961f0c42cea7b798b8d7e4082a88e334297`。

| 上游位置 | 本切片的含义 |
| --- | --- |
| `src/libslic3r/PresetBundle.cpp:556–650::resolve_preset_config` | 按类型选择 collection；同文件身份匹配；歧义拒绝；替换 canonical config；是否允许 source manifest。 |
| `src/OrcaSlicer.cpp:2085–2116::ensure_cli_preset_bundle/ensure_system_preset_resolver` | user 的已加载 bundle 与 system 的 source resolver 不是一个状态；同一次调用复用状态。 |
| `src/OrcaSlicer.cpp:2118–2161::resolve_preset` | typed 且无 inherits 的提前返回；其余输入选择 user/system 规则，再调用上面的成员函数。 |
| `src/OrcaSlicer.cpp:2202–2278::load_config_file` | 原文件 metadata、解析次序、失败诊断和 canonical config 的使用。 |
| `src/OrcaSlicer.cpp:2281–2433` | machine/process 及 filament 文件循环调用解析器；失败先结束，不进入切片。 |
| `src/OrcaSlicer.cpp:1803–1807` | 3MF 内 post_process 先于 CLI preset 文件的准入，维持错误优先级。 |
| `src/libslic3r/PresetBundle.cpp:655–688::load_source_vendor` | manifest 路径的 canonical vendor/library 加载依赖，不是按父名称任意找文件。 |

这不是新增 Ares pipeline。现有 `profiles/load_settings.rs` 把 CLI 文件
当作独立 import，`Catalog::enable_direct_imports` 全局绕过父链，不能代表
当前 beta 的 `PresetBundle`。本切片用上游逐文件准入替换这个豁免壳；
其后的已有组合、项目加载及 Print 生命周期不另建一套。

`native-cli-preset-input-transport.md` 只规定另一批测试的输入传递，
不授权把本切片的原 user 文件写入 Native datadir、补 version、去掉 inherits
或改成 system snapshot 来制造成功。`ard-cli-load-settings-filaments.md`
的 core/CLI 职责继续适用；旧 direct-load 源码说明只能用于 typed 无 inherits
的分支，不能再推出“所有 CLI 文件都忽略继承”。

## 2. 已有证据与缺口

测试切片 `e5e7363c2bdebd14419678e1308b2849c9d6c858` 保留原
`load_settings_3mf_filament_inherits_no_chain` 的文件、3MF、参数和成功流比较。
官方与 seeded 都是五个控制通过、目标失败：Native 拒绝原 child filament，
Ares CLI 接受并写 G-code。实测进程码为 Native 251、Ares 0；这些是观察，
不是新测试常量，也不代替 `CLI_CONFIG_FILE_ERROR` 的源码语义。

该差异表明：带 inherits 的独立 JSON 并不等于已加载的 preset。
是否能找到父名称，也不能替代对输入源文件身份的匹配。

新的 live Native 前提已到达真实退出码差异；之后的通道/存在性断言及
原成功流完整比较在这个见证上未执行。不能称为目标完整 G-code 一致，
也没有目标 API/browser、spec 新提交运行或组合/main 信用。

## 3. 不可协商的决定

1. 准入归 `ares-core::profiles::load_settings::compose_load_settings`，
   为 `PresetBundle::resolve_preset_config` 的数据逻辑重写。core 无文件 IO、
   目录扫描、环境读取、进程、terminal 或 UI；WASM/Windows/macOS/Linux 同一逻辑。
2. CLI adapter 读取字节并提供源身份与 bundle 数据。源身份表达
   `boost::filesystem::equivalent` 的同文件关系，不是 JSON name、inherits、
   basename、内容 SHA 或仅对路径作 canonicalize。符号链接/硬链接的同文件
   关系由 host adapter 求得；不同文件即使内容相同也不自动获得相同身份。
3. core 在输入身份与真实已加载 collection 的记录间匹配，不把 CLI 叶子本身
   自动加入 collection，也不借 `add_import` 的成功制造“已加载”。没有源身份的
   默认记录对应上游空 `preset.file`，不参与匹配；不能让两个空身份相等。
4. `User`/`user` 不允许 source-manifest 路径。`system` 才按源码选择它，
   并且不能借用 user bundle 的记录放行。无匹配不是按 name 再试一次的理由。
5. 唯一匹配消费该记录的 canonical config；不能只验证身份后继续使用未解析
   的 CLI leaf，也不能把 leaf 重新覆盖回 canonical config。
6. 原文件 `name/from/inherits/filament_id` 描述与解析结果分开。源码在解析前
   取得 `config_name/config_from`，不能用 system parent 的身份改变 user 的
   CLI source status。既有 `PresetDescriptor` 与既有 config membership/defaults 切片的分组语义不改写。
7. 不增兼容模式或回退开关。删除全局 `direct_imports` 的准入豁免；
   typed standalone 或解析成功后的逐文件 config 仍可复用已有单片段组装，
   但它不再是继承输入绕过 resolver 的后门。

## 4. 精确的行为顺序

本次生产目标限有显式 `machine/process/filament` 类型的文件：

- 按原顺序读取/解析文件与 `from`；3MF 准入仍先于 preset 文件。
  每个文件先完成解析/准入，再判断 machine/process 重复；前一失败获优先权。
  空 filament 槽位仍按原规则处理，不移动到新的切片阶段。
- typed 且 inherits 缺失/为空，按 `resolve_preset` 提前返回原 config；
  不能要求它存在于 bundle，也不为它触发完整 vendor 加载。
- 其他 typed 文件按 `from` 选择 collection/context。按上游 collection 的
  实际已加载记录检查身份，忽略没有文件来源的记录和无法证实等价的候选。
  不使用仅按 name 去重的 `selectable` map 作为歧义判断的替代品。
- 一个匹配：用该记录完整的 canonical config，清除本次解析错误后继续。
  第二个匹配：立刻返回 `Preset identity is ambiguous`，不能挑第一项，
  也不能转去 source manifest。
- 无匹配且不允许 source manifest：返回
  `Preset was not found in the loaded bundle`。
- system 无匹配时，CLI 给出从源文件父目录向上、以 sibling manifest
  为依据的候选数据。祖先遍历基于 `absolute(source_file).lexically_normal()`，
  不是跟随 symlink 后的真实路径；身份等价与 manifest 所在位置分开表达。
  遇到源码中第一个可用候选，复用 canonical vendor
  加载和 library/子路径语义；只在其对应类型 collection 中再次查源身份。
  loader 失败或文件并非实例化 preset，保留错误并返回；不继续向更高祖先
  或任意同名父 preset 搜索。没有候选仍是 loaded-bundle not-found。
- 只有完成这一准入才进入既有 machine/process/filament 配置组合与切片。

`resolve_preset_config` 开头的两个 compatibility-rule 映射也属于
resolver 的规则；复用现有 option/catalog 解析政策，不新建另一套
unknown-key/nullable/vector 解释器或用户可调的规则开关。

## 5. Rust 目的地与数据接口

| 目的地 | 最小职责 |
| --- | --- |
| `crates/ares-core/src/profiles/load_settings.rs::{LoadSettingsFile,LoadSettingsRequest,compose_load_settings}` | 在原文件字节/label 旁携带源身份；接收可借用的已加载 catalog 与 source-vendor 数据；实现第4节规则。解析 label 不承担同文件判断。 |
| `crates/ares-core/src/profiles/catalog.rs::Record` 及 `catalog/{store,compose,chain,document,includes}.rs` | 对真正进入 collection 的记录关联源身份；复用原有 vendor/import 文档解析和 canonical 层合并，不把所有 raw definition 当作实例化记录。不新建并行的配置 registry。 |
| `crates/ares-core/src/profiles/entry.rs::{LoadSettingsSliceRequest,with_load_settings_source}` | 把同一份数据上下文传给既有 compose；core 入口同样复用现有字节准入来守住3MF优先级；原文件描述符仍供已有 compatibility 路径使用，不新增切片阶段。 |
| `crates/ares-cli/src/main.rs::{run_slice,read_preset_files}` 或其小型拆分文件 | 从实际文件及既有目录/catalog 数据构造字节和源身份；标明实际已加载记录与 source-manifest 候选，不在 CLI 自行合并 slicer options。 |
| `crates/ares-core/src/lib.rs::SliceError` 与 `crates/ares-cli/src/failure.rs::slice_error_to_failure` | 最小的 preset-resolution 错误类型，保留 file label/reason；仅 adapter 把它映射为上游 config-file 失败协议，不能用字符串猜错误类别。 |

源身份只作元数据，不能流入 `ProjectSettingsRaw`、动态 option registry
或 G-code footer。core 不设计各 OS 的文件 ID 编码；host 先建立同文件
等价类，再传给 core。浏览器由输入资源/目录会话提供同一关系；不能用
内容 hash 混同两个不同源文件，也不能让 core 为验证来源自行访问路径。

同一次调用构造/解析上下文一次，按引用选记录，沿用现有 chain/option
合并；不为每个 filament 复制整个 catalog 或解析整个 vendor 树。只在
上游需要替换 config 的地方生成该文件的结果。需要拆分时模块不超过400行。

### 数据生产前置条件

上下文必须来自真实数据生产者，而非一个总为空的 stub 或测试名字表：

- user inventory 对应 `src/libslic3r/PresetBundle.cpp::load_presets` 和
  `src/libslic3r/Preset.cpp::PresetCollection::load_presets` 的已加载状态；
  原 input 不在其中时才是
  not-found。同 JSON name/父名称已存在都不等于同源文件已加载。
- system source context 对应 `load_source_vendor` 的 JSON manifest、
  `m_preserve_vendor_source_paths` 和 canonical vendor/library 路径。
  可复用既有 `PresetCatalogHandle` 的 catalog 逻辑，但必须补足 provenance
  绑定及准入检查，不能把“任何能 push 的 JSON”当作上游 loaded preset。
- CLI 的目录来源与 `OrcaSlicer.cpp:8236` 的 `set_data_dir`、上述 bundle
  producer 对齐；Native datadir 与 resources_dir 不是同一个概念。
  若当前 adapter 不能产生这些数据，实施计划须先列出相应的 source-cited
  producer 子切片；不把缺上下文解释为已证实空 bundle，不硬造准入或拒绝。

本 spec 不承诺新的 CLI 配置开关。显式数据目录、默认目录及 app-config
读取是否需要新增 adapter 入口，必须按上述上游数据来源补齐计划与源码
引用后再实施；不能用宿主个人目录、Native 输出或 fixture 特例当生产源。
adapter 可复用 core 的同一份轻量文件描述读取来确定数据需求，不复制
resolver 规则；standalone 不应因无关 bundle 加载错误而失败。实施计划
必须明确数据准备次序与生产者，不能留下“稍后再补来源”的可运行壳。
数据生产子切片须先有独立的 source-cited 计划；本 spec 不授予任何实现许可。

## 6. 包含与延期

包含：typed 文件的 early return、三种类型的 identity/ambiguity 准入、
user/system 规则、canonical config 选择、source-manifest 的数据选择契约、
原文件 metadata 保留、最小错误/CLI 协议连接和既有 core compose 复用。

只有原始 vendor JSON/manifest 数据生产者在本轮证据中具备条件时，才给
system source 路径运行信用。完整 vendor 发现、安装选择、app-config 全量
行为、`.opc` 缓存读写及跨调用缓存生命周期不是本 spec 的实现许可；其
数据生产缺口须在实施计划中显式列为依赖，不能用旧 direct-import 壳代替。

延期：`resolve_preset_config_type` 的缺 type 探测、SLA 类型、
`resolve_system_preset` 的 name/vendor 搜索、`cli_different_settings` 导出
列扩展和完整库级继承新行为。既有 catalog 已有父链能力继续复用，不重写。

不包含既有 config membership/defaults 切片，或 acceleration/jerk、J1、minimal-purge
materialization，或其它 process vectors、prime、FlushMode、geometry、
运动/time/statistics、viewer、UI 行为。若正确的 config 替换暴露这些后续
差异，逐项保留并归还其 owner，不在准入层补默认或改输出比较。

## 7. 失败模式与先写 E2E 的次序

生产代码前先保留原见证和控制，并为源码分支写可重复的 E2E：

| 先写的见证/控制 | 必须排除的错误实现 |
| --- | --- |
| 原 e5e child-filament 与原五控制，双模式原输入重跑 | 总是接受、硬编码251、替换 fixture、跳过 Ares、破坏 standalone 成功流。 |
| typed standalone 无/空 inherits，三种类型 | 为所有 JSON 强制 bundle 准入，或无故加载所有 vendor。 |
| 带 post_process 的原3MF与未加载 user 同时存在；重复 machine/process 的后项解析失败 | 错误优先级倒置；CLI 先过但 core 入口先后顺序不同。 |
| 未加载 user 与同 name/同字节的不同文件；parent 存在与不存在的独立新控制，但源文件始终不在 loaded 集合 | 只看 name、内容 hash 或 parent 是否可解析；不得改原见证文件。 |
| 位于真实加载目录中的 user，名字与文件名不同也按源身份接受；先让 Native 证明 canonical 解析确实发生 | 把所有 user 一律拒绝；把 CLI leaf 自己加入已加载集；按 JSON name 放行。 |
| 同源 identity 的符号链接/硬链接与两个实际已加载记录形成歧义的见证；另测指向 vendor 文件但自身位于 vendor 外的 symlink | 路径文本比较、canonicalize-only、第一项优先、歧义后去 manifest，或把 manifest 祖先误换成真实路径。Native 未到达分支就不得称已覆盖。 |
| system manifest 列出的实例化 preset、非实例化/未列出的文件、最近候选失败 | 任何邻近 manifest 都放行；只因存在文件便接受；向祖先或别的 vendor 再找同名项。 |
| unique match 后完整配置与原 source descriptor 的观测 | 只过身份校验却不使用 canonical config；把 user 变成 system source；丢 vector 元素。 |
| 错误与成功的同一 public core 入口/CLI；browser 提供对应字节+身份数据 | core 偷做文件 IO、CLI 独立合并、以退出码一致代替完整结果、以控制执行代替目标执行。 |

正向依赖需要独立 Native datadir 时，只给新增控制使用隔离、源码有据的
数据生产者；原 e5e 见证、原 Native flags/seed/cwd 方案和文件不动。
不能把新控制成功迁移成原目标成功。先记录有效 Native loaded-state/
manifest/identity 证据，再认定它是正向或歧义控制，不给空操作信用。

错误以实时 Native status/stdout/stderr 为准；core 检查对应的 typed error，
CLI 检查全部通道及 G-code 存在性。`CLI_CONFIG_FILE_ERROR` 的固定语义来自
源码，不从一次观察生成常量。`OrcaSlicer.cpp:485–502::flush_and_exit` 与
`:535–590::record_exit_reson` 归 adapter；Linux 与其它平台的协议差异
保留。原见证未生成 Native result.json；保存存在性和原字节，不替 Native
造 JSON。若新增控制覆盖该文件，按其真实输出目录及源码条件逐字段核对，
时间等差异也保留，不能新增归一化或默认字段来凑一致。

成功保留原完整 G-code 比较和唯一的 generator-line 例外；原 fixture
完整性 hash 只校验输入。输出 hash/count/exit 不能变成新语义常量。
API/CLI 各自真实调用，保留参数、退出、配置、原输出与 raw diff；错误前提
通过不等于完整测试通过。只读反例和外部审计错误不能被生产 patch 掩盖。

## 8. 实施前与交付核对

1. 先核对本 spec、具体输入数据生产子切片和新增 E2E 失败证据，再给生产
   变更范围；当前没有生产修改许可。精确修改点仍以第5节为准。
2. 最小 core admission/adapter 连接不得携带无来源 stub、名字白名单或
   fallback；保留所有既有 case 身份及原比较，范围外失败不隐藏。
3. 各提交独立 `cargo fmt --check`、
   `cargo clippy --locked --workspace --all-targets -- -D warnings`；
   使用 `cargo nextest run` 进行有界 Rust E2E，不以编译代替执行。生产入口
   变化另做 release browser-WASM 实际调用，跨平台编译与真实执行分开列。
   浏览器调用必须实际到达 `slice_with_load_settings` 的字节/身份数据入口；
   未经过本 resolver 的 `sliceWithProfiles` 成功不计此切片。若现有 binding
   不能表达该输入，先列 `ares-wasm` 的薄绑定子切片，不把解析逻辑移入 web。
4. Native CPU13、Ares28–31/jobs4，原 source/profile/binary 字节绑定、
   独立精确 tip、前后逐身份结果、P→F/F→P、全通道/输入/输出/diff/哈希及
   receipt 绑定。原证明和已接收交付不改写，不运行未经授权的 FULL。
5. 只有原目标完整实时错误协议一致，才能称该准入差异修复；成功控制的
   完整比较另计。新 spec 提交本身没有生产运行、全家族或 main 一致性结论。
