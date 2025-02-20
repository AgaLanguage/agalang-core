use std::{cell::RefCell, rc::Rc};

use crate::{
  runtime::values::{
    self,
    complex::{self, AgalPromise},
    internal::{self, AgalThrow},
    primitive::AgalNumber,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
  OnError, ToResult,
};

pub fn get_module(prefix: &str) -> values::DefaultRefAgalValue {
  let mut module_name = get_name(prefix);

  let mut hashmap = std::collections::HashMap::new();

  hashmap.insert(
    "suelo".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::suelo"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          arguments
            .get(0)
            .to_result()
            .on_error(AgalThrow::Params {
              type_error: parser::internal::ErrorNames::TypeError,
              message: "Se esperaba un número".into(),
              stack: stack.clone(),
            })?
            .to_agal_number(stack)?
            .to_agal_int()
            .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "min".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::min"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let mut val = AgalNumber::Infinity;
          for argument in arguments {
            let n = argument.to_agal_number(stack.clone())?;
            if n.less_than(&val) {
              val = n;
            }
          }
          val.to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "max".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::max"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let mut val = AgalNumber::NegInfinity;
          for argument in arguments {
            let n = argument.to_agal_number(stack.clone())?;
            if val.less_than(&n) {
              val = n;
            }
          }
          val.to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
  format!("{}{}", prefix, "mate")
}
