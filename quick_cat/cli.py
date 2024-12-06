#!/usr/bin/env python3

import os
import logging
import platform
import subprocess
import fnmatch
import re
from pathlib import Path

import click


logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler()],
)


def is_wayland():
    """Check if the current display server is Wayland."""
    return os.environ.get("WAYLAND_DISPLAY") is not None

def get_files_recursively(
    paths, exclude_patterns=None, ignore_comments=False, ignore_docstrings=False
):
    """
    Recursively find files in given paths, with optional exclusion.
    """
    # Predefined exclusion patterns for non-text and system directories
    default_exclude = [
        "*.pyc",
        "*.pyo",
        "*.pyd",
        ".git*",
        ".svn*",
        ".hg*",
        ".DS_Store",
        "*.jpg",
        "*.jpeg",
        "*.png",
        "*.gif",
        "*.bmp",
        "*.svg",
        "*.webp",
        "*.ico",
        "*.wav",
        "*.mp3",
        "*.mp4",
        "*.mov",
        "*.avi",
        "*.zip",
        "*.tar",
        "*.gz",
        "*.rar",
        "*.7z",
        "*.lock",
        "LICENSE",
        # Common directories to exclude
        "*/.git*",
        "*/.vscode*",
        "*/.idea*",
        "*/node_modules*",
        "*/venv*",
        "*/.env*",
        "*/__pycache__*",
        "*/.mypy_cache*",
        "*/.pytest_cache*",
        "*/build*",
        "*/dist*",
        "*/egg-info*",
    ]

    # Combine default exclusions with user-provided exclusions
    exclude_patterns = list(set(default_exclude + (exclude_patterns or [])))

    found_files = []

    for path in paths:
        path_obj = Path(path)

        if path_obj.is_file():
            # Check for empty files and exclusion patterns
            if _is_valid_file(path_obj, exclude_patterns, ignore_comments, ignore_docstrings):
                found_files.append(path_obj)
        elif path_obj.is_dir():
            for root, dirs, files in os.walk(path_obj):
                root_path = Path(root)

                # Filter out excluded directories
                dirs[:] = [
                    d
                    for d in dirs
                    if not any(
                        fnmatch.fnmatch(str(root_path / d), pattern)
                        or fnmatch.fnmatch(d, pattern.split("/")[-1])
                        for pattern in exclude_patterns
                    )
                ]

                for file in files:
                    file_path = root_path / file
                    # Check for empty files and exclusion patterns
                    if _is_valid_file(
                        file_path, exclude_patterns, ignore_comments, ignore_docstrings
                    ):
                        found_files.append(file_path)

    return found_files


def _is_valid_file(file_path, exclude_patterns, ignore_comments, ignore_docstrings):
    if file_path.stat().st_size == 0:
        return False

    # Check exclusion patterns with improved pattern matching
    if any(_advanced_pattern_match(str(file_path), pattern) for pattern in exclude_patterns):
        return False

    if ignore_comments or ignore_docstrings:
        try:
            with open(file_path, "r") as f:
                content = f.read()
                if ignore_comments:
                    content = _remove_comments(file_path, content)
                if ignore_docstrings:
                    content = _remove_docstrings(content)

                # If content becomes empty after removing comments/docstrings
                if not content.strip():
                    return False
        except Exception as e:
            logging.error(f"Error processing file {file_path}: {e}")
            return False

    return True


def _advanced_pattern_match(path, pattern):
    # Enhanced pattern matching with more robust wildcard handling
    regex_pattern = pattern.replace(".", r"\.").replace("*", ".*")
    return re.match(regex_pattern, path) is not None


def _remove_comments(file_path, content):
    ext = file_path.suffix.lower()
    if ext in [".py", ".pyw"]:
        return re.sub(
            r'(#.*?$)|(\'\'\'.*?\'\'\'|""".*?""")', "", content, flags=re.MULTILINE | re.DOTALL
        )
    elif ext in [".js", ".java", ".c", ".cpp", ".cs", ".swift"]:
        return re.sub(r"(//.*?$)|(\/\*.*?\*\/)", "", content, flags=re.MULTILINE | re.DOTALL)
    elif ext in [".html", ".xml"]:
        return re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)
    return content


def _remove_docstrings(content):
    # Remove Python docstrings
    return re.sub(r'(\'\'\'.*?\'\'\'|""".*?""")', "", content, flags=re.MULTILINE | re.DOTALL)


def generate_directory_structure(files):
    """
    Generates a compact text representation of the directory structure for the given files.
    """
    structure = []
    root = Path(".").resolve()

    # Group files by their directory
    file_dict = {}
    for file in sorted(files):
        try:
            relative_path = file.relative_to(root)
        except ValueError:
            relative_path = file  # If the path is not relative, use the absolute path

        parts = list(relative_path.parts)

        # Build the directory hierarchy
        current = file_dict
        for part in parts[:-1]:
            current = current.setdefault(part, {})
        current[parts[-1]] = None

    def build_tree(tree, prefix=""):
        """Recursively build the directory tree."""
        items = list(tree.items())
        for i, (name, subtree) in enumerate(items):
            is_last = i == len(items) - 1

            # Choose the right tree connector
            connector = "└── " if is_last else "├── "

            # Add the current item
            structure.append(f"{prefix}{connector}{name}")

            # Recursively build subtree
            if subtree is not None:
                new_prefix = prefix + ("    " if is_last else "│   ")
                build_tree(subtree, new_prefix)

    # Start building the tree
    build_tree(file_dict)

    return structure


def get_file_type(file):
    """
    Determines the file type based on the file extension.

    Parameters:
        file (Path): Path object of the file.

    Returns:
        str: File type string for markdown syntax highlighting.
    """
    ext = file.suffix.lower()
    file_types = {
        ".py": "python",
        ".js": "javascript",
        ".html": "html",
        ".css": "css",
        ".json": "json",
        ".log": "",
        ".txt": "",
        ".md": "markdown",
        ".xml": "xml",
        ".yaml": "yaml",
        ".yml": "yaml",
        ".sh": "shell",
        ".c": "c",
        ".cpp": "cpp",
        ".java": "java",
        ".php": "php",
        ".rb": "ruby",
        ".go": "go",
        ".swift": "swift",
        ".rs": "rust",
        ".pl": "perl",
        ".ps1": "powershell",
        ".bat": "batch",
        ".vbs": "vbscript",
        ".ini": "ini",
        ".toml": "toml",
        ".csv": "csv",
        ".tsv": "tsv",
        ".rst": "rst",
        ".tex": "tex",
        ".org": "org",
        ".jsx": "jsx",
        ".tsx": "tsx",
    }
    return file_types.get(ext, "")


def concatenate_files(files, output_file=None):
    """
    Concatenates the content of multiple files, adds directory structure and file type annotations.

    Parameters:
        files (list of Path): List of file paths to concatenate.
        output_file (str): The name of the output file.
    """
    output = []

    output.append("# Project Structure\n\n")
    output.append("├── ./\n")
    directory_structure = generate_directory_structure(files)
    output.extend(line + "\n" for line in directory_structure)
    output.append("\n# File Contents\n\n")

    for file in sorted(files):
        try:
            relative_path = file.relative_to(Path(".").resolve())
        except ValueError:
            relative_path = file

        file_type = get_file_type(file)

        output.append(f"## {relative_path}\n\n")
        output.append(f"```{file_type}\n")

        try:
            with open(file, "r") as f:
                # Read the content and strip trailing newlines
                content = f.read().rstrip("\n")
                output.append(content)
        except Exception as e:
            logging.error(f"Error reading file {file}: {e}")

        output.append("\n```\n\n")

    result = "".join(output)

    if output_file:
        try:
            with open(output_file, "w") as out_file:
                out_file.write(result)
            logging.info(f"Successfully created output file: {output_file}")
        except Exception as e:
            logging.error(f"Error writing output file: {e}")

    return result


def copy_to_clipboard(content):
    """
    Copies the contents of the output file to the clipboard.

    Parameters:
        output_file (str): The name of the output file.
    """
    try:
        system = platform.system()
        if system == "Linux":
            # Check for Wayland or X11
            if is_wayland():
                try:
                    subprocess.run(["wl-copy"], input=content.encode("utf-8"), check=True)
                    click.echo("Copied using wl-copy (Wayland)")
                except FileNotFoundError:
                    try:
                        subprocess.run(
                            ["xclip", "-selection", "clipboard"],
                            input=content.encode("utf-8"),
                            check=True,
                        )
                        click.echo("Copied using xclip (X11)")
                    except FileNotFoundError:
                        click.echo("Neither wl-copy nor xclip found. Unable to copy to clipboard.")
            else:
                try:
                    subprocess.run(
                        ["xclip", "-selection", "clipboard"],
                        input=content.encode("utf-8"),
                        check=True,
                    )
                    click.echo("Copied using xclip (X11)")
                except FileNotFoundError:
                    click.echo("xclip not found. Unable to copy to clipboard.")
        elif system == "Darwin":
            subprocess.run(["pbcopy"], input=content.encode("utf-8"), check=True)
            click.echo("Copied using pbcopy (macOS)")
        elif system == "Windows":
            subprocess.run(["clip"], input=content.encode("utf-8"), check=True)
            click.echo("Copied using clip (Windows)")
        else:
            click.echo(f"Clipboard copy not supported on {system}.")

    except Exception as e:
        logging.error(f"Error copying contents to clipboard: {e}")


@click.command(help="Concatenate files with directory structure and content.")
@click.argument("paths", nargs=-1, type=click.Path(exists=True))
@click.option("-o", "--output", default=None, help="Output file name (optional)")
@click.option("--copy/--no-copy", default=False, help="Copy the output to the clipboard")
@click.option("--exclude", multiple=True, help="Additional patterns to exclude from file search")
@click.option("--ignore-comments", is_flag=True, default=False, help="Ignore code comments")
@click.option("--ignore-docstrings", is_flag=True, default=False, help="Ignore docstrings")
def main(paths, output, copy, exclude, ignore_comments, ignore_docstrings):
    """Main function to execute file concatenation."""
    files = get_files_recursively(
        paths, list(exclude), ignore_comments=ignore_comments, ignore_docstrings=ignore_docstrings
    )

    result = concatenate_files(files, output)

    if copy:
        copy_to_clipboard(result)
    elif not output and click.confirm("Do you want to copy the contents to the clipboard?"):
        copy_to_clipboard(result)


if __name__ == "__main__":
    main()
