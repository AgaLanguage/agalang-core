use crate::{
  compiler::{ChunkGroup, Function, Object, Value}, OnError
};

const TYPE: &str = "<function>";

const CALL: &str = "llama";

pub fn prototype() -> Value {
  let hashmap = crate::compiler::Instance::new(TYPE.to_string());

  hashmap.set_instance_property(
    CALL,
    Value::Object(
      Function::Native {
        path: "".to_string(),
        name: format!("{TYPE}::{CALL}"),
        func: |funtion, mut args, thread, _| {
          let this = args
            .first()
            .on_error(|_| format!("{CALL}: se esperaba minimo 1 argumento y se recibieron 0"))?
            .clone();
          args.remove(0);
          let fun = match funtion {
            Value::Object(Object::Function(fun)) => fun,
            // Para llegar a este punto debes haber llamado esta funcion desde un tipo diferente a 
            _ => unreachable!(),
          };
          // Nos abstenemos de dar un valor propio haciendo que el valor que se da es el de la funcion llamada
          let _ = thread.call_function(this, fun, args);
          Ok(thread.pop())
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
