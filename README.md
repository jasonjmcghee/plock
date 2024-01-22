# Plock

100% Local.

(Because you pluck then plop. I know very creative.)

## What is this?

Ever wanted to use an LLM directly in an app without using a chat ui or terminal? That's what this does.

Write a prompt, select it, and hit `Cmd+Shift+.`. It will replace your prompt with the output in a streaming fashion.

Also! You can first put something on your clipboard (as in copy some text) before writing / selecting your prompt, and it `Cmd+Shift+/` and it will use the copied text as context to answer your prompt.

For Linux, use `Ctrl` instead of `Cmd`.

Note: Something not work properly? I won't know! Please log an issue or take a crack at fixing it yourself and submitting a PR! Have feature ideas? Log an issue!

## Demo
<a href="https://www.loom.com/share/fed267e695d145c88e6bff7e631da8e0">
  <img style="max-width:300px;" src="https://cdn.loom.com/sessions/thumbnails/fed267e695d145c88e6bff7e631da8e0-with-play.gif">
</a>

(in the video I mention [rem](https://github.com/jasonjmcghee/rem), another project I'm working on)

## Getting Started

Install [ollama](https://github.com/jmorganca/ollama) and make sure to run `ollama pull openhermes2.5-mistral` or swap it out in the code for something else.

Launch "plock"

Shortcuts:

`Ctrl / Cmd + Shift + .`: Replace the selected text with the output of the model.

`Ctrl / Cmd + Shift + /`: Feed whatever is on your clipboard as "context" and the replace the selected text with the output of the model.

Mac will request access to keyboard accessibility.

Linux (untested), may require X11 libs for clipboard stuff and key simulation using enigo. [Helpful instructions](https://github.com/enigo-rs/enigo/tree/main#runtime-dependencies)

Windows (untested), you'll need to swap out Ollama for something else, as it doesn't support windows yet.

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
