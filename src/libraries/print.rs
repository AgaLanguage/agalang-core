use std::{cell::RefCell, rc::Rc};

use crate::runtime::values::{
  self, complex, internal,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalValue,
};

pub fn get_module(prefix: &str) -> values::DefaultRefAgalValue {
  let mut module_name = get_name(prefix);

  let mut hashmap = std::collections::HashMap::new();

  hashmap.insert(
    "pintar".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::pintar"),
        func: Rc::new(|arguments, stack, env, modules_manager, this| {
          for arg in arguments {
            let data = arg.to_agal_console(stack.clone(), env.clone())?;
            print!("{} ", data.to_string());
          }
          print!("\n");
          Ok(AgalValue::Never.as_ref())
        }),
      }
      .to_ref_value(),
    },
  );
  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
  format!("{}{}", prefix, "consola")
}
