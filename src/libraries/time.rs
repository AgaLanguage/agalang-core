use std::{cell::RefCell, rc::Rc};

use crate::runtime::values::{
  self,
  complex::{self, AgalPromise},
  internal,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalValue,
};

pub fn get_module(prefix: &str) -> values::DefaultRefAgalValue {
  let mut module_name = get_name(prefix);

  let mut hashmap = std::collections::HashMap::new();

  hashmap.insert(
    "esperar".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::esperar"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let arg_clone = arguments.clone();
          AgalPromise::new(Box::pin(async move {
            let secs = if let Some(value) = arg_clone.get(0) {
              value.to_agal_number(stack)?.to_float()
            } else {
              0f32
            };
            tokio::time::sleep(std::time::Duration::from_secs_f32(secs)).await;
            AgalValue::Never.to_result()
          }))
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
  format!("{}{}", prefix, "tmp")
}
