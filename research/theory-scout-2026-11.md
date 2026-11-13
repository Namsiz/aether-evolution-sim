# DS Bot 侦察报告：开放式演化→智能→文明的涌现路线（调研截至 2026-11-13）

> 执行：ds-scout（Chief 派发，2026-11-13）。用途：为《自主发展宇宙核心路线图》（self-developing-universe-roadmap.md）提供现实理论依据。
> 报告性质：全部结论基于本次调研中核实过的来源（每条带 URL 与发表时间）；本人的推断单独标注「**推断**」。已核实项目现状：`~/workspace/world/ARCHITECTURE.md`（v2.0，2026-11-09）与 `world/core/` crate 骨架（aether-rng/coords/clock/sim-ecology/libm/biome/render-packet）。本报告不改任何文件、未跑构建。

---

## Q1 开放式演化（OEE）

### 核心概念（来源事实）
- **定义**：OEE = 永不枯竭地产生新颖、有趣、可学习的产物的过程；被称为「最后的重大挑战」——Stanley/Lehman/Soros《Open-endedness: The last grand challenge you've never heard of》，O'Reilly Radar，2017-12-19，https://www.oreilly.com/radar/open-endedness-the-last-grand-challenge-youve-never-heard-of；正式学术版：Stanley《Why Open-Endedness Matters》，Artificial Life 25(3):232-235，2019，https://doi.org/10.1162/artl_a_00294。
- **四条必要条件**（Soros & Stanley，Chromaria 实验世界提出并做了消融验证）：①可能出现新的可遗传个体/特征；②特征的变化要带来**进一步变化的潜能**；③能出现多样性的变化；④存在「最低生存标准」（minimal criterion）而非标量适应度。来源：Soros & Stanley《Identifying Necessary Conditions for OEE through the Artificial Life World of Chromaria》，ALIFE 14，2014，https://stars.library.ucf.edu/scopus2010/9117；消融实验：Soros/Cheney/Stanley，ALIFE 15，2016，https://www.uvm.edu/neurobotics/pubs/pdf/2016_SorosCheneyStanley_HowTheStrictnessOfTheMinimalCriterionImpactsOpenEndedEvolution_ALIFE.pdf；验证更新：OEE4 Workshop，2021-05，http://workshops.alife.org/oee4/papers/soros-oee4-camera-ready.pdf；博士论文全本：Soros 2018，https://stars.library.ucf.edu/etd/5965。
- **里程碑为何平台化**：
  - **Tierra**（Ray，1991）：寄生者→免疫→超寄生→合作聚集体→欺骗者自发涌现（真涌现），但长期运行中「生态往往只剩一种寄生物+一种宿主，动态停滞」——Frontiers in Ecology and Evolution 综述，2021-11-29，https://www.frontiersin.org/journals/ecology-and-evolution/articles/10.3389/fevo.2021.750779/full；Ray 本人记录的尺寸级停滞/点断平衡：https://tomray.me/pubs/tierra/node23.html。
  - **Avida**（Ofria & Wilke 2004；Adami/Ofria/Collier，PNAS 97(9):4463，2000，https://www.pnas.org/doi/10.1073/pnas.97.9.4463）：复杂度信息论度量下持续增长，但增长由「环境奖励任务」驱动——一旦环境固定，新颖性枯竭；Wiser 等指出早期适应性投影会低估演化潜能（「有界性错觉」，OEE3，2018，http://workshops.alife.org/oee3/papers/wiser-oee3-final.pdf）。
  - **超循环谱系**（Eigen & Schuster 1977/1979）：信息论上优雅，但**对寄生者演化不稳定，必须靠区室化（原细胞）才能存活**——Kun 等《The dynamics of the RNA world》，2015，https://real.mtak.hu/19624。这是「跃迁需要冲突调解结构」的最早教训。
  - **Polyworld**（Yaeger，1994，ALife IV）：生态+神经演化+Hebbian 学习的全栈鼻祖，自发出现捕食、拟态等行为；算力限制种群仅百级——https://webarchive.ars.electronica.art/en/archiv_files/19931/E1993_122.pdf。
- **近十年可操作进展**（全部有代码/论文可参考）：
  - Novelty Search（放弃目标，只搜新颖行为）：Lehman & Stanley，Evolutionary Computation 19(2):189-223，2011，https://doi.org/10.1162/EVCO_a_00025；前身 ALIFE XI 2008 会议文（https://mitpress-request.mit.edu/sites/default/files/titles/alife/0262287196chap43.pdf）。
  - MCNS（新颖性+最低标准）：Lehman & Stanley，GECCO 2010（出处见上引 2016 论文参考文献）。
  - Quality-Diversity / MAP-Elites（「照亮」行为空间，每个生态位留最优解）：Mouret & Clune，arXiv:1504.04909，2015，https://arxiv.org/abs/1504.04909；QD 综述：Pugh/Soros/Stanley，Frontiers in Robotics and AI，2016，https://www.frontiersin.org/journals/robotics-and-ai/articles/10.3389/frobt.2016.00040/full。
  - **POET**（环境与解成对共演化+跨环境迁移解）：Wang/Lehman/Clune/Stanley，GECCO 2019/arXiv:1901.01753，https://arxiv.org/abs/1901.01753；Enhanced POET，ICML 2020，https://proceedings.mlr.press/v119/wang20l/wang20l.pdf。
  - 当代延伸：**AlphaEvolve**（DeepMind，2025）= 进化档案（MAP-Elites 风格）+ LLM 变异 + 自动评估器，刷新 4×4 复数矩阵乘法（48 次乘法，56 年来首次超越 Strassen 路线）——arXiv:2506.13131，2025-06-16，https://arxiv.org/abs/2506.13131。
  - 领域共识文档：OEE York 工作坊报告（开放获取），Taylor 等，Artificial Life 22(3):408-423，2016，https://direct.mit.edu/artl/article/22/3/408/2841/。

### 对本项目的可操作建模建议
1. **保持适应度不可显式**（已在做）：agent 只有能量收支/繁殖/死亡——这就是 Chromaria 条件④的天然实现。任何「物种评分」「进化进度条」都会毁掉 OEE。
2. **把新颖性压力外包给生态位构建**：让 agent 的代谢产物/尸体/行为改变环境（项目的化学营养循环+地质层已经是基础），使「解决方案创造新解决方案的空间」（POET 的生态版=共演化）。
3. **统计影子（L0）也必须演化**：未观察区域的多样性/能量通量按确定性规则继续演化，否则观察范围外成为新颖性死区；L0 的聚合指标（物种数×生态位矩阵）可直接当 OEE 仪表盘（Vostinar/Dolson/Wiser/Ofria 的 change metric 思路，http://workshops.alife.org/oee2/abstracts/vostinar-oee2-submission.pdf）。
4. **检测平台化而非治疗平台化**：用 Bedau 式「进化活动统计」监测新增行为/形态/生态位类型的速率；速率→0 说明规则集该动（调环境动态，不是调目标）。

### 已知坑
- 目标函数伪装成「环境参数」又溜回来（Stanley 2019 反复警告）。
- 静态环境：适应完成后必然平台（POET 的立论动机）。灾变/地质扰动是对的，但扰动本身要从世界规则涌现，不要用「每 N tick 触发陨石」的上帝脚本（**推断**，与 OEE 文献对「机制内生」的要求一致）。
- 伪 OEE：无限随机噪声 ≠ 无限新颖性（Taylor 2016 与《Open-Endedness for the Sake of Open-Endedness》，ALife 25(2)，2019，https://direct.mit.edu/artl/article/25/2/198/2923/，明确批评了「能永远跑就算开放」的定义）。

---

## Q2 演化大跃迁（Major Transitions）

### 核心概念（来源事实）
- **框架**：Maynard Smith & Szathmáry《The Major Transitions in Evolution》（OUP，1995；综述版 Nature 374:227-232，https://joelvelasco.net/teaching/167win10/szathmary%20and%20maynard%20smith%2095-major%20evo%20transitions.pdf；书目：https://en.wikipedia.org/wiki/The_Major_Transitions_in_Evolution）。八次跃迁：复制分子→区室化分子群；独立复制子→染色体；RNA（基因+酶双职）→DNA+蛋白质分工；原核→真核（内共生）；无性→有性；单细胞→多细胞（细胞分化）；独居个体→有非繁殖品级的社会群体；灵长类社会→语言社会。
- **每次跃迁的共同机制**（Szathmáry & Maynard Smith 原文）：①跃迁前能独立复制的单位，跃迁后只能作为更大整体的一部分复制（个体性转移）；②合作+冲突调解；③信息存储/传递方式改变（Jablonka & Lamb 补强：神经系统与行为传递也算大跃迁信息通道，J Theor Biol 240:175-189，2006，https://www.sciencedirect.com/science/article/abs/pii/S0022519305003851）。
- **数字生命里「跃迁真的涌现过」的成功案例**（都在 Avida 谱系）：
  - **生殖分工涌现**：给「干代谢活」附带致突变副作用 → 生殖细胞/体细胞分化自发出现——Goldsby/Knoester/Ofria/Kerr，PLOS Biology，2014-05-13，https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.1001858（「脏活假说」）。
  - **任务切换成本 → 劳动分工与个体性转移**：GoldsBy/Dornhaus/Kerr/Ofria，PNAS 109(34):13686，2012，https://pubmed.ncbi.nlm.nih.gov/22872867。
  - **专门综述**：Moreno & Ofria《Toward Open-Ended Fraternal Transitions in Individuality》，Artificial Life 25:117-133，2019，https://doi.org/10.1162/artl_a_00284。
- **失败模式**（文献明示）：群体级性状涌现后，若没有 germline 瓶颈/程序性死亡/监督惩罚等「冲突调解器」，作弊者会摧毁群体（Goldsby 2014 的 cheater 分析；超循环寄生者问题是同一数学结构）。

### 对本项目的可操作建模建议
1. **把「多细胞/社会/语言」做成可涌现的数据结构而非 flag**：低层 agent 是一等实体；允许出现「群体实体」拥有自己的生死与繁殖（繁殖=单个体瓶颈 propagule），群体的基因型只是低层基因型的包装+少量调控基因。
2. **跃迁的燃料=真实经济学**：task-switching cost、代谢致突变（脏活）、能量节省——这三样全部可量化进现有能量系统，是文献验证过的最小触发条件。
3. **冲突调解从简单机制起步**：germ line 指定位、程序性死亡开关、亲缘识别——让它们的成本由能量账本支付，能否保住由生态压力裁决。
4. **把「层级数增长」当 OEE 里程碑**：监测选择层级（个体/群体/超群体）数量，而不是「解锁了多细胞」。

### 已知坑
- 「到达 X 解锁 Y」的脚本化跃迁 = 又一个目标函数（见 Q1）。
- 没有瓶颈的群体选择太弱，群体适应性性状无法积累（Godfrey-Smith《Darwinian Populations and Transitions in Individuality》，MIT Press 2011，https://petergodfreysmith.com/wp-content/uploads/2013/06/PGS-Darwinian-Pops-and-Transitions-Vienna-07-Q.pdf；**推断到工程**：群体繁殖必须经过单点）。
- 每次跃迁之后 OEE 需要「重新点火」：新层级=新的信息通道与资源空间（Jablonka & Lamb 2006）——不要指望单一机制撑全程。

---

## Q3 智能的演化（神经演化 / 最小认知）

### 核心概念（来源事实）
- **NEAT/CPPN/HyperNEAT 现状**：领域综述——Stanley/Clune/Lehman/Miikkulainen《Designing neural networks through neuroevolution》，Nature Machine Intelligence 1:24-35，2019，https://www.nature.com/articles/s42256-018-0006-z。要点：神经演化与深度学习并行活跃；NEAT 的 speciation（物种保护）与项目的物种层天然同构。
- **轻量确定性实现候选对比**（crates.io 核实，截至 2026-11）：

| 候选 | 状态 | 特性 | 结论 |
|---|---|---|---|
| **radiate v1.3.2** | 活跃（52 个版本，最近 2026-09；GitHub 2949 commits） | NEAT（opt-in）、novelty search、speciation、GA/GP 一体，MIT，纯 Rust+Python 绑定 | **首选参考实现**。https://crates.io/crates/radiate，https://github.com/pkalivas/radiate |
| neat v1.0.2 | 小型（约 2K SLoC，8 个月前更新，下载量低） | 基于 genetic-rs 的 NEAT | 可读源码参考 https://crates.io/crates/neat |
| rustneat v0.2.1 | **8+ 年未更新** | 最早的 Rust NEAT | **勿依赖** https://crates.io/crates/rustneat |
| vivalaakam_neuro_neat | 小、社区极小 | NEAT 变体 | 忽略 |

- **大脑是能量昂贵器官**：人脑占体重 ~2%、消耗 ~20% 静息代谢（Attwell & Laughlin 2001，J Cereb Blood Flow Metab 21:1133-1145，https://journals.sagepub.com/doi/full/10.1097/00004647-200110000-00001；Raichle & Gusnard 2002，PNAS 99:10237，https://www.pnas.org/doi/10.1073/pnas.172399499；总量 ~20W：NCBI Basic Neurochemistry，https://www.ncbi.nlm.nih.gov/books/NBK28194）；儿童期可达 ~50%。
- **每神经元能量预算近似恒定**（跨物种）：人脑 20% 只是神经元多——Herculano-Houzel，PLoS ONE 6(3):e17514，2011，https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0017514 → **工程上可把脑成本 = 常数 × 神经元数**。
- **能量限制塑造神经系统演化**（wiring/coding 效率化）：Niven & Laughlin，J Exp Biol 211:1792-1804，2008，https://journals.biologists.com/jeb/article/211/11/1792/9506/。
- **最小认知阶梯（有实证底座）**：
  1. **趋化 + 秒级记忆**：E. coli 做的是「过去 1 秒正加权、之前 3 秒负加权」的时间比较——Segall/Block/Berg，PNAS 83:8987-8991，1986，https://www.pnas.org/doi/10.1073/pnas.83.23.8987；记忆介质=受体甲基化（CheR/CheB 慢变积分反馈），并具对数感知/fold-change detection——Krembel 等，PLoS ONE，2015，https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0145582；FEMS Microbiol Rev 2021 综述，https://academic.oup.com/femsre/article/45/6/fuab038/6316109。
  2. **适应回路 = 免费的时间差分器**：慢积分负反馈让「当前感受 − 近期基线」成为输出（同上文献）。
  3. 在此之上加 Hebbian 强化与表格式 Q 更新（Rescorla-Wagner 一类经典 RL），奖励=能量变化率——**推断**（RL 教科书内容，未单独引文献），但趋化时间比较与 TD(0) 的同构是可以在 sim 里直接复用的。

### 对本项目的可操作建模建议
1. **大脑进能量账本**：`brain_cost = c_per_neuron × n_neurons + spike_traffic × c_spike`（常数按 Herculano-Houzel 缩放）。脑大要付真实代谢价，进化才能在「脑大」与「早生多生」间权衡——这是防智能「免费午餐化」的根本机制。
2. **学习也要有成本**：突触更新按次数扣能量（Niven & Laughlin 的能量压力论），否则最优学习策略会统治一切，多样性平台化（**推断**，接 Q1）。
3. **认知阶梯做成数据结构**：感官原始值 + 慢变内部变量（甲基化式）→ Hebbian/RL 更新 → 逐级扩展；第一级「趋化智能」几行代码就能写，与现有 grassfield 感知半径机制兼容。
4. **NEAT 用于个体大脑编码**：参考 radiate 的实现，但自研进 `aether-*` crate（体量 ~2K SLoC 级，换掉 `rand::thread_rng` 为 SimRng、激活函数走 aether-libm compat）——确定性可控性优先于依赖现成库（**推断**；radiate 本身 wasm 兼容性未验证）。
5. 大脑成本与 MTE（Q6）联动：代谢率按体型缩放后，脑占比就是可演化参数，人脑 20% 成为可达而昂贵的端点。

### 已知坑
- 免费大脑/免费学习 → 竞争维度塌缩（前述）。
- 每 agent 学习状态内存膨胀——给学习状态设硬上限（**推断**，工程约束）。
- rustneat 等停更 crate 勿引入；radiate 的 NEAT 是 opt-in 特性，抽取其 speciation 逻辑时注意其 RNG 路径全部替换。

---

## Q4 文化演化与基因-文化协同演化

### 核心概念（来源事实）
- **双重继承理论**（Cavalli-Sforza & Feldman 1981；Boyd & Richerson 1985《Culture and the Evolutionary Process》）。核心机制清单：①guided variation（个体试错改写文化项）；②biased transmission——direct/content bias（按内容好坏选）、frequency/conformist bias（从众）、model/prestige bias（模仿成功者/声望者）；③漂变；④对文化项的自然选择。来源：Boyd & Richerson 论文集 https://www.des.ucdavis.edu/faculty/richerson/BoydRicherson1982.pdf 与 https://www.des.ucdavis.edu/faculty/richerson/Stanford%20Cultural%20evolution.pdf；Henrich & McElreath《Dual-inheritance theory》章节 PDF：https://www2.psych.ubc.ca/~henrich/pdfs/Ch38%20-%20Dual%20Inheritance%20Theory.pdf。
- **Rogers 悖论**：只省钱的社会学习可以入侵种群但**不提高种群平均适应度**（Rogers 1988，Am Anthropol 90:819-831；Boyd & Richerson 1995《Why does culture increase human adaptability?》，https://www.des.ucdavis.edu/faculty/richerson/es95.pdf；Rendell 等 2010《Rogers' paradox recast and resolved》，https://lalandlab.wp.st-andrews.ac.uk/files/2015/08/Publication144.pdf）。成立条件：文化必须让个体学习更准/更便宜；年龄结构也不自动解决（Deffner & McElreath，PLoS ONE，2022，https://pmc.ncbi.nlm.nih.gov/articles/PMC9012401）。
- **累积文化/棘轮的成立条件**：创新率 ∝ 社会性 × 传播保真度 × 文化方差（Muthukrishna & Henrich《Innovation in the collective brain》，Phil Trans B 371:20150192，2016，https://doi.org/10.1098/rstb.2015.0192）；创新三来源：偶然、重组、增量改进——**全部可直接编码**。
- **技术可失传**：Henrich 塔斯马尼亚模型——有效学习群体规模跌破阈值→复杂技能衰减直至消失（Am Antiq 69:197-214，2004，https://www.cambridge.org/core/journals/american-antiquity/article/demography-and-cultural-evolution-how-adaptive-cultural-processes-can-produce-maladaptive-lossesthe-tasmanian-case/8CD08CC61E6FACF59EC02659B2BDA43C）。**注意争议**：Vaesen 等《Population size does not explain past changes in cultural complexity》，PNAS 113(16):E2241，2016，https://www.pnas.org/doi/10.1073/pnas.1520288113——人口规模不是万能变量，保真度是补充轴。
- **语言涌现不需要硬编码**：iterated learning + 传播瓶颈 + 压缩压力 → 组合结构自动涌现（Kirby 等，PNAS 105:10681-10687，2008，https://www.pnas.org/doi/10.1073/pnas.0707835105；压缩-表达权衡：Kirby/Tamariz/Cornish/Smith，Cognition 141:87-102，2015，https://www.sciencedirect.com/science/article/pii/S0010027715000815）。
- **农业起源的 agent-based 先例**：Bowles & Choi《Coevolution of farming and private property during the early Holocene》，PNAS 110(22):8830-8835，2013（演化博弈+ABM，含气候波动，重现考古时点），https://pmc.ncbi.nlm.nih.gov/articles/PMC3670368；参数扫描复现：Gallagher/Shennan/Thomas，PNAS 112(46):14218，2015（小群体+保守性+产权是关键，生产率优势非必需），https://pubmed.ncbi.nlm.nih.gov/26578766。

### 对本项目的可操作建模建议
1. **文化=第二继承通道**：每 agent 带文化向量（知识项），传播按观察半径+偏见规则执行（copy-successful / copy-majority / content-evaluate），全部走 SimRng。
2. **两个参数统治全部现象**：个体学习成本 c>0、传播保真度 f<1。Rogers 悖论、累积文化、塔斯马尼亚效应全部是 (c, f, N) 的函数——让 f 依赖「教学行为/语言复杂度」存在与否，语言与教学的演化就获得真实回报。
3. **技术=带保真度的知识项**：每项有 prevalence、fidelity、复杂度；群体规模收缩时按 Henrich 模型自然衰减——技术涨落免费获得，不写科技树。
4. **语言层从「示范-模仿」起步**：先有动作模仿通道，需要向同伴描述未见对象时，Kirby 式压缩压力会自发产生结构（2008/2015 两文是可复制的最小实验设计）。
5. 农业等「革命」用 Bowles&Choi 的协同演化条件生成：技术需要制度（如可防御的产权=资源驻留结构），制度需要技术——两者互为前提，缺一不出现。

### 已知坑
- 社会学习免费 → 文化瞬传播、无梯度；必须付时间/能量/风险。
- 只放 conformist bias → 单一文化锁死；需 prestige/content bias + 群体间竞争并存（Henrich & Boyd 1998 框架，见 SEP 条目 https://plato.sydney.edu.au/entries/evolution-cultural/）。
- 不要把「人口规模→复杂度」写成硬规则（Vaesen 2016 争议）——让 (社会性, 保真度, 方差) 三轴共同决定。

---

## Q5 文明涌现的先例与失败模式

### 核心概念（来源事实）
- **先例对比表**（核实时间 2026-11）：

| 系统 | 演化真实性 | 认知/文化层 | 现状 | 关键来源 |
|---|---|---|---|---|
| **The Bibites**（Léo Caussan/Omnia Studios，2017 起） | 真演化：基因+NEAT 式神经脑+能量/消化/信息素 | 神经脑（无文化层） | 活跃：Steam EA 2025-03-04，「96% 好评」，计划「module-based systems for unbounded evolution」 | https://www.thebibites.com/、https://store.steampowered.com/app/2736860/、https://thebibites.itch.io/the-bibites（观察到的涌现：信息素踪迹狩猎、囤食） |
| **Species: ALRE**（Quasar，Steam 2018） | 真演化（基因/变异/物种分化/食物网） | **无**神经脑/文化层 | **>3 年未更新**（Steam 页面标注） | https://store.steampowered.com/app/774541/ |
| **Polyworld**（Yaeger 1994） | 真演化+Hebbian 学习 | 有（脑）无文化 | 停滞 | https://en.wikipedia.org/wiki/Polyworld |
| **Creatures**（Steve Grand，1996） | 生化+神经脑+激素+遗传、简单动词-宾语语言习得 | 有学习无生态选择压力 | 停更 | Grand/Cliff/Malhotra 技术报告：https://www.sussex.ac.uk/informatics/cogslib/reports/csrp/csrp434.pdf |
| **Sugarscape**（Epstein & Axtell 1996） | 无生物演化 | 无技术变化 | 经典已封 | https://www.brookings.edu/books/growing-artificial-societies/（涌现部落/贸易/财富分布/生态崩溃） |
| **Dwarf Fortress worldgen** | **无**（文明由硬编码事件系统+启发式驱动；生物不演化、无技术演化） | 无 | 活跃 | Wiki：https://dwarffortresswiki.org/index.php/World_generation；Toady 本人的表述被 wiki 记录：「文明……不是智能体，历史只是记录。用模拟日志生成故事是可行路径，但有缺点」 |
| **Project Sid**（Altera，arXiv:2411.00114，2024-10） | 非演化（LLM 驱动） | 有（职业/宗教/法律涌现） | 活跃 | https://arxiv.org/abs/2411.00114（10-1000+ agent，Minecraft；**LLM 非确定性、按 token 计费**） |

- **文明度量做成涌现指标的方法**：
  - Kardashev 尺度：按能量利用分型（I≈4×10¹²W / II≈4×10²⁶W / III≈4×10³⁷W）——Kardashev《Transmission of Information by Extraterrestrial Civilizations》，Soviet Astronomy 8:217，1964（https://en.wikipedia.org/wiki/Kardashev_Scale；原始数据见《The Extended Kardashev Scale》，AJ，2020，https://iopscience.iop.org/article/10.3847/1538-3881/ab792b——该文还把**信息存储量**与**质量**作为附加坐标轴）。
  - **宏观历史可由少数规则重现**：Turchin/Currie/Turner/Gavrilets《War, space, and the evolution of Old World complex societies》，PNAS 110(41):16384-16389，2013——农业+战争+军技扩散+地理的 4 参数文化演化模型解释 65% 大帝国时空分布方差，https://doi.org/10.1073/pnas.1308825110（作者页：https://peterturchin.com/publications/war-space-and-the-evolution-of-old-world-complex-societies）。
  - **复杂度崩溃理论**：Tainter《The Collapse of Complex Societies》（Cambridge UP，1988）——社会是问题求解组织；对复杂度的投资边际回报递减，崩溃是理性简化（摘要与关键引文：https://centerforneweconomics.org/publications/excerpts-from-the-collapse-of-complex-societies）。

### 对本项目的可操作建模建议
1. **度量=遥测，不是奖励**：每个群体持续统计——(a) 能量通量（Kardashev 连续化）；(b) 信息存量（存活知识项数×保真度，对接 Extended Kardashev 的信息轴）；(c) 组织复杂度（选择层级数/分工维度）；(d) **Tainter 斜率**（新增复杂度的能量净收益，做边际曲线）。全部从 sim 状态计算，绝不回写逻辑。
2. **把 Tainter 做成选择压力**：复杂度维护成本随规模超线性（协调/管理能耗上升），能量流收缩时群体自动分裂/简化——崩溃不写脚本就能发生，且崩掉的群体空出生态位（对接现有灾变系统）。
3. **历史日志像 DF Legends 一样输出**：模拟日志→叙事记录（Toady 本人认可该路线，见上 wiki 引文）；放在 sim 的输出端而非规则端。
4. **文明动力学用 Turchin 式小规则集**：群体冲突由资源/地理涌现；技术扩散有距离衰减——4 个参数就能重现帝国兴衰的统计规律（2013 文献直接可抄）。
5. LLM agent 不可进 sim 核心（破坏 bit-exact、成本失控）；若用于「观察者叙事/文明档案总结」，只放 sim 外环（**推断**，约束来自项目自身确定性铁律与 Project Sid 的 token 成本事实）。

### 已知坑
- DF 路线上限：没有演化压力的「文明事件系统」最终是**生成历史**，不是**生成历史的生成器**。
- 阶段奖励=脚本化：能量阈值解锁行为清单（科技树）会把 Kardashev 从望远镜变成鞭子。
- Species: ALRE 的教训：只有形态演化、无认知层，玩家生态会枯竭——它是 Bibites（持续加深神经/生态层）的反面镜像。
- Sugarscape 的教训：社会结构可以涌现（不平等、迁徙、贸易），但没有技术变化，永远停留在「原始史」。

---

## Q6 现实规律基底（热力学 / 信息论 / 化学层时机）

### 核心概念（来源事实）
- **生态代谢理论（MTE）**：`B = B₀ · M^(3/4) · e^(−Ea/kT)`——个体代谢率由体重与温度决定，并连锁预测世代时间、死亡率、种群增长、种群密度等生态量。Brown/Gillooly/Allen/Savage/West，Ecology 85(7):1771-1789，2004，https://doi.org/10.1890/03-9000（开放 PDF：https://www.sccs.swarthmore.edu/users/08/bblonder/phys120/docs/brown.pdf）。**注意**：3/4 指数有争议但占优（Savage 等，Functional Ecology 18:257-282，2004，https://besjournals.onlinelibrary.wiley.com/doi/abs/10.1111/j.0269-8463.2004.00856.x）——做成可调参数。
- **营养级能量流**：Lindeman《The trophic-dynamic aspect of ecology》，Ecology 23:399-417，1942，https://www.michele.scardi.name/ecologia2/lindeman.pdf——营养级间传递效率 ~10%（Lindeman 自己的实测范围 0.1%–37.5%，见 LibreTexts 综述 https://bio.libretexts.org/Courses/Gettysburg_College/01%3A_Ecology_for_All/19%3A_Food_Webs/19.02%3A_Quantifying_Food_Webs）→ 食物链长度天然受限。
- **信息论**：基因组复杂性=「序列中存储的关于环境的信息」（信息论定义，Maxwell 式压缩）——Adami/Ofria/Collier，PNAS 97(9):4463-4468，2000，https://www.pnas.org/doi/10.1073/pnas.97.9.4463（**「信息存量」指标有直接的信息论定义**）。文化传播=有噪声信道：噪声率×瓶颈决定结构（Kirby 2008/2015，见 Q4）。
- **化学层（超循环/组合代谢）什么阶段才需要**：
  - **错误阈值**：复制错误率限制可维持的基因组长度（Eigen 1971，《Selforganization of matter…》，Naturwissenschaften 58:465-523；综述见 PMC：https://pmc.ncbi.nlm.nih.gov/articles/PMC5686403）；超循环靠互催化闭环突破该上限，但**对寄生者演化不稳定，需区室化/原细胞**才能存活（Eigen & Schuster 1977；Kun 等 RNA 世界综述，2015，https://real.mtak.hu/19624；PMC 教程：https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1004853）。
  - **自催化集（RAF）理论**：给定食物集，催化闭合网络的出现是可判定的（多项式算法）——Kauffman 1986，J Theor Biol 119:1-24；Hordijk/Steel/Kauffman 系列综述：https://pmc.ncbi.nlm.nih.gov/articles/PMC6315399/。最小细胞模型 chemoton（Gánti）：代谢+膜+模板三子系统——https://pmc.ncbi.nlm.nih.gov/articles/PMC6315399/ 与 https://doi.org/10.1007/tcc001。

### 对本项目的可操作建模建议
1. **MTE 作为统一账本**：所有 agent 代谢率=f(体型, 温度)，寿命/繁殖延迟/种群密度同源缩放——把现有「化学营养循环」的能量侧写成一条幂律+激活能公式，物种参数差异自动落在合理结构上；温度场（已有地质/气候层）直接成为代谢带=多样性带。
2. **营养级效率做参数（~5-15%）**：能量塔决定顶级捕食者密度上限，防「无限食物链」造成的多样性假象。
3. **知识保真度=信道容量**：知识项传播按 (教学时间, 语言复杂度) 决定的噪声率执行——Kirby 框架；信息存量指标用 Adami 式定义（对环境规律的压缩记录）。
4. **化学层推迟，且用 RAF 图模型而非浓度 ODE**：组合化学的收益场景只有两个——(a) 生命起源弧线（分子→区室→细胞跃迁）：用 RAF 判定自催化闭合是现成数学；(b) 发明/材料层（冶炼、合成）：需要有限反应规则空间。在那之前，零阶营养循环已够：**演化丰富性来自机制条件而非底层保真度**（Tierra/Avida 证明不需要分子细节也能演化；OEE 文献一致强调机制条件）——**推断（有文献依据）**。
5. **熵记账**：第二定律在 sim 中表现为「一切能量转换都有损耗」（Lindeman 1942 的立论基础）——把每个转换（消化/运动/学习/建造）的效率<1 写成统一约定，文明层就能继承同一物理感。

### 已知坑
- 3/4 指数与 10% 律都是经验均值而非常数——硬编码会限制演化探索（做参数）。
- 组合化学空间爆炸：无限反应空间吃光 CPU；用「食物集+闭合规则」约束（RAF 框架的 food set 概念）。
- 不要过早做分子层：在认知/文化层产生新颖性瓶颈之前，化学细节是算力负资产（**推断**）。

---

## 最高优先级建议（5 条）

1. **铁律：度量永远只当望远镜，不当鞭子。** 文明/智能的一切指标（Kardashev 化能量通量、Adami 式信息存量、Tainter 边际斜率、选择层级数）只做遥测输出，绝不回写 sim 逻辑。这是 OEE 文献（Stanley 2019；POET 2019）与项目「无剧本」方法论的唯一交集。来源：https://doi.org/10.1162/artl_a_00294；https://arxiv.org/abs/1901.01753。
2. **下一层做「能量-大脑-学习」三件套**：MTE 账本（Brown 2004）+ 神经脑按神经元数收费（Herculano-Houzel 2011 的恒定每神经元预算）+ 趋化式时间差分作为最低学习（Segall/Block/Berg 1986）。大脑昂贵且学习有价，智能涌现才有选择压力，同时天然防 OEE 平台化（新维度=新生态位）。
3. **文化层以两个参数落地**：个体学习成本 c、传播保真度 f——Rogers 悖论→累积文化→塔斯马尼亚式技术失传全部是 (c,f,N) 的涌现输出；技术=带保真度的知识项，不存在科技树。来源：https://www.des.ucdavis.edu/faculty/richerson/es95.pdf；https://doi.org/10.1098/rstb.2015.0192；Henrich 2004（注意 Vaesen 2016 反方）。
4. **大跃迁用 MLS 机制涌现**：群体=可繁殖个体（单点瓶颈）+ 脏活分工（代谢致突变）+ 任务切换成本——三个已验证的最小触发条件（Goldsby 2014/2012），全部可直接量化进能量系统；把「选择层级数增长」当 OEE 里程碑检测。来源：https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.1001858。
5. **化学层（超循环/组合代谢）推迟，且优先 RAF 图模型**：现有营养循环池已满足生态层需求；组合化学只在生命起源弧或发明层才有回报，届时用 Kauffman/Steel 的现成可判定框架。来源：https://pmc.ncbi.nlm.nih.gov/articles/PMC6315399/。

---

## 额外发现（顺带情报，不喧宾夺主）

- **Microcosmos**（ALIFE 2026 接收，arXiv:2607.02954，https://arxiv.org/abs/2607.02954）：GPU 原生、可微分、物理接地的 ALife 平台（弹性丝链生物+2D 流体），作者含 Kenneth O. Stanley，目标大规模 OEE——与项目 wgpu 远期路线同向，值得跟踪其开放代码。
- **Bibites Multiverse**（社区实验，2026-08-14 至 11-14，https://bibitesmultiverse.com/）：把 The Bibites 接成分布式多世界地图接力——「多宇宙协同演化」已被验证有社区需求。
- **WASM 3.0 确定性 profile**（2025-09-17 官方发布，https://webassembly.org/news/2025-09-17-wasm-3.0/）：标准现在为浮点 NaN 与 relaxed SIMD 规定确定性行为，明确面向「可重放系统」。对项目 bit-exact 路线是利好；现阶段 NaN 位模式仍是非确定来源——sim 内避免产生 NaN 或统一 canonicalize（spec numerics：https://webassembly.github.io/spec/core/exec/numerics.html）。
- **AlphaEvolve 的架构启示**（https://arxiv.org/abs/2506.13131）：进化档案+变异算子+**自动评估器**是当代 OEE 范式。本项目不需要 LLM——世界本身就是评估器（能量/存活/繁殖），这正是 deterministic 路线的结构性优势。
- Rust 神经进化生态现状：唯一活跃库是 radiate（v1.3.2，2026-09）；NEAT 体量约 2K SLoC，读透 radiate 后自研进 `aether-brain` 更符合确定性基座要求（crates.io 事实 + 推断）。

**未完成/未验证项**：未对 wet-lab 跃迁文献（如 Ratcliff 酵母多细胞实验）、Ostrom 公地治理、Morris《The Measure of Civilization》做检索核实，故未引用；Q4 中「乳糖酶持久性」等基因-文化案例本次未验证来源，以机制框架替代。如 Chief 需要这些方向，可派第二轮专项调研。
