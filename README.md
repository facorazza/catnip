# quick-cat 🐱📋

![Quick Cat Logo](quickcat.png)

**Rapid Project File Concatenation for LLM Comprehension**

## Overview

`quick-cat.py` is a versatile Python script designed to simplify project understanding by concatenating multiple files into a single, well-formatted markdown document. It's specifically crafted to help Large Language Models (LLMs) and collaborators quickly grasp the structure and content of a project.

## Key Features

- 📂 **Comprehensive Directory Mapping**
  - Generates a visual representation of project structure
  - Supports recursive file discovery
  - Intelligent file and directory exclusion

- 🔍 **Intelligent File Parsing**
  - Supports multiple programming languages
  - Automatic syntax highlighting
  - Filters out non-text and system files

- 📋 **Flexible Clipboard Integration**
  - Optional clipboard copying
  - Cross-platform support (Linux, macOS, Windows)
  - Wayland and X11 compatibility

## Prerequisites

- Python 3.9+
- Click library (`pip install click`)
- Optional clipboard utilities:
  - Linux: `xclip` or `wl-copy`
  - macOS: Built-in `pbcopy`
  - Windows: Built-in `clip`

## Installation

```bash
python -m pip install quick-cat
```

or from source

```bash
git clone https://github.com/yourusername/quick-cat.git
cd quick-cat
pip install -r requirements.txt
```

## Usage Examples

### Basic Usage

```bash
python quick-cat.py app.py ./source/config.py /static/js/script.js
```

### Advanced Usage

```bash
# Specify output file and copy to clipboard
python quick-cat.py app.py config.py -o project_summary.md --copy

# Exclude specific file patterns
python quick-cat.py . --exclude "*.pyc" --exclude ".git/*"
```

## Command Line Options

- `paths`: One or more files or directories to concatenate
- `-o, --output`: Custom output filename (default: `output.md`)
- `--copy/--no-copy`: Automatically copy output to clipboard
- `--exclude`: Patterns to exclude from file search

## Performance Considerations

- Filters out large binary files
- Skips system and version control directories
- Optimized for quick project overview generation

## Troubleshooting

- Ensure all required dependencies are installed
- Check file and directory permissions
- Verify clipboard utility installation on Linux

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

Apache 2.0 License - See [LICENSE](LICENSE) for details
