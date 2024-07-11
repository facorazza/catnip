# quick-cat.py

import os
import argparse
import logging
import platform
import subprocess
from pathlib import Path

"""
This script concatenates multiple files, adds a directory structure, and formats the content for markdown.

Purpose:
    The main goal of this script is to provide an organized and easy-to-read markdown document that includes
    multiple code files along with their directory structure. This is particularly useful for sharing with
    a Large Language Model (LLM) or other collaborators to quickly bring them up to speed with what you are
    working on. By providing both the content and the structure, it becomes easier to understand the context
    and relationships between the files.

Usage example:
    python quick-cat.py app.py ./source/config.py /static/js/javascript.js /templates/index.html -o combined_output.md

    This command concatenates file1.py and file2.js into a markdown file named combined_output.md, including a directory
    structure and syntax highlighting for each file's content.

Arguments:
    files (list of str): List of file paths to concatenate.
    -o, --output (str): Output file name (default: output.md).

Output:
    The script produces a markdown file with the following features:
    - A visual representation of the directory structure of the input files.
    - Concatenated contents of each file, with appropriate markdown syntax highlighting based on file type.
"""

# Sets up logging
def setup_logging(script_name):
    """Setup logging configuration to output logs to a file and console."""
    log_dir = './logs'
    os.makedirs(log_dir, exist_ok=True)
    log_file = os.path.join(log_dir, f'{script_name}.log')

    logging.basicConfig(level=logging.DEBUG,
                        format='%(asctime)s - %(levelname)s - %(message)s',
                        handlers=[
                            logging.FileHandler(log_file),
                            logging.StreamHandler()
                        ])

# Directory structure
def generate_directory_structure(files):
    """
    Generates a text representation of the directory structure for the given files.

    Parameters:
        files (list of str): List of file paths.

    Returns:
        list of str: Directory structure lines.
    """
    structure = []
    root = Path('.').resolve()
    for file in files:
        path = Path(file).resolve()
        try:
            relative_path = path.relative_to(root)
        except ValueError:
            relative_path = path  # If the path is not relative, use the absolute path
        parts = list(relative_path.parts)
        for i in range(len(parts)):
            part = parts[:i + 1]
            line = '│   ' * (len(part) - 1) + '├── ' + part[-1]
            if line not in structure:
                structure.append(line)
    return structure

# File types
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
        '.py': 'python',
        '.js': 'javascript',
        '.html': 'html',
        '.css': 'css',
        '.json': 'json',
        '.log': '',
        '.txt': '',
        '.md': 'markdown',
        '.xml': 'xml',
        '.yaml': 'yaml',
        '.yml': 'yaml',
        '.sh': 'shell',
        '.c': 'c',
        '.cpp': 'cpp',
        '.java': 'java',
        '.php': 'php',
        '.rb': 'ruby',
        '.go': 'go',
        '.swift': 'swift',
        '.rs': 'rust',
        '.pl': 'perl',
        '.ps1': 'powershell',
        '.bat': 'batch',
        '.vbs': 'vbscript',
        '.ini': 'ini',
        '.toml': 'toml',
        '.csv': 'csv',
        '.tsv': 'tsv',
        '.rst': 'rst',
        '.tex': 'tex',
        '.org': 'org',
        '.jsx': 'jsx',
        '.tsx': 'tsx'
    }
    return file_types.get(ext, '')

# Concatenation
def concatenate_files(files, output_file):
    """
    Concatenates the content of multiple files, adds directory structure and file type annotations.

    Parameters:
        files (list of str): List of file paths to concatenate.
        output_file (str): The name of the output file.
    """
    with open(output_file, 'w') as out_file:
        # Write the LLM prompt
        out_file.write("You are a super helpful coding assistant!\n\n")
        out_file.write("Below you will find relevant files for a project I am working on. Please analyze these different files and confirm that you understand their purpose. Do not offer updates or show me code at this time. If you do not understand something, please ask me questions for clarification.\n\n")
        out_file.write("\n# When providing code help, please adhere to the following guidelines\n")
        out_file.write("\n1. **Provide Code in Sections**: Do not provide whole files. Instead, provide specific sections, functions, or lines as needed.\n")
        out_file.write("\n2. **Example for Line Changes**:\n")
        out_file.write("    - Change this line:\n")
        out_file.write("\n      ```python\n")
        out_file.write("      Old line here\n")
        out_file.write("      ```\n")
        out_file.write("\n    - To this:\n")
        out_file.write("\n      ```python\n")
        out_file.write("      New line here\n")
        out_file.write("      ```\n")
        out_file.write("\n**Note**: When providing markdown code boxes, escape internal backticks by using triple backslashes before the backticks.\n")
        out_file.write("\n## Style Guide for Writing Tools\n")
        out_file.write("\n### 1. **Introduction**\n")
        out_file.write("\n- Purpose of the Style Guide\n")
        out_file.write("- Importance of Consistency\n")
        out_file.write("\n## 2. **Script Header**\n")
        out_file.write("\n- **Script Name Comment**: Include the name of the script at the top as a comment. This helps identify the log and output files.\n")
        out_file.write("\n  ```python\n")
        out_file.write("  # script_name.py\n")
        out_file.write("  ```\n")
        out_file.write("\n- **Summary Comment**: Include a brief summary of the script's purpose and functionality in comments at the top, after the filename and a line break.\n")
        out_file.write("\n  ```python\n")
        out_file.write("  # script_name.py\n")
        out_file.write("\n  # This script combines multiple text files into a single text file.\n")
        out_file.write("  # It prompts the user for the directory containing the text files,\n")
        out_file.write("  # reads each file, and writes their contents into an output file.\n")
        out_file.write("  ```\n")
        out_file.write("\n## 3. **Imports**\n")
        out_file.write("\n- **Grouping Imports**: Group imports in the following order: standard library imports, related third-party imports, local application/library-specific imports.\n")
        out_file.write("- **Absolute Imports**: Use absolute imports when possible.\n")
        out_file.write("\n## 4. **Configuration Management**\n")
        out_file.write("\n- **Configuration Files**: Use configuration files for storing settings that might change frequently. This can help make your scripts more flexible and easier to manage.\n")
        out_file.write("\n  ```python\n")
        out_file.write("  import json\n")
        out_file.write("\n  def load_config(config_file):\n")
        out_file.write("      \"\"\"Load configuration settings from a file.\"\"\"\n")
        out_file.write("      with open(config_file, 'r') as f:\n")
        out_file.write("          config = json.load(f)\n")
        out_file.write("      return config\n")
        out_file.write("  ```\n")
        out_file.write("\n## 5. **Function Definitions**\n")
        out_file.write("\n- **Comment Titles**: Provide a concise title for every function as a comment above the function. Example:\n")
        out_file.write("\n  ```python\n")
        out_file.write("  # Data processing\n")
        out_file.write("  def process_data(data):\n")
        out_file.write("    \"\"\"\n")
        out_file.write("    Process the given data.\n")
        out_file.write("\n    Parameters:\n")
        out_file.write("        data (list): A list of data items to be processed.\n")
        out_file.write("\n    Returns:\n")
        out_file.write("        list: The processed data.\n")
        out_file.write("    \"\"\"\n")
        out_file.write("    print(f\"Processing data: {data}\")\n")
        out_file.write("    processed_data = [item * 2 for item in data]\n")
        out_file.write("    print(\"Data processing complete.\")\n")
        out_file.write("    return processed_data\n")
        out_file.write("  ```\n")
        out_file.write("\n- **Function Docstrings**: Provide a clear explanation of the function's purpose, parameters, and return values. Example:\n")
        out_file.write("\n  ```python\n")
        out_file.write("  # Example function\n")
        out_file.write("  def example_function(param1, param2):\n")
        out_file.write("      \"\"\"\n")
        out_file.write("      Brief description of the function.\n")
        out_file.write("\n      Parameters:\n")
        out_file.write("          param1 (type): Description of param1.\n")
        out_file.write("          param2 (type): Description of param2.\n")
        out_file.write("      \n")
        out_file.write("      Returns:\n")
        out_file.write("          return_type: Description of the return value.\n")
        out_file.write("      \"\"\"\n")
        out_file.write("  ```\n")
        out_file.write("\n- **Modular Code**: Break down tasks into small, reusable functions.\n")
        out_file.write("\n## 6. **Code Formatting**\n")
        out_file.write("\n- **Indentation**: Use 4 spaces for indentation.\n")
        out_file.write("- **Line Length**: Limit all lines to a maximum of 79 characters.\n")
        out_file.write("- **Blank Lines**: Use blank lines to separate top-level function and class definitions.\n")
        out_file.write("- **String Quotes**: In general, use single quotes for short strings and double quotes for longer strings or when a string contains a single quote.\n")
        out_file.write("- **Whitespace in Expressions and Statements**:\n")
        out_file.write("  - Avoid extraneous whitespace in the following situations: immediately inside parentheses, brackets or braces; immediately before a comma, semicolon, or colon; immediately before the open parenthesis that starts the argument list of a function call.\n")
        out_file.write("  - Use a single space around binary operators and after a comma.\n")
        out_file.write("- **Naming Conventions**:\n")
        out_file.write("  - Use `CamelCase` for class names.\n")
        out_file.write("  - Use `lower_case_with_underscores` for functions and variable names.\n")
        out_file.write("  - Use `UPPER_CASE_WITH_UNDERSCORES` for constants.\n")
        out_file.write("\n## 7. **Commenting**\n")
        out_file.write("\n- **Inline Comments**: Use comments to explain complex logic or important sections of code.\n")
        out_file.write("\n  ```python\n")
        out_file.write("  # This loop processes each file in the directory\n")
        out_file.write("  for file in files:\n")
        out_file.write("      ...\n")
        out_file.write("  ```\n")
        out_file.write("\n## 8. **Logging**\n")
        out_file.write("\n- **Setup Logging**: Configure logging to output to both the console and a log file in the `./logs` directory. The log file should be named according to the script name.\n")
        out_file.write("\n  ```python\n")
        out_file.write("  def setup_logging(script_name):\n")
        out_file.write("      \"\"\"Setup logging configuration.\"\"\"\n")
        out_file.write("      log_dir = './logs'\n")
        out_file.write("      os.makedirs(log_dir, exist_ok=True)\n")
        out_file.write("      log_file = os.path.join(log_dir, f'{script_name}.log')\n")
        out_file.write("\n      logging.basicConfig(level=logging.DEBUG,\n")
        out_file.write("                          format='%(asctime)s - %(levelname)s - %(message)s',\n")
        out_file.write("                          handlers=[\n")
        out_file.write("                              logging.FileHandler(log_file),\n")
        out_file.write("                              logging.StreamHandler()\n")
        out_file.write("                          ])\n")
        out_file.write("  ```\n")
        out_file.write("\n- **Logging Levels**: Use appropriate logging levels (`DEBUG`, `INFO`, `ERROR`) to provide detailed and useful log messages.\n")
        out_file.write("\n    ```python\n")
        out_file.write("    logging.debug(f\"Processing file: {file_path}\")\n")
        out_file.write("    logging.info(\"Operation completed successfully.\")\n")
        out_file.write("    logging.error(f\"Error processing file: {e}\")\n")
        out_file.write("    ```\n")
        out_file.write("\n## 9. **Error Handling**\n")
        out_file.write("\n- **Try-Except Blocks**: Use try-except blocks to handle potential errors gracefully.\n")
        out_file.write("\n    ```python\n")
        out_file.write("    try:\n")
        out_file.write("        # code that may raise an exception\n")
        out_file.write("    except Exception as e:\n")
        out_file.write("        logging.error(f\"An error occurred: {e}\")\n")
        out_file.write("    ```\n")
        out_file.write("\n- **Using Whole New Functions**: Implement whole new functions within try-except blocks to keep the code modular and maintainable.\n")
        out_file.write("\n## 10. **Testing and Debugging**\n")
        out_file.write("\n- **Unit Tests**: Write unit tests for your functions to ensure they work correctly.\n")
        out_file.write("\n  ```python\n")
        out_file.write("  import unittest\n")
        out_file.write("\n  class TestExampleFunction(unittest.TestCase):\n")
        out_file.write("      def test_example_function(self):\n")
        out_file.write("          self.assertEqual(example_function(param1, param2), expected_result)\n")
        out_file.write("\n  if __name__ == '__main__':\n")
        out_file.write("      unittest.main()\n")
        out_file.write("  ```\n")
        out_file.write("\n## 12. **PEP 8 Compliance**\n")
        out_file.write("\n- Summary of Key Points\n")
        out_file.write("  - **Indentation**: Use 4 spaces per indentation level.\n")
        out_file.write("  - **Line Length**: Limit all lines to a maximum of 79 characters.\n")
        out_file.write("  - **Blank Lines**: Use blank lines to separate top-level function and class definitions.\n")
        out_file.write("  - **Imports**:\n")
        out_file.write("    - Imports should usually be on separate lines.\n")
        out_file.write("    - Group imports in the following order: standard library imports, related third-party imports, local application/library-specific imports.\n")
        out_file.write("    - Use absolute imports when possible.\n")
        out_file.write("  - **String Quotes**: In general, use single quotes for short strings and double quotes for longer strings or when a string contains a single quote.\n")
        out_file.write("  - **Whitespace in Expressions and Statements**:\n")
        out_file.write("    - Avoid extraneous whitespace in the following situations: immediately inside parentheses, brackets or braces; immediately before a comma, semicolon, or colon; immediately before the open parenthesis that starts the argument list of a function call.\n")
        out_file.write("    - Use a single space around binary operators and after a comma.\n")
        out_file.write("  - **Naming Conventions**:\n")
        out_file.write("    - Use `CamelCase` for class names.\n")
        out_file.write("    - Use `lower_case_with_underscores` for functions and variable names.\n")
        out_file.write("    - Use `UPPER_CASE_WITH_UNDERSCORES` for constants.\n")
        out_file.write("\nRemember to follow the guidelines provided in the style guide to maintain consistency and readability in the code.\n")

        
        # Write the directory structure
        out_file.write('├── ./\n')
        directory_structure = generate_directory_structure(files)
        for line in directory_structure:
            out_file.write(line + '\n')
        out_file.write('\n')
        
        # Concatenate the contents of each file
        for file in files:
            path = Path(file).resolve()
            try:
                relative_path = path.relative_to(Path('.').resolve())
            except ValueError:
                relative_path = path  # If the path is not relative, use the absolute path
            file_type = get_file_type(path)
            
            out_file.write(f'./{relative_path}\n\n')
            out_file.write(f'```{file_type}\n')
            
            try:
                with open(file, 'r') as f:
                    out_file.write(f.read())
            except Exception as e:
                logging.error(f"Error reading file {file}: {e}")
            
            out_file.write('\n```\n\n')

# Copy contents to clipboard
def copy_to_clipboard(output_file):
    """
    Copies the contents of the output file to the clipboard.

    Parameters:
        output_file (str): The name of the output file.
    """
    try:
        with open(output_file, 'r') as f:
            output_content = f.read()
        
        system = platform.system()
        if system == 'Linux':
            subprocess.run(['xclip', '-selection', 'clipboard'], input=output_content.encode('utf-8'))
        elif system == 'Darwin':  # macOS
            subprocess.run(['pbcopy'], input=output_content.encode('utf-8'))
        elif system == 'Windows':
            subprocess.run(['clip'], input=output_content.encode('utf-8'))
        else:
            print(f"Clipboard copy not supported on {system}.")
            
        print("Contents copied to clipboard.")
    except Exception as e:
        logging.error(f"Error copying contents to clipboard: {e}")

# Main function
def main():
    """
    Main function to parse arguments and execute file concatenation.
    """
    usage_example = """\
    Usage example:
        python quick-cat.py app.py ./source/config.py /static/js/javascript.js /templates/index.html -o my_files_combined.md --copy
    """
    description = "Concatenate files with directory structure and content.\n\n" + usage_example

    parser = argparse.ArgumentParser(description=description, formatter_class=argparse.RawTextHelpFormatter)
    parser.add_argument('files', nargs='+', help='List of files to concatenate')
    parser.add_argument('-o', '--output', default='output.md', help='Output file name (default: output.md)')
    parser.add_argument('--copy', action='store_true', help='Copy the output file contents to the clipboard')
    
    args = parser.parse_args()
    
    setup_logging('concatenate_files')
    
    concatenate_files(args.files, args.output)
    logging.info("File concatenation completed successfully.")
    
    # Ask the user if they want to copy the contents to the clipboard
    if args.copy or input("Do you want to copy the contents to the clipboard? (yes/no): ").strip().lower() in ['yes', 'y', '']:
        copy_to_clipboard(args.output)


if __name__ == '__main__':
    main()