use std::net::TcpListener;
use std::sync::Arc;

use crate::compiler::{Function, NativeValue, Promise, Value};
use crate::interpreter::stack::CallFrame;
use crate::interpreter::vm::AsyncThread;
use crate::interpreter::VarsManager;
use crate::util::{OnError, OnSome};
use crate::MultiRefHash;

pub const LIB_NAME: &str = ":red";
const TCP_SERVER: &str = "ServidorTCP";
const TCP_SERVER_PROMISE: &str = "promesa";

const TCP_SOCKET: &str = "SocketTCP";
const TCP_SOCKET_WRITE: &str = "escribe";
const TCP_SOCKET_CLOSE: &str = "cierra";
const TCP_SOCKET_READ: &str = "lee";

const TCP_DATA_PORT: &str = "puerto";
const TCP_DATA_IP: &str = "ip";

fn handle_client(
  stream: std::net::TcpStream,
  socket: std::net::SocketAddr,
  callback: MultiRefHash<crate::compiler::Function>,
  module: MultiRefHash<crate::interpreter::ModuleThread>,
) {
  let data = crate::compiler::Instance::new(format!("<{LIB_NAME}>::{TCP_SOCKET}"));

  let stream: MultiRefHash<crate::compiler::NativeValue> = stream.into();

  let locals = VarsManager::crate_child(callback.read().get_scope().unwrap());
  let frame = CallFrame::new(callback, vec![locals.into()]);
  let (thread, promise) = AsyncThread::from_frame(frame);
  thread.write().set_module(module.clone());
  thread.write().print_on_error();

  data.set_instance_property(
    TCP_SOCKET_READ,
    Value::Object(crate::compiler::Object::Function(MultiRefHash::new(
      Function::Native {
        name: format!("<{TCP_SOCKET}>::{TCP_SOCKET_READ}"),
        path: format!("<{LIB_NAME}>::{TCP_SERVER}({TCP_SOCKET})"),
        chunk: Default::default(),
        func: |_, _, _, stream| {
          let mut binding = stream.write();
          let stream = binding
            .mut_tcp_stream()
            .on_error(|_| format!("{TCP_SOCKET_READ}: El socket TCP no esta abierto"))?;
          let mut buf = vec![0; 1024];
          use std::io::Read;
          match stream.read(&mut buf) {
            Ok(bytes_read) => {
              if bytes_read > 0 {
                let value = Value::Object(buf[..bytes_read].to_vec().into());
                Ok(value)
              } else {
                Err(format!("{TCP_SOCKET_READ}: No hay datos para leer"))
              }
            }
            Err(e) => Err(format!(
              "{TCP_SOCKET_READ}: Error al leer del socket: {}",
              e
            )),
          }
        },
        custom_data: stream.clone(),
      },
    ))),
    true,
  );
  data.set_instance_property(
    TCP_SOCKET_WRITE,
    Value::Object(crate::compiler::Object::Function(MultiRefHash::new(
      Function::Native {
        name: format!("<{TCP_SOCKET}>::{TCP_SOCKET_WRITE}"),
        path: format!("<{LIB_NAME}>::{TCP_SERVER}({TCP_SOCKET})"),
        chunk: Default::default(),
        func: |_, args, thread, stream| {
          let data = args
            .first()
            .on_some_option(|t| t.as_strict_buffer(thread).ok())
            .on_error(|_| {
              format!("{TCP_SOCKET_WRITE}: Se esperaba un valor buffer como primer argumento")
            })?;
          let mut binding = stream.write();
          let stream = binding
            .mut_tcp_stream()
            .on_error(|_| format!("{TCP_SOCKET_WRITE}: El socket TCP no esta abierto"))?;
          use std::io::Write;
          stream
            .write_all(&data)
            .on_error(|e| format!("{TCP_SOCKET_WRITE}: Error al escribir en el socket: {e}"))?;
          Ok(Value::Never)
        },
        custom_data: stream.clone(),
      },
    ))),
    true,
  );
  data.set_instance_property(
    TCP_SOCKET_CLOSE,
    Value::Object(crate::compiler::Object::Function(MultiRefHash::new(
      Function::Native {
        name: format!("<{TCP_SOCKET}>::{TCP_SOCKET_CLOSE}"),
        path: format!("<{LIB_NAME}>::{TCP_SERVER}({TCP_SOCKET})"),
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
  data.set_instance_property(TCP_DATA_IP, Value::String(socket.ip().to_string()), true);
  data.set_instance_property(TCP_DATA_PORT, Value::Number(socket.port().into()), true);

  thread
    .read()
    .push(Value::Object(crate::compiler::Object::Map(
      Default::default(),
      data.into(),
    )));

  module
    .read()
    .get_process_manager()
    .read()
    .push_interrupt_thread(thread);
}

pub fn lib_value() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    TCP_SERVER,
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{TCP_SERVER}"),
        path: format!("<{LIB_NAME}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, thread, _| {
          let addr = args
            .first()
            .on_some_option(|t| {
              if t.is_string() {
                Some(t.to_aga_string(thread))
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

          let listener = Arc::new(
            TcpListener::bind(&addr)
              .map_err(|e| format!("{TCP_SERVER}: No se pudo iniciar el servidor TCP: {}", e))?,
          );

          let clone_listener = Arc::clone(&listener);
          let module = thread.get_async().read().get_module();
          std::thread::spawn(move || loop {
            match clone_listener.accept() {
              Ok((stream, addr)) => handle_client(stream, addr, callback.clone(), module.clone()),
              Err(e) => {
                promise.set_err(format!("{TCP_SERVER}: Error al aceptar conexión: {}", e));
                break;
              }
            }
          });
          let data = crate::compiler::Instance::new(format!("<{LIB_NAME}>::{TCP_SERVER}"));
          data.set_instance_property(TCP_SERVER_PROMISE, value, true);
          let addr = listener
            .local_addr()
            .on_error(|e| format!("{TCP_SERVER}: Error al verificar el servidor: {}", e))?;
          data.set_instance_property(TCP_DATA_IP, Value::String(addr.ip().to_string()), true);
          data.set_instance_property(TCP_DATA_PORT, Value::Number(addr.port().into()), true);
          Ok(Value::Object(crate::compiler::Object::Map(
            Default::default(),
            data.into(),
          )))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  Value::Object(crate::compiler::Object::Map(
    Default::default(),
    hashmap.into(),
  ))
}
