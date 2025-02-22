use std::{cell::RefCell, rc::Rc};

use crate::{
  libraries::{self, Modules}, runtime::values::{
    self, complex, internal,
    traits::{AgalValuable as _, ToAgalValue as _}, AgalValue,
  }, OnError as _, ToResult as _
};

pub fn get_name() -> String {
  format!("Numero")
}
pub fn get_full_name(prefix: &str) -> String {
  format!("{}/{}", super::get_name(prefix), get_name())
}

pub fn get_sub_module(
  prefix: &str,
  args: &str,
  modules_manager: libraries::RefModules,
) -> values::DefaultRefAgalValue {
  let module_name = get_full_name(prefix);
  if modules_manager.has(&module_name) {
    return modules_manager.get(&module_name);
  }

  let mut hashmap = std::collections::HashMap::new();
  hashmap.insert(
    "aCadena".into(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::aCadena"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          arguments
            .get(0)
            .or_else(||Some(&this))
            .ok_or_else(||internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::TypeError,
              message: "Se esperaba un argumento".into(),
              stack: stack.clone(),
            })?
            .to_agal_number(stack.clone())?
            .to_agal_string(stack)?
            .to_result()
        }),
      }
      .to_ref_value(),
    },
  );

  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  modules_manager.add(&module_name, complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value())
}
