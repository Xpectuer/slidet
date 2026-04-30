use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slide {
    pub path: PathBuf,
    pub title: String,
    pub raw_markdown: String,
}

// --- Tree types ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideNode {
    Leaf(Slide),
    Group {
        name: String,
        path: PathBuf,
        children: Vec<Slide>,
        expanded: bool,
    },
}

impl SlideNode {
    pub fn name(&self) -> &str {
        match self {
            SlideNode::Leaf(s) => &s.title,
            SlideNode::Group { name, .. } => name,
        }
    }

    pub fn resolve_slide<'a>(nodes: &'a [SlideNode], slide_ref: &SlideRef) -> &'a Slide {
        match slide_ref {
            SlideRef::Root(i) => {
                if let SlideNode::Leaf(s) = &nodes[*i] {
                    s
                } else {
                    panic!("Root SlideRef pointed to a Group")
                }
            }
            SlideRef::InGroup {
                group_index,
                child_index,
            } => {
                if let SlideNode::Group { children, .. } = &nodes[*group_index] {
                    &children[*child_index]
                } else {
                    panic!("InGroup SlideRef pointed to a Leaf")
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SlideRef {
    Root(usize),
    InGroup {
        group_index: usize,
        child_index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct VisibleItem {
    pub kind: VisibleItemKind,
    pub depth: u8,
    pub group_index: usize,
    pub slide_ref: Option<SlideRef>,
}

#[derive(Debug, Clone)]
pub enum VisibleItemKind {
    RootLeaf {
        slide_index: usize,
    },
    Group {
        name: String,
        expanded: bool,
        child_count: usize,
    },
    GroupChild {
        child_index: usize,
    },
}

pub fn compute_visible_items(nodes: &[SlideNode]) -> Vec<VisibleItem> {
    let mut items = Vec::new();
    for (group_index, node) in nodes.iter().enumerate() {
        match node {
            SlideNode::Leaf(_) => {
                items.push(VisibleItem {
                    kind: VisibleItemKind::RootLeaf {
                        slide_index: group_index,
                    },
                    depth: 0,
                    group_index,
                    slide_ref: Some(SlideRef::Root(group_index)),
                });
            }
            SlideNode::Group {
                name,
                children,
                expanded,
                ..
            } => {
                items.push(VisibleItem {
                    kind: VisibleItemKind::Group {
                        name: name.clone(),
                        expanded: *expanded,
                        child_count: children.len(),
                    },
                    depth: 0,
                    group_index,
                    slide_ref: None,
                });
                if *expanded {
                    for child_index in 0..children.len() {
                        items.push(VisibleItem {
                            kind: VisibleItemKind::GroupChild { child_index },
                            depth: 1,
                            group_index,
                            slide_ref: Some(SlideRef::InGroup {
                                group_index,
                                child_index,
                            }),
                        });
                    }
                }
            }
        }
    }
    items
}

pub fn compute_flat_refs(nodes: &[SlideNode]) -> Vec<SlideRef> {
    let mut refs = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        match node {
            SlideNode::Leaf(_) => {
                refs.push(SlideRef::Root(i));
            }
            SlideNode::Group { children, .. } => {
                for j in 0..children.len() {
                    refs.push(SlideRef::InGroup {
                        group_index: i,
                        child_index: j,
                    });
                }
            }
        }
    }
    refs
}

pub fn load_slides(dir: &Path) -> Result<Vec<SlideNode>> {
    if !dir.exists() {
        bail!("slides directory does not exist: {}", dir.display());
    }
    if !dir.is_dir() {
        bail!("slides path is not a directory: {}", dir.display());
    }

    let entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to enumerate {}", dir.display()))?;

    let mut nodes: Vec<SlideNode> = Vec::new();

    for entry in &entries {
        if entry.is_dir() {
            // Scan one level of subdirectory
            let subdir_entries: Vec<_> = fs::read_dir(entry)
                .with_context(|| format!("failed to read directory {}", entry.display()))?
                .map(|e| e.map(|e| e.path()))
                .collect::<std::result::Result<Vec<_>, _>>()
                .with_context(|| format!("failed to enumerate {}", entry.display()))?;

            // Fail-fast: check for nested subdirectories containing .md files
            for subdir_entry in &subdir_entries {
                if subdir_entry.is_dir() {
                    let has_md = fs::read_dir(subdir_entry)
                        .with_context(|| {
                            format!("failed to read directory {}", subdir_entry.display())
                        })?
                        .any(|e| {
                            e.map(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
                                .unwrap_or(false)
                        });
                    if has_md {
                        bail!(
                            "nested subdirectories not supported (max 2 levels): {} contains .md files inside {}",
                            subdir_entry.display(),
                            entry.display()
                        );
                    }
                }
            }

            // Collect .md files from subdirectory
            let mut md_files: Vec<_> = subdir_entries
                .iter()
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
                .cloned()
                .collect();
            if md_files.is_empty() {
                continue;
            }
            md_files.sort();

            let children: Vec<Slide> = md_files
                .into_iter()
                .map(|path| {
                    let raw_markdown = fs::read_to_string(&path)
                        .with_context(|| format!("failed to read {}", path.display()))?;
                    let title = path
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().into_owned())
                        .unwrap_or_else(|| String::from("untitled"));
                    Ok(Slide {
                        path,
                        title,
                        raw_markdown,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            let name = entry
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| String::from("unknown"));

            nodes.push(SlideNode::Group {
                name,
                path: entry.clone(),
                children,
                expanded: false,
            });
        } else if entry.extension().and_then(|s| s.to_str()) == Some("md") {
            let raw_markdown = fs::read_to_string(entry)
                .with_context(|| format!("failed to read {}", entry.display()))?;
            let title = entry
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| String::from("untitled"));
            nodes.push(SlideNode::Leaf(Slide {
                path: entry.clone(),
                title,
                raw_markdown,
            }));
        }
    }

    // Sort root-level nodes by name (file stem for leaves, dir name for groups)
    nodes.sort_by(|a, b| a.name().cmp(b.name()));

    if nodes.is_empty() {
        bail!("no markdown slides found in {}", dir.display());
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::{compute_visible_items, load_slides, SlideNode};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("slidet-{label}-{nanos}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn load_slides_sorts_markdown_files_by_name() {
        let dir = TempDir::new("loader-order");
        fs::write(dir.path().join("02-agenda.md"), "## Agenda").unwrap();
        fs::write(dir.path().join("01-intro.md"), "# Intro").unwrap();
        fs::write(dir.path().join("notes.txt"), "ignored").unwrap();

        let nodes = load_slides(dir.path()).unwrap();

        assert_eq!(nodes.len(), 2);
        assert!(matches!(&nodes[0], SlideNode::Leaf(s) if s.title == "01-intro"));
        assert!(matches!(&nodes[1], SlideNode::Leaf(s) if s.title == "02-agenda"));
    }

    #[test]
    fn load_slides_errors_for_missing_directory() {
        let err = load_slides(Path::new("/definitely/missing/slides"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("slides directory does not exist"));
    }

    #[test]
    fn load_slides_errors_for_non_directory_paths() {
        let dir = TempDir::new("loader-file");
        let file = dir.path().join("slides.md");
        fs::write(&file, "# Slide").unwrap();

        let err = load_slides(&file).unwrap_err().to_string();
        assert!(err.contains("slides path is not a directory"));
    }

    #[test]
    fn load_slides_errors_for_empty_directory() {
        let dir = TempDir::new("loader-empty");

        let err = load_slides(dir.path()).unwrap_err().to_string();
        assert!(err.contains("no markdown slides found"));
    }

    #[test]
    fn load_slides_collects_subdirectory_into_group() {
        let dir = TempDir::new("loader-group");
        fs::write(dir.path().join("00-intro.md"), "# Intro").unwrap();
        let subdir = dir.path().join("01-topic");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("01-a.md"), "# A").unwrap();
        fs::write(subdir.join("02-b.md"), "# B").unwrap();
        fs::write(dir.path().join("02-summary.md"), "# Summary").unwrap();

        let nodes = load_slides(dir.path()).unwrap();

        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[0], SlideNode::Leaf(s) if s.title == "00-intro"));
        assert!(
            matches!(&nodes[1], SlideNode::Group { name, children, .. } if name == "01-topic" && children.len() == 2)
        );
        assert!(matches!(&nodes[2], SlideNode::Leaf(s) if s.title == "02-summary"));
    }

    #[test]
    fn load_slides_rejects_nested_subdirectories_with_md() {
        let dir = TempDir::new("loader-nested");
        let subdir = dir.path().join("01-topic");
        fs::create_dir_all(&subdir).unwrap();
        let subsubdir = subdir.join("02-deep");
        fs::create_dir_all(&subsubdir).unwrap();
        fs::write(subsubdir.join("01.md"), "# Deep").unwrap();

        let err = load_slides(dir.path()).unwrap_err().to_string();
        assert!(err.contains("nested subdirectories not supported"));
    }

    #[test]
    fn load_slides_ignores_empty_subdirectories() {
        let dir = TempDir::new("loader-empty-sub");
        fs::write(dir.path().join("01-slide.md"), "# Slide").unwrap();
        let empty = dir.path().join("assets");
        fs::create_dir_all(&empty).unwrap();

        let nodes = load_slides(dir.path()).unwrap();

        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], SlideNode::Leaf(s) if s.title == "01-slide"));
    }

    #[test]
    fn compute_visible_items_collapse_hides_children() {
        let dir = TempDir::new("vis-collapse");
        let nodes = vec![
            SlideNode::Leaf(super::Slide {
                path: dir.path().join("01.md"),
                title: "01".into(),
                raw_markdown: "# One".into(),
            }),
            SlideNode::Group {
                name: "group".into(),
                path: dir.path().join("group"),
                children: vec![
                    super::Slide {
                        path: dir.path().join("group/a.md"),
                        title: "a".into(),
                        raw_markdown: "# A".into(),
                    },
                    super::Slide {
                        path: dir.path().join("group/b.md"),
                        title: "b".into(),
                        raw_markdown: "# B".into(),
                    },
                ],
                expanded: false,
            },
        ];

        let visible = compute_visible_items(&nodes);
        assert_eq!(visible.len(), 2); // leaf + group header (no children)
    }

    #[test]
    fn compute_visible_items_expand_shows_children() {
        let dir = TempDir::new("vis-expand");
        let nodes = vec![SlideNode::Group {
            name: "group".into(),
            path: dir.path().join("group"),
            children: vec![super::Slide {
                path: dir.path().join("group/a.md"),
                title: "a".into(),
                raw_markdown: "# A".into(),
            }],
            expanded: true,
        }];

        let visible = compute_visible_items(&nodes);
        assert_eq!(visible.len(), 2); // group header + 1 child
        assert_eq!(visible[1].depth, 1);
    }
}
