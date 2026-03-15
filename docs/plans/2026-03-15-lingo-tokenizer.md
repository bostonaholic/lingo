# LLM Tokenizer Research: Programming Language Token Mechanics

Research date: 2026-03-15

---

## Executive Summary

BPE tokenizers like GPT-4's `cl100k_base` and GPT-4o's `o200k_base` treat
programming languages very unevenly. Formatting overhead alone consumes 6–35%
of all tokens depending on the language. Across equivalent programs, the best
mainstream language (Clojure, ~109 tokens average) uses roughly 2.6x fewer
tokens than the worst (C, ~283 tokens). The ASCII array language J achieves
~70 tokens—less than half of Clojure—while APL, despite being terser in
character count, is penalized by exotic Unicode glyphs that BPE splits into
multiple tokens each. LLMs overwhelmingly prefer Python (90–97% of
spontaneous code generation) and degrade significantly on low-resource
languages. The evidence on whether terse code degrades LLM reasoning is
contested: one widely-cited study was later corrected and its conclusion
reversed.

---

## 1. Tokenizer Mechanics for Code

### 1.1 How BPE Splits Programming Constructs

BPE tokenizers are trained by iteratively merging the most frequent byte
pairs in a corpus until a vocabulary limit is reached. The practical effect
on code:

- **Common keywords become single tokens.** Words like `def`, `class`,
  `return`, `import`, `if`, `else`, `for`, `while` appear frequently enough
  in training corpora that they receive dedicated single-token IDs in
  `cl100k_base`. The same applies to common JavaScript keywords (`function`,
  `const`, `let`, `var`) and Python builtins. The reverse-engineering of the
  GPT-5/o200k_base vocabulary confirmed 100% single-token coverage for all
  common keywords including `return`, `import`, `export`, `async`, `await`,
  `null`, `undefined`, `true`, and `false`.
- **Less common keywords fragment.** Keywords from lower-frequency languages
  (e.g., Haskell's `deriving`, `newtype`, Rust's `impl`, `trait`) may merge
  into single tokens or split depending on corpus size during tokenizer
  training.
- **Operators tokenize compactly.** Short operators (`+`, `-`, `*`, `/`,
  `=`, `==`, `!=`, `->`, `=>`, `::`, `&&`, `||`) tend to be single tokens
  because they are extremely frequent in code. Multi-character compound
  operators are usually learned as single merges.
- **Identifiers split at subword boundaries.** A descriptive name like
  `calculateTotalCost` is typically split into 3–5 tokens
  (`calculate`, `Total`, `Cost` or similar subword pieces), while a
  single-letter variable like `x` or `i` is always exactly one token.
- **The low token IDs reveal training priorities.** In cl100k_base, 62 of
  the first 1,000 token IDs relate directly to code: parentheses, brackets,
  statement terminators, and indentation patterns. Token ID 257 is a 4-space
  indent. Token ID 269 is an 8-space indent. Token ID 362 is `);` followed
  by a newline. Token ID 393 is `//`. These assignments signal that code
  dominated OpenAI's pre-training corpus.

### 1.2 The Regex Pretokenizer

Before BPE merges run, `cl100k_base` applies a regex pretokenizer that
enforces hard split boundaries. No merge may cross these boundaries. The
pattern:

```text
(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{2,}|
[^\r\n\p{L}\p{N}]?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+(?!\S)|\s+
```

Key implications for code:

- Numbers with 2+ digits are grouped (e.g., `100` is one token, not three).
- Letters and non-letter characters (like `_`, `$`, `@`) cannot merge across
  word boundaries, preventing `def_foo` from collapsing into one token.
- Newlines are preserved as separate split points.

`o200k_base` upgrades `\p{L}` to granular Unicode categories (`\p{Lu}`,
`\p{Lt}`, `\p{Lm}`, `\p{Lo}`, `\p{M}`, `\p{Ll}`), capturing diacritical
marks within word boundaries—primarily a multilingual improvement with
limited effect on ASCII code. It also introduces native camelCase/PascalCase
splitting: `ChatGPT` divides into separate chunks at the regex level before
BPE processing, enabling more regular handling of camelCase identifiers.

### 1.3 Whitespace and Indentation

This is the most practically significant difference between tokenizer
generations for Python and other indented code:

- **GPT-2:** Each space was an individual token. Four spaces of indentation
  = four tokens. A Python FizzBuzz script encoded to **149 tokens**.
- **GPT-4 (`cl100k_base`):** The very first BPE merge learned during
  training was two spaces → one token (token ID 256), a direct signal that
  code dominated the training set. Four consecutive spaces became token ID
  257 (a single token). The tokenizer has dedicated tokens for whitespace
  runs up to 83 consecutive spaces. The same FizzBuzz script encodes to
  **77 tokens**—a ~48% reduction.
- **`o200k_base`:** Continues this approach with a doubled vocabulary;
  improvements for code over `cl100k_base` are modest (roughly 9% for
  JavaScript documentation comments in one measured example) compared to the
  large multilingual gains.

Measured across 10 LLMs (5 commercial, 5 open-source) on the McEval
benchmark (arXiv:2508.13666), formatting elements consume tokens as follows:

For Claude-3.7: newlines contribute ~14.6% of tokens on average, indentation
~7.9%, whitespace ~5.2%. For Gemini-1.5: newlines ~17.5%, indentation ~8.9%,
whitespace ~6.3%. For GPT-4o: whitespaces lead at ~10.7%, indentation ~9.6%,
newlines ~7.5%.

**Removing all optional formatting reduced input tokens by:**

- Java: 34.9% average
- C++: 31.1% average
- C#: 25.3% average
- Python: 6.5% (limited, since indentation is semantic and cannot be removed)

Commercial models (Claude-3.7, Gemini-1.5, GPT-4o) saw input reductions of
23–28%. Critically, output tokens only shrank by 2.5% on average—LLMs tend
to reproduce the formatting style of what they generate regardless of input
formatting, so context savings do not automatically translate to output
savings.

### 1.4 cl100k_base vs. o200k_base for Code

cl100k_base (used by GPT-4, Claude-class models) has approximately 100,000
vocabulary entries. o200k_base (used by GPT-4o, o3-mini, o4-mini) doubled
the vocabulary to approximately 200,000.

The expanded vocabulary enables o200k_base to assign dedicated tokens to more
common code patterns. A JavaScript function with JSDoc comments costs about
68 tokens in cl100k_base and 62 tokens in o200k_base, roughly a 9%
improvement. The gains are more pronounced for non-English natural language
text—Turkish drops from 28 to 22 tokens (21% reduction)—but code benefits
measurably from the larger vocabulary. The o200k_base tokenizer achieves 100%
single-token coverage for all common programming keywords.

---

## 2. Language Token Efficiency Comparison

### 2.1 The Primary Benchmark (Rosetta Code / GPT-4 Tokenizer)

Martin Alderson's analysis (January 2026) tokenized solutions to 1,000+
Rosetta Code tasks across 19 languages using the Xenova/GPT-4 (`cl100k_base`)
tokenizer on Hugging Face, comparing only tasks with solutions in all 19
languages.

**Complete rankings (average tokens per solution):**

- J: 70 tokens (Array, ASCII)
- Clojure: 109 tokens (Functional/Lisp)
- APL: 110 tokens (Array, Unicode) — on a smaller dataset
- Haskell: 115 tokens (Functional/Static)
- F#: 118 tokens (Functional/Static)
- Python: ~130 tokens (Dynamic)
- JavaScript: ~148 tokens (Dynamic)
- C: 283 tokens (Procedural, worst)

The full 19-language table was not published in sources accessed. The 2.6x
gap between first and last place is confirmed.

**Note on APL vs. J:** APL ranks 2nd by token count but this is misleadingly
favorable—its unique Unicode glyphs (⍳, ⍴, ⌽, etc.) are each split into
multiple BPE tokens. APL solutions are extremely short in character count,
which partially compensates, but tokenizers are not optimized for its symbol
set. J, which expresses the same array programming paradigm using pure ASCII
symbols, achieves ~half the tokens of Clojure with no glyph penalty.

**Why C ranks last:** C's syntactic terseness is deceptive at the program
level. Operations that other languages provide through standard libraries
(string manipulation, data structures, memory management) must be implemented
explicitly in C, dramatically inflating token counts for equivalent
functionality.

### 2.2 Corroborating Real-Project Evidence

The adriangalilea/vibe-coding-lang-bench study measured complete idiomatic
implementations of two real programs: a CLI task manager and a REST API with
CRUD. Using cl100k_base:

- Python: 4,322 total tokens (baseline)
- Elixir: 5,147 tokens (1.2x Python)
- Rust: 6,064 tokens (1.4x Python)

For the REST API specifically: Python 2,487 tokens vs Rust 3,669 tokens
(1.5x). The methodology included configuration files (Cargo.toml, pyproject.toml,
mix.exs), making this a more realistic measure of actual project cost.

An iteration test adding computed fields and cascade deletion found Python
requiring changes to 2 files (+154 tokens), while Rust required changes to 3
files (+219 tokens). Rust's type system creates cascading modifications when
data shapes change.

### 2.3 Ruby

Ivan Turtkovic's 2026 analysis (drawing on Alderson's dataset) reported Ruby
ranked second among mainstream languages in the Rosetta Code comparison, behind
only Clojure. Ruby's efficiency comes from eliminating semicolons, type
annotations, and curly braces; from symbol-to-proc syntax (`&:method`) and
implicit returns; and from Rails convention patterns like `has_many :posts`
that pack complex functionality into minimal tokens.

### 2.4 J vs APL: The ASCII Paradox

J achieves 70 tokens average—about half of Clojure—while APL sits at 110
despite APL being arguably more terse visually. J uses pure ASCII for its
operators (`+/`, `{.`, `#:`), while APL uses Unicode glyphs (`⍳`, `⍴`, `⌽`).

BPE tokenizers are trained overwhelmingly on ASCII text. APL's exotic Unicode
symbols appear so rarely in training data that they never get merged into
multi-character tokens. Each glyph costs one to three tokens instead of one.
APL's surface-level terseness becomes token-inefficient because the tokenizer
handles its symbol set poorly. J avoids this entirely by committing to ASCII.

The implication: terseness in ASCII is rewarded by current BPE tokenizers;
terseness via exotic Unicode symbols is penalized.

### 2.5 Go, Rust, Perl, Lua, K

These languages were not part of the primary 19-language Rosetta Code
benchmark.

**Go** is consistently described across sources as more verbose than Python
due to explicit error handling patterns (`if err != nil { return err }`),
required return types on every function, and limited type inference beyond
`:=` for local variables. Multiple analyses place Go in the "verbose/explicit
typing" tier alongside Java and C#, which consume the most tokens among
mainstream languages.

**Rust** falls between Haskell (aggressive global inference) and Go/Java
(annotation-heavy) based on measured benchmarks. Rust's local type inference
eliminates annotation cost within function bodies, but function signatures
require explicit parameter and return types. The practical result is ~1.4-1.5x
Python for real projects. Rust's biggest token penalty comes from structural
type changes that cascade across multiple files when a data shape is modified.

**Perl** was not benchmarked. Perl's sigil system (`$scalar`, `@array`,
`%hash`) adds tokens to every variable reference. However, Perl's regex
integration and terse idioms compress logic significantly. No systematic
comparison data was found.

**Lua** was not benchmarked. Lua is dynamically typed with minimal syntax and
no type annotations. It would likely perform comparably to Python, possibly
slightly worse due to verbose `function`/`end` block delimiters (no closing
keyword is free; `end` costs a token).

**K** (the Kx Systems array language) was not benchmarked. K shares J's
commitment to ASCII-based operators but has an even smaller character set.
Based on J's performance, K would likely tokenize similarly well, but this
is inference from first principles rather than measured data.

### 2.6 Haskell and F# Outperforming Intuition

Haskell (115 tokens) and F# (118 tokens) nearly match dynamically typed
Python (130 tokens) despite being statically typed. The explanation is
Hindley-Milner global type inference: the compiler infers the type of every
expression without annotation. In practice Haskell programmers write type
signatures as documentation, but idiomatic Rosetta Code solutions often omit
them entirely. The result is code that is compact without the annotation
overhead that weighs down Java or Go.

---

## 3. Token-Saving Syntactic Patterns

### 3.1 Short Keywords vs. Long Keywords

Short reserved words are almost universally single tokens, and their brevity
compounds across a codebase:

- Function declaration: `fn foo` (Rust, 1 token) vs `function foo` (JS, 1
  token). Both keywords are single tokens; the character difference matters
  more for identifiers that don't have dedicated vocabulary slots.
- End delimiter: `end` (Ruby/Lua) vs `}` (C/Java). Both are roughly 1 token;
  the difference is negligible for delimiters specifically.
- Absence of access modifiers: Python, Ruby, Haskell, and Clojure have no
  `public`/`private`/`protected` on methods by default. Java and C# add 1-2
  tokens to every method declaration.
- Module system: Rust `mod foo` vs Java `namespace foo {` — fewer tokens.

The token count for `function` in JavaScript cl100k_base is 1 token (it is
common enough to be a merged token). Rust's `fn` is also 1 token. The savings
from short keywords are modest per occurrence but matter at scale across an
entire codebase.

### 3.2 Sigils and Operators vs. Spelled-Out Words

Operators are generally single tokens because they are extremely frequent in
training data. Languages that express operations symbolically are at an
advantage:

- `&&` vs `and`, `||` vs `or`: In Python, `and`/`or` are single tokens too,
  so the gap is minimal here.
- `->`, `=>`, `::`: All typically single tokens in BPE vocabularies trained
  on code.
- Method chaining with `.` is 1 token for the dot, versus verbose OOP
  accessor patterns that may span many tokens.
- Ruby's `&:` sigil shorthand (`array.map(&:upcase)`) packs a full method
  reference into 2 tokens; the equivalent lambda `{ |x| x.upcase }` costs
  roughly 8–10 tokens.

Perl's sigils (`$`, `@`, `%`) are each single characters and likely single
tokens, but they appear in front of every variable name, adding one token per
variable access. In a program with heavy variable manipulation, this
accumulates.

### 3.3 Whitespace Sensitivity vs. Delimiter-Based

Python's mandatory indentation imposes a token cost that cannot be optimized
away—removing formatting only reduced Python token counts by 6.5% in the
arXiv study, compared to 25–35% for delimiter-based languages. However,
Python's lack of explicit type annotations and concise syntax elsewhere
partially compensates.

Ruby uses `end` keywords rather than braces. Both are single tokens. The
difference is that brace languages incur both the brace tokens and
conventional indentation, doubling the structural overhead.

### 3.4 Type Inference vs. Explicit Types

Every explicit type annotation is tokens spent on information the compiler
could derive. This is a primary driver of differences between language tiers:

- **Go** requires explicit type declarations in many contexts: `var x int =
  5` vs Python's `x = 5`. Each `int`, `string`, `bool` annotation is an
  extra token.
- **Java** requires type annotations everywhere: `List<String> names = new
  ArrayList<>()` vs Python's `names = []`. A Java class definition reduced
  from 60 to 47 tokens through format removal alone (14.7% savings from
  whitespace), but its type annotations are semantically required and
  unremovable.
- **Haskell/F#** use Hindley-Milner type inference; the type signature
  `f :: Int -> Int` is optional in most cases, and the compiler infers it.
  This is why Haskell and F# reach near-Python efficiency (115–118 tokens)
  despite being statically typed.
- **Rust** uses type inference within function bodies but requires explicit
  signatures at function boundaries. One analysis reported type inference
  yielding a 30–40% token reduction for array-heavy data structures in
  applications that previously required explicit generic type parameters.
  Still more efficient than Java or C#.
- **Python with type hints**: Optional but increasingly common. When used
  extensively, costs roughly what Java costs.

### 3.5 Identifier Length

Single-letter identifiers (`x`, `i`, `n`, `f`) are always exactly 1 token.
Descriptive names like `calculateTotalRevenue` split into 3–5 subword tokens.
In dense mathematical/algorithm code, short identifiers are a meaningful
token saver; in large codebases, descriptive names are worth the cost for
the LLM's ability to reason about intent (see section 4 for the quantified
tradeoff).

### 3.6 Data Format Comparison (Structural Notation)

From mattrickard.com's analysis of data serialization formats:

- XML: 201 tokens (~400 characters)
- Standard JSON: 162 tokens (337 characters)
- TOML: 91 tokens (~230 characters)
- YAML: 85 tokens (227 characters)
- HCL: 79 tokens (~210 characters)
- INI: 84 tokens (~200 characters)
- Minified JSON: 64 tokens (223 characters)

YAML and minified JSON are roughly equivalent and both dramatically more
efficient than XML or formatted JSON. Tokenizer choice matters: LLaMA's
tokenizer produced different counts than OpenAI's for the same inputs.

---

## 4. LLM Reasoning Quality vs. Token Count

### 4.1 LLM Language Bias

LLMs exhibit strong systematic bias toward Python (arXiv:2503.17181, 2025):

- Across diverse benchmarks, LLMs generate Python for 90–97% of all tasks.
- Even when Python is demonstrably unsuitable (high-performance computing,
  system programming), models choose Python 58% of the time in project
  initialization tasks.
- LLMs only use 6–14 different languages despite hundreds being available.
- **Critically: LLMs contradict their own language recommendations 83% of
  the time.** When asked which language is best, their answer has near-zero
  correlation (Kendall's τ ≈ 0) with what they actually generate.

This bias means LLM reasoning quality for most languages other than Python
and JavaScript is empirically lower, regardless of tokenizer efficiency.

### 4.2 Low-Resource Language Performance

From the ACM survey on LLM code generation for low-resource languages
(arXiv:2410.03981, 2024):

- **Pass@1 accuracy:** Python/JavaScript/Java achieve 50–75%. R, Racket,
  Perl, Swift, Golang: ≤30%.
- APL, J, BQN are extreme cases—classified as "low-resource programming
  languages" (LRPLs) with severe data scarcity. No standardized benchmarks
  exist for these languages in LLM evaluation.
- Performance degrades not from tokenizer inefficiency but from insufficient
  training examples. A language can be token-efficient but comprehensible to
  LLMs only if the model has seen enough examples to learn its idioms.

### 4.3 Programming Language Confusion (PLC)

From arXiv:2503.13620 (March 2025), studying 10 LLMs on 6 multilingual
datasets:

- LLMs frequently generate syntactically valid code in the wrong language
  when asked to use less common languages.
- Confusion follows systematic patterns: strong default to Python, and
  consistent confusion between syntactically similar pairs (C#/Java,
  JavaScript/TypeScript).
- **Model quantization significantly amplifies language confusion**, a
  relevant consideration for on-device deployment.
- Unintended language switches can introduce subtle, difficult-to-detect bugs
  that compromise security—a practical risk, not just an aesthetic problem.

### 4.4 Identifier Naming and LLM Reasoning: Quantified Tradeoffs

Multiple studies have measured the impact of identifier names on LLM code
comprehension and generation:

From Shin et al. (arXiv:2307.12488), studying code search and clone detection:

- Anonymizing all variable names simultaneously caused ~75% performance
  collapse on code search tasks (from ~70% MRR baseline to 17–24%).
- Anonymizing only function definition names caused ~9.5% performance loss.
- Python is more sensitive than Java because Python relies more heavily on
  naming for semantic context; Java compensates with type declarations.
- ChatGPT "cannot provide useful review of obfuscated code," confirming that
  transformer-based models treat code similarly to natural language text and
  extract meaning primarily from explicit identifier labels.

From a separate empirical study on AI code completion:

- Descriptive names use 41% more tokens but achieve 8.9% better semantic
  performance.
- Exact match rate: 34.2% for descriptive names vs 16.6% for obfuscated names.
- Performance ranking: descriptive > SCREAM_SNAKE_CASE > snake_case >
  PascalCase > minimal > obfuscated.

The exchange rate is unfavorable for terseness: a 41% token reduction in
identifiers produces roughly a 50% drop in task performance. Shortening
identifiers saves tokens at the cost of disproportionately large reasoning
degradation.

### 4.5 Terse Code and LLM Reasoning: The q/kdb+ Controversy

A Medium post (October 2025) by Gabi Teodoru argued that LLMs should not be
forced to write terse q/kdb+ code, using perplexity as a proxy for
generation quality. The core claim: terse code has lower redundancy, causing
LLMs to make more errors.

**This conclusion was retracted.** Community scrutiny revealed the original
experiments used incorrect methodology (missing chat templates for the
instruct model). When re-run with correct setup, **the terse version
outperformed the verbose version**. The information-theory framing was sound
but the empirical execution was flawed. The author committed to publishing
corrected findings.

An independent observation remains: "LLMs haven't had much training on Q
[as] there's little publicly available code"—the performance issue for niche
terse languages is a training data problem, not a perplexity/redundancy
problem.

### 4.6 Code Obfuscation Studies

Multiple studies have evaluated LLM performance on obfuscated or minified
code. The key distinction is between structural obfuscation (removing
formatting, compressing whitespace) and semantic obfuscation (renaming
identifiers, injecting dead code, transforming control flow):

- State-of-the-art models (GPT-4o, DeepSeek-V3, Claude) show negligible
  pass@1 degradation from structural obfuscation—near-zero impact from
  removing whitespace and formatting.
- Semantic obfuscation (identifier renaming, dead branch injection) degrades
  LLM fault localization and code review performance substantially.
- Researchers propose four dimensions of LLM code comprehension difficulty:
  reasoning depth, pattern recognition, noise filtering, and context
  integration. Structural changes affect the first; identifier changes affect
  all four.

### 4.7 LLM Confusion Mirrors Human Confusion

From arXiv:2508.18547v1 (2025): LLM perplexity spikes correlate with human
EEG signals of confusion (Spearman's ρ=0.47, p<0.001) at the same code
locations. This suggests LLMs and humans share similar comprehension
difficulties—complex operator precedence, misleading variable names, and
non-obvious control flow confuse both equally. This is largely orthogonal to
token count: confusing code isn't confusing because it uses more tokens, but
because its logic is hard to trace.

### 4.8 Verbose Code Does Not Automatically Aid Reasoning

GPT-5 reasoning models generate more than double the lines of code of GPT-4o
for the same Java tasks—and this verbosity improves functional correctness
but produces code that is harder to maintain. The trade-off is real: verbose
output aids the model's step-by-step reasoning process but does not
translate to more readable or maintainable output for humans.

### 4.9 The Iteration Tax

Raw token count per solution is not the right metric for LLM-assisted
development. The true cost includes tokens consumed during debugging
iterations when generated code is incorrect, tokens spent in error messages
and correction prompts, and downstream modification tokens when the type
system forces cascading changes.

Untyped languages (Python, Clojure) produce lower first-draft token counts
but may require more correction cycles. Typed languages (Rust, Haskell)
produce more verbose first drafts but the type checker catches errors before
they become multi-turn debugging sessions. One developer reported "almost all
code being decent the first try" in TypeScript with LLMs, while C# required
significant iteration.

J's low token count is similarly undermined: models produce incorrect J code
at higher rates due to training data scarcity, and the token savings can be
consumed by debugging iterations. The HackerNews thread on Alderson's study
flagged this directly.

---

## 5. Claude's Tokenizer

Anthropic does not publicly release Claude's tokenizer vocabulary.
Third-party reverse-engineering
([github.com/javirandor/anthropic-tokenizer](https://github.com/javirandor/anthropic-tokenizer))
has established:

- **Vocabulary size: ~65,000 tokens** (plus 5 special tokens), vs. 100K for
  `cl100k_base` and 200K for `o200k_base`.
- **45.2% overlap (70% of Claude's vocabulary)** with GPT-4's `cl100k_base`
  token set.
- Architecture: BPE, same class as tiktoken.
- Average: ~4 characters per token, ~1.5 tokens per word.
- Code tokenization is noted to be more efficient than natural language
  (shorter average token length per character of code).
- Python is explicitly called out as more token-efficient than Rust or C++
  under Claude's tokenizer—consistent with the GPT-4 findings.

The smaller vocabulary relative to `o200k_base` implies Claude's tokenizer
may split some code constructs into more tokens than GPT-4o does, though the
practical gap for English-language code is small given the high vocabulary
overlap.

---

## 6. Practical Synthesis

### For Language Selection in AI-Assisted Development

Balancing token efficiency, LLM training data coverage, and type system
feedback:

- Fewest tokens, mainstream only: Clojure > Haskell > F# > Python > Ruby
- Fewest tokens, any language: J (70 avg) — but nearly no LLM training data
- Best LLM reasoning support: Python >> JavaScript > TypeScript > Java > C#
- Static typing + token efficiency: Haskell, F# (type inference, no annotation cost)
- Worst combination: C (most tokens) + low LLM domain familiarity

### Key Design Principles for Token-Efficient Code

1. **Prefer type inference over explicit annotations.** Every redundant
   `: string` or `int x` is a token with no semantic value for the LLM.
2. **Use method chaining and functional composition.** `array.map(&:f)` over
   `[f(x) for x in array]`—both readable, but the former denser.
3. **Remove formatting when passing code to LLMs as context.** Java/C#/C++
   lose 25–35% of tokens by stripping whitespace and comments from input.
   Python cannot do this safely.
4. **Prefer short variable names in algorithmic code; use descriptive names
   in domain logic.** Short names save tokens but cost reasoning clarity.
   The measured exchange rate is unfavorable: a 41% token reduction from
   shortening identifiers yields roughly a 50% degradation in task
   performance.
5. **Avoid esoteric Unicode operators.** APL-style glyphs are
   multiply-tokenized in all mainstream LLM tokenizers, eliminating their
   character-count advantage. If using array programming idioms, J's ASCII
   subset is far more efficient.
6. **LLM output verbosity is hard to control.** Even if input code is
   stripped of formatting, LLMs maintain their own formatting conventions in
   outputs (output savings are only ~2.5%). This limits practical benefit of
   aggressive input compression strategies.

---

## Sources

1. **Martin Alderson - "Which programming languages are most token-efficient?"**
   (January 2026)
   [martinalderson.com](https://martinalderson.com/posts/which-programming-languages-are-most-token-efficient/)
   Primary study using Rosetta Code + GPT-4 tokenizer. Methodology
   acknowledged as non-scientific but data is the most comprehensive
   available.

2. **Hacker News thread on above** (46582728)
   [news.ycombinator.com](https://news.ycombinator.com/item?id=46582728)
   Expert commentary including counterarguments, the iteration tax, C's
   library-inflation problem, and training data observations.

3. **Lobsters thread on above**
   [lobste.rs](https://lobste.rs/s/nx9uwg/which_programming_languages_are_most)
   Additional technical commentary on tokenizer mechanics and
   counterarguments.

4. **Ivan Turkovic - "Why Ruby Might Be the Most AI-Friendly Language
   Nobody's Talking About"** (January 2026)
   [ivanturkovic.com](https://www.ivanturkovic.com/2026/01/17/ruby-token-efficiency-llm-ai-friendly-language/)
   Ruby-specific analysis corroborating the Rosetta Code findings.

5. **UBOS.tech - "Programming Languages Ranked by Token Efficiency for
   AI-Assisted Development"**
   [ubos.tech](https://ubos.tech/news/programming-languages-ranked-by-token-efficiency-for-ai%E2%80%91assisted-development/)
   Publishes the J=70, APL=110, Clojure=109, Haskell=115, F#=118,
   Python=~130 ranking table.

6. **Julia Programming Forum - "Julia is one of the most token-efficient
   programming languages"**
   [discourse.julialang.org](https://discourse.julialang.org/t/julia-is-one-of-the-most-token-efficient-programming-languages/134996/9)
   Community discussion; corroborates APL Unicode penalty and J ASCII
   advantage.

7. **arXiv:2508.13666 - "The Hidden Cost of Readability: How Code Formatting
   Silently Consumes Your LLM Budget"** (August 2025)
   [arxiv.org](https://arxiv.org/html/2508.13666v1)
   Rigorous 10-model study showing Java loses 34.9%, C++ 31.1%, C# 25.3%,
   Python only 6.5% of tokens to formatting. Primary source for formatting
   overhead figures.

8. **arXiv:2503.17181 - "LLMs Love Python: A Study of LLMs' Bias for
   Programming Languages and Libraries"** (2025)
   [arxiv.org](https://arxiv.org/html/2503.17181v1)
   90–97% Python bias finding, 83% self-contradiction in language
   recommendations.

9. **arXiv:2503.13620 - "Evaluating Programming Language Confusion"**
   (March 2025)
   [arxiv.org](https://arxiv.org/html/2503.13620v2)
   PLC study across 10 LLMs; systematic language confusion and quantization
   effects.

10. **arXiv:2410.03981 - "A Survey on LLM-based Code Generation for
    Low-Resource and Domain-Specific Programming Languages"** (2024)
    [arxiv.org](https://arxiv.org/abs/2410.03981)
    Pass@1 ≤30% for low-resource languages; training data scarcity as root
    cause for LRPL degradation.

11. **arXiv:2508.18547 - "How do Humans and LLMs Process Confusing Code?"**
    (2025)
    [arxiv.org](https://arxiv.org/html/2508.18547v1)
    LLM perplexity-human EEG confusion correlation (ρ=0.47); atoms of
    confusion affect LLMs and humans similarly.

12. **Gabi Teodoru / Hacker News - "Don't Force Your LLM to Write Terse
    Code"** (October 2025)
    [news.ycombinator.com](https://news.ycombinator.com/item?id=45567746)
    The terse-hurts-LLMs thesis and its subsequent retraction/correction.

13. **Matt Rickard - "A Token-Efficient Language for LLMs"**
    [mattrickard.com](https://mattrickard.com/a-token-efficient-language-for-llms)
    Data format token comparison (XML 201 → minified JSON 64 tokens).

14. **njkumar.com - "Multilingual token compression in GPT-o family models"**
    [njkumar.com](https://www.njkumar.com/gpt-o-multilingual-token-compression/)
    o200k_base regex pattern changes vs cl100k_base; Tamil 3.2x compression
    example.

15. **Christopher Samiullah - "The Technical User's Introduction to LLM
    Tokenization"**
    [christophergs.com](https://christophergs.com/blog/understanding-llm-tokenization)
    FizzBuzz GPT-2 (149 tokens) vs cl100k_base (77 tokens) concrete
    comparison.

16. **fast.ai - "Let's Build the GPT Tokenizer" (based on Karpathy's work)**
    [fast.ai](https://www.fast.ai/posts/2025-10-16-karpathy-tokenizers.html)
    cl100k_base regex pattern, first BPE merge = two spaces (token 256),
    training signal that code dominated the corpus.

17. **javirandor/anthropic-tokenizer (GitHub)**
    [github.com](https://github.com/javirandor/anthropic-tokenizer)
    Claude tokenizer reverse-engineering: ~65K vocabulary, 70% overlap with
    cl100k_base.

18. **metehan.ai - "Reverse-Engineering the OpenAI's GPT-5 Tokenizer"**
    [metehan.ai](https://metehan.ai/blog/reverse-engineering-the-gpt-5-tokenizer-aeo-geo/)
    Single-token programming keyword coverage in o200k_base; code token IDs
    in vocabulary (ID 257=4-space indent, 362=`);`, 393=`//`).

19. **adriangalilea/vibe-coding-lang-bench (GitHub)**
    [github.com](https://github.com/adriangalilea/vibe-coding-lang-bench)
    Real-project benchmarks: Python 4,322 tokens, Elixir 5,147, Rust 6,064
    using cl100k_base.

20. **Shin et al. - "How Does Naming Affect LLMs on Code Analysis Tasks?"**
    (arXiv:2307.12488)
    [arxiv.org](https://arxiv.org/html/2307.12488v5)
    ~75% performance collapse on full anonymization; method names matter more
    than variable names; Python more sensitive than Java.

21. **Yakubov - "Do Variable Names Matter for AI Code Completion?"** (2025)
    [yakubov.org](https://yakubov.org/blogs/2025-07-25-variable-naming-impact-on-ai-code-completion)
    Descriptive names use 41% more tokens but achieve 8.9% better semantic
    performance; 34.2% vs 16.6% exact match rate.

22. **llm-calculator.com - "GPT-4o vs GPT-4: Tokenization Differences"**
    [llm-calculator.com](https://llm-calculator.com/blog/gpt-4o-vs-gpt-4-tokenization/)
    Concrete measurement: JavaScript function costs 68 tokens (cl100k_base)
    vs 62 tokens (o200k_base), ~9% improvement.

23. **Chris McCormick - "Advantages of Generating Clojure with LLMs"**
    [mccormick.cx](https://mccormick.cx/news/entries/advantages-of-generating-clojure-with-llms)
    Clojure LISP syntax reduces superfluous tokens; niche training data
    caveat (models frequently generate non-existent functions).

---

## Confidence Assessment

- J=70, Clojure=109, C=283 token averages: Medium confidence. Single study;
  Rosetta Code has task selection bias. Consistent with corroborating sources.
- 2.6x gap between best/worst languages: Medium. Confirmed across multiple
  independent sources.
- cl100k_base groups 4 spaces as 1 token: High. Confirmed by multiple
  sources, mechanistically sound.
- FizzBuzz 149 tokens (GPT-2) vs 77 tokens (GPT-4): High. Direct
  measurement, widely cited.
- Java formatting overhead 34.9%, Python 6.5%: High. Peer-reviewed study,
  10 models tested.
- LLMs generate Python 90–97% of the time: High. Large-scale multi-benchmark
  study.
- APL Unicode glyphs are multi-token: High. Mechanistic explanation
  confirmed; community consensus.
- Terse code degrades LLM reasoning: Low. Only study retracted; insufficient
  data.
- Identifier names cause ~75% collapse when fully anonymized: High.
  Replicated across multiple tasks and languages.
- Claude vocabulary size ~65K: Medium. Third-party reverse-engineering only.
- Go and Perl token counts vs Python: Low. No direct benchmark found; inferred
  from language characteristics.

## Known Gaps

- No study has directly benchmarked LLM generation quality (Pass@1 or
  similar) specifically for J, BQN, or K against equivalent Python solutions.
- Rust's exact average token count from the Rosetta Code study was not
  published in sources accessed; it likely falls between JavaScript and Java.
- Perl, Kotlin, Scala, Nim, and K were not in the primary 19-language study
  and their exact placements were not surfaced in available sources.
- Claude-specific token counts per language have not been benchmarked; the
  ~65K vocabulary may produce meaningfully different splits from GPT-4 for
  some code constructs.
- No controlled study exists comparing LLM reasoning quality on semantically
  identical programs written in high-token vs. low-token languages while
  controlling for training data availability.
- The iteration tax (tokens spent correcting LLM errors) has not been
  systematically measured across languages in a way that accounts for token
  count per attempt alongside correction frequency.
