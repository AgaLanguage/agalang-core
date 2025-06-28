use std::collections::HashMap;

use crate::compiler::{Class, Value};
use crate::functions_names::{CONSOLE, CONSTRUCTOR, STRING};
use crate::util::{OnError, OnSome, SetColor};

pub const LIB_NAME: &str = ":sa";

const PATH: &str = "Ruta";
const PATH_IS_FILE: &str = "es_archivo";
const PATH_IS_DIR: &str = "es_carpeta";
const PATH_GET_PARENT: &str = "obtener_padre";
const PATH_GET_NAME: &str = "obtener_nombre";
const PATH_GET_EXTENSION: &str = "obtener_extension";

const READ_FILE: &str = "leer_archivo";
const READ_DIR: &str = "leer_carpeta";

pub fn absolute_path(path: &str) -> String {
  let path = std::path::Path::new(path);
  let path_str = path
    .canonicalize()
    .ok()
    .map(|path| path.to_string_lossy().to_string())
    .unwrap_or_else(|| {
      let path = std::env::current_dir().unwrap().join(path);
      let mut string_path: Vec<String> = vec![];
      for part in path.components() {
        match part {
          std::path::Component::CurDir => {}
          std::path::Component::Normal(n) => string_path.push(n.to_string_lossy().to_string()),
          std::path::Component::RootDir => {}
          std::path::Component::Prefix(p) => {
            string_path.push(p.as_os_str().to_string_lossy().to_string())
          }
          std::path::Component::ParentDir => {
            string_path.pop();
          }
        }
      }
      string_path.join("\\")
    });

  path_str
    .strip_prefix(r"\\?\")
    .unwrap_or(&path_str)
    .to_string()
}
pub fn lib_value() -> Value {
  let path_class = Class::new(PATH.to_string());
  path_class.read().set_instance_property(
    CONSTRUCTOR,
    Value::Object(crate::compiler::Object::Function(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{PATH}::{CONSTRUCTOR}"),
        path: format!("<{LIB_NAME}>::{PATH}"),
        chunk: Default::default(),
        func: |this, args, thread, _| {
          let path = args
            .first()
            .map(|path| absolute_path(&path.as_string(thread)))
            .on_error(|_| format!("{PATH}: Se esperaba una ruta"))?;
          this.set_instance_property(
            CONSOLE,
            Value::String(format!("{PATH}({path})").set_color(crate::util::Color::Magenta)),
            true,
            true,
            thread,
          );
          this.set_instance_property(STRING, Value::String(path), true, true, thread);
          Ok(this)
        },
        custom_data: ().into(),
      }
      .into(),
    )),
  );
  path_class.read().get_instance().on_ok(|a| {
    a.set_instance_property(
      PATH_IS_FILE,
      Value::Object(crate::compiler::Object::Function(
        crate::compiler::Function::Native {
          name: format!("<{LIB_NAME}>::{PATH}()::{PATH_IS_FILE}"),
          path: format!("<{LIB_NAME}>::{PATH}"),
          chunk: Default::default(),
          func: |this, _, thread, _| {
            Ok(
              match std::path::Path::new(&this.as_string(thread)).is_file() {
                true => Value::True,
                false => Value::False,
              },
            )
          },
          custom_data: ().into(),
        }
        .into(),
      )),
      true,
    );
    a.set_instance_property(
      PATH_IS_DIR,
      Value::Object(crate::compiler::Object::Function(
        crate::compiler::Function::Native {
          name: format!("<{LIB_NAME}>::{PATH}()::{PATH_IS_DIR}"),
          path: format!("<{LIB_NAME}>::{PATH}"),
          chunk: Default::default(),
          func: |this, _, thread, _| {
            Ok(
              match std::path::Path::new(&this.as_string(thread)).is_dir() {
                true => Value::True,
                false => Value::False,
              },
            )
          },
          custom_data: ().into(),
        }
        .into(),
      )),
      true,
    );
    a.set_instance_property(
      PATH_GET_PARENT,
      Value::Object(crate::compiler::Object::Function(
        crate::compiler::Function::Native {
          name: format!("<{LIB_NAME}>::{PATH}()::{PATH_GET_PARENT}"),
          path: format!("<{LIB_NAME}>::{PATH}"),
          chunk: Default::default(),
          func: |this, _, thread, _| {
            std::path::Path::new(&this.as_string(thread))
              .parent()
              .map(|p| Value::String(p.to_string_lossy().to_string()))
              .on_error(|_| format!("{PATH_GET_PARENT}: La ruta no tiene padre"))
          },
          custom_data: ().into(),
        }
        .into(),
      )),
      true,
    );
    a.set_instance_property(
      PATH_GET_NAME,
      Value::Object(crate::compiler::Object::Function(
        crate::compiler::Function::Native {
          name: format!("<{LIB_NAME}>::{PATH}()::{PATH_GET_NAME}"),
          path: format!("<{LIB_NAME}>::{PATH}"),
          chunk: Default::default(),
          func: |this, _, thread, _| {
            std::path::Path::new(&this.as_string(thread))
              .file_name()
              .map(|p| Value::String(p.to_string_lossy().to_string()))
              .on_error(|_| format!("{PATH_GET_NAME}: La ruta no es un archivo"))
          },
          custom_data: ().into(),
        }
        .into(),
      )),
      true,
    );
    a.set_instance_property(
      PATH_GET_EXTENSION,
      Value::Object(crate::compiler::Object::Function(
        crate::compiler::Function::Native {
          name: format!("<{LIB_NAME}>::{PATH}()::{PATH_GET_EXTENSION}"),
          path: format!("<{LIB_NAME}>::{PATH}"),
          chunk: Default::default(),
          func: |this, _, thread, _| {
            std::path::Path::new(&this.as_string(thread))
              .extension()
              .map(|p| Value::String(p.to_string_lossy().to_string()))
              .on_error(|_| format!("{PATH_GET_EXTENSION}: La ruta no es un archivo"))
          },
          custom_data: ().into(),
        }
        .into(),
      )),
      true,
    );
    Some(())
  });
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    READ_FILE,
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{READ_FILE}"),
        path: format!("<{LIB_NAME}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, thread, _| {
          let path = args
            .first()
            .map(|t| t.as_string(thread))
            .on_error(|_| format!("{READ_FILE}: Se esperaba una ruta"))?;
          std::fs::File::open(&path)
            .ok()
            .on_some_option(|mut file| {
              use std::io::Read;
              let mut buffer_writer = Vec::new();
              file
                .read_to_end(&mut buffer_writer)
                .ok()
                .map(|i| Value::Object(buffer_writer[..i].to_vec().into()))
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
        func: |_, args, thread, _| {
          let path = args
            .first()
            .map(|t| t.as_string(thread))
            .on_error(|_| format!("{READ_DIR}: Se esperaba una ruta"))?;
          std::fs::read_dir(&path)
            .ok()
            .on_some_option(|dir| {
              let mut files = Vec::new();
              for file in dir {
                files.push(Value::String(
                  file
                    .ok()
                    .map(|file| file.file_name().to_string_lossy().to_string())?,
                ))
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
  hashmap.set_instance_property(
    PATH,
    Value::Object(crate::compiler::Object::Class(path_class)),
    true,
  );
  Value::Object(crate::compiler::Object::Map(
    HashMap::new().into(),
    hashmap.into(),
  ))
}
