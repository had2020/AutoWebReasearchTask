## Project Post-Mortem: Visual Web-Agent Trapped in High-Entropy State Spaces

An empirical exploration into headless web-automation using small, quantized, vision-language models (`smolvlm-500m`). This project investigates why complex, multi-role prompt sequencing fails to overcome the fundamental data-loss and structural limitations inherent to processing web-states purely via rendered pixel matrices.

---

## ## Executive Summary

This architecture was designed to operate a headless instance of Chrome (`headless_chrome`) via an autonomous, multi-agent loop driven entirely by local LLM/VLM execution (`localhost:1234`). The control loop attempted to decompose web browsing into specialized behavioral primitives:

1. **Visual State Encoder (`OBSERVER_PROMPT`)** – Compresses base64 JPEG screenshots into an interactive state description.
2. **Planning Agent (`PLANPROMPT`)** – Evaluates current state against global objectives.
3. **Deterministic State Machine (`ACTIONPROMPT`)** – Emits precise execution string commands (`$RUN TAB!`, `$RUN TYPING>...<`).
4. **State Compression Engineer (`MEMORYPROMPT`)** – Maximize entropy per token to bypass context limits.

### ### Verdict

**Architectural Failure.** Despite high-fidelity prompt roles and specialized micro-contexts, the agent consistently stalled, diverged, or generated malformed state primitives. Relying on visual representations of text-heavy DOM frameworks introduces structural inefficiencies that defy algorithmic scaling at small model weights.

---

## ## System Architecture

```
                 [ Headless Chrome Instance ]
                             │
                     (Capture Screenshot)
                             ▼
                    ┌──────────────────┐
                    │ base64 JPEG + DOM│
                    └────────┬─────────┘
                             ▼
                    ┌──────────────────┐
                    │   VLM Observer   │ ──(Textual Scene Description)──┐
                    └──────────────────┘                                │
                                                                        ▼
┌──────────────────┐                                           ┌──────────────────┐
│ Memory Compress  │◄──(Update State)─── [ Execution Loop ] ──►│  Planning Agent  │
└──────────────────┘                                           └────────┬─────────┘
                                                                        ▼
                                                               ┌──────────────────┐
                                                               │  Action Dispatch │
                                                               └──────────────────┘

```

The underlying loop runs synchronously in Rust, performing multi-stage pipeline orchestration:

```rust
// Core pipeline loop snippet
loop {
    let jpeg_data = tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true).unwrap_or_default();
    let base64_encoded = general_purpose::STANDARD.encode(&jpeg_data);

    // Decode Visual State
    let response = send_image_to_llm(&client, &format!("{} ### METADATA...", OBSERVER_PROMPT), &base64_encoded, 500).unwrap();

    // Plan Next Action
    let plan = llm_response(&client, &format!("{}{}{}{}", PLANPROMPT, scout_task, long_term_context, response), 500).unwrap();

    // Parse String Command to Hardware Primitives
    let action = llm_response(&client, &plan, 50).unwrap();
    
    // Execute state transition across the CDP (Chrome DevTools Protocol)
    // ... Match execution blocks
}

```

---

## ## Microarchitectural & Theoretical Failure Analysis

The project’s failure is not a flaw in the Rust implementation, but an unavoidable result of processing highly dense text-and-layout domains using low-capacity parameter matrices via visual modalities.

### 1. Spatial Resolution and Information Laundering

Websites are natively represented as structured, symbolic text files (HTML DOM tree). Converting this highly structured data into a 2D matrix of RGB pixels, and then asking a 500M parameter VLM to reconstruct the text strings (e.g., parsing a tiny GitHub URL link out of a 1080p rendered block) represents massive information laundering.

* **Spatial Subsampling:** At standard resolutions, crucial interactable data spans tiny sub-grids of pixels. Small vision transformers patchify image inputs ($14 \times 14$ or $32 \times 32$ pixel patches). Small text elements are blurred across patch boundaries, corrupting the feature vectors.
* **Token Overhead:** Describing a screen visually takes up hundreds of vision tokens, leaving little room for dense reasoning or causal step history.

### 2. Context Cascade Corruption (Cascade of Failure)

The system depends on a sequential string pipeline where the output of phase $N$ serves as the input to phase $N+1$:

$$\text{Visual Input} \longrightarrow \text{Observation} \longrightarrow \text{Plan} \longrightarrow \text{Command} \longrightarrow \text{Execution}$$

In a 500M parameter local deployment model, the error rate per step is non-trivial ($e > 0.15$). Because the actions are strongly coupled, the errors multiply exponentially:

$$P(\text{Success}) = (1 - e_{\text{obs}}) \times (1 - e_{\text{plan}}) \times (1 - e_{\text{cmd}})$$

By the third loop iteration, noise completely dominates the prompt context, causing the agent to hallucinate actions or emit illegal, unparseable command schemas.

### 3. Lack of Token-Level Lookahead & Latency Overhead

Using web-automation requires precise execution matching. Small models lack the deep semantic pathways needed to safely follow strict format constraints without fine-tuning.

Furthermore, the pipeline introduces severe latency. For every simulated step, the host machine handles a full browser layout render, a JPEG compression pass, a base64 string copy, and four distinct LLM inference steps over HTTP. The CPU and GPU cycles consumed are orders of magnitude greater than the cost of native DOM parsing and linear tree searching.

---

## ## How to Run (For Academic Evaluation)

If you wish to test the limitations of local VLM agent loops on your hardware cluster, ensure your local inference engine is listening on port `1234`.

### Prerequisites

* Google Chrome / Chromium installed on the host.
* Local model provider (e.g., LM Studio, Ollama, or custom vLLM instance serving `smolvlm` compatible JSON variants).

### Execution

```bash
cargo run --release

```

---

## Alternative Solutions: Specialized Architectures over Generative LLMs

To eliminate the latency and reliability overhead of autoregressive LLMs, web orchestration must pivot to paradigms designed specifically for structural state spaces.

# Solution A: Specialized Web-Searching Neural Networks

Rather than using broad-knowledge foundational models, web navigation should be handled by specialized, non-generative neural networks (such as Graph Neural Networks (GNNs) or dedicated Reinforcement Learning policy networks) trained specifically on DOM structures:

Direct State Encoding: The input layer processes the DOM tree directly as a graph, where HTML elements are nodes and relationships are edges. This keeps information dense and completely removes the need for expensive visual rendering or image patch tracking.

Action Classification: Instead of generating text strings and hoping they match a schema, the network acts as a strict classifier. The output layer maps directly to a discrete probability distribution over valid browser commands.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
