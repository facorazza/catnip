# catnip 🛠️📋

[![Lint, test and build](https://github.com/facorazza/catnip/actions/workflows/build.yml/badge.svg)](https://github.com/facorazza/catnip/actions/workflows/build.yml)

**Rapid Project File Concatenation and Code Patching for LLM Analysis**

## Overview

`catnip` is a high-performance Rust CLI tool designed to simplify project understanding and code modification workflows. It concatenates multiple files into a single, well-formatted markdown document and provides automated code patching capabilities using regex-based updates.

## Key Features

- 📂 **Comprehensive Directory Mapping**

  - Generates a visual tree representation of project structure
  - Supports recursive file discovery
  - Intelligent file and directory exclusion with comprehensive default patterns

- 🔍 **Advanced File Processing**

  - Supports 50+ programming languages and file types
  - Automatic syntax highlighting in code blocks
  - Binary file detection and exclusion
  - Optional comment and docstring removal
  - Configurable file size limits

- 🔧 **Automated Code Patching**

  - JSON-based patch specification system
  - Exact string matching for precise code updates
  - Multi-file batch operations
  - Safe replacement with backup options

## Installation

### From Source

```bash
git clone <repository-url>
cd catnip
cargo build --release
cargo install --path .
```

Make sure the binary is in your PATH.

## Usage

### File Concatenation (`cat` command)

```bash
# Process files and output to stdout
catnip cat src/main.rs src/lib.rs

# Process entire directory
catnip cat src

# Append prompt for JSON updates to be used with catnip patch
catnip cat --prompt src

# Save to file
catnip cat src -o project_summary.md

# Exclude additional patterns
catnip cat . --exclude "*.log" --exclude "temp/*"

# Include only specific patterns
catnip cat . --include "*.rs" --include "*.toml"

# Remove comments and docstrings
catnip cat src --ignore-comments --ignore-docstrings

# Set maximum file size (default: 10MB)
catnip cat . --max-size-mb 5
```

### Code Patching (`patch` command)

```bash
# Apply patches from JSON content in the clipboard
catnip patch

# Apply patches from JSON file
catnip patch updates.json

# Apply patches with dry-run (preview changes)
catnip patch updates.json --dry-run

# Create backups before applying patches
catnip patch updates.json --backup
```

## Command Line Options

### `cat` subcommand

- `<PATHS>...`: One or more files or directories to process
- `-o, --output <FILE>`: Optional output filename
- `-e, --exclude <PATTERN>`: Additional patterns to exclude
- `-i, --include <PATTERN>`: Additional patterns to include
- `--ignore-comments`: Strip code comments from output
- `--ignore-docstrings`: Remove docstrings from output
- `--max-size-mb <SIZE>`: Maximum file size in MB (default: 10)
- `-p, --prompt`: Include prompt instructions for LLM analysis

### `patch` subcommand

- `<JSON_FILE>`: JSON file containing patch specifications
- `--dry-run`: Preview changes without applying them
- `-b, --backup`: Create backup files before modifications

## Patch JSON Format

```json
{
  "analysis": "Brief description of changes being made",
  "files": [
    {
      "path": "src/main.rs",
      "updates": [
        {
          "old_content": "fn old_function() {\n    println!(\"old implementation\");\n}",
          "new_content": "fn new_function(param: &str) {\n    println!(\"Updated: {}\", param);\n}",
          "description": "Replace old_function with improved version"
        }
      ]
    }
  ]
}```

## Default Exclusion Patterns

The tool automatically excludes:

- **Build artifacts**: `*.pyc`, `*.o`, `*.class`, `target/`, `build/`, `dist/`
- **Dependencies**: `node_modules/`, `__pycache__/`, `.venv/`, `venv/`
- **Version control**: `.git/`, `.svn/`, `.hg/`, `.bzr/`
- **System files**: `.DS_Store`, `Thumbs.db`, `*.tmp`, `*.bak`
- **Media files**: `*.jpg`, `*.png`, `*.mp4`, `*.zip`, `*.tar`
- **IDE files**: `.vscode/`, `.idea/`
- **Logs and databases**: `*.log`, `*.sqlite`, `*.db`
- **Lock files**: `Cargo.lock`, `package-lock.json`, `yarn.lock`
- **Certificates**: `*.pem`, `*.key`, `*.crt`

## Default Inclusion Patterns

Automatically includes common development files:

- **Programming languages**: `*.rs`, `*.py`, `*.js`, `*.ts`, `*.java`, `*.go`, `*.cpp`, `*.c`, `*.h`
- **Web technologies**: `*.html`, `*.css`, `*.scss`, `*.vue`, `*.svelte`
- **Configuration**: `*.json`, `*.yaml`, `*.toml`, `*.ini`, `*.conf`
- **Documentation**: `*.md`, `*.txt`, `README*`, `CHANGELOG*`
- **Build files**: `Cargo.toml`, `package.json`, `Makefile`, `Dockerfile*`
- **Scripts**: `*.sh`, `*.bash`, `*.ps1`, `*.bat`
- **Database**: `*.sql`, `*.pgsql`

## Supported Languages

The tool recognizes and provides syntax highlighting for 40+ languages including:

- **Systems**: Rust, C, C++, Go
- **Web**: JavaScript, TypeScript, HTML, CSS, Vue, Svelte
- **Backend**: Python, Java, C#, PHP, Ruby
- **Mobile**: Swift, Kotlin, Dart
- **Functional**: Scala, Clojure, F#
- **Scripting**: Bash, PowerShell, Lua, Perl
- **Data**: SQL, R, YAML, JSON

## Output Format

The `cat` command generates a structured markdown document with:

1. **Project Structure**: ASCII tree representation of directories and files
2. **File Contents**: Each file's content in appropriately highlighted code blocks

## Workflow Example

1. **Analyze codebase**:

   ```bash
   catnip cat src/ > codebase.md
   ```

2. **Get LLM recommendations** (using the generated markdown)

3. **Create patch file** (following JSON format)

4. **Apply changes**:

   ```bash
   catnip patch updates.json --dry-run  # Preview
   catnip patch updates.json --backup   # Apply with backup
   ```

## Environment Variables

- `RUST_LOG`: Set logging level (`error`, `warn`, `info`, `debug`, `trace`)
