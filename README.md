# Plock

Use an LLM directly from literally anywhere you can type.

Write a prompt, select it, and hit `Cmd+Shift+.`. It will replace your prompt with the output in a streaming fashion.

Also! You can first put something on your clipboard (as in copy some text) before writing / selecting your prompt, and it `Cmd+Shift+/` and it will use the copied text as context to answer your prompt.

For Linux, use `Ctrl` instead of `Cmd`.

**100% Local** by default. (If you want to use an API or something, [you can call any shell script you want](https://github.com/jasonjmcghee/plock/blob/d82b9286fcad310e3045970a401b2a6e1399309d/src-tauri/src/generator.rs#L32) - just [set `USE_OLLAMA` to `false`](https://github.com/jasonjmcghee/plock/blob/d82b9286fcad310e3045970a401b2a6e1399309d/src-tauri/src/main.rs#L20))

_Note: Something not work properly? I won't know! Please log an issue or take a crack at fixing it yourself and submitting a PR! Have feature ideas? Log an issue!_

## Demo using Ollama
<a href="https://www.loom.com/share/fed267e695d145c88e6bff7e631da8e0">
  <img style="max-width:300px;" src="https://cdn.loom.com/sessions/thumbnails/fed267e695d145c88e6bff7e631da8e0-with-play.gif">
</a>

(in the video I mention [rem](https://github.com/jasonjmcghee/rem), another project I'm working on)

## Demo using GPT-3.5 and GPT-4
<a href="https://www.loom.com/share/756220f3f5e249d5b4d5b759e9f9add3">
  <img style="max-width:300px;" src="https://cdn.loom.com/sessions/thumbnails/756220f3f5e249d5b4d5b759e9f9add3-with-play.gif">
</a>

If you are going to use this with remote APIs, consider environment variables for your API keys... make sure they exist wherever you launch, or directly embed them (just don't push that code anywhere)

## Getting Started

Install [ollama](https://github.com/jmorganca/ollama) and make sure to run `ollama pull openhermes2.5-mistral` or swap it out in the code for something else.

Launch "plock"

Shortcuts:

`Ctrl / Cmd + Shift + .`: Replace the selected text with the output of the model.

`Ctrl / Cmd + Shift + /`: Feed whatever is on your clipboard as "context" and the replace the selected text with the output of the model.

`Escape`: Stop any streaming output

**Mac** will request access to keyboard accessibility.

**Linux** (untested), may require X11 libs for clipboard stuff and key simulation using enigo. [Helpful instructions](https://github.com/enigo-rs/enigo/tree/main#runtime-dependencies)

Also [system tray icons require some extras](https://tauri.app/v1/guides/features/system-tray/#linux-setup)

**Windows** (untested), you'll need to swap out Ollama for something else, as it doesn't support windows yet.

## Building Plock
If you don't have apple silicon or don't want to blindly trust binaries (you shouldn't), here's how you can build it yourself!

## Prerequisites

- Node.js (v14 or later)
- Rust (v1.41 or later)
- Bun (latest version)

## Installation Steps

### Node.js

Download from: https://nodejs.org/

### Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Bun

```bash
curl https://bun.sh/install | bash
```

## Project Setup

```bash
git clone <repo_url>
cd path/to/project
bun install
bun run tauri dev
```

## Build

```bash
bun run tauri build
```

## Another demo

Another demo where I use the perplexity shell script to generate an answer super fast.
Not affiliated, was just replying to a thread lol

https://github.com/jasonjmcghee/plock/assets/1522149/6166af73-545f-4a8e-ad46-ea8aacd84969

## Secrets

Curious folks might be wondering what `ocr` feature is. I took a crack at taking a screenshot,
running OCR, and using that for context, instead of copying text manually. Long story short,
rusty-tesseract _really_ dissapointed me, which is awkward b/c it's core to [xrem](https://github.com/jasonjmcghee/xrem).

If someone wants to figure this out... this could be really cool, especially with multi-modal models.
