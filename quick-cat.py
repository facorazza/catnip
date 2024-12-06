# quick-cat.py

import os
import logging
import platform
import subprocess
import fnmatch
from pathlib import Path

import click


def setup_logging(script_name):
    """Setup logging configuration to output logs to a file and console."""
    log_dir = "./logs"
    os.makedirs(log_dir, exist_ok=True)
    log_file = os.path.join(log_dir, f"{script_name}.log")

    logging.basicConfig(
        level=logging.DEBUG,
        format="%(asctime)s - %(levelname)s - %(message)s",
        handlers=[logging.FileHandler(log_file), logging.StreamHandler()],
    )


def is_wayland():
    """
    Check if the current display server is Wayland.

    Returns:
        bool: True if running on Wayland, False otherwise.
    """
    return os.environ.get("WAYLAND_DISPLAY") is not None


def get_files_recursively(paths, exclude_patterns=None):
    """
    Recursively find files in given paths, with optional exclusion.

    Parameters:
        paths (list): List of file or directory paths to search.
        exclude_patterns (list, optional): List of patterns to exclude.

    Returns:
        list: List of file paths.
    """
    exclude_patterns = exclude_patterns or []
    found_files = []

    for path in paths:
        path_obj = Path(path)

        # If it's a file, add directly
        if path_obj.is_file():
            if not any(
                fnmatch.fnmatch(path_obj.name, pattern) for pattern in exclude_patterns
            ):
                found_files.append(path_obj)
            continue

        # If it's a directory, walk recursively
        if path_obj.is_dir():
            for root, _, files in os.walk(path_obj):
                for file in files:
                    full_path = Path(root) / file
                    # Check against exclusion patterns
                    if not any(
                        fnmatch.fnmatch(file, pattern) for pattern in exclude_patterns
                    ):
                        found_files.append(full_path)

    return found_files


def generate_directory_structure(files):
    """
    Generates a text representation of the directory structure for the given files.

    Parameters:
        files (list of Path): List of file paths.

    Returns:
        list of str: Directory structure lines.
    """
    structure = []
    root = Path(".").resolve()
    for file in sorted(files):
        path = file.resolve()
        try:
            relative_path = path.relative_to(root)
        except ValueError:
            relative_path = path  # If the path is not relative, use the absolute path
        parts = list(relative_path.parts)
        for i in range(len(parts)):
            part = parts[: i + 1]
            line = "│   " * (len(part) - 1) + "├── " + part[-1]
            if line not in structure:
                structure.append(line)
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


def concatenate_files(files, output_file):
    """
    Concatenates the content of multiple files, adds directory structure and file type annotations.

    Parameters:
        files (list of Path): List of file paths to concatenate.
        output_file (str): The name of the output file.
    """
    try:
        with open(output_file, "w") as out_file:
            # Write the directory structure
            out_file.write("# Project Structure\n\n")
            out_file.write("├── ./\n")
            directory_structure = generate_directory_structure(files)
            for line in directory_structure:
                out_file.write(line + "\n")
            out_file.write("\n# File Contents\n\n")

            # Concatenate the contents of each file
            for file in sorted(files):
                try:
                    relative_path = file.relative_to(Path(".").resolve())
                except ValueError:
                    relative_path = file

                file_type = get_file_type(file)

                out_file.write(f"## {relative_path}\n\n")
                out_file.write(f"```{file_type}\n")

                try:
                    with open(file, "r") as f:
                        out_file.write(f.read())
                except Exception as e:
                    logging.error(f"Error reading file {file}: {e}")

                out_file.write("\n```\n\n")

        logging.info(f"Successfully created output file: {output_file}")
    except Exception as e:
        logging.error(f"Error concatenating files: {e}")


def copy_to_clipboard(output_file):
    """
    Copies the contents of the output file to the clipboard.

    Parameters:
        output_file (str): The name of the output file.
    """
    try:
        with open(output_file, "r") as f:
            output_content = f.read()

        system = platform.system()
        if system == "Linux":
            # Check for Wayland or X11
            if is_wayland():
                try:
                    subprocess.run(
                        ["wl-copy"], input=output_content.encode("utf-8"), check=True
                    )
                    click.echo("Copied using wl-copy (Wayland)")
                except FileNotFoundError:
                    try:
                        subprocess.run(
                            ["xclip", "-selection", "clipboard"],
                            input=output_content.encode("utf-8"),
                            check=True,
                        )
                        click.echo("Copied using xclip (X11)")
                    except FileNotFoundError:
                        click.echo(
                            "Neither wl-copy nor xclip found. Unable to copy to clipboard."
                        )
            else:
                try:
                    subprocess.run(
                        ["xclip", "-selection", "clipboard"],
                        input=output_content.encode("utf-8"),
                        check=True,
                    )
                    click.echo("Copied using xclip (X11)")
                except FileNotFoundError:
                    click.echo("xclip not found. Unable to copy to clipboard.")
        elif system == "Darwin":  # macOS
            subprocess.run(["pbcopy"], input=output_content.encode("utf-8"), check=True)
            click.echo("Copied using pbcopy (macOS)")
        elif system == "Windows":
            subprocess.run(["clip"], input=output_content.encode("utf-8"), check=True)
            click.echo("Copied using clip (Windows)")
        else:
            click.echo(f"Clipboard copy not supported on {system}.")

    except Exception as e:
        logging.error(f"Error copying contents to clipboard: {e}")

@click.command(help="Concatenate files with directory structure and content.")
@click.argument("paths", nargs=-1, type=click.Path(exists=True))
@click.option(
    "-o", "--output", default="output.md", help="Output file name (default: output.md)"
)
@click.option(
    "--copy/--no-copy",
    default=False,
    help="Copy the output file contents to the clipboard",
)
@click.option(
    "--exclude",
    multiple=True,
    help="Patterns to exclude from file search (e.g., *.pyc .git)",
)
def main(paths, output, copy, exclude):
    """
    Main function to execute file concatenation.

    Args:
        paths (tuple): Paths to files or directories to concatenate
        output (str): Output file name
        copy (bool): Whether to copy to clipboard
        exclude (tuple): Patterns to exclude
    """
    # Setup logging
    setup_logging("concatenate_files")

    # Find files with exclusion
    try:
        files = get_files_recursively(paths, list(exclude))

        # Concatenate files
        concatenate_files(files, output)

        # Copy to clipboard if requested
        if copy:
            copy_to_clipboard(output)
        elif click.confirm("Do you want to copy the contents to the clipboard?"):
            copy_to_clipboard(output)

    except Exception as e:
        logging.error(f"An error occurred: {e}")
        click.echo(f"Error: {e}")


if __name__ == "__main__":
    main()
