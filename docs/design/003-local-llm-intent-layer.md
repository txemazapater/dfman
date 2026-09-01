# Design 003 — Optional local LLM intent layer

Status: Proposed  
Date: 2026-09-01

## Intent

dfman should support natural-language interaction without surrendering determinism, auditability or control of filesystem operations.

The local LLM is therefore **not** a filesystem agent and **not** an operation executor. It is an optional intent interpreter placed in front of dfman's own formal semantics.

The architectural rule is:

> Natural language may generate dfman intent, but only validated dfman intent can generate an OperationPlan.

And, more strictly:

> The LLM may interpret language; dfman alone defines operational semantics.

## Architecture

```text
User input
   |
   +--> native dfman DSL ------------------+
   |                                       |
   +--> deterministic intent parser -------+--> Intent AST
   |                                       |
   +--> optional local LLM ----------------+
                                               |
                                               v
                                         DSL compiler
                                               |
                                               v
                                           Basket
                                               |
                                               v
                                        OperationPlan
                                               |
                                               v
                                          Validator
                                               |
                                               v
                                           Executor
                                               |
                                               v
                                           Journal
```

The LLM must have no direct API for copy, move, delete, rename, shell execution or arbitrary filesystem access.

Its contract should be deliberately narrow:

```text
IntentResolver.resolve(text, context) -> IntentResult
```

A possible `IntentResult` contains:

```text
Action
SourceSet
Predicates[]
Destination
Modifiers[]
Confidence
UnresolvedTerms[]
InterpretationNotes[]
```

The result is validated against dfman's schema before it can be compiled into the formal command representation.

## Why local

A local model is attractive for dfman because:

- filesystem paths and filenames may be private or commercially sensitive;
- the required task is narrow enough that a small model may be sufficient;
- latency should remain low for interactive use;
- dfman should continue working offline;
- the intent layer should not become dependent on a cloud account or API;
- local inference makes the feature optional and replaceable.

## Hybrid interpretation strategy

The LLM should not be invoked for every command.

Recommended resolution order:

```text
1. native DSL parser
2. deterministic natural-language patterns / aliases
3. local LLM when free language or ambiguity remains
4. unresolved intent if no safe interpretation is possible
```

Examples that should not require an LLM:

```text
move *.jpg right
select pdf larger than 20MB
undo OP-1042
```

Examples where an LLM may help:

```text
move all the photos from this folder to my photos folder
remove tiny screenshots from the basket
undo the failed files from the big move I did earlier
```

## Context supplied to the LLM

The model should receive structured context rather than uncontrolled filesystem visibility.

Useful context includes:

```text
current path
left path
right path
known bookmarks
Basket summary
available dfman actions
available predicates
formal grammar / JSON schema
recent operation identifiers and summaries
configured semantic aliases
```

The model should normally not receive complete file contents.

Directory entry names may be supplied only when needed by the requested interpretation and subject to configurable privacy limits.

## Output format

The preferred interface is structured output rather than generated executable text.

Example user request:

```text
Move all files in this folder to the photos folder
```

Desired model result:

```json
{
  "action": "move",
  "source": "current",
  "predicates": [
    {"field": "type", "operator": "eq", "value": "file"}
  ],
  "destination": {"kind": "bookmark", "value": "photos"},
  "confidence": 0.99,
  "unresolved": []
}
```

The dfman compiler may then render an equivalent formal command for explanation:

```text
MOVE FROM current
TO bookmark('photos')
WHERE type = file
```

The model never decides what `MOVE`, `DELETE`, `UNDO` or any other operation means internally.

## Ambiguity handling

The model must be allowed to return unresolved concepts.

Example:

```text
move the good photos to archive
```

Possible interpretation:

```text
action: MOVE
kind: photo
destination: bookmark('archive')
unresolved: ['good']
```

This is preferable to silently inventing a definition of "good".

A configurable semantic alias may later define terms such as:

```text
large = size > 100MB
old = modified < now - 1y
photo = extension in (...)
```

The LLM can map natural phrases to these known concepts but should not create permanent definitions implicitly.

## Safety boundary

The local model must never receive general-purpose tools such as:

```text
run_shell
open_file
write_file
delete_file
move_file
```

The strongest permitted capability is to emit a schema-valid intent object.

The following path is forbidden:

```text
LLM -> filesystem
```

The required path is:

```text
LLM -> Intent AST -> dfman compiler -> OperationPlan -> Validator -> Executor
```

## Explainability

Natural-language interpretation should be visible when useful.

Example:

```text
> move the large photos to archive

Interpreted as:
  kind = photo
  large = size > 100 MB
  destination = D:\Archive

347 files / 18.4 GB

MOVE TO bookmark('archive')
WHERE kind = photo
AND size > 100MB
```

`EXPLAIN` should work identically regardless of whether an intent originated from the DSL, deterministic natural-language parsing or an LLM.

## Runtime abstraction

The intent layer should not depend on a specific inference engine.

A local provider interface may expose an OpenAI-compatible endpoint or an internal adapter.

Potential runtimes include:

- llama.cpp;
- Ollama;
- LM Studio;
- vLLM for larger/local-server deployments;
- another compatible local runtime.

No runtime is selected by this document.

## Candidate models

The initial evaluation should focus on small instruct models capable of reliable structured output or function/tool calling.

### 1. Qwen3-1.7B

Role: **minimum-footprint baseline**.

Reasons to test:

- very small compared with conventional chat models;
- Qwen3 documentation explicitly discusses agentic/tool-calling use;
- suitable for determining how little model capacity dfman's narrow intent task actually needs;
- Apache-2.0 model license is attractive for experimentation and redistribution analysis.

Risk:

- complex references, elliptical language and ambiguous Spanish commands may exceed its reliable semantic capacity.

Recommendation: **first performance baseline**, not assumed production winner.

### 2. Microsoft Phi-4-mini-instruct

Role: **primary structured-intent candidate**.

Reasons to test:

- roughly 4B-class small model;
- official model documentation includes a tool-enabled function-calling format;
- well matched to the constrained `IntentResult` / function-call style required by dfman;
- MIT-licensed model distribution simplifies experimentation.

Recommendation: **high-priority candidate**.

### 3. Qwen3-4B

Role: **primary semantic-quality candidate**.

Reasons to test:

- still small enough for realistic local deployment;
- official Qwen documentation highlights tool-calling capability;
- likely to provide more robust multilingual and elliptical-language interpretation than the 1.7B variant;
- good candidate for Spanish natural-language commands.

Recommendation: **high-priority candidate**, especially if 1.7B is insufficient.

### 4. Gemma 3 4B IT

Role: **quality/efficiency comparison candidate**.

Reasons to test:

- designed for local deployment in desktop-class environments;
- 4B size is within the intended dfman envelope;
- strong general instruction following.

Caveat:

- dfman specifically values constrained structured intent and tool-like outputs, so it should be benchmarked against Phi/Qwen rather than selected from general quality alone.

### 5. Llama 3.2 3B Instruct

Role: **ecosystem baseline**.

Reasons to test:

- mature local-runtime support including llama.cpp / GGUF ecosystem;
- modest resource requirements;
- useful baseline for portability and inference speed.

Caveat:

- not currently the strongest architectural fit compared with models explicitly emphasizing tool calling / structured interfaces.

### 6. Hermes 3 Llama 3.2 3B

Role: **structured-output specialist comparison**.

Reasons to test:

- explicit function-calling and JSON structured-output prompting documented by the model publisher;
- interesting if base Llama 3.2 proves lightweight but insufficiently constrained.

Caveat:

- derivative/community model rather than the preferred initial vendor-maintained baseline.

## Initial shortlist

Recommended first benchmark matrix:

```text
Qwen3-1.7B             minimum-resource baseline
Phi-4-mini-instruct    structured-intent candidate
Qwen3-4B               semantic-quality candidate
```

Only add Gemma 3 4B, Llama 3.2 3B and Hermes 3 3B if they provide useful comparison or deployment advantages.

## Benchmark tasks

Model selection should be based on dfman-specific tests, not generic LLM benchmarks.

Build a corpus containing at least:

```text
simple action + destination
compound predicates
relative destinations (left/right/parent)
bookmarks by natural name
Basket references
history references
undo / retry requests
Spanish and English commands
elliptical follow-up commands
ambiguous terms
negative constraints
commands that must be rejected as unresolved
```

Examples:

```text
Move all JPG files larger than 20 MB to the right panel.
Mueve los PDF antiguos a Archivo.
Quita de la cesta las capturas pequeñas.
Deshaz sólo los que fallaron en la operación anterior.
Mueve esas mismas fotos a la carpeta padre.
Borra los duplicados excepto el más reciente.
```

Metrics should include:

```text
valid-schema rate
exact action accuracy
predicate accuracy
destination resolution accuracy
unresolved/ambiguity detection accuracy
false-confidence rate
latency
RAM/VRAM footprint
```

The most important failure metric is **unsafe confident misinterpretation**, not ordinary language-model perplexity.

## Model selection principle

> Choose the smallest local model that reliably produces correct, conservative and schema-valid intent for dfman's command corpus.

A larger model should only be adopted if it materially reduces semantic errors or ambiguity failures.

## Future possibility: task-specific fine-tuning

If a small general model performs reasonably but inconsistently, dfman's formal DSL provides an unusually convenient supervised target.

Pairs can be generated in the form:

```text
natural-language request -> IntentResult
```

This makes LoRA / QLoRA fine-tuning of a small local model a plausible later optimization.

Fine-tuning is explicitly out of scope for the first implementation.

## Open questions

1. Should the first LLM integration use Ollama, llama.cpp or a generic OpenAI-compatible provider interface?
2. How much directory context is safe and necessary to send to the model?
3. Should confidence thresholds have any semantic meaning, or should validation rely only on structural ambiguity?
4. Should potentially destructive natural-language commands always display the generated formal command before execution?
5. How much conversational state should be supported (`those`, `the previous ones`, `same destination`)?
6. Can Qwen3-1.7B reliably handle Spanish command interpretation, or is the practical floor closer to 4B?
7. Should the model emit the Intent AST directly or select from function-like dfman operations?
