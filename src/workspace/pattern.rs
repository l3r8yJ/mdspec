use crate::diagnostics::Diagnostic;
use globset::{GlobBuilder, GlobMatcher};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct Pattern {
    pub root: PathBuf,
    pub display_root: PathBuf,
    matcher: GlobMatcher,
}

impl Pattern {
    pub fn compile(path: &Path) -> Result<Self, Box<Diagnostic>> {
        if path.exists() {
            return Self::literal(path);
        }
        let prefix: PathBuf = path
            .components()
            .take_while(|part| !has_magic(Path::new(part.as_os_str())))
            .collect();
        let root = if has_magic(path) {
            prefix.as_path()
        } else {
            path.parent().unwrap_or_else(|| Path::new(""))
        };
        let suffix = path
            .strip_prefix(root)
            .map_err(|error| invalid(path, error))?;
        if suffix.components().any(|part| part == Component::ParentDir) {
            return Err(invalid(path, "Parent traversal after a wildcard is not supported").into());
        }
        let text = suffix
            .to_str()
            .ok_or_else(|| invalid(path, "Glob patterns must be valid UTF-8"))?;
        Self::build(path, root, text)
    }

    fn literal(path: &Path) -> Result<Self, Box<Diagnostic>> {
        let normalized = normalize(path).map_err(|error| invalid(path, error))?;
        let parent = normalized.parent().unwrap_or(&normalized);
        let name = normalized
            .strip_prefix(parent)
            .map_err(|error| invalid(path, error))?;
        let text = name
            .to_str()
            .ok_or_else(|| invalid(path, "Exclusion patterns must be valid UTF-8"))?;
        Self::build(path, parent, &globset::escape(text))
    }

    fn build(path: &Path, root: &Path, text: &str) -> Result<Self, Box<Diagnostic>> {
        let matcher = GlobBuilder::new(text)
            .literal_separator(true)
            .build()
            .map_err(|error| invalid(path, error))?
            .compile_matcher();
        let root = if root.as_os_str().is_empty() {
            Path::new(".")
        } else {
            root
        };
        let root_path = fs::canonicalize(root)
            .or_else(|_| std::path::absolute(root))
            .map_err(|error| invalid(path, error))?;
        Ok(Self {
            root: root_path,
            display_root: root.to_path_buf(),
            matcher,
        })
    }

    pub fn matches(&self, path: &Path) -> bool {
        path.strip_prefix(&self.root)
            .is_ok_and(|suffix| self.matcher.is_match(suffix))
    }
}

pub fn has_magic(path: &Path) -> bool {
    path.as_os_str()
        .as_encoded_bytes()
        .iter()
        .any(|byte| matches!(byte, b'*' | b'?' | b'[' | b'{' | b'\\'))
}

pub fn normalize(path: &Path) -> std::io::Result<PathBuf> {
    if path.is_dir() {
        return fs::canonicalize(path);
    }
    let absolute = std::path::absolute(path)?;
    let parent = absolute.parent().unwrap_or(&absolute);
    let name = absolute
        .strip_prefix(parent)
        .map_err(std::io::Error::other)?;
    Ok(fs::canonicalize(parent)?.join(name))
}

fn invalid(path: &Path, error: impl std::fmt::Display) -> Diagnostic {
    Diagnostic::error(
        "MDS901",
        path,
        None,
        format!("Invalid selection pattern: {error}"),
    )
}
