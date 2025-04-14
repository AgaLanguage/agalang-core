use std::{
  io::{Read as _, Write as _},
  sync::Arc,
};

use futures_util::{lock::Mutex, FutureExt};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt as _},
  net::{TcpListener, TcpStream, UdpSocket},
  sync::RwLock,
};

use crate::{
  parser,
  runtime::{
    await_result, values::{
      self,
      complex::{self, AgalArray, AgalObject, AgalPromise, AgalPrototype},
      internal::{self, AgalNativeFunction, AgalThrow},
      primitive::{AgalNumber, AgalString},
      traits::{self, AgalValuable, ToAgalValue},
      AgalValue, DefaultRefAgalValue, ResultAgalValue,
    }, RefStack, Stack
  },
  util::OnError as _,
};

use super::RefModules;

async fn handle_client(
  mut stream: TcpStream,
  callback: &DefaultRefAgalValue,
  stack: RefStack,
  modules: RefModules,
) -> Result<(), AgalThrow> {
  let mut tcp_proto = std::collections::HashMap::new();
  let arc_stream_ = Arc::new(RwLock::new(stream));
  let arc_stream = arc_stream_.clone();
  tcp_proto.insert(
    "leer".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: false,
      value: AgalNativeFunction {
        name: "leer".to_string(),
        func: Arc::new(move |arguments, stack, modules, this| {
          let arc_stream = arc_stream.clone();
          AgalPromise::new(
            async move {
              let mut buf = vec![0; 1024];
              let mut total_data = Vec::new();

              let bytes_read = arc_stream
                .write()
                .await
                .read(&mut buf)
                .await
                .on_error(|_| AgalThrow::Params {
                  type_error: parser::ErrorNames::TypeError,
                  message: "Error al leer del buffer".into(),
                  stack: stack.clone(),
                })?;

              total_data.extend_from_slice(&buf[..bytes_read]);

              AgalArray::from(&total_data).to_result()
            }
            .boxed(),
          )
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );

  let arc_stream = arc_stream_.clone();
  tcp_proto.insert(
    "escribir".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: false,
      value: AgalNativeFunction {
        name: "escribir".to_string(),
        func: Arc::new(move |arguments, stack, modules, this| {
          let arc_stream = arc_stream.clone();
          AgalPromise::new(
            async move {
              let buf = arguments
                .get(0)
                .on_error(|_| AgalThrow::Params {
                  type_error: parser::ErrorNames::TypeError,
                  message: "Se esperaba un argumento".into(),
                  stack: stack.clone(),
                })?
                .to_agal_array(stack.clone(), modules.clone())?
                .un_ref()
                .to_buffer(stack.clone(), modules.clone())?;
              arc_stream
                .write()
                .await
                .write_all(&buf)
                .await
                .on_error(|_| AgalThrow::Params {
                  type_error: parser::ErrorNames::TypeError,
                  message: "Error al escribir al buffer".into(),
                  stack: stack.clone(),
                })?;
              AgalValue::Never.to_result()
            }
            .boxed(),
          )
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  let tcp_stream = AgalObject::from_prototype(
    AgalPrototype::new(Arc::new(std::sync::RwLock::new(tcp_proto)), None).as_ref(),
  );

  let res = callback.clone().call(
    stack,
    callback.clone(),
    vec![tcp_stream.to_result()?],
    modules,
  )?;
  await_result(res).await?;

  Ok(())
}

pub fn get_module(prefix: &str) -> values::DefaultRefAgalValue {
  let mut module_name = get_name(prefix);
  let mut hashmap = std::collections::HashMap::new();

  hashmap.insert(
    "servidorTCP".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::servidorTCP"),
        func: Arc::new(|arguments, stack, modules, this| {
          let addr = arguments
            .clone()
            .get(0)
            .on_error(|_| AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba un acceso".into(),
              stack: stack.clone(),
            })?
            .to_agal_string(stack.clone(), modules.clone())?
            .clone();
          let callback = arguments
            .get(1)
            .on_error(|_| AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba una funcion".into(),
              stack: stack.clone(),
            })?
            .clone();

          let addr = addr.to_string();
          AgalPromise::new(
            async move {
              let stack = &stack;
              let listener =
                TcpListener::bind(addr)
                  .await
                  .on_error(move |_| AgalThrow::Params {
                    type_error: parser::ErrorNames::TypeError,
                    message: "Error al crear el servidor TCP".into(),
                    stack: stack.clone(),
                  })?;
              loop {
                let (mut stream, _) =
                  listener
                    .accept()
                    .await
                    .on_error(move |_| AgalThrow::Params {
                      type_error: parser::ErrorNames::TypeError,
                      message: "Error al aceptar la conexión".into(),
                      stack: stack.clone(),
                    })?;
                let mut buf = vec![];
                let data = stream.read_to_end(&mut buf);
                let data = AgalArray::from(&buf);
                handle_client(stream, &callback, stack.clone(), modules.clone()).await?;
              }
            }
            .boxed(),
          )
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "\0servidorUDP".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::servidorUDP"),
        func: Arc::new(|arguments, stack, modules, this| {
          let addr = arguments
            .clone()
            .get(0)
            .on_error(|_| AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba un acceso".into(),
              stack: stack.clone(),
            })?
            .to_agal_string(stack.clone(), modules.clone())?
            .clone();
          let callback = arguments
            .get(1)
            .on_error(|_| AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "Se esperaba una funcion".into(),
              stack: stack.clone(),
            })?
            .clone();

          let addr = addr.to_string();
          AgalPromise::new(
            async move {
              let stack = &stack;
              let listener = UdpSocket::bind(addr).await
              .on_error(move |_| AgalThrow::Params {
                type_error: parser::ErrorNames::TypeError,
                message: "Error al crear el servidor UDP".into(),
                stack: stack.clone(),
              })?;
              loop {
                let mut buf = vec![];
                let (_,mut stream) =
                  listener
                    .recv_from(&mut buf).await
                    .on_error(move |_| AgalThrow::Params {
                      type_error: parser::ErrorNames::TypeError,
                      message: "Error al aceptar la conexión".into(),
                      stack: stack.clone(),
                    })?;
                println!("{buf:?}");
                listener.send_to(&buf, stream).await;
              }
            }
            .boxed(),
          )
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  let prototype = complex::AgalPrototype::new(Arc::new(std::sync::RwLock::new(hashmap)), None);
  complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
  format!("{}{}", prefix, "red")
}
