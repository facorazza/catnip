# quick-cat 🐱📋

**Rapid Project File Concatenation for LLM Comprehension**

## Overview

`quick-cat` is a high-performance Rust CLI tool designed to simplify project understanding by concatenating multiple files into a single, well-formatted markdown document. It's specifically crafted to help Large Language Models (LLMs) and collaborators quickly grasp the structure and content of a project.

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

- 📋 **Flexible Output Options**
  - Automatic clipboard integration (default behavior)
  - Optional file output
  - Cross-platform clipboard support (Linux Wayland/X11, macOS, Windows)

- ⚡ **Performance Optimized**
  - Async file processing
  - Comprehensive exclusion patterns for build artifacts, dependencies, and system files
  - Structured logging with tracing

## Prerequisites

- Rust 1.70+ (for building from source)
- Optional clipboard utilities:
  - Linux Wayland: `wl-clipboard`
  - Linux X11: `xclip`
  - macOS: Built-in `pbcopy`
  - Windows: Built-in `clip`

## Installation

### From Source
```bash
git clone <repository-url>
cd quick-cat
cargo build --release
```

### Binary Usage
```bash
./target/release/quick-cat [OPTIONS] <PATHS>...
```

## Usage Examples

### Basic Usage
```bash
# Process files and copy to clipboard (default)
quick-cat src/main.rs src/lib.rs

# Process entire directory
quick-cat ./src

# Save to file instead of clipboard
quick-cat src/ -o project_summary.md

# Disable clipboard copy
quick-cat src/ --no-copy
```

### Advanced Usage
```bash
# Exclude additional patterns
quick-cat . --exclude "*.log" --exclude "temp/*"

# Include only specific patterns
quick-cat . --include "*.rs" --include "*.toml"

# Remove comments and docstrings
quick-cat src/ --ignore-comments --ignore-docstrings

# Set maximum file size (default: 10MB)
quick-cat . --max-size-mb 5
```

## Command Line Options

- `<PATHS>...`: One or more files or directories to process
- `-o, --output <FILE>`: Optional output filename
- `--no-copy`: Don't copy to clipboard (clipboard is default)
- `--exclude <PATTERN>`: Additional patterns to exclude
- `--include <PATTERN>`: Additional patterns to include
- `--ignore-comments`: Strip code comments from output
- `--ignore-docstrings`: Remove docstrings from output
- `--max-size-mb <SIZE>`: Maximum file size in MB (default: 10)

## Default Exclusion Patterns

The tool automatically excludes:
- **Build artifacts**: `*.pyc`, `*.o`, `*.class`, `target/`, `build/`, `dist/`
- **Dependencies**: `node_modules/`, `__pycache__/`, `.venv/`
- **Version control**: `.git/`, `.svn/`, `.hg/`
- **System files**: `.DS_Store`, `Thumbs.db`, `*.tmp`
- **Media files**: `*.jpg`, `*.png`, `*.mp4`, `*.zip`
- **IDE files**: `.vscode/`, `.idea/`
- **Logs and databases**: `*.log`, `*.sqlite`

## Default Inclusion Patterns

Automatically includes common development files:
- **Programming languages**: `*.rs`, `*.py`, `*.js`, `*.ts`, `*.java`, `*.go`, `*.cpp`
- **Web technologies**: `*.html`, `*.css`, `*.scss`, `*.vue`
- **Configuration**: `*.json`, `*.yaml`, `*.toml`, `*.env.example`
- **Documentation**: `*.md`, `README*`, `CHANGELOG*`
- **Build files**: `Cargo.toml`, `package.json`, `Makefile`, `Dockerfile`

## Output Format

The tool generates a structured markdown document with:
1. **Project Structure**: Visual tree representation of directories and files
2. **File Contents**: Each file's content in appropriately highlighted code blocks

## Environment Variables

- `RUST_LOG`: Set logging level (`error`, `warn`, `info`, `debug`, `trace`)

## Performance Considerations

- Efficiently filters binary files using null-byte detection
- Skips large files (configurable limit)
- Uses async I/O for better performance
- Comprehensive pattern matching to avoid processing unnecessary files

## Troubleshooting

- **Clipboard issues**: Install appropriate clipboard utility for your system
- **Large output**: Use `--max-size-mb` to limit file sizes or `--exclude` for specific patterns
- **Permission errors**: Ensure read access to target files and directories
- **Logging**: Set `RUST_LOG=debug` for detailed operation logs
