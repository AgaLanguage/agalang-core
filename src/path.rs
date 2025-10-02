use std::path::{Path, PathBuf};

pub fn absolute_path(path: &str) -> String {
  let path_buf = PathBuf::from(path);
  let canonicalize = path_buf.canonicalize();
  let path_str = if let Ok(canonicalized) = canonicalize {
    canonicalized
  } else {
    path_buf
  }
  .to_string_lossy()
  .to_string();
  if path_str.starts_with(r"\\?\") {
    path_str[4..].to_string()
  } else {
    path_str
  }
}
pub fn relative_path<'a>(absolute_path: &'a Path, base_path: &'a Path) -> Option<&'a Path> {
  absolute_path.strip_prefix(base_path).ok()
}
pub fn get_dir_path(file_path: &Path) -> Option<&Path> {
  file_path.parent()
}
pub fn path_buf_to_string(path: &PathBuf) -> String {
  path.to_string_lossy().to_string()
}
