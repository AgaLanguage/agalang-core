use crate::{runtime::RefAgalValue, Modules};
mod print;
mod fs;

type EvalResult = Result<RefAgalValue, ()>;

pub const PREFIX_NATIVE_MODULES: &str = ":";

pub fn get_module(key: &str, modules_manager: &Modules) -> EvalResult {
  if modules_manager.has(key) {
    let v = modules_manager.get(key);
    return Ok(v);
  }
  if key == print::get_name(PREFIX_NATIVE_MODULES) {
    let value = print::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.add(key, value.clone());
    return Ok(value);
  }
  if key == fs::get_name(PREFIX_NATIVE_MODULES) {
    let value = fs::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.add(key, value.clone());
    return Ok(value);
  }
  Err(())
}