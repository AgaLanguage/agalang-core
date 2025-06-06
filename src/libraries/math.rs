use std::sync::{Arc, RwLock};

use crate::{
  parser,
  runtime::values::{
    self,
    complex::{self, AgalPromise},
    internal::{self, AgalThrow},
    primitive::AgalNumber,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
  util::OnError as _,
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
        func: Arc::new(|arguments, stack, modules, this| {
          arguments
            .get(0)
            .on_error(|_| AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba un número".into(),
              stack: stack.clone(),
            })?
            .to_agal_number(stack, modules)?
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
        func: Arc::new(|arguments, stack, modules, this| {
          let mut val = AgalNumber::Infinity;
          for argument in arguments {
            let n = argument.to_agal_number(stack.clone(), modules.clone())?;
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
        func: Arc::new(|arguments, stack, modules, this| {
          let mut val = AgalNumber::NegInfinity;
          for argument in arguments {
            let n = argument.to_agal_number(stack.clone(), modules.clone())?;
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
  let prototype = complex::AgalPrototype::new(Arc::new(RwLock::new(hashmap)), None);
  complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
  format!("{}{}", prefix, "mate")
}
