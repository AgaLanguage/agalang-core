use crate::{
  compiler::{ChunkGroup, Function, Object, Value},
  OnError,
};

const TYPE: &str = "<function>";

const CALL: &str = "llamar";

pub fn prototype() -> Value {
  let hashmap = crate::compiler::Instance::new(TYPE.to_string());

  hashmap.set_instance_property(
    CALL,
    Value::Object(
      Function::Native {
        path: "".to_string(),
        name: format!("{TYPE}::{CALL}"),
        func: |funtion, mut args, thread, _| {
          let this = args.first().on_error(|_|format!(
            "{CALL}: se esperaba minimo 1 argumento y se recibieron 0"
          ))?.clone();
          args.remove(0);
          let fun = match funtion {
            Value::Object(Object::Function(fun)) => fun,
            _ => unreachable!(),
          };
          thread.call_function(this, fun, args)?;
          
          Ok(Value::Never)
        },
        chunk: ChunkGroup::default().into(),
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );

  Value::Object(Object::Map(Default::default(), hashmap.into()))
}
