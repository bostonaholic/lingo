# Existing Work: Programming Languages and DSLs for LLM Token Efficiency

**Date:** 2026-03-15
**Purpose:** Survey of prior art relevant to designing a programming language (lingo) optimized for
LLM code generation efficiency.

---

## Executive Summary

There is a small but growing body of work on designing programming languages and representations for
LLM efficiency. The most directly relevant academic work is SimPy (ISSTA 2024), which created an
"AI-oriented grammar" for Python, achieving 10-14% token reductions without sacrificing
performance. At the framework level, Wasp claims 10-40x token reductions via a high-level DSL for
full-stack web apps. Community analysis (January 2026) confirmed large token-count differences
across languages — up to 2.6x between most and least efficient — with Clojure and J topping the
rankings. No general-purpose language has yet been designed from the ground up for LLM generation
efficiency, making this a largely open space.

---

## 1. LLM-Optimized Languages and Grammars

### 1.1 SimPy / AI-Oriented Grammar (ISSTA 2024)

The most directly relevant academic work. Sun et al. (ISSTA 2024, arXiv:2404.16333) introduced the
concept of **AI-oriented grammar**: a grammar designed for LLMs rather than humans. They
implemented **Simple Python (SimPy)**, which strips formatting tokens and grammatical ceremony from
Python while preserving identical AST structure. This enables lossless, bidirectional conversion
between Python and SimPy.

**Key findings:**

- 13.5% token reduction for CodeLlama; 10.4% for GPT-4
- Performance maintained or improved on code tasks despite fewer tokens
- The AST is preserved exactly, so SimPy and Python are semantically equivalent

**Significance for lingo:** This is the strongest academic precedent for an LLM-target language
dialect. The paper explicitly frames the problem as "current programs are designed for humans; we
need one designed for AI," which is exactly the lingo premise.

**Source:** [arXiv:2404.16333](https://arxiv.org/abs/2404.16333) |
[GitHub: v587su/SimPy](https://github.com/v587su/SimPy)

---

### 1.2 MoonBit: AI-Friendly Language Design (LLM4Code @ ICSE 2024)

Fei et al. presented "MoonBit: Explore the Design of an AI-Friendly Programming Language" at the
1st International Workshop on Large Language Models for Code (LLM4Code 2024, co-located with ICSE,
Lisbon, April 2024).

MoonBit is a general-purpose language targeting cloud and edge computing that was designed from the
start with LLM integration in mind. Key design features for LLM-friendliness:

- **Strong type system** — helps models recognize patterns and generate more accurate code
- **Real-time semantics-based sampler** — validates and guides LLM generation token-by-token
  against the type system
- **Local and global resampling** — corrects AI-generated code in real time without full
  regeneration
- **Built-in expect testing** — enables rapid correctness feedback with no external tooling
- **"Blocks" architecture** — hierarchical code organization that simplifies context provision to
  LLMs
- **WebAssembly sandboxing** — safe isolated execution of AI-generated code

**Significance for lingo:** MoonBit demonstrates that language design choices (type systems, AST
structure, testing affordances) directly improve LLM code quality and can reduce "hallucination"
errors by providing tighter feedback loops.

**Source:** [ACM DL](https://dl.acm.org/doi/10.1145/3643795.3648376) |
[MoonBit blog: AI-friendly design](https://www.moonbitlang.com/blog/ai-coding) |
[MoonBit blog: AI-Native Toolchain](https://www.moonbitlang.com/blog/moonbit-ai)

---

### 1.3 LMQL: "Prompting Is Programming" (PLDI 2023)

Developed at ETH Zurich, LMQL (Language Model Query Language) is a Python superset that treats LLM
generation as a programmable, constraint-guided process. Published at PLDI 2023
(arXiv:2212.06094).

**Design philosophy:** LLM prompts are programs. LMQL embeds generation variables directly in
Python code with declarative constraints (`where VAR in [...]`). The runtime eagerly evaluates
constraints at the token level to produce token masks during decoding.

**Key results:**

- 26-85% cost savings versus unconstrained API calls
- Maintained or improved accuracy on downstream tasks
- Supports typed outputs, beam search, constrained decoding

**Significance for lingo:** LMQL approaches the problem from the host-language side (Python) rather
than the target-language side. It shows that structured, constraint-aware generation frameworks can
significantly reduce wasted token production.

**Source:** [arXiv:2212.06094](https://arxiv.org/abs/2212.06094) |
[lmql.ai](https://lmql.ai/)

---

### 1.4 SudoLang: Pseudocode for LLMs (2023)

Eric Elliott designed SudoLang in 2023 (co-designed with GPT-4) as a pseudolanguage combining
natural language with programming constructs for LLM interaction. The core claim is that SudoLang
was not invented but "discovered" — it captures latent pseudocode understanding already present in
LLMs.

**Design goals:** Structured, predictable LLM interactions; fewer misunderstandings;
easier-to-review outputs. Intended as a language LLMs write *for* LLMs, not a language humans
primarily read.

**Significance for lingo:** An early non-academic example of deliberately designing a language for
the LLM generation target, though without rigorous token-efficiency evaluation.

**Source:** Medium: "SudoLang: A Powerful Pseudocode Programming Language for LLMs" by Eric
Elliott (April 2023) |
[Hacker News discussion](https://news.ycombinator.com/item?id=35424835) |
[O'Reilly Radar][sudolang-oreilly]

[sudolang-oreilly]: https://www.oreilly.com/radar/unlocking-the-power-of-ai-driven-development-with-sudolang/

---

## 2. Token Efficiency Measurements Across Existing Languages

### 2.1 Comparative Analysis: 19 Languages (January 2026)

Martin Alderson published a widely-discussed analysis measuring token counts across 19 programming
languages using 1,000+ RosettaCode tasks and the GPT-4 tokenizer. The results circulated heavily on
Hacker News (January 2026) and spawned discussions on Lobste.rs and Julia Discourse.

**Top performers (most token-efficient):**

| Language | Avg Tokens | Notes |
| -------- | ---------- | ----- |
| J | ~70 | Array language; ASCII-only symbols; very terse |
| Clojure | ~109 | Lisp-family; minimal ceremony; most efficient "mainstream" language |
| APL | ~110 | Symbol-heavy; BUT tokenizer handles its glyphs poorly |
| F# | low | Functional; type inference eliminates verbosity |
| Haskell | low | Type inference; declarative style |

**Least efficient:**

- C: ~283 tokens average — 2.6x more than the most efficient languages
- JavaScript: most verbose of the dynamic languages

**Key design insights from community discussion:**

- Dynamic languages are generally more token-efficient (no type annotations to write)
- Functional languages with type inference partially close the gap to dynamic languages
- Tokenizer optimization for a language's character set matters as much as brevity: APL's exotic
  symbols are each tokenized as multiple tokens, neutralizing its surface terseness
- Strong type systems act as "scaffolding" that reduces LLM error rate even if they cost some tokens
- F# is a statistically significant outlier: a statically-typed language with near-dynamic-language
  token counts, due to inference

**Significance for lingo:** These results reveal that optimal LLM language design is not simply
"write less" — it requires co-optimization of the character set, the tokenizer, and the type system.

**Source:** [Martin Alderson's post][alderson-post] |
[Hacker News thread](https://news.ycombinator.com/item?id=46582728) |
[Lobsters](https://lobste.rs/s/nx9uwg/which_programming_languages_are_most) |
[F# Weekly #3, 2026][fsharp-weekly]

[alderson-post]: https://martinalderson.com/posts/which-programming-languages-are-most-token-efficient/
[fsharp-weekly]: https://sergeytihon.com/2026/01/17/f-weekly-3-2026-most-token-efficient-static-language/

---

### 2.2 OckBench: Measuring LLM Reasoning Efficiency (NeurIPS 2025 Workshop)

Du et al. (arXiv:2511.05722) published OckBench, the first benchmark that jointly measures LLM
accuracy *and* token efficiency across reasoning and coding tasks. Presented at the NeurIPS 2025
Workshop on Efficient Reasoning.

**Key finding:** Models solving the same problem with similar accuracy can use up to **5.0x more
tokens** than the most efficient model. Token efficiency is largely unoptimized in current frontier
models.

**Significance for lingo:** Establishes that output token count is a first-class metric, not a
secondary concern. Provides an evaluation framework that a lingo benchmark could adopt.

**Source:** [arXiv:2511.05722](https://arxiv.org/abs/2511.05722) |
[OckBench project page](https://ockbench.github.io/)

---

## 3. Compressed and Intermediate Code Representations

### 3.1 IRCoder: Compiler IR as Shared Representation (ACL 2024)

Paul et al. (ACL 2024, arXiv:2403.03894) trained LLMs on compiler intermediate representations
(LLVM IR) alongside source code. The hypothesis: compiler IR is a language-agnostic, semantically
dense representation that can serve as a shared backbone for multilingual code generation.

**Method:** Created SLTrans, a ~4M sample parallel corpus of source code paired with compiler IR.
Fine-tuned code LLMs to learn and align the IR with surface languages.

**Results:** Consistent gains on multilingual code generation benchmarks (Multipl-E), with up to
+2.23 pass@1 points for CodeLlama 6.7B. Also improves prompt robustness and instruction following.

**Significance for lingo:** Compiler IR is a form of "semantic compression" — it expresses meaning
densely without syntactic ceremony. This is the spirit of what lingo could formalize as a designed
target representation.

**Source:** [ACL Anthology](https://aclanthology.org/2024.acl-long.802/) |
[arXiv:2403.03894](https://arxiv.org/abs/2403.03894) |
[GitHub: UKPLab/acl2024-ircoder](https://github.com/UKPLab/acl2024-ircoder)

---

### 3.2 TreeDiff: AST-Guided Diffusion Code Generation (arXiv, August 2025)

Zhang et al. (arXiv:2508.01473) propose a diffusion-based code generation framework that uses
Abstract Syntax Trees to guide the noise corruption and denoising process.

**Core insight:** Random token masking is wrong for code because it ignores syntactic structure.
TreeDiff uses AST-guided masking — it corrupts and recovers whole subtrees, preserving structural
integrity during denoising.

**Significance for lingo:** Demonstrates that AST structure is a natural intermediate representation
for generation. A lingo target that serializes to a tree-structured format may reduce structural
errors versus flat token generation.

**Source:** [arXiv:2508.01473](https://arxiv.org/abs/2508.01473)

---

### 3.3 ShortCoder: Syntax-Level Code Simplification (arXiv, January 2026)

Li et al. (arXiv:2601.09703) published ShortCoder, the most recent and directly relevant work: a
framework for training LLMs to generate more token-efficient Python by applying AST-preserving
syntax simplification rules.

**Method:**

- Defines 10 syntax-level simplification rules for Python (AST-preserving transformations)
- Builds ShorterCodeBench: a corpus of (original, simplified) code pairs
- Fine-tunes LLMs with "conciseness awareness" using this corpus

**Results:**

- 18.1% token reduction (rule-based baseline)
- Up to 37.8% improvement in generation efficiency over prior methods
- Performance maintained on HumanEval benchmarks

**Key difference from SimPy:** ShortCoder trains the model to produce shorter code; SimPy modifies
the grammar/format. Both target the same problem from different angles.

**Significance for lingo:** ShortCoder demonstrates that learned compression of code (not just
rule-based) is viable and yields larger gains than grammar-level changes alone. A lingo compiler
could combine both approaches.

**Source:** [arXiv:2601.09703](https://arxiv.org/abs/2601.09703) |
[Quantum Zeitgeist writeup](https://quantumzeitgeist.com/37-8-percent-shortcoder-achieves-more-efficient-code/)

---

### 3.4 LLMLingua: Prompt Compression (EMNLP 2023 / ACL 2024)

Microsoft Research's LLMLingua family (Jiang et al.) compresses *input* prompts to LLMs using a
small auxiliary model that identifies and removes low-information tokens.

**Results:**

- Up to 20x compression with minimal performance loss on reasoning tasks
- LLMLingua-2 (ACL 2024): up to 6x faster than LLMLingua, handles out-of-domain data better

**Distinction from lingo:** LLMLingua compresses the *input context* given to an LLM, not the
*output code* the LLM generates. However, the technique of using perplexity to identify token
information density is applicable to designing target representations.

**Source:** [GitHub: microsoft/LLMLingua](https://github.com/microsoft/LLMLingua) |
[EMNLP 2023](https://aclanthology.org/2023.emnlp-main.825/)

---

### 3.5 CodeFast: Excess Token Prevention at Inference (ISSTA 2024)

Sun et al. (arXiv:2407.20042, ISSTA 2024) built CodeFast, a lightweight model (GenGuard) that
predicts at each decoding step whether the LLM has finished generating useful code and should stop.

**Results:** 34-452% speedup across five code LLMs and four datasets, without compromising code
quality.

**Significance for lingo:** Addresses wasted tokens from the *output end* — models frequently
over-generate. A lingo grammar with unambiguous termination signals could make this unnecessary.

**Source:** [arXiv:2407.20042](https://arxiv.org/abs/2407.20042)

---

## 4. Languages and DSLs Designed for AI Agents

### 4.1 AgentSpec: Runtime Enforcement DSL (ICSE 2026)

Wang et al. (arXiv:2503.18666, ICSE 2026) designed AgentSpec, a DSL for specifying and enforcing
safety constraints on LLM agents at runtime.

**Language design:** Rules are structured as `trigger → predicate → enforcement_action`. Predicates
can be user-written or LLM-generated. The runtime intercepts agent actions before execution and
evaluates them against the rule set.

**Results:** >90% prevention of unsafe code agent executions; 100% compliance for autonomous
vehicle agents in 5 of 8 law-enforcement scenarios.

**Significance for lingo:** A concrete example of a small, purpose-built language that LLMs can
generate reliably for a specific domain. The trigger/predicate/action structure is easy for LLMs to
produce because it is compositional and syntactically regular.

**Source:** [arXiv:2503.18666](https://arxiv.org/abs/2503.18666)

---

### 4.2 Declarative LLM Agent Workflow Languages (December 2025)

Multiple papers and tools converged on YAML/JSON-based declarative languages for LLM agent
orchestration in late 2025:

- **Open Agent Specification (OAS)** (arXiv:2510.04173, October 2025): A declarative language
  defining agents and workflows in a framework-agnostic way. Separates behavioral intent from
  implementation; one definition runs on Java, Python, or Go backends.
- **ADL (Agent Definition Language)** (arXiv:2504.14787): Four agent types (Knowledge Base, LLM,
  Flow, Ensemble) expressed in a declarative config. Flow agents support traditional control flow
  via DSL.
- **Declarative LLM Agent Workflows** (arXiv:2512.19769, December 2025): Survey/proposal for
  separating agent workflow specification from imperative code. Common patterns (RAG retrieval, API
  calls, filtering) expressed as a unified DSL.

**Significance for lingo:** These works show strong demand for LLM-writable specification languages
with minimal boilerplate. They favor YAML/JSON as syntax, which is token-heavy but human-readable.
A lingo design could explore whether a more compact structural format achieves the same
expressiveness.

**Source:** [arXiv:2510.04173 (OAS)](https://arxiv.org/pdf/2510.04173) |
[arXiv:2512.19769](https://arxiv.org/abs/2512.19769) |
[arXiv:2504.14787 (ADL)](https://arxiv.org/html/2504.14787)

---

### 4.3 AutoDSL: Automated DSL Design for Procedural Constraints (ACL 2024)

Shi et al. (ACL 2024, arXiv:2406.12324) presented AutoDSL, a framework that automatically designs a
DSL for a target domain by optimizing syntactic and semantic constraints from a corpus of domain
examples.

**Method:** Two-stage: (1) design a DSL via bidirectional grammar optimization, (2) use the DSL as
a plug-and-play constraint module for LLMs, preventing free-form/non-deterministic procedural
outputs.

**Significance for lingo:** AutoDSL inverts the typical approach — rather than humans designing a
DSL that LLMs then generate, it uses LLMs to design a DSL that constrains subsequent LLM
generation. Relevant if lingo's grammar is intended to be derived or evolved automatically.

**Source:** [ACL Anthology](https://aclanthology.org/2024.acl-long.659/) |
[arXiv:2406.12324](https://arxiv.org/abs/2406.12324)

---

### 4.4 Wasp: Token-Efficient Full-Stack DSL (Production, 2023-present)

Wasp is an open-source, production DSL (with 15,000+ GitHub stars) for building full-stack web apps
(React + Node.js + Prisma). Its design predates but directly anticipates LLM integration: a
high-level `.wasp` config file replaces boilerplate, and the Wasp compiler generates all imperative
code.

**Token efficiency claim:** "AI-generated Wasp apps use ~10-40x less tokens (input and output text)
than comparable tools." The claim is that the DSL's high-level abstractions give LLMs clear
structural guardrails with minimal boilerplate to generate.

**Significance for lingo:** The most mature real-world example of a DSL achieving large token-count
reductions through abstraction rather than compression. Validates the hypothesis that raising the
abstraction level of the target language is more powerful than syntactic trimming.

**Source:** [Wasp GitHub](https://github.com/wasp-lang/wasp) |
[Wasp blog: LLM-powered framework](https://wasp.sh/blog/2025/04/01/wasp-first-full-stack-framework-powered-by-llm)

---

### 4.5 DSL-Xpert: LLM-Driven DSL Code Generation (MODELS 2024)

Garcia-Gonzalez et al. (MODELS 2024) built DSL-Xpert, a tool for generating code in arbitrary,
unpublished DSLs by providing the grammar and few-shot examples as context to an LLM.

**Key finding:** LLMs can generate reliable code for novel DSLs when given the grammar specification
as context. Grammar prompting + few-shot learning is sufficient for reliable DSL code generation
without fine-tuning.

**Significance for lingo:** Demonstrates that a new language does not need to be pre-trained into an
LLM to be useful. A well-specified grammar in the prompt is sufficient, lowering the barrier for
deploying lingo.

**Source:** [ACM DL](https://dl.acm.org/doi/abs/10.1145/3652620.3687782) |
[PDF](https://victorjlamas.github.io/assets/papers/LLMXpertMODELS2024.pdf)

---

## 5. Community Discussions and Blog Analysis

### 5.1 Hacker News: Token Efficiency Across Languages (January 2026)

The January 2026 Hacker News thread on Martin Alderson's analysis generated substantive discussion.
Key themes from highly-voted comments:

- **Functional programming is "outsize" for LLMs:** "Pure statically typed functional languages are
  incredibly well suited for LLMs" due to referential transparency (no hidden state to track) and
  the ability to validate generated code against types without full execution.
- **Type systems as scaffolding:** Strong types provide immediate feedback without requiring full
  codebase context. "Constraining LLM output through types has an outsize effect."
- **Training data vs. syntax:** Less common languages (Haskell) perform surprisingly well despite
  small training corpora — once a baseline exists, tooling quality matters more than corpus size.
- **Active building:** Several commenters mentioned building LLM-optimized languages, including a
  dependently-typed theorem proving language emphasizing regular syntax and mathematical
  expressiveness.
- **Tokenizer co-design:** The APL case (terse but tokenizer-hostile) was frequently cited as
  evidence that language designers must think about tokenizer behavior alongside syntax.

**Source:** [Hacker News thread](https://news.ycombinator.com/item?id=46582728) |
[Lobsters thread](https://lobste.rs/s/nx9uwg/which_programming_languages_are_most)

---

### 5.2 "The Return of Language-Oriented Programming" (Blog, November 2025)

Enrico Vacchi's blog post argues that LLMs reverse the economics of DSL creation: LLMs can generate
DSL implementations *and* documentation automatically, eliminating the maintenance overhead that
previously made small DSLs impractical.

**Design principles proposed:**

- Design for the LLM's context window, not human readability
- Prefer domain-specific scope (small, focused DSLs) over general-purpose languages
- Use "middle-out" development: define domain syntax first, then build the compiler and the system
  simultaneously using that syntax

**Source:**
[Blog post](https://blog.evacchi.dev/posts/2025/11/09/the-return-of-language-oriented-programming/)

---

### 5.3 Token-Aware Code Repair: NanoSurge (arXiv, April 2025)

Hu et al. (arXiv:2504.15989) studied token consumption in LLM-driven code repair. They found that
code "smells" in chain-of-thought reasoning inflate token counts substantially.

**Results:**

- Refactored code (with code smells removed) reduces token consumption by up to 50%
- Explicit role/responsibility prompting reduces token usage 24.5-30%
- The "NanoSurge" approach (Context Awareness + Responsibility Tuning + Cost Sensitive strategies)
  significantly reduces chain-of-thought verbosity

**Significance for lingo:** Token waste often comes from LLM reasoning *about* code, not just
writing it. A lingo grammar with unambiguous, compositional semantics may reduce reasoning
verbosity by making the model's job more deterministic.

**Source:** [arXiv:2504.15989](https://arxiv.org/abs/2504.15989)

---

## 6. Summary Table: Prior Art Landscape

| Work | Type | Year | Venue | Token Reduction | Approach |
| ---- | ---- | ---- | ----- | --------------- | -------- |
| SimPy (Sun et al.) | Academic | 2024 | ISSTA | 10-14% | AI-oriented grammar; removes formatting tokens |
| ShortCoder (Li et al.) | Academic | 2026 | arXiv | 18-38% | Fine-tunes LLMs to generate simplified code |
| MoonBit (Fei et al.) | Academic | 2024 | LLM4Code | Qualitative | Type-system-guided LLM feedback loops |
| LMQL (Beurer-Kellner et al.) | Academic | 2023 | PLDI | 26-85% cost | Constrained decoding; query language |
| IRCoder (Paul et al.) | Academic | 2024 | ACL | N/A | Compiler IR as shared intermediate representation |
| TreeDiff (Zhang et al.) | Academic | 2025 | arXiv | N/A | AST-guided diffusion; structural generation |
| CodeFast (Sun et al.) | Academic | 2024 | ISSTA | 34-452% speed | Stops generation at task completion |
| LLMLingua (Jiang et al.) | Academic | 2023/24 | EMNLP/ACL | 20x (input) | Prompt compression; input-side only |
| OckBench (Du et al.) | Benchmark | 2025 | NeurIPS WS | N/A | Jointly measures accuracy + token efficiency |
| AgentSpec (Wang et al.) | Academic | 2025 | ICSE 2026 | N/A | Runtime enforcement DSL for agent safety |
| AutoDSL (Shi et al.) | Academic | 2024 | ACL | N/A | Automated DSL design for structured output |
| DSL-Xpert (Garcia-Gonzalez) | Tool | 2024 | MODELS | N/A | Grammar-prompted LLM generation for DSLs |
| Wasp | Production | 2023+ | OSS | 10-40x (claimed) | High-level DSL; abstraction vs. compression |
| SudoLang (Elliott) | Blog/OSS | 2023 | Medium | Qualitative | Pseudolanguage for LLM interaction |
| Martin Alderson analysis | Empirical | 2026 | Blog | 2.6x range | Token count comparison across 19 languages |

---

## 7. Key Gaps and Open Problems

Based on this survey, the following problems remain largely unsolved:

1. **No general-purpose LLM-native language exists.** SimPy is a Python dialect; ShortCoder
   fine-tunes models on simplified Python; MoonBit is a new language but focused on correctness,
   not token economy. No one has designed a full general-purpose language from first principles for
   LLM generation efficiency.

2. **Tokenizer co-design is ignored.** All existing work uses existing tokenizers and optimizes
   code representation around them. The inverse — designing a language and tokenizer together —
   has not been explored.

3. **Abstraction-level tradeoffs are unmeasured.** Wasp claims 10-40x reduction via high
   abstraction, but there is no systematic study of how abstraction level affects token efficiency
   across domains. The SimPy approach (~14%) and the Wasp approach (~10-40x) operate at different
   layers and have never been compared rigorously.

4. **Output vs. input compression.** LLMLingua compresses input; ShortCoder/SimPy compress output.
   No system jointly optimizes both directions.

5. **Semantics-preserving compression as a compilation target.** The idea of a language whose
   compiler outputs both a human-readable form *and* an LLM-efficient form (as SimPy partially
   realizes) has not been generalized into a language design framework.

6. **Community but no consensus.** The Hacker News discussion shows substantial interest in
   LLM-optimized language design, but no community has coalesced around a shared design or
   benchmark.

---

## 8. Confidence Assessment

- **High confidence:** SimPy, MoonBit, LMQL, IRCoder, ShortCoder, LLMLingua, CodeFast, AgentSpec,
  AutoDSL — all peer-reviewed with reproducible results.
- **High confidence:** Martin Alderson token analysis — methodology is transparent; results are
  consistent with Julia Discourse and community observations.
- **Medium confidence:** Wasp's 10-40x token reduction claim — plausible given the abstraction
  level, but self-reported and domain-specific (web apps), not independently benchmarked.
- **Medium confidence:** SudoLang — influential community artifact but no peer-reviewed evaluation.
- **Low confidence:** Claims about which languages LLMs "prefer" absent fine-tuning — confounded by
  training data distribution.

---

## 9. Recommended Next Steps for lingo Research

1. **Read SimPy in full** (arXiv:2404.16333) — the closest prior art; understand which grammar
   rules were changed and why.
2. **Examine ShortCoder's 10 simplification rules** (arXiv:2601.09703) — likely overlapping with
   lingo design space.
3. **Benchmark lingo against OckBench's methodology** (arXiv:2511.05722) — adopt joint accuracy +
   token-efficiency evaluation from the start.
4. **Study MoonBit's type-system feedback loop** — the real-time semantics-based sampler may be
   more valuable than raw token reduction for correctness.
5. **Consider tokenizer co-design** — the APL lesson (surface terseness neutralized by tokenizer
   hostility) suggests lingo's character set must be chosen with the target tokenizer in mind.
6. **Differentiate from Wasp** — Wasp proves high-abstraction DSLs can dramatically reduce tokens,
   but only in a narrow domain. Lingo's value proposition should articulate what "general-purpose"
   means in this context.

---

*Research conducted March 15, 2026. All links verified as of research date.*
