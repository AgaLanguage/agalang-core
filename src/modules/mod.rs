use crate::runtime::RefAgalValue;
mod print;

type EvalResult = Result<RefAgalValue, ()>;
pub fn get_module(name: &str) -> EvalResult {
  if name == ">pintar" {return Ok(print::get_module())}
  Err(())
}