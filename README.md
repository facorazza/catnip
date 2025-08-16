# catnip 🛠️📋

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
  - Regex pattern matching for precise code updates
  - Multi-file batch operations
  - Safe replacement with backup options

- ⚡ **Performance Optimized**

  - Async file processing
  - Optimized pattern matching with fast lookups
  - Structured logging with tracing
  - Memory-efficient directory traversal


## Installation

### From Source

```bash
git clone <repository-url>
cd catnip
cargo build --release
```

## Usage

### File Concatenation (`cat` command)

```bash
# Process files and output to stdout
catnip cat src/main.rs src/lib.rs

# Process entire directory
catnip cat src

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
- `--exclude <PATTERN>`: Additional patterns to exclude
- `--include <PATTERN>`: Additional patterns to include
- `--ignore-comments`: Strip code comments from output
- `--ignore-docstrings`: Remove docstrings from output
- `--max-size-mb <SIZE>`: Maximum file size in MB (default: 10)

### `patch` subcommand

- `<JSON_FILE>`: JSON file containing patch specifications
- `--dry-run`: Preview changes without applying them
- `--backup`: Create backup files before modifications

## Patch JSON Format

```json
{
  "analysis": "Brief description of changes being made",
  "files": [
    {
      "path": "src/main.rs",
      "updates": [
        {
          "pattern": "fn old_function\\(.*?\\) \\{[^}]+\\}",
          "replacement": "fn new_function(param: &str) {\n    println!(\"Updated: {}\", param);\n}",
          "case_insensitive": false,
          "multiline": true,
          "dot_matches_newline": true,
          "max_replacements": null,
          "description": "Replace old_function with improved version"
        }
      ]
    }
  ]
}
```

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

## Performance Features

- **Fast pattern matching**: Separate exact matches from glob patterns
- **Optimized directory traversal**: Skip common build/cache directories early
- **Async I/O**: Non-blocking file operations
- **Memory efficient**: Stream processing for large codebases
- **Binary detection**: Quick null-byte scanning to skip binary files

## Troubleshooting

- **Large output**: Use `--max-size-mb` to limit file sizes or `--exclude` for specific patterns
- **Permission errors**: Ensure read/write access to target files and directories
- **Regex patterns**: Test patterns carefully - use online regex testers
- **Logging**: Set `RUST_LOG=debug` for detailed operation logs
- **Path issues**: Use forward slashes in patterns, even on Windows
