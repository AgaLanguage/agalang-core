use std::{cell::RefCell, rc::Rc};

use super::{Class, MultiRefHash, Value};
use crate::interpreter::VarsManager;
use crate::util::{Color, Location, OnError as _, OnSome as _, SetColor as _};
use crate::{compiler::ChunkGroup, parser::NodeFunction};
use crate::{Decode, StructTag};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Function {
  Function {
    arity: usize,
    chunk: ChunkGroup,
    name: String,
    is_async: bool,
    in_class: MultiRefHash<Option<MultiRefHash<Class>>>,
    location: crate::util::Location,
    scope: MultiRefHash<Option<Rc<RefCell<VarsManager>>>>,
    has_rest: bool,
  },
  Script {
    chunk: ChunkGroup,
    path: String,
    scope: MultiRefHash<Option<Rc<RefCell<VarsManager>>>>,
  },
  Native {
    name: String,
    path: String,
    chunk: ChunkGroup,
    func: fn(Value, Vec<Value>, &mut crate::interpreter::Thread) -> Result<Value, String>,
  },
}
impl Function {
  pub fn set_rest(&mut self, rest: bool) {
    match self {
      Self::Function { has_rest, .. } => *has_rest = rest,
      _ => {}
    }
  }
  pub fn get_type(&self) -> &'static str {
    match self {
      Self::Function { .. } => "funcion",
      Self::Script { .. } => "script",
      Self::Native { .. } => "nativo",
    }
  }
  pub fn set_in_class(&self, class: MultiRefHash<Class>) {
    match self {
      Self::Function { in_class, .. } => *in_class.borrow_mut() = Some(class),
      Self::Script { .. } | Self::Native { .. } => {}
    }
  }
  pub fn get_in_class(&self) -> Option<MultiRefHash<Class>> {
    match self {
      Self::Function { in_class, .. } => in_class.cloned(),
      Self::Script { .. } | Self::Native { .. } => None,
    }
  }
  pub fn set_scope(&self, vars: Rc<RefCell<VarsManager>>) {
    match self {
      Self::Function { scope: v, .. } => *v.borrow_mut() = Some(vars),
      Self::Script { scope: v, .. } => *v.borrow_mut() = Some(vars),
      Self::Native { .. } => {}
    }
  }
  pub fn get_scope(&self) -> Option<Rc<RefCell<VarsManager>>> {
    match self {
      Self::Function { scope: vars, .. } => vars.borrow().clone(),
      Self::Script { scope: vars, .. } => vars.borrow().clone(),
      Self::Native { .. } => None,
    }
  }
  pub fn chunk(&mut self) -> &mut ChunkGroup {
    match self {
      Self::Function { chunk, .. } => chunk,
      Self::Script { chunk, .. } => chunk,
      Self::Native { chunk, .. } => chunk,
    }
  }
  pub fn location(&self) -> String {
    match self {
      Self::Function {
        name,
        is_async,
        location,
        ..
      } => format!(
        "en {} <{}:{}:{}>",
        if *is_async {
          format!("asinc {name}")
        } else {
          name.to_string()
        },
        location.file_name.set_color(Color::Cyan),
        (location.start.line + 1)
          .to_string()
          .set_color(Color::Yellow),
        (location.start.column + 1)
          .to_string()
          .set_color(Color::Yellow)
      ),
      Self::Script { path, .. } => {
        format!(
          "en <{}:{}>",
          path.set_color(Color::Cyan),
          "script".to_string().set_color(Color::Gray)
        )
      }
      Self::Native { path, name, .. } => {
        if path.is_empty() {
          return format!(
            "en {name} <{}>",
            "nativo".to_string().set_color(Color::Gray)
          );
        }
        format!(
          "en {name} <{}:{}>",
          path.set_color(Color::Cyan),
          "nativo".to_string().set_color(Color::Gray)
        )
      }
    }
  }
}
impl ToString for Function {
  fn to_string(&self) -> String {
    match self {
      Self::Function { name, is_async, .. } => {
        format!("<{} {name}>", if *is_async { "asinc fn" } else { "fn" })
      }
      Self::Script { path, .. } => format!("<script '{path}'>"),
      Self::Native { name, .. } => format!("<nativo fn {name}>"),
    }
  }
}
impl From<&NodeFunction> for Function {
  fn from(value: &NodeFunction) -> Self {
    Self::Function {
      arity: value.params.len(),
      chunk: ChunkGroup::new(),
      name: value.name.clone(),
      is_async: value.is_async,
      location: value.location.clone(),
      scope: None.into(),
      has_rest: false,
      in_class: None.into(),
    }
  }
}
impl std::fmt::Debug for Function {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
impl crate::Encode for Function {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::Function as u8];
    match self {
      Function::Function {
        arity,
        chunk,
        name,
        is_async,
        location,
        has_rest,
        ..
      } => {
        encode.push(0);
        encode.extend(arity.encode()?);
        encode.extend(chunk.encode()?);
        encode.extend(name.encode()?);
        encode.extend(is_async.encode()?);
        encode.extend(location.encode()?);
        encode.extend(has_rest.encode()?);
      }
      Function::Script { chunk, path, .. } => {
        encode.push(1);
        encode.extend(path.encode()?);
        encode.extend(chunk.encode()?);
      }
      Function::Native { .. } => return Err("No se puede compilar una funcion nativa".to_string()),
    };

    Ok(encode)
  }
}
impl Decode for Function {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Function as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba una funcion".to_string())?;
    let type_byte = vec
      .pop_front()
      .on_error(|_| "Se esperaba un tipo de funcion".to_string())?;
    return match type_byte {
      0 => Ok(Self::Function {
        in_class: Default::default(),
        scope: Default::default(),
        arity: usize::decode(vec)?,
        chunk: ChunkGroup::decode(vec)?,
        name: String::decode(vec)?,
        is_async: bool::decode(vec)?,
        location: Location::decode(vec)?,
        has_rest: bool::decode(vec)?,
      }),
      1 => Ok(Self::Script {
        scope: Default::default(),
        path: String::decode(vec)?,
        chunk: ChunkGroup::decode(vec)?,
      }),
      _ => Err("Se esperaba una funcion".to_string()),
    };
  }
}
