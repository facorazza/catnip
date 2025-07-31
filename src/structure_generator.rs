use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum TreeNode {
    File,
    Directory(BTreeMap<String, TreeNode>),
}

pub fn generate_directory_structure(files: &[PathBuf]) -> Vec<String> {
    let mut structure = Vec::new();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Build tree structure
    let mut root = BTreeMap::new();

    for file in files {
        let relative_path = file.strip_prefix(&current_dir)
            .unwrap_or(file);

        add_to_tree(&mut root, relative_path);
    }

    // Generate structure recursively
    build_tree_lines(&root, &mut structure, "");

    structure
}

fn add_to_tree(tree: &mut BTreeMap<String, TreeNode>, path: &Path) {
    let components: Vec<_> = path.components().collect();

    if components.is_empty() {
        return;
    }

    add_components_to_tree(tree, &components, 0);
}

fn add_components_to_tree(
    tree: &mut BTreeMap<String, TreeNode>,
    components: &[std::path::Component],
    index: usize,
) {
    if index >= components.len() {
        return;
    }

    let component_name = components[index].as_os_str().to_string_lossy().to_string();

    if index == components.len() - 1 {
        // This is a file
        tree.insert(component_name, TreeNode::File);
    } else {
        // This is a directory
        let entry = tree.entry(component_name)
            .or_insert_with(|| TreeNode::Directory(BTreeMap::new()));

        if let TreeNode::Directory(ref mut subtree) = entry {
            add_components_to_tree(subtree, components, index + 1);
        }
    }
}

fn build_tree_lines(
    tree: &BTreeMap<String, TreeNode>,
    lines: &mut Vec<String>,
    prefix: &str,
) {
    let items: Vec<_> = tree.iter().collect();

    for (i, (name, node)) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };

        lines.push(format!("{}{}{}", prefix, connector, name));

        if let TreeNode::Directory(subtree) = node {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            build_tree_lines(subtree, lines, &new_prefix);
        }
    }
}
