# ARD：多核一致性与 Ares / Native 分离绑核

本文只有文档，不引入并行或实现门禁。

## 1. 决定与范围

相同请求在不同可用 CPU 数、不同调度及重复运行下，Ares 的当前公共输出必须
逐字节相同。准备两道独立合入门禁：

- **门禁 A：Ares 自一致性**。全量去重请求分别运行1、2、7、16 CPU，16 CPU
  再运行一次；均与新执行的1 CPU基线比较。批次跑全量，Linux Tier1跑精简子集。
- **门禁 B：分离绑核 FULL**。Ares 测试进程使用 K 个 CPU，所有 Native 启动
  路径只使用其中1个；既有官方/seeded FULL 比较、断言和缺 supplier 失败政策不变。

每个引入并行的上游 rewrite 切片，合入前必须同时具有这两道门禁的批次证据。
作者仍只做本切片的定向公共 RED/GREEN；全量门禁由集成批次承担，不要求作者
私自重复 FULL。CI 精简子集不替代批次全量。

**本切片不实现公共3MF导出**。Ares 当前 CLI 只有 `slice`，没有生产3MF writer。
比较 CLI 原始 G-code，以及 API-only 调用的返回字节、公开结果结构的无损序列化。
不能把 Native 导出的3MF、输入3MF或测试 fixture ZIP 当作 Ares 导出结果。
未来 Ares 对应上游真实导出边界后，才将其输出加入本门禁。

## 2. 上游依据与 Rust 边界

上游唯一引用基线为 OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`。

|上游文件 / 函数|可借鉴的具体契约|Rust 目的边界 / 本次处理|
|---|---|---|
|`PrintObject.cpp:PrintObject::make_perimeters`（520起）|TBB按层执行，层身份不是完成顺序|未来 `project_slice/perimeters` 对应源切片；本次只设合入约束，不改算法|
|`PrintObject.cpp:project_triangles_to_slabs`（5305、5330–5470、5475–5488）|预分配 `projections_of_triangles[idx]`，并行写各 triangle 的槽位，之后按 vector 索引顺序汇入各层|未来 painted-facet / mesh slicing 对应源边界；不得用完成队列决定 polygon 顺序|
|`PrintObject.cpp:PrintObject::detect_surfaces_type`（1941起，2239–2269的 extra-bridge 两阶段段落）|先收集 `bridge_polys_per_layer[i]`，barrier后各自修改 `m_layers[i+1]`，禁止相邻迭代读写别人的输入|未来相应 `project_slice` surface/bridge rewrite；端口前逐函数核对 owning boundary|
|`PrintObject.cpp:PrintObject::bridge_over_infill`（3235–3309）|从未修改输入收集 `needs_lightning_expansion[lidx]`，再执行互不别名的写阶段|未来 fill / lightning 对应源切片；不是引入新 Ares 调度管线|
|`TriangleMeshSlicer.cpp:slice_make_lines`（603–635）|并行 facet 写入会改变行顺序，上游明确按完整 edge/vertex/坐标/type/flags tuple 恢复定义顺序|已有 `mesh_slicer` / `segments::slice_layers` 边界；只可移植源定义的排序，不能全局排序输出掩盖错误|
|`TriangleMeshSlicer.cpp:slice_mesh_ex` 的 layer 转换段（2158–2183）|预分配 layers，按 `layer_id` 写入、按索引返回|已有 `segments.rs::slice_layers` 等相应 API；本次不并行化|
|`Format/bbs_3mf.cpp:store_bbs_3mf`（9723）、`_BBS_3MF_Exporter::save_model_to_file`（6318）、`_save_model_to_file`（6418）|真实项目导出生命周期|**deferred**；本基础设施不发明 writer / CLI export 命令|

源链接：[PrintObject.cpp](https://github.com/OrcaSlicer/OrcaSlicer/blob/345efec64c7b95a05ce5cf96df0dde66a158835c/src/libslic3r/PrintObject.cpp)、
[TriangleMeshSlicer.cpp](https://github.com/OrcaSlicer/OrcaSlicer/blob/345efec64c7b95a05ce5cf96df0dde66a158835c/src/libslic3r/TriangleMeshSlicer.cpp)、
[bbs_3mf.cpp](https://github.com/OrcaSlicer/OrcaSlicer/blob/345efec64c7b95a05ce5cf96df0dde66a158835c/src/libslic3r/Format/bbs_3mf.cpp)。

这不是对上游所有 TBB 路径天然确定的证明，更不授权引入自研 Ares pipeline。
现有顺序执行代码是相应上游概念的当前实现；只有后续逐源切片可以替换执行策略。

## 3. 不可协商的并行规则

1. 结果槽位、层/面/对象身份及输出汇合顺序由输入索引确定。不得将完成顺序、
   channel到达顺序、worker编号、指针地址或锁竞争胜负用于输出排序。
2. 每个任务只写自己的槽位；跨层依赖采用源定义的 snapshot / barrier / apply
   阶段。无 data race 不等于有确定顺序。
3. 浮点归约不得按 CPU 数或 worker 数分块。块大小、边界、块内顺序和块间归约
   树必须固定且与核数无关。若源要求顺序累加，则并行计算独立项后按原索引串行
   累加，不能因使用固定块就改变既有位级语义；FULL 同样必须通过。
4. 输出路径禁止依赖 HashMap / HashSet 遍历顺序。纯 lookup 可以保留；可观测
   遍历必须采用源定义的稳定序列/键顺序，不得靠比较器再排序 G-code、顶点或路径。
5. 禁止依赖调度的 ID 分配（包括原子计数器完成即取号）。预分配索引或源定义的
   稳定 prefix offsets；容器、ZIP成员和配置序列化同样受约束。
6. RNG、时间和名称仍是请求输入，不允许用 worker/CPU数换 seed，也不能随意
   clone RNG 以制造新序列。涉及随机性的切片必须另述源定义的 draw 顺序。
7. 不增 epsilon、归一化、忽略字段、旧算法 fallback 或“只在单核正确”的合入通道。

## 4. 完整、轻量的请求捕获

输入集合是 **orca_parity 中全部实际 Ares 请求**，不是 fixture 文件名列表。
包括共享 helper 调用、直接 CLI、API-only 调用、运行时改写项目、presets、失败
输入、processed结果，以及有状态 Plater 的原输入与公开 edit 操作序列。
构造请求的数据类型构造器不另伪装成 slicing job，但 ledger 必须说明分类，不能
把未适配的真实公共调用归为 helper 排除。

- 只在测试适配层包裹既有公共调用，先记录原参数，再原样执行并返回原结果。
  CLI 捕获真实 Command 的 argv/cwd/显式语义环境；API 捕获调用前的原始请求参数。
- 不改 parity 的 assertion、比较器、期望、执行分支、错误传播或可选 Native 政策。
  不在 `ares-core` 加文件 I/O / 追踪 hook，不复制私有几何算法。
- **不改 ROOT `crates/ares-cli/tests/orca_parity.rs`**；仅允许的纯改名也非本任务所需。
  capture模块放现有 grouped adapter 下，replay程序经公开 `ares_core` API 调用。
- 内容寻址记录 input bytes、格式和语义文件名、按序配置/preset内容、resources
  内容树、overrides、plate、placement、metadata、seed、公开 edit 序列与入口种类。
  只排除无语义的临时输出目录及本门禁主动改变的 CPU 设置；不任意重排请求。
  `GenerationMetadata` / `SlicerContext` 的内部字段不公开：在原始构造参数处记录
  时间六元组/seed，重放用原公共构造器，不添加getter、读取私有RNG或猜测默认值。
- 一个请求 ID 可关联多个调用来源；去重不删除来源、次数和配置组合。ID 不含
  核数和调度序号。资源/输入快照必须在调用结束或临时目录销毁前保存。
- 不抓取完整宿主环境、认证信息或私有路径；运行时路径转为内容对象引用，语义
  相对路径/文件名保留。新语义参数未被 schema 表达即 BLOCK，不做静默默认。

覆盖率 ledger 必须列出源 revision、entry / callsite、请求 ID 集合、运行时次数、
全部测试是否完成、Native导致的提前退出和未适配项。静态调用清单负责发现漏网
路径，真实运行负责发现动态参数；不能用静态扫描代替实际捕获。
全量声明只在清单与运行覆盖都闭合时成立；覆盖不完整、0请求、过期 corpus、
只收到了通过测试的子集或未到达预期请求的注册用例，均为 **BLOCK**，不是 PASS。
有意不执行的条件分支须在 ledger 记录对应 registered case 的不适用理由；这不是
要求枚举无限输入路径。未适配公共入口或缺 Native 导致的提前退出不能归为不适用。

## 5. 重放和比较契约

独立测试程序/脚本按请求 ID 顺序调度；CLI 请求执行真正的 `ares slice`，API-only
请求由独立 Rust replay adapter 调用原公共 API，而不是强行换成另一种 CLI语义。
不新增生产 CLI 功能或3MF导出以满足测试。

每个去重请求做五次真实执行：1、2、7、16、16-repeat CPU。`taskset` 在独立进程
启动前生效；记录实际 affinity与 `available_parallelism`，线程数环境变量不能
替代 CPU mask。所有执行使用同一构建、同一请求、同一可见输入/输出文件名、
metadata、seed和语义环境。工作目录按次顺序复用固定路径，再将结果复制到证据
目录，避免临时路径进入模板输出造成伪差异。每次都重新调用 Ares，禁止命中
上一次 Ares 输出缓存来证明“重复一致”。

输出合同：
- CLI G-code、API `Vec<u8>` / G-code 不修改任何字节；不使用 Native/Ares 的
  generator-only比较器，也不去时间、路径、注释、换行或footer。
- 公开结果结构用版本化的无损序列化：完整公开字段、数组/顶点原顺序、enum种类、
  f32/f64的位模式（含负零和非有限值），不缩窄或按显示小数比较。已有 Serialize
  不能完整保留位模式时，test adapter投影公开字段；不能访问私有内部或漏新字段。
- admission/error请求保留原成功/失败分类、公开错误与输出文件集合；基线必须符合
  捕获时的 outcome类型。不能把全部请求变成相同失败后计为通过。
- 1 CPU是本轮实际基线，后四次每一份公共产物都与它逐字节比较；两次16 CPU亦
  记录独立执行证据。哈希可用于清单，但失败必须报首个字节offset、长度/EOF、
  两侧值和有限上下文；结构产物另给字段/元素定位，不代替原字节差异。

`taskset` 的 CPU 数指可用逻辑 CPU 数；记录拓扑，但不将两个 SMT 线程冒称两个
物理核。分配来自显式授权 CPU 集与继承 affinity 的交集，不调用 `nproc --all`
扩大权限。少于16个授权 CPU 时全量门禁明确 BLOCK，不能降级、超售一个CPU
冒充16CPU或跳过。

## 6. Native 缺失不产生空集通过

重放器不需要 Native，也不得调用 Native产生比较期待值。完整已冻结的请求集合
在无 Native 环境仍可跑门禁 A；报告明确写“Native未参与”，不宣称 Native parity。

采集所依赖的原测试若缺 Native而 return / skip，则 ledger不能完整。采集入口须
显式报告缺 supplier / 未到达 Ares 调用，阻止全量声明。没有完整 corpus、Native
又不可用时 BLOCK；不能让“0请求全部相同”返回成功。
Linux CI 子集使用仓库已有 authored 输入和显式配置，不靠 Native现场生成输入，
也不提交 Native生成的项目、G-code或golden。子集清单是公开声明的子集，不冒称全量。

## 7. 分离绑核与 Native cache identity

当前 `scripts/ci/pin-runner.sh` 同时把测试与子进程钉到1CPU，且用 `nproc --all`
选 CPU；`cache_identity.rs::identify` 将父测试进程的 available_parallelism放进
Native producer components。在 Ares 使用 K CPU后，这个值不再代表 Native。

新模式要求：
- pin-runner仅给 Ares测试/其AresCLI后代一个授权 K CPU子集，并声明其中一个
  Native CPU；资源不足则失败。nextest并发测试数与每测试 K CPU是不同参数。
- 单一仓库相对路径的 **Native启动脚本** 在内部把 Native执行限制为该子集的
 1CPU。官方、seeded、diagnostic、Node kernel supplier、GDB启动链都必须覆盖。
  不能只修 seeded wrapper，漏掉直接执行的 Nix launcher。
- 优先把纯 taskset/exec策略放在外层启动脚本和 test adapters；不修改已声明的
  Native ELF、resources、原launcher/endpoint字节、argv、seed、locale或ASLR策略。
  原bin仍是 producer身份；外层CPU封装不是冒充另一供应商的 alias。
  本机制新增的 Ares-only 核数/捕获控制变量不得泄漏为 Native 的不同执行输入；
  原有Native语义环境保持，不增新的shell副作用或输出。实际差异必须被验收发现。
- `cache_identity.rs` 的 parallelism来自与真实Native启动完全相同策略、已经
 进入子 affinity的查询，不读取父进程核数，也不盲信一个环境变量写死1。
  Native实际执行的子进程/exec前mask要有独立证据。CPU编号仍仅是运行证据，
  不进入内容cache key；Native可用CPU数仍为1，既有schema和其他components不变。

验收须证明：同一输入及 Native producer，父进程1CPU与KCPU两模式中 Native始终
1CPU、producer/request key一致，同一既有entry热命中且所有存储产物hash不变；
不能删缓存、改key版本、偷换launcher或靠重建导致“看起来相同”。另用确定性的
真实Native公共调用验证两模式的原始输出相同，官方/seeded均覆盖。选择无隐式
时钟的输出，既有时间策略不变；发现时间差异原样记录，不能新增归一化掩盖。

## 8. 默认 FULL 的永久切换时点

基础设施先准备两种明确运行模式，当前仍可保持既有单CPU默认。
**第一个引入并行的切片合入时，批次 FULL 与对应 CI FULL 必须同时切到 Ares
多CPU作为默认，并从此一直如此。** K在该切片前确定，建议4；不能把
nextest测试并发数当作 K，也不能在红时退回 Ares1CPU取得合入证据。
Native仍为每调用1CPU，官方/seeded两套比较和缺supplier失败政策保持。

## 9. 平台与证据

Linux affinity属于测试/CI/CLI适配层；ares-core仍无文件I/O、终端或平台绑核。
WASM、Windows、macOS公共API不因这个Linux门禁引入TBB/native runtime依赖。
本门禁不宣称跨平台浮点字节相同；平台各自仍受其 source-parity合同约束。

每次产出可重放的请求CAS、coverage ledger、构建/source身份、每次实际mask、
命令/outcome、产物长度/hash、首差异及最终summary。批次证据保存在调用方指定的
仓库外目录；CI子集保留机器可读summary/日志与复现命令，不借Native cache
塞测试输出，不改额度/删除cache。产物写入或要求的发布失败不能计作通过。

关联：[实施计划](multicore-consistency-gates-plan.md)、
[已有确定时间/seed合同](ard-0095-reproducible-generation-inputs.md)。
3MF导出、第一处并行算法移植、跨平台字节一致性及性能优化均不包含在本次实现。
