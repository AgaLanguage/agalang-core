fn main() {
  #[cfg(target_os = "windows")]
  println!(
    "cargo:rustc-link-arg-bin={}=icon.res",
    env!("CARGO_PKG_NAME")
  );

  println!("cargo:rustc-check-cfg=cfg(gnu_time_bits64)");
}
