use regex::Regex;

pub fn remove_comments_and_docstrings(
    content: &str,
    language: &str,
    ignore_comments: bool,
    ignore_docstrings: bool,
) -> String {
    if !ignore_comments && !ignore_docstrings {
        return content.to_string();
    }

    let mut result = content.to_string();

    if ignore_comments || ignore_docstrings {
        match language {
            "rust" | "javascript" | "typescript" | "java" | "kotlin" | "scala" | "c" | "cpp"
            | "csharp" | "go" | "swift" | "dart" => {
                if ignore_comments {
                    let re = Regex::new(r"//.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();

                    let re = Regex::new(r"/\*.*?\*/").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            "python" => {
                if ignore_comments {
                    let re = Regex::new(r"#.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
                if ignore_docstrings {
                    let re = Regex::new(r#"""".*?""""#).unwrap();
                    result = re.replace_all(&result, "").to_string();
                    let re = Regex::new(r"'''.*?'''").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            "ruby" | "bash" | "sh" | "zsh" | "fish" => {
                if ignore_comments {
                    let re = Regex::new(r"#.*$").unwrap();
                    result = re.replace_all(&result, "").to_string();
                }
            }
            _ => {}
        }
    }

    result
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
