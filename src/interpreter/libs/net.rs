use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Arc;

use crate::compiler::{Function, NativeValue, Promise, Value};
use crate::interpreter::stack::CallFrame;
use crate::interpreter::vm::AsyncThread;
use crate::interpreter::VarsManager;
use crate::util::{OnError, OnSome};
use crate::MultiRefHash;

pub const NET_LIB: &str = ":red";
const TCP_SERVER: &str = "servidorTCP";
const TCP_DATA: &str = "DatosTCP";

const TCP_DATA_WRITE: &str = "escribe";
const TCP_DATA_CLOSE: &str = "cierra";
const TCP_DATA_READ: &str = "lee";

fn handle_client(
  stream: std::net::TcpStream,
  callback: MultiRefHash<crate::compiler::Function>,
  module: MultiRefHash<crate::interpreter::ModuleThread>,
) {
  let data = crate::compiler::Instance::new(format!("<{NET_LIB}>::{TCP_DATA}"));

  let stream: MultiRefHash<crate::compiler::NativeValue> = stream.into();

  let locals = VarsManager::crate_child(callback.read().get_scope().unwrap());
  let frame = CallFrame::new(callback, vec![locals.into()]);
  let (thread, promise) = AsyncThread::from_frame(frame);
  thread.write().set_module(module.clone());

  data.set_instance_property(
    TCP_DATA_READ,
    Value::Object(crate::compiler::Object::Function(MultiRefHash::new(
      Function::Native {
        name: format!("<{TCP_DATA}>::{TCP_DATA_READ}"),
        path: format!("<{NET_LIB}>::{TCP_SERVER}({TCP_DATA})"),
        chunk: Default::default(),
        func: |_, _, _, stream| {
          let mut binding = stream.write();
          let stream = binding
            .mut_tcp_stream()
            .on_error(|_| format!("{TCP_DATA_READ}: El socket TCP no esta abierto"))?;
          let mut buf = vec![0; 1024];
          use std::io::Read;
          match stream.read(&mut buf) {
            Ok(bytes_read) => {
              if bytes_read > 0 {
                let value = Value::Object(buf[..bytes_read].to_vec().into());
                Ok(value)
              } else {
                Err(format!("{TCP_DATA_READ}: No hay datos para leer").into())
              }
            }
            Err(e) => Err(format!("{TCP_DATA_READ}: Error al leer del socket: {}", e).into()),
          }
        },
        custom_data: stream.clone(),
      },
    ))),
    true,
  );
  data.set_instance_property(
    TCP_DATA_WRITE,
    Value::Object(crate::compiler::Object::Function(MultiRefHash::new(
      Function::Native {
        name: format!("<{TCP_DATA}>::{TCP_DATA_WRITE}"),
        path: format!("<{NET_LIB}>::{TCP_SERVER}({TCP_DATA})"),
        chunk: Default::default(),
        func: |_, args, thread, stream| {
          let data = args
            .get(0)
            .on_some_option(|t| t.as_strict_buffer(thread).ok())
            .on_error(|_| {
              format!("{TCP_DATA_WRITE}: Se esperaba un valor buffer como primer argumento")
            })?;
          let mut binding = stream.write();
          let stream = binding
            .mut_tcp_stream()
            .on_error(|_| format!("{TCP_DATA_WRITE}: El socket TCP no esta abierto"))?;
          use std::io::Write;
          stream
            .write_all(&data)
            .on_error(|e| format!("{TCP_DATA_WRITE}: Error al escribir en el socket: {e}"))?;
          Ok(Value::Never)
        },
        custom_data: stream.clone(),
      },
    ))),
    true,
  );
  data.set_instance_property(
    TCP_DATA_CLOSE,
    Value::Object(crate::compiler::Object::Function(MultiRefHash::new(
      Function::Native {
        name: format!("<{TCP_DATA}>::{TCP_DATA_CLOSE}"),
        path: format!("<{NET_LIB}>::{TCP_SERVER}({TCP_DATA})"),
        chunk: Default::default(),
        func: |_, _, _, values| {
          *values.read().get_value().unwrap().write() = NativeValue::None;
          values
            .write()
            .get_promise()
            .unwrap()
            .set_value(Value::Never);
          Ok(Value::Never)
        },
        custom_data: (stream, promise).into(),
      },
    ))),
    true,
  );

  thread
    .read()
    .push(Value::Object(crate::compiler::Object::Map(
      HashMap::new().into(),
      data.into(),
    )));

  module
    .read()
    .get_process_manager()
    .read()
    .push_interrupt_thread(thread);
}

pub fn net_lib() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{NET_LIB}>"));

  hashmap.set_instance_property(
    TCP_SERVER,
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{NET_LIB}>::{TCP_SERVER}"),
        path: format!("<{NET_LIB}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, thread, _| {
          let addr = args
            .get(0)
            .on_some_option(|t| {
              if t.is_string() {
                Some(t.as_string())
              } else {
                None
              }
            })
            .on_error(|_| format!("{TCP_SERVER}: Se esperaba un acceso como primer argumento"))?;
          let callback = args
            .get(1)
            .on_some_option(|t| {
              if t.is_function() {
                Some(t.as_function())
              } else {
                None
              }
            })
            .on_error(|_| {
              format!("{TCP_SERVER}: Se esperaba una función como segundo argumento")
            })?;

          let promise = Promise::new();
          let value = Value::Promise(promise.clone());

          let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("{TCP_SERVER}: No se pudo iniciar servidor TCP: {}", e))?;

          let module = thread.get_async().read().get_module();
          std::thread::spawn(move || {
            let listener = Arc::new(listener);
            loop {
              match listener.accept() {
                Ok((stream, _addr)) => handle_client(stream, callback.clone(), module.clone()),
                Err(e) => {
                  promise.set_err(format!("{TCP_SERVER}: Error al aceptar conexión: {}", e));
                  break;
                }
              }
            }
          });
          Ok(value)
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  Value::Object(crate::compiler::Object::Map(
    HashMap::new().into(),
    hashmap.into(),
  ))
}
