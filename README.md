# mobile-inspector-cli

A Rust CLI to inspect the view hierarchy of **Android** apps (via `adb`) and **iOS** apps (via the Appium REST API) **without opening Appium Inspector**. Built to find locators from the terminal in seconds: pipe-friendly (`| jq`, `| grep`), regex and XPath filters, and an interactive REPL for exploration sessions.

```bash
mobile-inspector dump android --id "btn_continue" --format json | jq '.nodes[].attrs.bounds'
```

## Table of contents

- [Why it exists](#why-it-exists)
- [Requirements](#requirements)
- [Setup](#setup)
  - [Install Rust](#1-install-rust)
  - [Android setup](#2-android-setup)
  - [iOS setup](#3-ios-setup)
  - [Build and install](#4-build-and-install)
- [Usage](#usage)
  - [General structure](#general-structure)
  - [`dump`](#dump-one-shot)
  - [`repl`](#repl-interactive)
  - [`config`](#config)
- [Attribute mapping Android &harr; iOS](#attribute-mapping-android--ios)
- [Output formats](#output-formats)
- [Common recipes](#common-recipes)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)
- [Development](#development)
- [Roadmap](#roadmap)
- [License](#license)

## Why it exists

Appium Inspector is a heavy Electron GUI: slow to start, not scriptable, and it requires an existing session id before you can inspect anything. For everyday tasks (finding a button id, checking an element's bounds, listing all `clickable` nodes) a CLI is faster and composes well with `jq`, `grep`, and scripts. `mobile-inspector` covers exactly that.

| Use case | Appium Inspector | `mobile-inspector` |
|----------|------------------|---------------------|
| List all buttons by regex | Manual click per node | `dump --id "btn_.*"` |
| Pipe to `jq`/`grep` | No | Yes (`--format json`) |
| Android without Appium | No (driver required) | Yes (just `adb`) |
| Scriptable in CI | No | Yes |
| Visual inspection with overlay | Yes | No (roadmap) |
| Interactive tap/swipe | Yes | No (roadmap) |

## Requirements

- **Rust 1.85+** (edition 2024). [Install Rust](https://rustup.rs).
- **Android:**
  - `adb` on your `PATH` (part of Android Platform Tools).
  - An authorized physical device or a running emulator.
- **iOS:**
  - A running **Appium server** (typically `http://localhost:4723`).
  - An **active session** against the app you want to inspect (created by your test suite or manually).

> iOS **cannot** be inspected without Appium: unlike Android, there is no native CLI equivalent to `uiautomator dump`. You need WDA (WebDriverAgent) behind Appium.

## Setup

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# or, on macOS with Homebrew:
brew install rustup-init && rustup-init
```

Verify:

```bash
rustc --version   # rustc 1.85+
cargo --version
```

### 2. Android setup

Install Android Platform Tools (includes `adb`):

```bash
# macOS
brew install --cask android-platform-tools

# Linux (Debian/Ubuntu)
sudo apt install android-tools-adb

# Windows: download from https://developer.android.com/tools/releases/platform-tools
```

Connect a device and authorize USB debugging. Verify:

```bash
adb devices
# List of devices attached
# RFCT40XDYAX	device
```

If it shows `unauthorized`, accept the RSA prompt on the phone and retry.

> If you have multiple devices connected, you'll need `--serial <id>` on every command, or set `ANDROID_SERIAL` in your env.

### 3. iOS setup

Install Appium 2:

```bash
npm install -g appium
appium driver install xcuitest
```

Start the server:

```bash
appium --base-path /
```

Create a session from your code or with a manual `POST /session`. If your test suite already starts sessions, you can use those; `mobile-inspector` resolves the single active session automatically.

Configure the server URL if it's not the default:

```bash
mobile-inspector config set appium.url http://localhost:4723
```

### 4. Build and install

Clone and compile release:

```bash
git clone https://github.com/gmassello/mobile-inspector-cli.git
cd mobile-inspector-cli
cargo build --release
```

The binary ends up at `target/release/mobile-inspector`. To install it into `$CARGO_HOME/bin` (which should be on your `PATH`):

```bash
cargo install --path .
mobile-inspector --version
```

You can also move the binary manually:

```bash
sudo cp target/release/mobile-inspector /usr/local/bin/
```

## Usage

### General structure

```bash
mobile-inspector [-v|--verbose] <COMMAND> [OPTIONS]
```

Available commands:

| Command  | What it does |
|----------|--------------|
| `dump`   | Takes a snapshot of the view hierarchy and prints it (with optional filters). |
| `repl`   | Interactive session with caching of the last dump. |
| `config` | Reads/writes `~/.config/mobile-inspector/config.toml`. |

Detailed help:

```bash
mobile-inspector --help
mobile-inspector dump --help
mobile-inspector repl --help
mobile-inspector config --help
```

### `dump` (one-shot)

Takes a snapshot, optionally filters, and prints to stdout. Designed for pipes and scripts.

```bash
mobile-inspector dump <PLATFORM> [FILTERS] [--format xml|json|table] [--serial <s>] [--session <id>]
```

#### Without filters (full XML)

```bash
mobile-inspector dump android
mobile-inspector dump ios --session 7a3f...
```

#### Attribute filters (regex)

All filters accept regex (Rust regex syntax). Combining them is an AND.

```bash
mobile-inspector dump android --id "btn_.*"
mobile-inspector dump android --text "^Continue$"
mobile-inspector dump android --class "Button"
mobile-inspector dump android --content-desc "next"

# Combined (AND)
mobile-inspector dump android --id "btn_.*" --text "Continue"
```

The same flags work against iOS and the CLI maps attribute names automatically (see [Mapping](#attribute-mapping-android--ios)).

#### XPath

If you pass `--xpath`, attribute filters are ignored:

```bash
mobile-inspector dump android --xpath "//node[@clickable='true']"
mobile-inspector dump android --xpath "//node[contains(@resource-id, 'btn')]"
mobile-inspector dump ios --xpath "//*[@type='XCUIElementTypeButton']"
```

Supports XPath 1.0 (provided by `sxd-xpath`).

#### Device / session selector

| Flag | Platform | When to use |
|------|----------|-------------|
| `--serial <id>` | Android | Multiple devices/emulators in `adb devices`. |
| `--session <id>` | iOS | Multiple active sessions or you want to target a specific one. |

### `repl` (interactive)

Spawns a persistent session that **caches the last dump** (XML + parsed tree). Useful when you're exploring a screen and want to try several filters without hitting the device each time.

```bash
mobile-inspector repl android
mobile-inspector repl ios --session <id>
```

Commands available inside the REPL:

| Command | Description |
|---------|-------------|
| `dump` | Prints the full view hierarchy. Uses cache if it exists. |
| `refresh` | Forces a new dump from the device and invalidates the cache. |
| `find --id|--text|--class|--content-desc <regex>` | Filters the cached dump. Same flags as `dump`. |
| `xpath <expr>` | Applies XPath against the cached dump. |
| `help` or `?` | List of commands. |
| `exit` or `quit` | Exits. `Ctrl+D` or `Ctrl+C` also work. |

Example session:

```
$ mobile-inspector repl android
mobile-inspector REPL (android). 'help' for commands, 'exit' to quit.
[android] > find --id "btn_.*"
id/name                                       class/type            text/label   bounds
...
[android] > xpath //node[@clickable='true']
...
[android] > refresh
refreshed
[android] > exit
```

### `config`

| Subcommand | What it does |
|------------|--------------|
| `config path` | Prints the path of the config file. |
| `config get <key>` | Reads a key. |
| `config set <key> <value>` | Writes a key. |

Supported keys:

| Key | Default | Description |
|-----|---------|-------------|
| `appium.url` | `http://localhost:4723` | Base URL of the Appium server (for iOS). |

```bash
mobile-inspector config set appium.url http://10.0.0.5:4723
mobile-inspector config get appium.url
mobile-inspector config path
# /Users/<you>/Library/Application Support/mobile-inspector/config.toml
```

## Attribute mapping Android &harr; iOS

`dump`/`find` flags use neutral names and match against the equivalent attributes on each platform:

| Flag | Android (`uiautomator`) | iOS (`XCUITest`) |
|------|-------------------------|------------------|
| `--id` | `resource-id` | `name` |
| `--text` | `text` | `label` |
| `--class` | `class` | `type` |
| `--content-desc` | `content-desc` | `value` |

The raw XML keeps each platform's original attributes; the mapping only applies to filters.

## Output formats

### `--format xml` (default)

Pretty-printed XML. If there are filters, results are wrapped in `<results>...</results>`.

```xml
<results>
  <node bounds="[40,2100][1040,2280]" class="android.widget.Button" resource-id="cl.mach.app:id/btn_continue" text="Continue"/>
</results>
```

### `--format json`

Parseable JSON with a different shape depending on whether filters are applied:

```json
// Without filters
{ "type": "tree", "root": { "tag": "hierarchy", "attrs": {...}, "children": [...] } }

// With filters
{ "type": "list", "count": 2, "nodes": [ { "tag": "node", "attrs": {...} }, ... ] }
```

Ideal to combine with `jq`:

```bash
mobile-inspector dump android --id "btn_.*" --format json | jq '.nodes[] | {id: .attrs."resource-id", bounds: .attrs.bounds}'
```

### `--format table`

Summary table with `id/name`, `class/type`, `text/label`, `bounds` columns. Designed for quick visual scanning. Long columns get truncated.

```
id/name                            class/type                text/label    bounds
--------------------------------------------------------------------------------------
cl.mach.app:id/btn_continue        android.widget.Button     Continue      [40,2100][1040,2280]
cl.mach.app:id/btn_cancel          android.widget.Button     Cancel        [40,1900][1040,2080]
```

## Common recipes

#### List all clickable buttons

```bash
mobile-inspector dump android --xpath "//node[@clickable='true']" --format table
```

#### Find the resource-id of a visible text

```bash
mobile-inspector dump android --text "Continue" --format json | jq '.nodes[].attrs."resource-id"'
```

#### Full snapshot to a file to diff between runs

```bash
mobile-inspector dump android > before.xml
# (perform action)
mobile-inspector dump android > after.xml
diff before.xml after.xml
```

#### Only elements whose bounds are inside the visible viewport

```bash
mobile-inspector dump android --format json | jq '.root | .. | objects | select(.attrs.bounds) | select(.attrs.bounds | startswith("[0,0]") | not)'
```

#### Search elements by partial content using XPath

```bash
mobile-inspector dump android --xpath "//node[contains(@content-desc, 'next')]"
```

#### List all non-empty `text` values

```bash
mobile-inspector dump android --format json | jq '.. | objects | select(.attrs.text? and .attrs.text != "") | .attrs.text'
```

## Troubleshooting

#### `adb: command not found`

Install Android Platform Tools (see [Setup](#2-android-setup)).

#### `error: no XML found in adb output`

The `uiautomator dump` may fail if the screen is transitioning or if a system dialog (e.g. permissions) is up. Retry after the UI stabilizes. It can also happen if the device was just unlocked.

#### `error: no active appium sessions`

There are no sessions created on the Appium server. Start one from your suite or create it manually with a `POST /session`. Verify with `curl http://localhost:4723/sessions`.

#### `error: N active sessions; pass --session <id>`

Multiple sessions exist on the server. List them and pick one:

```bash
curl -s http://localhost:4723/sessions | jq '.value[].id'
mobile-inspector dump ios --session <picked-id>
```

#### `appium responded 404 at /session/.../source`

The session exists but is closed or expired. Start a new one.

#### iOS XML does not look like Android's

That's expected: each platform uses a different representation (`hierarchy/node` on Android, `XCUIElementType*` on iOS). Filter flags map the relevant names; the rest of the XML is platform-specific.

#### `invalid regex '...': ...`

The regex doesn't compile. `mobile-inspector` uses the Rust [`regex`](https://docs.rs/regex/latest/regex/) crate's dialect (no lookahead/lookbehind). To match a literal `.`: `\.`.

## Architecture

```
mobile-inspector-cli/
|- Cargo.toml
|- src/
|  |- main.rs              entry point (dispatch to subcommands)
|  |- lib.rs               re-exports for integration tests
|  |- cli.rs               clap structs (Cli, DumpArgs, AttrFilters, ...)
|  |- attr.rs              constants for attribute names (android/ios mapping)
|  |- error.rs             InspectorError (thiserror) + Result alias
|  |- model.rs             UiNode + iter_descendants
|  |- config.rs            load/save TOML at ~/.config/mobile-inspector/
|  |- repl.rs              REPL (rustyline) + xml/tree Cache
|  |- platform/
|  |  |- mod.rs            trait Platform { fn dump_xml() }
|  |  |- android.rs        adb exec-out uiautomator dump /dev/tty
|  |  |- ios.rs            Appium REST: GET /sessions, /session/:id/source
|  |- filter/
|  |  |- mod.rs            FilterResult enum, apply_filters
|  |  |- attrs.rs          regex over UiNode (uses attr::*)
|  |  |- xpath.rs          sxd-xpath over the raw XML
|  |- output/
|     |- mod.rs            dispatch by OutputFormat
|     |- xml.rs            UiNode pretty-print
|     |- json.rs           serde_json (TreeOutput / ListOutput)
|     |- table.rs          per-column truncated table
|- tests/
   |- fixtures/            real Android and iOS dumps
   |- filters.rs           integration tests (attrs, xpath, output)
```

**Central trait:**

```rust
pub trait Platform {
    fn dump_xml(&self) -> Result<String>;
    fn name(&self) -> &'static str;
}
```

Android backend shells out to `adb`; iOS backend talks HTTP to Appium. Filters and formatters operate over the resulting XML and are platform-agnostic.

## Development

```bash
# Build
cargo build              # debug
cargo build --release    # optimized

# Tests (includes fixtures, no device required)
cargo test

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt
cargo fmt --check
```

Relevant tech stack:

| Concern | Crate |
|---------|-------|
| CLI parsing | `clap` v4 (derive) |
| REPL | `rustyline` |
| XML parsing | `roxmltree` |
| XPath | `sxd-document` + `sxd-xpath` |
| JSON | `serde_json` |
| HTTP | `reqwest` (blocking, rustls) |
| Errors | `thiserror` + `anyhow` |
| Config | `serde` + `toml` + `dirs` |

## Roadmap

- [ ] Interaction: `tap <selector>`, `swipe`, `input`.
- [ ] Screenshot with bounds overlay of the filtered element.
- [ ] `watch` mode: periodic poll until an element appears.
- [ ] Support for multiple Appium endpoints (profiles).
- [ ] Distribution via Homebrew tap.

## License

[MIT](LICENSE). (c) 2026 gmassello.
