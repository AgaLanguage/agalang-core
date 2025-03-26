use std::{
  future::Future,
  pin::Pin,
  sync::{Arc, RwLock},
};

use futures_util::FutureExt;

use crate::{
  functions_names, libraries, parser,
  runtime::{
    values::{
      self, complex, internal,
      traits::{AgalValuable, ToAgalValue},
      DefaultRefAgalValue, ResultAgalValue,
    },
    RefStack,
  },
};

pub fn get_name() -> String {
  format!("Promesa")
}
pub fn get_full_name(prefix: &str) -> String {
  format!("{}/{}", super::get_name(prefix), get_name())
}

fn then(
  arguments: Vec<DefaultRefAgalValue>,
  stack: RefStack,
  modules: libraries::RefModules,
  this: DefaultRefAgalValue,
) -> Pin<Box<dyn Future<Output = ResultAgalValue> + Send>> {
  let cb = arguments.get(0).ok_or_else(|| internal::AgalThrow::Params {
    type_error: parser::ErrorNames::TypeError,
    message: "Se esperaba un argumento".into(),
    stack: stack.clone(),
  });
  if let Err(e) = cb {
    return async { Err(e) }.boxed();
  }
  let cb = cb.unwrap();
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
    return async move { Ok(this) }.boxed();
  };
  let value = value.as_arc();
  let guard = value.read().unwrap();
  if let complex::AgalPromiseData::Resolved(r) = &guard.data {
    let resolved_value = r.clone();
    return async move { resolved_value }.boxed();
  }
  let value_clone = value.clone();
  let callback = cb.clone();
  async move {
    let agal_value = std::mem::replace(
      &mut value_clone.write().unwrap().data,
      complex::AgalPromiseData::Resolved(values::AgalValue::Never.to_result()),
    )
    .resolve();
    let agal_value = agal_value.await;
    value_clone.write().unwrap().data = complex::AgalPromiseData::Resolved(agal_value.clone());
    match agal_value {
      Ok(value) => callback.clone().call(stack, this, vec![value], modules),
      error => error,
    }
  }
  .boxed()
}

fn catch(
  arguments: Vec<DefaultRefAgalValue>,
  stack: RefStack,
  modules: libraries::RefModules,
  this: DefaultRefAgalValue,
) -> Pin<Box<dyn Future<Output = ResultAgalValue> + Send>> {
  let cb = arguments.get(0).ok_or_else(|| internal::AgalThrow::Params {
    type_error: parser::ErrorNames::TypeError,
    message: "Se esperaba un argumento".into(),
    stack: stack.clone(),
  });
  if let Err(e) = cb {
    return async { Err(e) }.boxed();
  }
  let cb = cb.unwrap();
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
    return async move { Ok(this) }.boxed();
  };
  let value = value.as_arc();
  let guard = value.read().unwrap();
  if let complex::AgalPromiseData::Resolved(r) = &guard.data {
    let resolved_value = r.clone();
    return async move { resolved_value }.boxed();
  }
  let value_clone = value.clone();
  let callback = cb.clone();
  async move {
    let agal_value = std::mem::replace(
      &mut value_clone.write().unwrap().data,
      complex::AgalPromiseData::Resolved(values::AgalValue::Never.to_result()),
    )
    .resolve();
    let agal_value = agal_value.await;
    value_clone.write().unwrap().data = complex::AgalPromiseData::Resolved(agal_value.clone());
    match agal_value {
      Err(error) => callback.clone().call(stack, this, vec![error.to_error().to_ref_value()], modules),
      value => value,
    }
  }
  .boxed()
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
    complex::PROMISE_THEN.into(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::{}", complex::PROMISE_THEN),
        func: Arc::new(|arguments, stack, modules, this| {
          complex::AgalPromise::new(then(arguments, stack, modules, this)).to_result()
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
        func: Arc::new(|arguments, stack, modules, this| {
          complex::AgalPromise::new(catch(arguments, stack, modules, this))
          .to_result()
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
