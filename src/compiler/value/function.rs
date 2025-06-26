use super::{Class, Value};
use crate::compiler::Promise;
use crate::interpreter::VarsManager;
use crate::parser::NodeBlock;
use crate::util::{Color, Location, MutClone};
use crate::{compiler::ChunkGroup, parser::NodeFunction, Decode, MultiRefHash, StructTag};

pub const FUNCTION_TYPE: &str = "funcion";
pub const SCRIPT_TYPE: &str = "script";
pub const NATIVE_FUNCTION_TYPE: &str = "funcion nativa";

pub enum NativeValue {
  None,
  TcpStream(std::net::TcpStream),
  Promise(Promise),
  ValuePromise(MultiRefHash<NativeValue>, Promise),
}
impl NativeValue {
  pub fn mut_tcp_stream(&mut self) -> Option<&mut std::net::TcpStream> {
    match self {
      NativeValue::TcpStream(stream) => Some(stream),
      _ => None,
    }
  }
  pub fn get_value(&self) -> Option<MultiRefHash<NativeValue>> {
    match self {
      Self::ValuePromise(value, _) => Some(value.clone()),
      _ => None,
    }
  }
  pub fn get_promise(&mut self) -> Option<&mut Promise> {
    match self {
      Self::Promise(promise) => Some(promise),
      Self::ValuePromise(_, promise) => Some(promise),
      _ => None,
    }
  }
}
impl From<()> for MultiRefHash<NativeValue> {
  fn from(_: ()) -> Self {
    MultiRefHash::new(NativeValue::None)
  }
}
impl From<std::net::TcpStream> for MultiRefHash<NativeValue> {
  fn from(stream: std::net::TcpStream) -> Self {
    MultiRefHash::new(NativeValue::TcpStream(stream))
  }
}
impl From<Promise> for MultiRefHash<NativeValue> {
  fn from(promise: Promise) -> Self {
    MultiRefHash::new(NativeValue::Promise(promise))
  }
}
impl From<(MultiRefHash<NativeValue>, Promise)> for MultiRefHash<NativeValue> {
  fn from(values: (MultiRefHash<NativeValue>, Promise)) -> Self {
    MultiRefHash::new(NativeValue::ValuePromise(values.0, values.1))
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Function {
  Function {
    arity: usize,
    chunk: MultiRefHash<ChunkGroup>,
    name: String,
    is_async: bool,
    in_class: MultiRefHash<Option<MultiRefHash<Class>>>,
    location: crate::util::Location,
    scope: MultiRefHash<Option<MultiRefHash<VarsManager>>>,
    has_rest: bool,
  },
  Script {
    chunk: MultiRefHash<ChunkGroup>,
    path: String,
    scope: MultiRefHash<Option<MultiRefHash<VarsManager>>>,
  },
  Native {
    name: String,
    path: String,
    chunk: MultiRefHash<ChunkGroup>,
    func: for<'a> fn(
      Value,
      Vec<Value>,
      &'a mut crate::interpreter::Thread,
      MultiRefHash<NativeValue>,
    ) -> Result<Value, String>,
    custom_data: MultiRefHash<NativeValue>,
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
      Self::Function { .. } => FUNCTION_TYPE,
      Self::Script { .. } => SCRIPT_TYPE,
      Self::Native { .. } => NATIVE_FUNCTION_TYPE,
    }
  }
  pub fn set_in_class(&self, class: MultiRefHash<Class>) {
    match self {
      Self::Function { in_class, .. } => *in_class.write() = Some(class),
      Self::Script { .. } | Self::Native { .. } => {}
    }
  }
  pub fn get_in_class(&self) -> Option<MultiRefHash<Class>> {
    match self {
      Self::Function { in_class, .. } => in_class.cloned(),
      Self::Script { .. } | Self::Native { .. } => None,
    }
  }
  pub fn set_scope(&self, vars: MultiRefHash<VarsManager>) {
    match self {
      Self::Function { scope: v, .. } => *v.write() = Some(vars),
      Self::Script { scope: v, .. } => *v.write() = Some(vars),
      Self::Native { .. } => {}
    }
  }
  pub fn get_scope(&self) -> Option<MultiRefHash<VarsManager>> {
    match self {
      Self::Function { scope: vars, .. } => vars.cloned(),
      Self::Script { scope: vars, .. } => vars.cloned(),
      Self::Native { .. } => None,
    }
  }
  pub fn chunk(&self) -> MultiRefHash<ChunkGroup> {
    match self {
      Self::Function { chunk, .. } => chunk.clone(),
      Self::Script { chunk, .. } => chunk.clone(),
      Self::Native { chunk, .. } => chunk.clone(),
    }
  }
  pub fn location(&self) -> String {
    use crate::util::SetColor as _;
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
      chunk: ChunkGroup::new().into(),
      name: value.name.clone(),
      is_async: value.is_async,
      location: value.location.clone(),
      scope: None.into(),
      has_rest: false,
      in_class: None.into(),
    }
  }
}
impl From<&NodeBlock> for Function {
  fn from(value: &NodeBlock) -> Self {
    Self::Script {
      chunk: ChunkGroup::new().into(),
      path: value.location.file_name.clone(),
      scope: None.into(),
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
        encode.extend(chunk.read().encode()?);
        encode.extend(name.encode()?);
        encode.extend(is_async.encode()?);
        encode.extend(location.encode()?);
        encode.extend(has_rest.encode()?);
      }
      Function::Script { chunk, path, .. } => {
        encode.push(1);
        encode.extend(path.encode()?);
        encode.extend(chunk.read().encode()?);
      }
      Function::Native { .. } => return Err("No se puede compilar una funcion nativa".to_string()),
    };

    Ok(encode)
  }
}
impl Decode for Function {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    use crate::util::{OnError as _, OnSome as _};
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
        chunk: ChunkGroup::decode(vec)?.into(),
        name: String::decode(vec)?,
        is_async: bool::decode(vec)?,
        location: Location::decode(vec)?,
        has_rest: bool::decode(vec)?,
      }),
      1 => Ok(Self::Script {
        scope: Default::default(),
        path: String::decode(vec)?,
        chunk: ChunkGroup::decode(vec)?.into(),
      }),
      _ => Err("Se esperaba una funcion".to_string()),
    };
  }
}
impl MutClone for Function {}
