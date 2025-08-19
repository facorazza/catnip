use anyhow::Result;
use std::process::Command;
use tracing::{debug, info};

#[derive(Debug)]
enum ClipboardType {
    Wayland,
    X11,
    MacOS,
    Windows,
    Unsupported,
}

fn detect_clipboard_system() -> ClipboardType {
    if cfg!(target_os = "windows") {
        return ClipboardType::Windows;
    }

    if cfg!(target_os = "macos") {
        return ClipboardType::MacOS;
    }

    // For Linux/Unix systems
    if std::env::var("WAYLAND_DISPLAY").is_ok() && command_exists("wl-copy") {
        return ClipboardType::Wayland;
    }

    if std::env::var("DISPLAY").is_ok() && command_exists("xclip") {
        return ClipboardType::X11;
    }

    ClipboardType::Unsupported
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

async fn copy_to_clipboard_native(content: &str) -> Result<()> {
    let clipboard_type = detect_clipboard_system();
    debug!("Detected clipboard system: {:?}", clipboard_type);

    let (cmd, args): (&str, Vec<&str>) = match clipboard_type {
        ClipboardType::Wayland => ("wl-copy", vec![]),
        ClipboardType::X11 => ("xclip", vec!["-selection", "clipboard"]),
        ClipboardType::MacOS => ("pbcopy", vec![]),
        ClipboardType::Windows => ("clip", vec![]),
        ClipboardType::Unsupported => {
            return Err(anyhow::anyhow!(
                "No supported clipboard system found. Install:\n\
                - Wayland: wl-clipboard\n\
                - X11: xclip\n\
                - Or use --output to save to file"
            ));
        }
    };

    let mut child = Command::new(cmd)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn {}: {}", cmd, e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write to {} stdin: {}", cmd, e))?;
    }

    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("Failed to wait for {}: {}", cmd, e))?;

    if !status.success() {
        return Err(anyhow::anyhow!("{} failed with status: {}", cmd, status));
    }

    info!("Content copied to clipboard using {}", cmd);
    println!("Content copied to clipboard");
    Ok(())
}

pub async fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Copying {} characters to clipboard", content.len());
    copy_to_clipboard_native(content).await
}

pub async fn read_from_clipboard() -> Result<String> {
    let clipboard_type = detect_clipboard_system();
    debug!("Reading from clipboard using: {:?}", clipboard_type);

    let (cmd, args): (&str, Vec<&str>) = match clipboard_type {
        ClipboardType::Wayland => ("wl-paste", vec![]),
        ClipboardType::X11 => ("xclip", vec!["-selection", "clipboard", "-o"]),
        ClipboardType::MacOS => ("pbpaste", vec![]),
        ClipboardType::Windows => ("powershell", vec!["-command", "Get-Clipboard"]),
        ClipboardType::Unsupported => {
            return Err(anyhow::anyhow!(
                "No supported clipboard system found. Install:\n\
                - Wayland: wl-clipboard\n\
                - X11: xclip\n\
                - Or provide a JSON file path"
            ));
        }
    };

    let output = Command::new(cmd)
        .args(&args)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run {}: {}", cmd, e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "{} failed with status: {}",
            cmd,
            output.status
        ));
    }

    let content = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in clipboard content: {}", e))?;

    if content.trim().is_empty() {
        return Err(anyhow::anyhow!("Clipboard is empty"));
    }

    info!("Read {} characters from clipboard", content.len());
    Ok(content)
}
