use std::{
  cell::RefCell,
  fs,
  io::{self, Read as _, Write as _},
  path::Path,
  rc::Rc,
};

use parser::{internal::ErrorNames, util};

use crate::{
  runtime::{
    self,
    values::{
      self, complex, internal, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
  Modules,
};
fn get_path(
  stack: runtime::RefStack,
  this: values::DefaultRefAgalValue,
) -> Result<primitive::AgalString, internal::AgalThrow> {
  let string = this.get_object_property(stack.clone(), "@ruta");
  string?.borrow().to_agal_string(stack)
}

pub fn get_module(prefix: &str) -> values::DefaultRefAgalValue {
  let mut module_name = get_name(prefix);

  let path_name = format!("{module_name}::Ruta");

  let path = complex::AgalClass::new(
    path_name.clone(),
    vec![
      (
        "es_archivo".to_string(),
        complex::AgalClassProperty {
          is_public: true,
          is_static: false,
          value: internal::AgalNativeFunction {
            name: format!("{path_name}::es_archivo"),
            func: Rc::new(|_, stack, _, this| {
              let string = get_path(stack.clone(), this)?;
              let binding = string.to_string();
              let path = std::path::Path::new(&binding);
              primitive::AgalBoolean::new(path.is_file()).to_result()
            }),
          }
          .to_ref_value(),
        },
      ),
      (
        "es_carpeta".to_string(),
        complex::AgalClassProperty {
          is_public: true,
          is_static: false,
          value: internal::AgalNativeFunction {
            name: format!("{path_name}::es_carpeta"),
            func: Rc::new(|_, stack, _, this| {
              let string = get_path(stack, this)?;
              let binding = string.to_string();
              let path = std::path::Path::new(&binding);
              primitive::AgalBoolean::new(path.is_dir()).to_result()
            }),
          }
          .to_ref_value(),
        },
      ),
      (
        "nombre".to_string(),
        complex::AgalClassProperty {
          is_public: true,
          is_static: false,
          value: internal::AgalNativeFunction {
            name: format!("{path_name}::nombre"),
            func: Rc::new(|_, stack, _, this| get_path(stack, this)?.to_result()),
          }
          .to_ref_value(),
        },
      ),
      (
        "obtener_padre".to_string(),
        complex::AgalClassProperty {
          is_public: true,
          is_static: false,
          value: internal::AgalNativeFunction {
            name: format!("{path_name}::obtener_padre"),
            func: Rc::new(|_, stack, _, this| {
              let string = get_path(stack, this)?;
              let binding = string.to_string();
              let path = std::path::Path::new(&binding);
              let parent = path.parent().unwrap();
              primitive::AgalString::from_string(parent.to_string_lossy().to_string()).to_result()
            }),
          }
          .to_ref_value(),
        },
      ),
    ],
    None,
  )
  .to_value();

  let mut hashmap = std::collections::HashMap::new();
  hashmap.insert(
    "leer_archivo".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::leer_archivo"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let path: Option<&values::DefaultRefAgalValue> = arguments.get(0);
          if !path.is_some() {
            return internal::AgalThrow::Params {
              type_error: ErrorNames::TypeError,
              message: "Falta el argumento path".to_string(),
              stack,
            }
            .to_result();
          }
          let path = path.unwrap().try_to_string(stack.clone())?;
          let mut file = std::fs::File::open(path);
          if let Ok(file) = &mut file {
            let mut buffer_writer = Vec::new();
            file.read_to_end(&mut buffer_writer);
            let buffer: &[u8] = &buffer_writer;
            return complex::AgalArray::from(buffer).to_result();
          }
          internal::AgalThrow::Params {
            type_error: parser::internal::ErrorNames::PathError,
            message: "No se pudo abrir el archivo".to_string(),
            stack,
          }
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "leer_carpeta".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::leer_carpeta"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let path: Option<&values::DefaultRefAgalValue> = arguments.get(0);
          if !path.is_some() {
            return AgalValue::Never.to_result();
          }
          let path = path.unwrap().try_to_string(stack.clone())?;
          let mut dir = std::fs::read_dir(path);
          if let Ok(dir) = &mut dir {
            let mut files = Vec::new();
            for i in dir {
              if let Ok(entry) = i {
                files.push(
                  primitive::AgalString::from_string(
                    entry.file_name().to_string_lossy().to_string(),
                  )
                  .to_ref_value(),
                );
              }
            }
            return complex::AgalArray::from(files).to_result();
          }
          internal::AgalThrow::Params {
            type_error: parser::internal::ErrorNames::PathError,
            message: "No se pudo abrir el archivo".to_string(),
            stack,
          }
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "obtener_ruta".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::obtener_ruta"),
        func: Rc::new(move |arguments, stack, modules_manager, this| {
          let p: Option<&values::DefaultRefAgalValue> = arguments.get(0);
          if !p.is_some() {
            return AgalValue::Never.to_result();
          }
          let value = p.unwrap().to_agal_string(stack.clone())?;
          complex::AgalPromise::new(Box::pin({
            let mut path = path.clone();
            async move {
              let mut path = path
                .call(stack.clone(), this, vec![], modules_manager)
                .await?;

              let p = value.to_string();
              let p = crate::path::absolute_path(&p);
              let value = primitive::AgalString::from_string(p);
              path.set_object_property(stack, "@ruta", value.to_ref_value());
              path.to_result()
            }
          }))
          .to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "escribir_archivo".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::escribir_archivo"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let path: Option<&values::DefaultRefAgalValue> = arguments.get(0);
          if !path.is_some() {
            return AgalValue::Never.to_result();
          }
          let path = path.unwrap().try_to_string(stack.clone())?;
          let exists = Path::new(&path).exists();
          if !exists {
            return internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: "El archivo no existe".to_string(),
              stack,
            }
            .to_result();
          }
          let file = fs::OpenOptions::new().write(true).truncate(true).open(path);
          if let Err(error) = &file {
            return internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: error.to_string(),
              stack,
            }
            .to_result();
          }
          let mut file = file.ok().unwrap();
          let content: Option<&values::DefaultRefAgalValue> = arguments.get(1);
          if !content.is_some() {
            return internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: "Se nesesita contenido para escribir en el archivo".to_string(),
              stack,
            }
            .to_result();
          }
          let binding = content.unwrap().to_agal_array(stack.clone())?;
          let content = &*binding.borrow();
          let mut buf: &[u8] = &content.to_buffer(stack.clone())?;
          let f = file.write_all(buf);
          if let Err(error) = f {
            return internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: error.to_string(),
              stack,
            }
            .to_result();
          }
          AgalValue::Never.to_result()
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "crear_archivo".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::crear_archivo"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let path: Option<&values::DefaultRefAgalValue> = arguments.get(0);
          if !path.is_some() {
            return AgalValue::Never.to_result();
          }
          let path = path.unwrap().try_to_string(stack.clone())?;
          let exists = Path::new(&path).exists();
          if exists {
            return internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: "La ruta no esta disponible".to_string(),
              stack,
            }
            .to_result();
          }
          let file = fs::File::create(path);
          if let Err(error) = file {
            internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: error.to_string(),
              stack,
            }
            .to_result()
          } else {
            AgalValue::Never.to_result()
          }
        }),
      }
      .to_ref_value(),
    },
  );
  hashmap.insert(
    "crear_carpeta".to_string(),
    complex::AgalClassProperty {
      is_public: true,
      is_static: true,
      value: internal::AgalNativeFunction {
        name: format!("{module_name}::crear_carpeta"),
        func: Rc::new(|arguments, stack, modules_manager, this| {
          let path: Option<&values::DefaultRefAgalValue> = arguments.get(0);
          if !path.is_some() {
            return AgalValue::Never.to_result();
          }
          let path = path.unwrap().try_to_string(stack.clone())?;
          let exists = Path::new(&path).exists();
          if exists {
            return internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: "La ruta no esta disponible".to_string(),
              stack,
            }
            .to_result();
          }
          let file = fs::create_dir_all(path);
          if let Err(error) = file {
            internal::AgalThrow::Params {
              type_error: parser::internal::ErrorNames::PathError,
              message: error.to_string(),
              stack,
            }
            .to_result()
          } else {
            AgalValue::Never.to_result()
          }
        }),
      }
      .to_ref_value(),
    },
  );

  let prototype = complex::AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
  complex::AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
  format!("{}{}", prefix, "sa")
}
