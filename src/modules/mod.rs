use crate::runtime::RefAgalValue;
mod print;
mod fs;

type EvalResult = Result<RefAgalValue, ()>;

pub const PREFIX_NATIVE_MODULES: &str = ":";

pub fn get_module(name: &str) -> EvalResult {
  if name == print::get_name(PREFIX_NATIVE_MODULES) {return Ok(print::get_module())}
  if name == fs::get_name(PREFIX_NATIVE_MODULES) {return Ok(fs::get_module())}
  Err(())
}