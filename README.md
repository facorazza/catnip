![Quick Cat](quickcat.png)

```txt
________        .__        __     _________         __   
\_____  \  __ __|__| ____ |  | __ \_   ___ \_____ _/  |_ 
 /  / \  \|  |  \  |/ ___\|  |/ / /    \  \/\__  \\   __\
/   \_/.  \  |  /  \  \___|    <  \     \____/ __ \|  |  
\_____\ \_/____/|__|\___  >__|_ \  \______  (____  /__|  
       \__>             \/     \/         \/     \/
```


# quick-cat
Script for concatenating files and prompting an LLM to quickly bring it up to speed on a project.

## Overview

`quick-cat.py` is a script designed to concatenate multiple files, add a directory structure, and format the content for markdown. This tool is especially useful for sharing projects with Large Language Models (LLMs) or collaborators, providing both content and structure to enhance understanding.

## Purpose

The main goal of this script is to provide an organized and easy-to-read markdown document that includes multiple code files along with their directory structure. By providing both the content and the structure, it becomes easier to understand the context and relationships between the files.

## Usage Example

```sh
python quick-cat.py app.py ./source/config.py /static/js/javascript.js /templates/index.html -o combined_output.md --copy
```

This command concatenates `app.py`, `config.py`, `javascript.js`, and `index.html` into a markdown file named `combined_output.md`, including a directory structure and syntax highlighting for each file's content. The `--copy` flag automatically copies the contents to the clipboard.

## Arguments

- `files` (list of str): List of file paths to concatenate.
- `-o`, `--output` (str): Output file name (default: `output.md`).
- `--copy`: Copy the output file contents to the clipboard.

## Output

The script produces a markdown file with the following features:
- A visual representation of the directory structure of the input files.
- Concatenated contents of each file, with appropriate markdown syntax highlighting based on file type.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

## Getting Started

To use the script, simply run the command with the appropriate file paths and options as shown in the usage example above. The script will generate a markdown file with the concatenated content and directory structure.

## Guidelines for LLMs

The script includes a detailed prompt for LLMs with specific guidelines to follow when providing code help. This ensures consistent and high-quality assistance from the LLMs.

## LLM Prompt

The LLM Prompt is embedded inside the script to keep it to one file, I have it separeted out and pulled into the final file automatically.  I also keep a readme on my project that is intended to help bring the LLM up to speed and I add that to the list of files I am concatenating.
