# Plock

Use an LLM (or anything else that can stream to stdout) directly from literally anywhere you can type. Outputs in real
time.

![demo](https://github.com/jasonjmcghee/plock/assets/1522149/737cb647-69aa-426c-884d-bbe29bac0637)

Write a prompt, select it, and (by default) hit `Cmd+Shift+.`. It will replace your prompt with the output in a
streaming fashion.

Also! You can first put something on your clipboard (as in copy some text) before writing / selecting your prompt, and
it (by default) `Cmd+Shift+/` and it will use the copied text as context to answer your prompt.

For Linux, use `Ctrl` instead of `Cmd`.

**100% Local** by default. (If you want to use an API or something, you can call any shell script you want specified
in `settings.json`)

I show an example `settings.json` in [Settings](#settings)

_Note: Something not work properly? I won't know! Please log an issue or take a crack at fixing it yourself and
submitting a PR! Have feature ideas? Log an issue!_

## Demo using Ollama

<a href="https://www.loom.com/share/fed267e695d145c88e6bff7e631da8e0">
  <img style="max-width:300px;" src="https://cdn.loom.com/sessions/thumbnails/fed267e695d145c88e6bff7e631da8e0-with-play.gif">
</a>

(in the video I mention [rem](https://github.com/jasonjmcghee/rem), another project I'm working on)

## Demo using GPT-3.5 and GPT-4

<a href="https://www.loom.com/share/756220f3f5e249d5b4d5b759e9f9add3">
  <img style="max-width:300px;" src="https://cdn.loom.com/sessions/thumbnails/756220f3f5e249d5b4d5b759e9f9add3-with-play.gif">
</a>

If you are going to use this with remote APIs, consider environment variables for your API keys... make sure they exist
wherever you launch, or directly embed them (just don't push that code anywhere)

## Getting Started

Install [ollama](https://github.com/jmorganca/ollama) and make sure to run `ollama pull openhermes2.5-mistral` or swap
it out in settings for something else.

Launch "plock"

Shortcuts:

`Ctrl / Cmd + Shift + .`: Replace the selected text with the output of the model.

`Ctrl / Cmd + Shift + /`: Feed whatever is on your clipboard as "context" and the replace the selected text with the
output of the model.

(these two are customizable in `settings.json`)

`Escape`: Stop any streaming output

**Mac** will request access to keyboard accessibility.

**Linux** (untested), may require X11 libs for clipboard stuff and key simulation using
enigo. [Helpful instructions](https://github.com/enigo-rs/enigo/tree/main#runtime-dependencies)

Also [system tray icons require some extras](https://tauri.app/v1/guides/features/system-tray/#linux-setup)

**Windows** (untested), you'll need to swap out Ollama for something else, as it doesn't support windows yet.

## [Settings]

There is a `settings.json` file which you can edit to change shortcuts, the model,
prompts, whether to use shell scripts and what they are, and other settings.

After updating, click the tray icon and select "Load Settings" or restart it.

On mac, It's at `~/Library/Application Support/today.jason.plock/settings.json`.

On linux, I think it's `~/$XDG_DATA_HOME/today.jason.plock/settings.json`.

Windows, I think it's `~\AppData\Local\today.jason.plock\settings.json`

Correct me if any of these are wrong.

<details>
  <summary>Show Example</summary>

```json
{
  "environment": {
    "PERPLEXITY_API": "",
    "OLLAMA_MODEL": "openhermes2.5-mistral",
    "OPENAI_API": ""
  },
  "processes": [
    {
      "command": [
        "bash",
        "/Users/jason/workspace/plock/scripts/gpt.sh"
      ]
    },
    {
      "command": []
    },
    {
      "command": [
        "bash",
        "/Users/jason/workspace/plock/scripts/p.sh"
      ]
    },
    {
      "command": [
        "bash",
        "/Users/jason/workspace/plock/scripts/dalle.sh"
      ]
    },
    "ollama"
  ],
  "prompts": [
    {
      "name": "default basic",
      "prompt": "$SELECTION"
    },
    {
      "name": "default with context",
      "prompt": "I will ask you to do something. Below is some extra context to help do what I ask. --------- $CLIPBOARD --------- Given the above context, please, $SELECTION. DO NOT OUTPUT ANYTHING ELSE."
    },
    {
      "name": "step",
      "prompt": "$STEP"
    },
    {
      "name": "say gpt",
      "prompt": "say \"$GPT\""
    }
  ],
  "triggers": [
    {
      "trigger_with_shortcut": "Command+Shift+,",
      "process": 1,
      "prompt": 0,
      "next_steps": [
        {
          "store_as_env_var": "STEP"
        },
        {
          "trigger": 4
        }
      ],
      "selection_action": null
    },
    {
      "trigger_with_shortcut": "Command+Shift+.",
      "process": 0,
      "prompt": 0,
      "next_steps": [
        "stream_text_to_screen"
      ],
      "selection_action": "newline"
    },
    {
      "trigger_with_shortcut": "Command+Shift+/",
      "process": 1,
      "prompt": 0,
      "next_steps": [
        "write_final_text_to_screen"
      ],
      "selection_action": "newline"
    },
    {
      "trigger_with_shortcut": "Command+Shift+'",
      "process": 3,
      "prompt": 0,
      "next_steps": [
        "write_image_to_screen"
      ],
      "selection_action": null
    },
    {
      "trigger_with_shortcut": null,
      "process": 0,
      "prompt": 2,
      "next_steps": [
        "stream_text_to_screen",
        {
          "store_as_env_var": "GPT"
        },
        {
          "trigger": 5
        }
      ],
      "selection_action": null
    },
    {
      "trigger_with_shortcut": null,
      "process": 0,
      "prompt": 3,
      "next_steps": [],
      "selection_action": null
    }
  ]
}

```

</details>

## Building Plock

If you don't have apple silicon or don't want to blindly trust binaries (you shouldn't), here's how you can build it
yourself!

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

### ~~Bun~~ NPM

_Whattt?? Why?_ - well, windows doesn't support `bun` in github actions afaict. So, I'm using npm instead.

[How to Install Node](https://nodejs.org/en/download/package-manager)

## Project Setup

```bash
git clone <repo_url>
cd path/to/project
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Another demo

Another demo where I use the perplexity shell script to generate an answer super fast.
Not affiliated, was just replying to a thread lol

https://github.com/jasonjmcghee/plock/assets/1522149/6166af73-545f-4a8e-ad46-ea8aacd84969

## Secrets

Curious folks might be wondering what `ocr` feature is. I took a crack at taking a screenshot,
running OCR, and using that for context, instead of copying text manually. Long story short,
rusty-tesseract _really_ dissapointed me, which is awkward b/c it's core
to [xrem](https://github.com/jasonjmcghee/xrem).

If someone wants to figure this out... this could be really cool, especially with multi-modal models.
