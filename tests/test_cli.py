import pytest
from pathlib import Path
from unittest.mock import patch, mock_open, MagicMock
from quick_cat.cli import (
    is_wayland,
    get_files_recursively,
    generate_directory_structure,
    get_file_type,
    concatenate_files,
    copy_to_clipboard,
)


def test_is_wayland():
    with patch.dict("os.environ", {"WAYLAND_DISPLAY": "wayland-0"}):
        assert is_wayland() is True

    with patch.dict("os.environ", {}, clear=True):
        assert is_wayland() is False


def test_get_files_recursively(tmp_path):
    # Create test directory structure
    test_dir = tmp_path / "test_dir"
    test_dir.mkdir()
    (test_dir / "file1.txt").touch()
    (test_dir / "file2.py").touch()
    (test_dir / "__pycache__").mkdir()
    (test_dir / "__pycache__/cache.pyc").touch()

    files = get_files_recursively([test_dir])
    assert len(files) == 2
    assert any(f.name == "file1.txt" for f in files)
    assert any(f.name == "file2.py" for f in files)


def test_generate_directory_structure():
    files = [Path("dir1/file1.txt"), Path("dir1/subdir/file2.py"), Path("dir2/file3.js")]

    structure = generate_directory_structure(files)
    assert len(structure) == 6
    assert "├── dir1" in structure
    assert "│   ├── file1.txt" in structure
    assert "│   └── subdir" in structure
    assert "│       └── file2.py" in structure
    assert "└── dir2" in structure
    assert "    └── file3.js" in structure


def test_get_file_type():
    assert get_file_type(Path("test.py")) == "python"
    assert get_file_type(Path("test.js")) == "javascript"
    assert get_file_type(Path("test.unknown")) == ""


def test_concatenate_files(tmp_path):
    # Create test files
    file1 = tmp_path / "test1.txt"
    file2 = tmp_path / "test2.py"

    file1.write_text("content1")
    file2.write_text("content2")

    output_file = tmp_path / "output.md"
    concatenate_files([file1, file2], str(output_file))

    assert output_file.exists()
    content = output_file.read_text()
    assert "content1" in content
    assert "content2" in content


@pytest.mark.parametrize(
    "platform,command", [("Linux", "xclip"), ("Darwin", "pbcopy"), ("Windows", "clip")]
)
def test_copy_to_clipboard(platform, command):
    mock_content = "test content"

    with (
        patch("platform.system", return_value=platform),
        patch("builtins.open", mock_open(read_data=mock_content)),
        patch("subprocess.run") as mock_run,
        patch("quick_cat.cli.is_wayland", return_value=False),
    ):

        copy_to_clipboard("test.txt")
        mock_run.assert_called_once()

        # Verify command was called with correct input
        call_args = mock_run.call_args[0][0]
        if platform == "Linux":
            assert call_args == ["xclip", "-selection", "clipboard"]
        else:
            assert call_args == [command]


def test_copy_to_clipboard_wayland():
    with (
        patch("platform.system", return_value="Linux"),
        patch("quick_cat.cli.is_wayland", return_value=True),
        patch("subprocess.run") as mock_run,
        patch("builtins.open", mock_open(read_data="test content")),
    ):

        copy_to_clipboard("test.txt")
        mock_run.assert_called_once()
        assert mock_run.call_args[0][0] == ["wl-copy"]
