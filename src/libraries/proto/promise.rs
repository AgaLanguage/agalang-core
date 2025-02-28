use std::{cell::RefCell, rc::Rc};

use crate::{
  functions_names, libraries, parser,
  runtime::values::{
    self, complex, internal,
    traits::{AgalValuable, ToAgalValue},
  },
};

pub fn get_name() -> String {
  format!("Promesa")
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
        func: Rc::new(|arguments, stack, modules, this| {
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
    complex::PROMISE_THEN.into(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::{}", complex::PROMISE_THEN),
        func: Rc::new(|arguments, stack, modules, this| {
          complex::AgalPromise::new(Box::pin(async move {
            let cb = arguments
              .get(0)
              .ok_or_else(|| internal::AgalThrow::Params {
                type_error: parser::ErrorNames::TypeError,
                message: "Se esperaba un argumento".into(),
                stack: stack.clone(),
              })?;
            let value = if let values::AgalValue::Complex(c) = this.un_ref() {
              Some(c.un_ref())
            } else {
              None
            };
            let value = if let Some(complex::AgalComplex::Promise(p)) = value {
              Some(p)
            } else {
              None
            };
            let value = if let Some(promise) = value {
              promise
            } else {
              return Ok(this);
            };
            let mut value = value.borrow_mut();
            if let complex::AgalPromiseData::Resolved(r) = &value.data {
              return r.clone();
            }
            let value = if let complex::AgalPromiseData::Unresolved(future) = std::mem::replace(
              &mut value.data,
              complex::AgalPromiseData::Resolved(values::AgalValue::Never.to_result()),
            ) {
              let agal_value = future.await;
              value.data = complex::AgalPromiseData::Resolved(agal_value.clone());
              agal_value?
            } else {
              values::AgalValue::Never.as_ref()
            };
            cb.clone().call(stack, this, vec![value], modules)
          }))
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    complex::PROMISE_CATCH.into(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::{}", complex::PROMISE_CATCH),
        func: Rc::new(|arguments, stack, modules, this| {
          complex::AgalPromise::new(Box::pin(async move {
            let cb = arguments
              .get(0)
              .ok_or_else(|| internal::AgalThrow::Params {
                type_error: parser::ErrorNames::TypeError,
                message: "Se esperaba un argumento".into(),
                stack: stack.clone(),
              })?;
            let value = if let values::AgalValue::Complex(c) = this.un_ref() {
              Some(c.un_ref())
            } else {
              None
            };
            let value = if let Some(complex::AgalComplex::Promise(p)) = value {
              Some(p)
            } else {
              None
            };
            let value = if let Some(promise) = value {
              promise
            } else {
              return Ok(this);
            };
            let mut value = value.borrow_mut();
            if let complex::AgalPromiseData::Resolved(r) = &value.data {
              return r.clone();
            }
            if let complex::AgalPromiseData::Unresolved(future) = std::mem::replace(
              &mut value.data,
              complex::AgalPromiseData::Resolved(values::AgalValue::Never.to_result()),
            ) {
              let agal_value = future.await;
              value.data = complex::AgalPromiseData::Resolved(agal_value.clone());
              match agal_value {
                Err(e) => cb
                  .clone()
                  .call(stack, this, vec![e.to_error().to_ref_value()], modules),
                agal_value => agal_value,
              }
            } else {
              values::AgalValue::Never.to_result()
            }
          }))
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );

  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  modules_manager.add(
    &module_name,
    complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value(),
  )
}
