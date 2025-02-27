use std::{cell::RefCell, rc::Rc};

use crate::{
  functions_names, libraries::{self, Modules}, parser, runtime::values::{
    self, complex, internal,
    traits::{AgalValuable as _, ToAgalValue as _},
    AgalValue,
  }
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
    functions_names::TO_AGAL_STRING.into(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::{}",functions_names::TO_AGAL_STRING),
        func: Rc::new(|arguments, stack, modules, this| {
          arguments
            .get(0)
            .or_else(|| Some(&this))
            .ok_or_else(|| internal::AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba un argumento".into(),
              stack: stack.clone(),
            })?
            .to_agal_number(stack.clone(), modules.clone())?
            .to_agal_string(stack, modules)?
            .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(functions_names::CALL.into(), complex::AgalClassProperty {
    is_public: true,
    is_static: true,
    value: internal::AgalNativeFunction {
      name: format!("{module_name}::{}",functions_names::CALL),
      func: Rc::new(|arguments, stack, modules, this| {
        arguments
          .get(0)
          .ok_or_else(|| internal::AgalThrow::Params {
            type_error: parser::ErrorNames::TypeError,
            message: "Se esperaba un argumento".into(),
            stack: stack.clone(),
          })?
          .to_agal_number(stack.clone(), modules.clone())
          .unwrap_or_default()
          .to_result()
      }),
    }
    .to_ref_value(),
  });

  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  modules_manager.add(
    &module_name,
    complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value(),
  )
}
