# Reframe Documentation

Create a `Reframe.toml` file at the root of your template directory. Below is a complete reference of every available configuration field.

## Complete `Reframe.toml` Reference

```toml
[reframe]
# Human-readable name of this template.
name = "My Template"

# Author name or identifier.
author = "your-username"

# Minimum reframe version required to run this template.
# If the installed version is lower, reframe will abort with an error.
min_version = "0.5.0"

# Operation mode: "generate" (default) or "apply".
#   generate — scaffold a new project directory.
#   apply   — overlay files into the current directory.
mode = "generate"

[project]
# Default project name. The user can override it during generation.
name = "MyProject"

# Default project version.
version = "0.1.0"

# Directories to skip during template processing.
# These are excluded from the output entirely.
ignore_dirs = [
    "target", "build", "dist"
]

# Files to skip during template processing.
# Supports glob-like pattern matching (e.g. "*.lock", "secret.*").
ignore_files = [
    "Cargo.lock"
]

# Message displayed after generation finishes successfully.
# Supports $variables$ — all param values and case variants are available.
finish_text = """
Usage:
    $ cd $name_kebab_case$
    $ pip install -r requirements.txt
"""

# ------------------------------------------------------------------
# Interactive Parameters
# ------------------------------------------------------------------
# Each [[param]] defines a question asked during generation.
# The param key (e.g. "author_name") becomes a variable available
# in templates as $author_name$.
#
# Supported param fields:
#   ask     — The question text shown to the user.
#   default — Optional default value.
#   options — Optional list of allowed values (user must pick one).
#   if      — Condition; only ask this param if another param is truthy.
#             Value checked: "true" or "false".
#
# String params auto-generate case variants:
#   $author_name_lower_case$, $author_name_upper_case$,
#   $author_name_snake_case$, $author_name_kebab_case$,
#   $author_name_camel_case$, $author_name_pascal_case$,
#   $author_name_shout_snake_case$
# ------------------------------------------------------------------

# Simple required param (no default means user must provide a value).
[[param]]
author_name = { ask = "Author name?" }

# Param with a default value.
[[param]]
author_email = { ask = "Author email?", default = "author@example.com" }

# Boolean-style param (default = "true" or "false").
[[param]]
with_tests = { ask = "Include tests?", default = true }

# Conditional param — only asked if with_tests was answered "true".
[[param]]
test_framework = { ask = "Test framework?", default = "pytest", if = "with_tests" }

# Options param — user must pick from the list.
[[param]]
license = { ask = "License type?", options = ["MIT", "Apache-2.0", "GPL-3.0"], default = "MIT" }

# ------------------------------------------------------------------
# Conditional Files / Directories
# ------------------------------------------------------------------
# [[present]] ensures a file or directory is present only when a
# condition is met. If the condition is false, the file/dir is
# removed from the output.

# Only keep frontends/web if with_web_frontends was answered "true".
[[present]]
path = "frontends/web"
if = "with_web_frontends"

# ------------------------------------------------------------------
# Post-Generation Actions
# ------------------------------------------------------------------
# [[post_generate]] runs actions after all files have been generated.
# Multiple [[post_generate]] entries can be defined.

# Make a specific file executable (Unix: chmod +x).
[[post_generate]]
make_executable = "scripts/run.sh"

# Run an arbitrary shell command in the output directory.
# The command supports $variables$ substitution.
[[post_generate]]
command = "cp _hooks/pre-commit .git/hooks/pre-commit"

# ------------------------------------------------------------------
# Built-in Variables
# ------------------------------------------------------------------
# Available in all templates without defining any [[param]]:
#
#   $year$        — Current year (e.g. "2026")
#   $month_name$  — Current month name (e.g. "June")

# ------------------------------------------------------------------
# Case Variants for Project Name
# ------------------------------------------------------------------
# The project name (from [project].name) auto-generates these:
#
#   $name_lower_case$      — my project
#   $name_upper_case$      — MY PROJECT
#   $name_snake_case$      — my_project
#   $name_kebab_case$      — my-project
#   $name_camel_case$      — myProject
#   $name_pascal_case$     — MyProject
#   $name_shout_snake_case$ — MY_PROJECT
```

## Operation Modes

### Generate Mode (default)

Scaffolds a new project directory:

```
reframe your-username/your-template
```

The output directory is created using the project name (kebab-case) inside the current directory. If the directory already exists, reframe asks for confirmation before overwriting.

### Apply Mode

Overlays template files directly into the current directory. The template must declare `mode = "apply"` in its `[reframe]` section. After generation, the `_hooks/` directory (if any) is automatically removed since hooks are consumed by `post_generate` commands.

```
reframe apply your-username/git-init
```

## Templating

Reframe supports Handlebars templating inside generated files for conditional blocks:

```handlebars
{{#if with_jwt}}
const jwt = require('jsonwebtoken');
{{/if}}
```

Files with binary extensions (png, jpg, gif, mp4, zip, pdf, etc.) are automatically skipped during template processing.

## Example Templates

See the [`examples/`](examples/) directory in this repository for working templates:

- [`basic.rs.rf`](examples/basic.rs.rf) — Simple Rust project with params, file copying, and asset loading.
- [`git-init.rf`](examples/git-init.rf) — Apply-mode template that installs a pre-commit git hook.
