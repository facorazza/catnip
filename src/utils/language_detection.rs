use std::path::Path;

pub fn get_language_from_extension(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => "rust",
        Some("py") | Some("pyw") => "python",
        Some("js") | Some("mjs") => "javascript",
        Some("ts") => "typescript",
        Some("tsx") | Some("jsx") => "jsx",
        Some("java") => "java",
        Some("kt") => "kotlin",
        Some("scala") => "scala",
        Some("clj") => "clojure",
        Some("c") => "c",
        Some("cpp") | Some("cc") | Some("cxx") => "cpp",
        Some("h") | Some("hpp") => "c",
        Some("cs") => "csharp",
        Some("fs") => "fsharp",
        Some("vb") => "vbnet",
        Some("php") => "php",
        Some("rb") => "ruby",
        Some("go") => "go",
        Some("swift") => "swift",
        Some("m") | Some("mm") => "objc",
        Some("dart") => "dart",
        Some("lua") => "lua",
        Some("pl") => "perl",
        Some("r") | Some("R") => "r",
        Some("html") | Some("htm") => "html",
        Some("css") => "css",
        Some("scss") => "scss",
        Some("sass") => "sass",
        Some("less") => "less",
        Some("vue") => "vue",
        Some("svelte") => "svelte",
        Some("json") | Some("jsonc") => "json",
        Some("yaml") | Some("yml") => "yaml",
        Some("toml") => "toml",
        Some("xml") => "xml",
        Some("sql") => "sql",
        Some("sh") | Some("bash") => "bash",
        Some("zsh") => "zsh",
        Some("fish") => "fish",
        Some("ps1") => "powershell",
        Some("bat") | Some("cmd") => "batch",
        Some("tf") => "hcl",
        Some("dockerfile") => "dockerfile",
        Some("md") | Some("markdown") => "markdown",
        Some("tex") => "latex",
        Some("cmake") => "cmake",
        _ => {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                match filename {
                    "Makefile" | "makefile" => "makefile",
                    "Dockerfile" => "dockerfile",
                    "Jenkinsfile" => "groovy",
                    _ => "text",
                }
            } else {
                "text"
            }
        }
    }
}
