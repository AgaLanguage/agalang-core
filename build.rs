fn main() {
  println!(
    "cargo:rustc-link-arg-bin={}=icon.res",
    env!("CARGO_PKG_NAME")
  );
}
