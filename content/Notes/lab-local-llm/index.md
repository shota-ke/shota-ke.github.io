---
title: "Building a Local LLM Stack on DGX Spark"
date: 2026-05-25
tags: ["DGX Spark", "RAGFlow", "vLLM", "Ollama", "Slack", "LLM", "Docker"]
description: "Setting up vLLM, Ollama, and RAGFlow on DGX Spark for a Slack bot that summarizes arXiv papers and answers questions about them."
---

# Building a Local LLM Stack on DGX Spark

Our lab recently got an [NVIDIA DGX Spark](https://www.nvidia.com/en-us/products/workstations/dgx-spark/), and since nobody had set anything up on it yet, I ended up being the one to build out the local LLM stack.
The goal was a Slack bot that reads arXiv links, summarizes them with a locally-running LLM, and stores papers in RAGFlow so you can ask questions about them later.
I ran into a few problems with ARM64 images and Slack formatting along the way.

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

The DGX Spark is a small desktop machine with a GB10 Blackwell Superchip.
I started with NVIDIA's [playbook repository](https://github.com/NVIDIA/dgx-spark-playbooks), which has setup guides for vLLM, Ollama, Open WebUI, LLaMA Factory, and Flux fine-tuning.

DGX Spark runs `linux/arm64`. The NGC containers I used worked, but several third-party Docker Hub images were `amd64`-only. For those, I had to find alternatives or build from source.
RAGFlow's ARM images worked for this setup.

For remote access I followed the `nvidia/tailscale` playbook to get [Tailscale](https://tailscale.com/) running.
That gave me a stable address for remote access and let me forward ports to my laptop without changing the lab's firewall rules.

---

## RAGFlow

[RAGFlow](https://ragflow.io/) chunks and indexes documents such as PDFs, web pages, and plain text, then provides an API for retrieval-augmented Q&A.
I chose it for its ARM support and Python SDK. Its web UI also let me check which documents had been ingested when debugging.

I started RAGFlow with Docker Compose:

```bash
cd ragflow
docker compose -f docker/docker-compose.yml up -d
```

Some dependencies still needed Dockerfile changes for ARM64.
The UI comes up at `http://localhost:80`. I created an API key there and registered vLLM as the chat model under **Settings → Model Providers**.
I used `http://host.docker.internal:8000/v1` as the Base URL to reach vLLM on the host. Using `localhost` here would point to the RAGFlow container itself.

RAGFlow has two main concepts — **Datasets** (your document collections) and **Chat Assistants** (a dataset + LLM config bundled together).
The Slack bot creates both automatically on first run via the SDK, so you don't have to do any of that by hand.

---

## vLLM and Ollama

I tried both and ended up using them for different things.

I mostly use **Ollama** to serve an embedding model (e.g., `bge-m3`) for RAGFlow.

**vLLM** is noticeably faster for chat, especially when a few summarization requests come in around the same time.
It does continuous batching, so multiple requests don't just queue up behind each other.
I started it with the NGC container:

```bash
docker run --rm -it --gpus all --ipc=host -p 8000:8000 \
    nvcr.io/nvidia/vllm:25.12-py3 \
    vllm serve Qwen/Qwen3-Coder-Next-FP8 \
        --max-model-len 32768 \
        --gpu-memory-utilization 0.80
```

I keep the chat model in one vLLM server process and serve embeddings through Ollama. RAGFlow lets me configure those backends separately.

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

The bot runs in Socket Mode, so the lab server behind NAT does not need a public webhook URL.
Both event handlers (`app_mention` and `message`) immediately react with 👀 to acknowledge the request while the summary is being generated.

### The arXiv pipeline

Pasting an arXiv link kicks off a multi-step pipeline:

{{< mermaid >}}
flowchart LR
    URL([arXiv URL]) --> FETCH["Fetch metadata\n+ extract text"]
    FETCH --> INGEST["Upload to RAGFlow"]
    INGEST --> SUM["Summarize\nvLLM / RAGFlow Chat"]
    SUM --> POST(["Post in thread"])
{{< /mermaid >}}

Two details needed extra handling:

- **Fetch**: the arXiv Atom API can return 503 errors under load, so the bot retries with exponential backoff. When `pypdf` cannot extract text from a figure-heavy paper, it falls back to the abstract page on arxiv.org. I only extract the first few pages for the summary.
- **Summarize**: controlled by a `RAGFLOW_SUMMARIZE` flag. When true, the bot uses RAGFlow Chat so the summary is grounded in the retrieved chunks. When false (or when RAGFlow returns nothing useful), it goes straight to vLLM.

### Summarization and Slack formatting

Slack uses "mrkdwn", where `*text*` means bold and `## headings` render as literal text.
Even with mrkdwn instructions in the prompt, the model often returned regular Markdown.

I added a post-processing pass to convert the output:

- `## Heading` and `**bold**` → `*bold*`
- `- item` / `* item` → `• item`
- Fenced code blocks stripped
- 3+ blank lines collapsed to 2

Setting `temperature=0.2` also made the formatting more consistent in my tests.

### RAG Q&A with source attribution

When someone asks a question (via `/ask ...` or just `@hakushu ...`), the bot queries RAGFlow and attaches source references to the reply.
RAGFlow returns metadata alongside the answer — things like `document_name` and `page` — but these field names have shifted across SDK versions, so I added a normalization layer that checks a few possible keys before building labels like `"arxiv-2312.12345.pdf (p.3)"`.

If RAGFlow doesn't have anything relevant (I detect this by checking for phrases like "not found in the dataset"), the bot falls back to asking vLLM directly.
That reply uses vLLM directly rather than an answer from the paper collection.

---

## Using it in the lab

I use the bot to look up papers shared in Slack. Once a paper is in RAGFlow, I can ask what it said about quantization, for example, without searching through old threads.

ARM64 compatibility took the most work: some third-party dependencies needed alternative images or source builds.

If I were starting over, I'd set up an embedding model in Ollama from day one instead of defaulting to RAGFlow's built-in option.
In this setup, Ollama gave me better embedding throughput than RAGFlow's built-in option, and I could manage it separately from the chat model.
