# Reframe

[![Crates](https://img.shields.io/crates/v/reframe.svg)](https://crates.io/crates/reframe)
[![Build Status](https://travis-ci.org/Ansvia/reframe.svg?branch=master)](https://travis-ci.org/Ansvia/reframe)

> Because *"don't repeat yourself"*

Reframe is a lightweight project scaffolding tool that enables rapid setup of new projects by generating the necessary directories, files, and code templates. It streamlines the development process from the outset.

![Reframe Demo](img/reframe.gif?raw=true)

For detailed usage, see the [Reframe Documentation](DOCS.md).

---

## Install

### Homebrew (macOS)

```bash
brew tap ansvia/tools
brew install reframe
```

### Cargo

If you have Rust installed:

```
$ cargo install reframe
```

### Download Binary

Download a pre-built binary for your platform from the [releases page](https://github.com/anvie/reframe/releases).

---

## Usage

```
$ reframe [SOURCE]
```

### Example

```
$ reframe anvie/basic-rust
```

`anvie/basic-rust` refers to the GitHub repository [basic-rust.rf](https://github.com/anvie/basic-rust.rf).

---

## Creating a Reframe Source

Creating a Reframe source is straightforward — write a `Reframe.toml` at the root of your template project:

```toml
[reframe]
name = "Basic Rust"
mode = "generate"        # "generate" (default) or "apply"
min_version = "0.1"

[project]
name = "Hello World"
version = "1.0"

[[param]]
with_serde = { ask = "With serde?", default = false }

[[param]]
serde_version = { ask = "Serde version?", default = "1.0", if = "with_serde" }

[[param]]
# No default value means required
author_name = { ask = "Author name?" }

[[param]]
author_email = { ask = "Author email?" }
```

Top-level configuration fields in `[reframe]`:

| Field | Description |
|---|---|
| `name` | Human-readable template name |
| `mode` | `"generate"` (default, scaffold into new directory) or `"apply"` (overlay into current directory) |
| `min_version` | Minimum reframe version required |

Every string parameter automatically gets case variants, e.g. `author_name` yields:
`author_name_lowercase`, `author_name_snake_case`, `author_name_kebab_case`, and more.

When you need the project name in snake case, write `$name_snake_case$`.

Test your template locally:

```
$ reframe /path/to/your/template
```

For hands-on examples, check the [`examples/`](examples/) directory in this repository — it contains real templates including `basic.rs.rf` and `git-init.rf`.

When ready, push the repository to GitHub with a `.rf` suffix on the repo name. For example, if your repo is named `unicorn`, name the remote repository `unicorn.rf`. Then use it anywhere:

```
$ reframe your-username/unicorn
```

For detailed usage, see the [Reframe Documentation](DOCS.md).

### Reframe Source Examples

- [anvie/basic-rust.rf](https://github.com/anvie/basic-rust.rf)
- [anvie/hello-world-py.rf](https://github.com/anvie/hello-world-py.rf)

---

## Operation Modes

Reframe supports two operation modes:

### Scaffold Mode (default)

Generates a new project directory with all template files. This is the standard scaffolding flow:

```
$ reframe anvie/basic-rust
```

No special configuration is needed — all templates work in scaffold mode by default.

### Apply Mode

Overlays template files directly into the **current directory**, rather than creating a new project directory. This is useful for tasks like installing git hooks, adding configuration files to an existing project, or running setup scripts.

```
$ reframe apply anvie/git-init
```

To make a template support apply mode, add `mode = "apply"` to the `[reframe]` section of its `Reframe.toml`:

```toml
[reframe]
name = "Git Init"
mode = "apply"
```

Templates in apply mode can also define `[[post_generate]]` actions and a `finish_text` message. See the [`examples/git-init.rf`](examples/git-init.rf) template for a complete example.

---

## Supported Case Variants

| Variant | Example Input | Output |
|---|---|---|
| `*_lower_case` | my cool app | my cool app |
| `*_snake_case` | my cool app | my_cool_app |
| `*_kebab_case` | my cool app | my-cool-app |
| `*_shout_snake_case` | my cool app | MY_COOL_APP |
| `*_upper_case` | my cool app | MY COOL APP |
| `*_camel_case` | my cool app | myCoolApp |
| `*_pascal_case` | my cool app | MyCoolApp |

### Built-in Variables

- `$year$` — Current year (e.g. 2026)
- `$month_name$` — Current month name (e.g. June)

---

## Templating

Reframe supports Handlebars templating inside generated files for conditional logic:

```handlebars
{{#if with_jwt}}
const jwt = require('jsonwebtoken');
{{/if}}
```

---

## Recent Updates (v0.5.12)

- **Automatic cleanup**: `_hooks/` directory is removed after a successful `apply`.
- **Smarter directory skipping**: `dist/` is now skipped alongside `node_modules/` and `.git` during template processing.
- **Binary file handling**: Binary files are automatically skipped during template processing.
- **Graceful error recovery**: Files that cause Handlebars template errors are skipped gracefully instead of aborting.

---

## Local Examples

This repository includes example templates in the [`examples/`](examples/) directory. Browse them to learn how `Reframe.toml`, templates, and hooks work in practice:

- [`examples/basic.rs.rf`](examples/basic.rs.rf) — A simple Rust project template with file copying and asset loading.
- [`examples/git-init.rf`](examples/git-init.rf) — An apply-mode template that installs a pre-commit git hook.

## Available Sources

- [anvie/basic-rust-cli.rf](https://github.com/anvie/basic-rust-cli.rf) — Basic CLI application.
- [anvie/rust-grpc.rf](https://github.com/anvie/rust-grpc.rf) — Rust gRPC application.

For more sources, see [SOURCES](SOURCES.md).

List available sources from the command line:

```bash
reframe --list
```
