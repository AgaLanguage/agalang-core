use parser::util::RefValue;

use crate::{runtime::values, Modules};
mod fs;
mod print;

type EvalResult = Result<values::DefaultRefAgalValue, ()>;

pub const PREFIX_NATIVE_MODULES: &str = ":";

pub fn get_module(key: &str, modules_manager: RefValue<Modules>) -> EvalResult {
  if modules_manager.borrow().has(key) {
    let v = modules_manager.borrow().get(key);
    return Ok(v);
  }
  if key == print::get_name(PREFIX_NATIVE_MODULES) {
    let value = print::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.borrow().add(key, value.clone());
    return Ok(value);
  }
  if key == fs::get_name(PREFIX_NATIVE_MODULES) {
    let value = fs::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.borrow().add(key, value.clone());
    return Ok(value);
  }
  Err(())
}
