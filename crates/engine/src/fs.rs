//! Reading the file system: folder listings, sub-folders for the tree, and the default Favorites.

use std::path::{Path, PathBuf};

use crate::model::{Asset, AssetKind};

/// Stops a pathological folder from exhausting memory; Bridge-sized folders are far below this.
const MAX_ENTRIES: usize = 200_000;

pub fn list_folder(path: &Path) -> Result<Vec<Asset>, String> {
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for entry in entries.flatten().take(MAX_ENTRIES) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        // Follows symbolic links for the kind and size, like Finder; a dangling link lists as Other.
        let meta = std::fs::metadata(&path).ok();
        let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());
        let ext = if is_dir { String::new() } else { path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default() };
        let kind = if is_dir { AssetKind::Folder } else { AssetKind::from_extension(&ext) };
        out.push(Asset {
            hidden: is_hidden(&name),
            size: meta.as_ref().map_or(0, |m| if is_dir { 0 } else { m.len() }),
            modified: meta.as_ref().and_then(|m| m.modified().ok()),
            created: meta.as_ref().and_then(|m| m.created().ok()),
            path,
            name,
            ext,
            kind,
        });
    }
    Ok(out)
}

/// Visible sub-folders, sorted by name, for the Folders panel.
pub fn list_subfolders(path: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut out: Vec<PathBuf> =
        entries.flatten().take(MAX_ENTRIES).filter(|e| !is_hidden(&e.file_name().to_string_lossy())).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    out.sort_by_key(|p| display_name(p).to_lowercase());
    Ok(out)
}

pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.') || name.eq_ignore_ascii_case("desktop.ini") || name.eq_ignore_ascii_case("thumbs.db")
}

/// The last path component, or the whole path for a root (`/`, `C:\`).
pub fn display_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string_lossy().into_owned())
}

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).filter(|p| p.is_dir())
}

/// The root of the "Computer" Favorite.
pub fn computer_root() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/Volumes")
    } else if cfg!(windows) {
        PathBuf::from("C:\\")
    } else {
        PathBuf::from("/")
    }
}

/// Bridge's default Favorites, in its order, that exist on this computer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Computer,
    Home,
    Desktop,
    Documents,
    Pictures,
    Downloads,
}

pub fn default_places() -> Vec<(Place, PathBuf)> {
    let mut out = vec![(Place::Computer, computer_root())];
    if let Some(home) = home_dir() {
        for (place, sub) in [(Place::Desktop, "Desktop"), (Place::Documents, "Documents"), (Place::Pictures, "Pictures"), (Place::Downloads, "Downloads")] {
            let p = home.join(sub);
            if p.is_dir() {
                out.push((place, p));
            }
        }
        out.insert(1, (Place::Home, home));
    }
    out
}

/// Ancestors from the root down to `path`, for the path bar.
pub fn breadcrumbs(path: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = path.ancestors().map(Path::to_path_buf).collect();
    v.reverse();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_files_and_folders_with_kinds() {
        let dir = std::env::temp_dir().join(format!("pier-fs-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.JPG"), b"x").unwrap();
        std::fs::write(dir.join(".hidden"), b"x").unwrap();
        let mut list = list_folder(&dir).unwrap();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        let kinds: Vec<_> = list.iter().map(|a| (a.name.as_str(), a.kind, a.hidden)).collect();
        assert_eq!(kinds, vec![(".hidden", AssetKind::Other, true), ("a.JPG", AssetKind::Image, false), ("sub", AssetKind::Folder, false)]);
        assert_eq!(list_subfolders(&dir).unwrap(), vec![dir.join("sub")]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_folder_is_an_error_not_a_panic() {
        assert!(list_folder(Path::new("/definitely/not/here/pier")).is_err());
    }

    #[test]
    fn breadcrumbs_run_root_first() {
        let b = breadcrumbs(Path::new("/a/b"));
        assert_eq!(b, vec![PathBuf::from("/"), PathBuf::from("/a"), PathBuf::from("/a/b")]);
    }
}
