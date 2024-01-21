# Plock

Because you pluck then plop. I know very creative.

## Getting Started

On Linux you need the x11 library (for clipboard), install it with something like:

```bash
sudo apt-get install xorg-dev
```

Mac will request access to keyboard accessibility.

Windows, you'll need to swap out Ollama for something else, as it doesn't support windows yet.

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
