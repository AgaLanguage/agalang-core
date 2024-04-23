pub fn to_cyan(s: &str) -> String {
  format!("\x1b[96m{}\x1b[0m", s)
}
pub fn split_meta(meta: &str) -> (&str, &str, &str) {
  let mut meta = meta.split("\0");
  let file_name = meta.next().unwrap();
  let data = meta.next();
  if data == None {
      return (file_name, "", "")
  }
  let mut meta = data.unwrap().split("\0");
  let line = meta.next().unwrap();
  let token = meta.next();
  if token == None {
      return (file_name, line, "")
  }
  let token = token.unwrap();
  (file_name, line, token)
}