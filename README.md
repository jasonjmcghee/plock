# Plock

Because you pluck then plop. I know very creative.

## Getting Started

Install [ollama](https://github.com/jmorganca/ollama) and make sure to run `ollama pull openhermes2.5-mistral` or swap it out in the code for something else.

`Ctrl / Cmd + Shift + .`: Replace the selected text with the output of the model.

`Ctrl / Cmd + Shift + /`: Feed whatever is on your clipboard as "context" and the replace the selected text with the output of the model.

Mac will request access to keyboard accessibility.

Linux (untested), may require X11 libs for clipboard stuff

Windows (untested), you'll need to swap out Ollama for something else, as it doesn't support windows yet.

## Developing Plock

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
