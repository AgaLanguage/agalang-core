use std::collections::HashMap;

use crate::compiler::Value;
use crate::util::{OnError, OnSome};

pub const LIB_NAME: &str = ":sa";

const READ_FILE: &str = "leer_archivo";
const READ_DIR: &str = "leer_carpeta";

pub fn lib_value() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    READ_FILE,
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{READ_FILE}"),
        path: format!("<{LIB_NAME}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let path = args
            .get(0)
            .on_some_option(|t| {
              if t.is_string() {
                Some(t.as_string())
              } else {
                None
              }
            })
            .on_error(|_| format!("{READ_FILE}: Se esperaba una ruta"))?;
          std::fs::File::open(&path)
            .ok()
            .on_some_option(|mut file| {
              use std::io::Read;
              let mut buffer_writer = Vec::new();
              file
                .read_to_end(&mut buffer_writer)
                .ok()
                .on_some(|i| Value::Object(buffer_writer[..i].to_vec().into()))
            })
            .on_error(|_| format!("{READ_FILE}: No se pudo leer el archivo: {path}"))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    READ_DIR,
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{READ_DIR}"),
        path: format!("<{LIB_NAME}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let path = args
            .get(0)
            .on_some_option(|t| {
              if t.is_string() {
                Some(t.as_string())
              } else {
                None
              }
            })
            .on_error(|_| format!("{READ_DIR}: Se esperaba una ruta"))?;
          std::fs::read_dir(&path)
            .ok()
            .on_some_option(|dir| {
              let mut files = Vec::new();
              for file in dir {
                files.push(Value::String(file.ok().on_some(|file|file.file_name().to_string_lossy().to_string())?))
              }
              Some(Value::Object(files.into()))
            })
            .on_error(|_| format!("{READ_DIR}: No se pudo leer la carpeta: {path}"))
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
