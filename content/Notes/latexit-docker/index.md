+++
date = 2026-05-19
draft = false
title = "Using LaTeXiT with a Docker Environment"
description = "A short note on configuring LaTeXiT on macOS to call LaTeX tools through Docker wrapper scripts."
tags = ["LaTeX", "Docker", "macOS", "LaTeXiT"]
categories = ["Notes"]
+++

I use LaTeXiT on macOS to typeset equations for slides and notes. It expects `pdflatex`, `ps2pdf`, and `gs` to be installed locally, but my LaTeX environment is in Docker.

To use that environment from LaTeXiT, I wrote three wrapper scripts and set them as LaTeXiT's command paths.

## Wrapper scripts

I created three scripts under my local `bin` directory:

```text
~/bin/pdflatex_docker.sh
~/bin/gs_docker.sh
~/bin/ps2pdf_docker.sh
```

Each script runs the corresponding command inside the same Docker image.
I use my existing `lualatex-template-tex-ubuntu:latest` image.

The `pdflatex` wrapper looks like this:

```bash
#!/bin/bash
CWD=$(pwd)

CACHE_DIR="/tmp/texmf-cache"
mkdir -p "$CACHE_DIR"

docker run --rm \
  --user $(id -u):$(id -g) \
  -e TEXMFVAR="$CACHE_DIR" \
  -v /tmp:/tmp \
  -v /var/folders:/var/folders \
  -v "$CWD":"$CWD" \
  -w "$CWD" \
  lualatex-template-tex-ubuntu:latest \
  pdflatex "$@"
```

The Ghostscript wrapper is almost the same:

```bash
#!/bin/bash
CWD=$(pwd)

docker run --rm \
  --user $(id -u):$(id -g) \
  -v /tmp:/tmp \
  -v /var/folders:/var/folders \
  -v "$CWD":"$CWD" \
  -w "$CWD" \
  lualatex-template-tex-ubuntu:latest \
  gs "$@"
```

And the `ps2pdf` wrapper is:

```bash
#!/bin/bash
CWD=$(pwd)

docker run --rm \
  --user $(id -u):$(id -g) \
  -v /tmp:/tmp \
  -v /var/folders:/var/folders \
  -v "$CWD":"$CWD" \
  -w "$CWD" \
  lualatex-template-tex-ubuntu:latest \
  ps2pdf "$@"
```

After creating these files, I made them executable and ensured that `~/bin` was included in my shell `PATH`.

```bash
chmod +x ~/bin/pdflatex_docker.sh
chmod +x ~/bin/gs_docker.sh
chmod +x ~/bin/ps2pdf_docker.sh
```

To add `~/bin` to the shell's `PATH`:

```bash
export PATH="$HOME/bin:$PATH"
```

Even so, I used absolute paths when configuring LaTeXiT itself, 
because GUI applications do not always inherit the same `PATH` as an interactive shell.

## Why these options matter

`--user $(id -u):$(id -g)` prevents Docker from creating root-owned output files.
Without this, generated files can become annoying to clean up or overwrite from macOS.

The current working directory is mounted into the container:

```bash
-v "$CWD":"$CWD"
-w "$CWD"
```

This lets the command inside Docker use the same working directory and relative paths as LaTeXiT.

I also mounted both `/tmp` and `/var/folders`:

```bash
-v /tmp:/tmp
-v /var/folders:/var/folders
```

On macOS, GUI applications often use temporary directories under `/var/folders`. If LaTeXiT asks `pdflatex` to compile a file in such a directory, the Docker container also needs to see it. 

`pdflatex_docker.sh` sets `TEXMFVAR` to a writable cache directory under `/tmp`. This avoids cache-related permission issues inside the container.

## LaTeXiT configuration

In LaTeXiT, I opened the preferences and replaced the command paths with the wrapper scripts:

```text
pdflatex  -> /Users/ikarishota/bin/pdflatex_docker.sh
ps2pdf    -> /Users/ikarishota/bin/ps2pdf_docker.sh
gs        -> /Users/ikarishota/bin/gs_docker.sh
```

{{< figure src="configuration.png" alt="LaTeXiT Typesetting preferences configured to use Docker wrapper scripts" caption="LaTeXiT configured to call Docker wrapper scripts for pdflatex, Ghostscript, and ps2pdf." >}}

I also enabled "Use current user's login shell" in LaTeXiT. The command paths are absolute, so locating the scripts does not depend on the shell's `PATH`.

## Result

I checked the setup by typesetting an equation. I could enter LaTeX and render it with the usual typesetting button.

{{< figure src="example.png" alt="LaTeXiT rendering an equation while using a Dockerized LaTeX environment" caption="A LaTeXiT equation rendered through the Docker-backed LaTeX toolchain." >}}

## A small trade-off

Each compilation starts a new Docker container, which adds some startup time. For the small equations I usually typeset, I am happy to accept that to keep using my existing LaTeX image.

If the wait becomes a problem, I may switch to a container that stays running. For now, I use `docker run --rm`.
