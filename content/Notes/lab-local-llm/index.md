---
title: "Building a Local LLM Stack on DGX Spark"
date: 2026-05-25
tags: ["DGX Spark", "RAGFlow", "vLLM", "Ollama", "Slack", "LLM", "Docker"]
description: "A casual note on setting up a local LLM environment — from DGX Spark to a RAG-enabled Slack bot that summarizes arXiv papers on the fly."
---

# Building a Local LLM Stack on DGX Spark

Our lab recently got an [NVIDIA DGX Spark](https://www.nvidia.com/en-us/products/workstations/dgx-spark/), and since nobody had set anything up on it yet, I ended up being the one to build out the local LLM stack.
The goal was a Slack bot that reads arXiv links, summarizes them with a locally-running LLM, and stores papers in RAGFlow so you can ask questions about them later.
Here are my notes on how I got there and what tripped me up along the way.

The overall stack looks like this:

{{< mermaid >}}
flowchart LR
    subgraph HW["DGX Spark (GB10 Blackwell)"]
        GPU["GPU Memory"]
    end

    subgraph Inference["Inference Layer"]
        vLLM["vLLM\n(chat model)\n:8000"]
        Ollama["Ollama\n(embeddings)\n:11434"]
    end

    subgraph RAG["RAGFlow"]
        KB["Knowledge Base\n(Dataset)"]
        Chat["Chat Assistant"]
    end

    subgraph Frontend["Front-end"]
        Bot["Slack Bot\n(Socket Mode)"]
    end

    GPU --> vLLM
    GPU --> Ollama
    vLLM -->|"OpenAI API"| Chat
    Ollama -->|"Embeddings"| KB
    KB --> Chat
    Chat -->|"RAG answer"| Bot
    vLLM -->|"Direct summarize"| Bot
    Bot -->|"Ingest PDF / text"| KB
{{< /mermaid >}}

---

## DGX Spark

The DGX Spark is a small desktop box running the GB10 Blackwell Superchip — basically a personal supercomputer that somehow fits under a monitor.
NVIDIA has a [playbook repository](https://github.com/NVIDIA/dgx-spark-playbooks) with step-by-step guides for things like vLLM, Ollama, Open WebUI, LLaMA Factory, and Flux fine-tuning, so I used that as my starting point instead of guessing from scratch.

The one gotcha is that DGX Spark runs `linux/arm64`.
NVIDIA's own NGC containers are multi-arch so those are fine, but third-party Docker Hub images are a coin flip — a handful I tried were `amd64`-only and I had to hunt for alternatives or build from source.
RAGFlow publishes ARM images, which saved me a lot of pain.

For remote access I followed the `nvidia/tailscale` playbook to get [Tailscale](https://tailscale.com/) running.
After that the machine has a stable address I can reach from anywhere, and port-forwarding to my laptop just works without touching firewall rules.

---

## RAGFlow

[RAGFlow](https://ragflow.io/) is the RAG part of the stack. You throw documents at it — PDFs, web pages, plain text — and it chunks and indexes them and gives you an API for retrieval-augmented Q&A.
I went with it over other options mainly because it has solid ARM support, a Python SDK, and a web UI where you can actually see what got ingested, which is really handy for debugging.

Getting it running is literally just a `docker compose up`:

```bash
cd ragflow
docker compose -f docker/docker-compose.yml up -d
```

As mentioned earlier, DGX Spark is ARM64, so I had to patch a few Dockerfiles to get everything running — RAGFlow's images were fine, but some dependencies weren't.
The UI comes up at `http://localhost:80`. I created an API key there and registered vLLM as the chat model under **Settings → Model Providers**.
One thing that got me: the Base URL has to be `http://host.docker.internal:8000/v1`, not `localhost`, because RAGFlow is inside Docker and can't reach the host's loopback directly.

RAGFlow has two main concepts — **Datasets** (your document collections) and **Chat Assistants** (a dataset + LLM config bundled together).
The Slack bot creates both automatically on first run via the SDK, so you don't have to do any of that by hand.

---

## vLLM and Ollama

I tried both and ended up using them for different things.

**Ollama** is dead simple — just pull a model and run it, no config needed.
I mostly use it for embeddings (e.g., `bge-m3`) since RAGFlow needs an embedding endpoint and Ollama handles that without any fuss.

**vLLM** is noticeably faster for chat, especially when a few summarization requests come in around the same time.
It does continuous batching, so multiple requests don't just queue up behind each other.
On DGX Spark the easiest way to start it is with the NGC container:

```bash
docker run --rm -it --gpus all --ipc=host -p 8000:8000 \
    nvcr.io/nvidia/vllm:25.12-py3 \
    vllm serve Qwen/Qwen3-Coder-Next-FP8 \
        --max-model-len 32768 \
        --gpu-memory-utilization 0.80
```

The catch with vLLM is that one server process = one model, full stop.
So I run vLLM for chat and Ollama for embeddings side by side.
RAGFlow is happy with that — you can point different model types at different backends.

---

## Slack Bot

Once the inference stack was working, I put together a Slack bot to connect all the pieces.
The basic idea: paste an arXiv link in a channel, get a summary in the thread, and have the paper automatically land in RAGFlow so you can ask questions about it later.

{{< mermaid >}}
flowchart TD
    E(["Message or @mention"]) --> U{"Contains URL?"}

    U -->|"arXiv URL"| AX["Fetch metadata + PDF\narXiv Atom API"]
    U -->|"General URL"| WEB["Scrape page text\nBeautifulSoup"]
    U -->|"No URL\n(mention or DM)"| RAG1["ask_ragflow_with_sources"]

    AX --> INGEST["Upload to RAGFlow\nasync_parse_documents"]
    WEB --> INGEST
    INGEST --> SUM["Summarize\nvLLM /v1/chat/completions"]
    SUM --> POST(["Post in thread"])

    RAG1 --> FOUND{"RAGFlow answered?"}
    FOUND -->|Yes| SRC["Append source labels"]
    FOUND -->|"No / not found"| DIRECT["chat_with_llm\nvLLM direct"]
    SRC --> POST
    DIRECT --> POST
{{< /mermaid >}}

The bot runs in Socket Mode, which means no public URL or webhook needed — pretty convenient for a lab server behind a NAT.
Both event handlers (`app_mention` and `message`) immediately react with 👀 so people know something is actually happening.

### The arXiv pipeline

Pasting an arXiv link kicks off a multi-step pipeline:

{{< mermaid >}}
flowchart LR
    URL([arXiv URL]) --> FETCH["Fetch metadata\n+ extract text"]
    FETCH --> INGEST["Upload to RAGFlow"]
    INGEST --> SUM["Summarize\nvLLM / RAGFlow Chat"]
    SUM --> POST(["Post in thread"])
{{< /mermaid >}}

A few things worth noting about each step:

- **Fetch**: the arXiv Atom API 503s under load, so there's exponential backoff built in. `pypdf` extraction also fails silently on figure-heavy papers — in that case it falls back to scraping the abstract page on arxiv.org. I only pull the first few pages anyway; the useful stuff (motivation, method) is always up front.
- **Summarize**: controlled by a `RAGFLOW_SUMMARIZE` flag. When true, the bot uses RAGFlow Chat so the summary is grounded in the retrieved chunks. When false (or when RAGFlow returns nothing useful), it goes straight to vLLM.

### Summarization and Slack formatting

This part was a bit annoying: Slack has its own markdown dialect called "mrkdwn" where `*text*` means bold (not italic), and `## headings` just render as literal text.
LLMs don't really know about mrkdwn, so even with explicit instructions in the prompt, the model would still spit out regular Markdown half the time.

I added a post-processing pass to convert the output:

- `## Heading` and `**bold**` → `*bold*`
- `- item` / `* item` → `• item`
- Fenced code blocks stripped
- 3+ blank lines collapsed to 2

Setting `temperature=0.2` also helped a lot — the output got way more consistent and the model stopped inventing its own formatting quirks.

### RAG Q&A with source attribution

When someone asks a question (via `/ask ...` or just `@hakushu ...`), the bot queries RAGFlow and attaches source references to the reply.
RAGFlow returns metadata alongside the answer — things like `document_name` and `page` — but these field names have shifted across SDK versions, so I added a normalization layer that checks a few possible keys before building labels like `"arxiv-2312.12345.pdf (p.3)"`.

If RAGFlow doesn't have anything relevant (I detect this by checking for phrases like "not found in the dataset"), the bot falls back to asking vLLM directly.
Not perfect, but at least you always get some kind of answer.

---

## Wrapping up

Overall the stack has been pretty useful as a daily research tool.
Papers pile up in RAGFlow over time, and it's kind of surprisingly handy to be able to ask things like "what did that paper from last week say about quantization" without having to dig through your memory or your Slack history.

The biggest headache was ARM64 compatibility — not all Docker images are multi-arch, and discovering that at runtime is really annoying.
RAGFlow and the NGC containers were both fine; it was only some third-party stuff where I hit dead ends.

If I were starting over, I'd set up an embedding model in Ollama from day one instead of defaulting to RAGFlow's built-in option.
The throughput is better and it keeps things cleaner to have the embedding and chat inference paths on separate backends.

It was a good excuse to actually get hands-on with the LLM ecosystem rather than just reading about it.
Things like RAGFlow and vLLM move fast, so I'll probably keep tinkering as new tools show up.