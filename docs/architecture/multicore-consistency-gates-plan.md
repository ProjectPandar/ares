# 多核一致性基础设施实施方案

约束见 [ARD](multicore-consistency-and-native-affinity.md)，本计划不授权并行化算法。

## 一、当前事实与明确边界

调查结论：

- `ares-cli/src/main.rs::Commands` 只有 `Slice`；已有 options、overrides、plate、
  resources、load-settings、load-filaments和dump-moves入口，没有3MF导出。
- `orca_parity` 静态候选中136个文件引用 `CARGO_BIN_EXE_ares`；另有大量公共
  API-only调用，包括 profile、processed、不同seed/metadata、Plater edits。
  这些只是扫描事实，不是用例数/去重数，也不是完整运行覆盖证明。
- `runner/cache.rs` 的 export步骤是 Native，不是 Ares exporter。不能复用它的
  3MF产物冒充本门禁中的 Ares输出。
- `runner/cache_identity.rs::identify` 当前记录父进程 available_parallelism；
  `scripts/ci/pin-runner.sh` 当前测试/Native共用1CPU，不能直接改父mask后仍用该值。
- 官方启动入口、seeded入口、`runner/command.rs`、processor diagnostics、
  Node Native kernels以及 Python/GDB observer必须分别检查，不能只绑 seeded。

**包含**：轻量公共请求捕获、覆盖率 ledger、独立 CLI/API replay程序、严格逐字节
比较、Linux小集成门禁、分离绑核模式、Native身份/热cache不变证据。
**不包含**：3MF导出、任何 slicing并行实现、算法重构、Rust parity行为修复、
Native配方重写、ROOT `orca_parity.rs`注册改动、断言放宽、全量输出归一化。

## 二、最小实现布局

|位置（拟定）|职责|
|---|---|
|现有 `orca_parity/runner` grouped adapter中的新小模块|记录真实CLI/API参数和来源，复用现有artifact写入能力；不在ROOT添加模块|
|`crates/ares-cli/examples/multicore_replay.rs`及按需小模块|独立测试程序；CLI请求执行真正ares；API-only请求只调用公开ares-core入口，序列化公开返回结果|
|`scripts/ci/multicore-consistency.py`|CAS/ledger完整性检查、固定五次affinity执行、原始产物比较与summary；不调用Native|
|`tests/parity/multicore-consistency/`|精简子集的显式请求manifest，引用已有authored inputs/config；不存Native输出或expected G-code|
|`scripts/ci/pin-runner.sh`|选择授权CPU子集给Ares测试，导出其中一个Native CPU；原单CPU模式和新分离模式明确区分|
|`scripts/ci/pin-native.sh`（拟定）|内部taskset/exec，仅约束Native；可单独查询进入实际子mask后的CPU数，不污染普通stdout/stderr|
|`runner/cache_identity.rs`和Native调用适配层|查询同一Native执行策略的实际CPU数；不把父Ares核数放入Native producer key|
|Linux Tier1与批次命令|CI精简集、批次全量集；记录coverage、mask、hash、首差异与Native cache证明|

这是测试/adapter代码，不向 ares-core 注入文件I/O、环境追踪、mock或私有访问。
复用当前依赖和artifact代码，避免新服务、数据库、通用工作流框架或重放另一套
slicing pipeline。新模块达到400行前按请求族拆分，不建巨型dispatcher。

## 三、捕获格式与覆盖率闭环

1. 对已执行的每个公共操作保存原请求：入口种类、输入bytes/格式/语义文件名、
   原options/overrides、preset顺序与内容、所需resources内容树、plate、placement、
   metadata、seed；stateful API保存原始输入及公开edit序列，不能序列化私有指针。
   对私有字段的GenerationMetadata/SlicerContext，只在构造调用处记录原时间六元组
   和seed，重放仍调用原公开构造器。找不到原参数则BLOCK，不能额外开放私有状态。
2. 以长度分帧的内容hash去重；多个callsite关联同一请求，不丢失来源和调用次数。
   相同模型但不同配置、顺序、入口、seed、metadata或edit序列不是同一请求。
   输出位置和门禁mask只是重放控制，不成为制造不同case的噪声。
3. CLI适配函数读取真实Command的公开参数/设置、保存依赖，再调用真实binary；
   API适配器保存调用前的原始参数，然后原样执行原公共API、返回原结果。
   不改变旧断言、fixture、发生错误的阶段或Native可选性。
4. 用源调用清单+nextest登记测试清单审计全部CLI/API入口。包装共享helper不足以
   覆盖直接Command/API调用；别名、宏、动态输入及多次调用都进入ledger核对。
   新的未适配调用、缺失语义参数、临时文件已消失或不完整快照都BLOCK。
5. 完整ledger绑定source/build身份并记录每个来源的request集合、次数、outcome、
   测试完成状态；条件分支的不适用须明示依据。Native缺失导致的早退不是不适用。
   对于非slicing的公开API也记录其真实返回值；数据构造器分类说明，不当作独立
   输出job。不能把难表达的真实公共调用当helper排除。
6. 原测试对Native的assertion/skip语义不改。采集控制程序负责把不完整覆盖判为
   BLOCK；不因测试进程exit0就宣称全量捕获。采集仅搭载集成者本来要跑的FULL，
   不为作者额外启动一轮全量或重复Native sweep。

成功的冻结corpus在无Native的环境仍可重放；无corpus且缺Native不能从空采集
得到PASS。CI子集直接使用已有authored输入，不等待Native生成3MF。

## 四、输出与首差异

- CLI记录exit/outcome、实际输出文件集合及原G-code；API `Vec<u8>`返回值原样保存。
- ProcessedGCode等返回对象完整投影公开字段。已有serde实现能无损表示时复用；
  否则测试适配器用公开字段/方法生成版本化表示。浮点保存宽度和原位模式，不能
  把NaN转null、负零转零或f64变f32；sequence/vertex顺序不改。
- 枚举match穷尽、公开struct可穷尽时不使用忽略剩余字段的展开；新结果字段/变体
  必须适配，不能静默漏掉。不能为了序列化访问私有内部或改业务算法。
- 只有捕获时真实预期为admission/error的请求才允许相应失败输出。运行超时、
  panic、missingbinary、缺产物、未知request kind是基础设施失败，不能“各核都错”
  就PASS。基线成功产物必须非空，所有case和产物类别有计数。
- 比较器从offset0开始流式逐字节比较，EOF也是差异；输出第一个差异offset、
  双方长度和值/EOF、有限十六进制上下文、请求ID及CPU标签。结构序列化另给
  字段路径帮助定位，但不替代原始bytes失败。
- 不复用Native generator-only比较器，不去header/time/comment/config/footer，
  不按近似float、几何集合或排序后的路径判相同。

## 五、门禁 A 调度与 CI 子集

每个请求在固定可见文件路径/cwd、同binary和依赖下依次运行：
`1 → 2 → 7 → 16 → 16-repeat`。复制产物后重用原可见路径，避免临时目录差异
成为模板输入。输出不跨执行复用；基线也必须新跑。结果逐一对新1CPU基线比较。

只使用调用方授权集合与继承affinity的交集；每次查询实际可用CPU数/集合，
核数不足明确BLOCK，不扩展到宿主全部CPU。当前单CPU许可不够跑矩阵，实现前
需另外分配至少16CPU或由获分配的远端CI/集成执行。执行矩阵不代表性能证明。

批次全量消费已闭合的corpus；CI清单是独立、明确的小集合，五种运行仍全部保留，
不把7/16CPU删成“精简”。拟覆盖以下现有请求形状，最终ID由捕获确认：

- 最小STL+options；stock3MF与第二plate；多对象/多instance；
- Arachne/经典墙、支持/bridge等按层路径；
- profile/preset叠加与ordered load-settings；
- processed public result与Plater edit序列；
- epoch/seed输入及显式admission结果。

子集只能减少请求数量，不能取代全量覆盖ledger。清单增删给出来源/形状覆盖
变化，不准用只选当前通过case的方式隐藏失败。

## 六、门禁 B 的分离策略与 cache 证明

优先使用外层纯绑核Native启动脚本，原官方/seeded/诊断launcher和ELF/resources
字节保持不变，避免改wrapper hash后谎称旧cachekey不变。Native bin仍按原身份
校验和记录；外层脚本仅改变执行affinity，正常执行不能增加stdout/stderr字节。
只去除本机制新增的Ares-only核数/捕获控制变量，不改原Native语义环境；shell执行
副作用也须纳入真实启动验证，不能假设新增一层脚本必然保持Native字节。

`pin-runner`用继承CPU集合分配每测试K CPU，并选一个包含于子集的Native CPU。
nextest测试并发和K分开配置，资源使用不可通过重叠/超售伪装成核数满足。
查询Native CPU数和实际exec使用同一脚本/策略；在已经收窄的子mask内查询，
真实Native执行保留exec前或子进程affinity证据，不只相信父环境变量。

适配所有直接Native调用：Rust stages/diagnostics、官方/seeded入口、Node probe，
以及Python/GDB observer链。Ares CLI/API replay不走Native helper。不要把整个
Ares测试重新钉回1CPU来满足旧observer的单CPU前置；只约束Native/tool链。

cache验收分为两种、官方与seeded均做：

1. **既有entry热命中**：同输入/配置/producer，基准模式填充一次；保留同一个
   cache目录，父进程由1CPU变KCPU后命中原entry。producer components/key、
   request key、Native parallelism=1、所有cached payload hash必须相同。
   不许清cache、换schema、改请求或重建来掩盖身份变化。
2. **真实启动等价**：两模式都真正启动Native，并记录实际子mask=1；使用已有
   无隐式时钟的公共输出逐字节比较，记录原argv、binary/resources/launcherhash。
   固定同一个Native CPU做定向对照，避免把CPU迁移噪声归因给Ares核数。
   既有时间字段如发生差异原样报告，不新增比较标准化。

若实现必须修改原producer launcher文件，launcherhash天然会变；不得先改后
声称命中旧entry。应停下审阅策略是否能保持纯外层affinity封装，再决定而非
删除cacheidentity字段。FULL需要Native，缺supplier或实际Native不是1CPU则失败。

基础设施先不改默认FULL核数。第一处并行切片落地时，同批次将FULL默认Ares K
设为大于1（建议4），之后永久保持；Native继续1CPU。
两道门禁的批次证据必须随该并行切片一起进入合入审查。

## 七、TDD 与实施顺序

|切片|先写/运行的真实公共 RED 场景|最小 GREEN / 证据|
|---|---|---|
|1 捕获CLI|现有最小CLI case没有可重放请求/ledger；捕获遗漏参数必须拒绝完整声明|透明捕获一个真实CLI请求，原结果/断言未变；内容去重仍保留两个来源|
|2 捕获API-only|非默认seed/metadata、processed结果或Plater edit缺字段时不能重放/闭合|只调用原公共API的适配；完整返回值位模式，新增入口未适配则BLOCK|
|3 独立replay/compare|用真实程序比较两份有独立字面首差异的文件，必须报指定offset；真实case无完整mask则BLOCK|流式首差异及同mask实际执行；不使用mockCPU或虚构16核通过|
|4 矩阵/CI子集|缺请求、0cases、缺requiredCPU、Native缺失造成空采集均不能PASS|授权环境下真实五次执行；无Native但完整corpus可以独立完成|
|5 分离pin与identity|父K导致Native多核或producerkey变动，定向cache证据失败|父K/Native1、旧entry热命中、Native原始产物不变；覆盖直接调用链|
|6 集成接线|子集不能冒充全量；首次并行切片未切FULL默认则不准合入|批次两门禁证据+CI子集，source/binary/request/mask/hash/report可复核|

每步先有定向公共失败证据再实现，不用private函数单测、mock或复制算法制造
expectation。新增名称描述行为，不重命名现有case/fixture。编排循环属于独立
批处理程序，不把测试断言改成计算出来的期待值。

格式/检查：变动Rust后使用 `cargo fmt --all -- --check`、
`cargo clippy --workspace --all-targets -- -D warnings`；定向Rust测试用nextest。
全量、16CPU矩阵及Native身份对照由独立的集成运行执行；证据缺失就是阻断，
不靠文档宣称完成。

## 八、实现退出标准

本文档覆盖ARD、范围、两个门禁、轻量捕获与ledger、Native身份保持策略、
未来默认切换规则；无运行门禁或已支持并行的主张。

实现退出需同时具备：全部调用来源闭合的corpus/ledger、独立可复现请求与报告、
全量五运行一致性、CI子集五运行、父K/Native1与热cache不变证据、原parity语义及
ROOT未改、相关格式/lint通过。3MF明确deferred，不作为当前PASS或隐式漏项。
性能提速、跨平台位级等价和任何具体并行算法都另作source-cited切片审查。
