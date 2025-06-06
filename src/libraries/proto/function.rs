use std::sync::{RwLock, Arc};

use crate::{
  functions_names, libraries, parser,
  runtime::values::{
    self,
    complex::{self, FUNCTION_CALL},
    internal,
    traits::{AgalValuable, ToAgalValue as _},
  },
};

pub fn get_name() -> String {
  format!("Funcion")
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
        name: format!("{module_name}::{}", functions_names::TO_AGAL_STRING),
        func: Arc::new(|arguments, stack, modules, this| {
          arguments
            .get(0)
            .or_else(|| Some(&this))
            .ok_or_else(|| internal::AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba un argumento".into(),
              stack: stack.clone(),
            })?
            .to_agal_string(stack, modules)?
            .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    FUNCTION_CALL.into(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::{FUNCTION_CALL}"),
        func: Arc::new(|arguments, stack, modules, real_this| {
          let this = arguments
            .get(0)
            .ok_or_else(|| internal::AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba un argumento".into(),
              stack: stack.clone(),
            })?
            .clone();
          let args = arguments
            .get(1)
            .ok_or_else(|| internal::AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba un argumento".into(),
              stack: stack.clone(),
            })?
            .to_agal_array(stack.clone(), modules.clone())?
            .get()
            .to_vec()
            .read().unwrap()
            .clone();
          real_this.call(stack, this, args, modules)
        }),
      }
      .to_ref_value(),
    },
  );

  let prototype = complex::AgalPrototype::new(Arc::new(RwLock::new(hashmap)), None);
  modules_manager.add(
    &module_name,
    complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value(),
  )
}
