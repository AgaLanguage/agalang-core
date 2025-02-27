use crate::runtime::values::{
  self, complex,
  traits::{AgalValuable as _, ToAgalValue as _},
};
use std::{cell::RefCell, rc::Rc};

mod array;
mod number;
mod string;

pub fn get_name(prefix: &str) -> String {
  format!("{}{}", prefix, "proto")
}
pub fn get_dir_module(
  prefix: &str,
  args: &str,
  modules_manager: super::RefModules,
) -> values::DefaultRefAgalValue {
  if array::get_name() == args {
    return array::get_sub_module(prefix, args, modules_manager);
  }
  if number::get_name() == args {
    return number::get_sub_module(prefix, args, modules_manager);
  }
  if string::get_name() == args {
    return string::get_sub_module(prefix, args, modules_manager);
  }
  let mut module_name = get_name(prefix);

  let mut hashmap = std::collections::HashMap::new();
  hashmap.insert(
    number::get_name(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: number::get_sub_module(prefix, args, modules_manager.clone()),
    },
  );
  hashmap.insert(
    array::get_name(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: array::get_sub_module(prefix, args, modules_manager.clone()),
    },
  );
  hashmap.insert(
    array::get_name(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: string::get_sub_module(prefix, args, modules_manager.clone()),
    },
  );
  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  modules_manager.add(
    &module_name,
    complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value(),
  )
}
