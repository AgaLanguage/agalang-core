use crate::{
  agal_parser::{Node, NodeOperator},
  util::{Location, Position},
  MultiRefHash, ToJSON,
};
use std::{
  collections::{HashMap, HashSet},
  ops::Deref,
  rc::Rc,
};

trait Resolvable {
  fn resolve(self, tokens: &Vec<SyntaxTokenData>) -> Self;
}
impl<T> Resolvable for Vec<T>
where
  T: Resolvable,
{
  fn resolve(self, tokens: &Vec<SyntaxTokenData>) -> Self {
    self.into_iter().map(|v| v.resolve(tokens)).collect()
  }
}

#[derive(Debug, Clone)]
enum SyntaxTokenType {
  Class,
  Function,
  Variable,
  Parameter,
  Module,
  KeywordControl,
}
impl ToJSON for SyntaxTokenType {
  fn to_json(&self) -> String {
    format!("{self:?}").to_json()
  }
}
#[derive(Debug, Clone)]
enum SyntaxTokenModifier {
  Constant,
  Iterable,
}
impl ToJSON for SyntaxTokenModifier {
  fn to_json(&self) -> String {
    format!("{self:?}").to_json()
  }
}
#[derive(Debug, Clone)]
struct SyntaxTokenData {
  pub definition: Position,
  pub token_type: SyntaxTokenType,
  pub token_modifier: Vec<SyntaxTokenModifier>,
  pub location: Location,
  pub data_type: DataType,
  pub is_original_decl: bool,
}
impl ToJSON for SyntaxTokenData {
  fn to_json(&self) -> String {
    format!(
      "{{\"definition\":{},\"location\":{},\"token_type\":{},\"token_modifier\":{},\"data_type\":{}{}}}",
      self.definition.to_json(),
      self.location.to_json(),
      self.token_type.to_json(),
      self.token_modifier.to_json(),
      self.data_type.to_json(),
      if self.is_original_decl {
        ",\"is_original_decl\":true"
      } else {
        ""
      }
    )
  }
}
impl Resolvable for SyntaxTokenData {
  fn resolve(self, tokens: &Vec<SyntaxTokenData>) -> Self {
    let data_type = self.data_type.resolve(tokens);
    Self {
      definition: self.definition,
      token_type: self.token_type,
      token_modifier: self.token_modifier,
      location: self.location,
      data_type,
      is_original_decl: self.is_original_decl,
    }
  }
}
#[derive(Eq, Clone, Debug)]
struct RefHash<T>(Rc<T>);
impl<T> RefHash<T> {
  fn new(value: T) -> Self {
    Self(Rc::new(value))
  }
}
impl<T> PartialEq for RefHash<T> {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0)
  }
}
impl<T> std::hash::Hash for RefHash<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    Rc::as_ptr(&self.0).hash(state);
  }
}
impl<T> Deref for RefHash<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    self.0.as_ref()
  }
}
impl<T> Default for RefHash<T>
where
  T: Default,
{
  fn default() -> Self {
    Self(Default::default())
  }
}

#[derive(Default, Clone)]
struct Scope {
  locals: MultiRefHash<HashMap<String, SyntaxTokenData>>,
  parent: Option<Rc<Scope>>,
}
impl Scope {
  fn get(&self, k: &String) -> Option<SyntaxTokenData> {
    if let Some(val) = self.locals.read().get(k).cloned() {
      Some(val)
    } else if let Some(ref parent) = self.parent {
      parent.get(k)
    } else {
      None
    }
  }
  fn insert(&self, k: String, v: SyntaxTokenData) -> Option<SyntaxTokenData> {
    self.locals.write().insert(k, v)
  }
  fn child(self: &Rc<Self>) -> Rc<Self> {
    Rc::new(Self {
      locals: Default::default(),
      parent: Some(self.clone()),
    })
  }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
enum DataType {
  #[default]
  Unknown,
  Char,
  String,
  StringLiteral(String),
  Number,
  Byte,
  Never,
  Boolean,
  Reference(Box<Self>),
  Iterable(Box<Self>),
  List(Box<Self>),
  Item(Box<Self>),
  Promise(Box<Self>),
  Mod(String),
  Fn {
    params: Vec<Self>,
    ret: Box<Self>,
  },
  Class {
    props: RefHash<HashMap<String, Self>>,
    instance_props: RefHash<HashMap<String, Self>>,
    name: String,
  },
  Constructor(Box<Self>),
  ConstructorFn {
    params: Vec<Self>,
    ret: Box<Self>,
  },
  Instance {
    props: RefHash<HashMap<String, Self>>,
    name: String,
  },
  Identifier(Location),
  Param(Location, usize),
  Params(Location),
  Return(Box<Self>),
  Multiple(Vec<Self>),
  Member {
    object: Box<Self>,
    member: Box<Self>,
    is_instance: bool,
  },
  UnPromise(Box<Self>),
  Call {
    callee: Box<Self>,
    args: Vec<Self>,
  },
}
impl DataType {
  fn on_call(&self, args: Vec<Self>) -> Self {
    match self {
      Self::Number => Self::Number,
      Self::Class {
        instance_props,
        name,
        ..
      } => Self::Instance {
        props: instance_props.clone(),
        name: name.clone(),
      },
      Self::ConstructorFn { ret, params } | Self::Fn { ret, params } => {
        let mut diff = 0;
        let mut type_params = vec![];
        for (index, param) in params.iter().enumerate() {
          if index < diff {
            type_params.push((param, Self::List(Default::default())));
            continue;
          }
          if matches!(param, Self::Params(_)) {
            let start_index = index - diff;
            let mut type_param_list = vec![];
            for i in start_index..args.len() {
              diff += 1;
              type_param_list.push(match args.get(i).cloned().unwrap_or_default() {
                Self::Iterable(inner) => Self::Item(inner),
                data_type => data_type,
              });
            }
            type_params.push((
              param,
              Self::List(Box::new(Self::Multiple(type_param_list).simplify())),
            ));
            continue;
          }
          let type_param = match args.get(index - diff).cloned().unwrap_or_default() {
            Self::Iterable(inner) => {
              diff += 1;
              Self::Item(inner)
            }
            data_type => data_type,
          };
          type_params.push((param, type_param));
        }
        ret.infer(&type_params)
      }
      Self::Constructor(val) => *val.clone(),
      Self::Reference(val) => val.on_call(args),
      _ => Self::Call {
        callee: Box::new(self.clone()),
        args,
      },
    }
  }
  fn or_else(self, cb: impl FnOnce() -> Self) -> Self {
    if matches!(self, Self::Never | Self::Unknown) {
      cb()
    } else {
      self
    }
  }
  fn prop_type(&self) -> SyntaxTokenType {
    match self {
      Self::Fn { .. } => SyntaxTokenType::Function,
      Self::Class { .. } | Self::Constructor(_) | Self::ConstructorFn { .. } => {
        SyntaxTokenType::Class
      }
      Self::Mod { .. } => SyntaxTokenType::Module,
      Self::Reference(data) => data.prop_type(),
      _ => SyntaxTokenType::Variable,
    }
  }
  fn into_no_ret(self) -> Self {
    match self {
      Self::Return(val) => (*val).into_no_ret(),
      Self::Multiple(val) => Self::Multiple(val.into_iter().map(|i| i.into_no_ret()).collect()),
      _ => self,
    }
  }
  fn into_no_repeat(self) -> Self {
    match self {
      Self::Multiple(vec) => {
        let mut set = HashSet::new();
        for data in vec {
          set.insert(data);
        }
        let mut vec = set.drain().into_iter().collect::<Vec<_>>();
        if vec.len() <= 1 {
          vec.pop().unwrap_or_default()
        } else {
          Self::Multiple(vec)
        }
      }
      _ => self,
    }
  }
  fn simplify(&self) -> Self {
    match self {
      Self::Multiple(vals) => {
        let mut unique: HashSet<DataType> = vals
          .into_iter()
          .map(|d| match d {
            Self::Multiple(inner) => inner.into_iter().map(|d| d.simplify()).collect(),
            _ => vec![d.simplify()],
          })
          .flatten()
          .collect();
        let mut unique_vec: Vec<DataType> = unique.drain().collect();
        if unique_vec.len() <= 1 {
          unique_vec.pop().unwrap_or_default()
        } else {
          DataType::Multiple(unique_vec)
        }
      }
      Self::Iterable(val) => match val.simplify() {
        Self::Iterable(inner) => Self::Iterable(inner),
        inner => Self::Iterable(Box::new(inner)),
      },
      Self::List(val) => Self::List(Box::new(val.simplify())),
      Self::Item(val) => match val.simplify() {
        Self::Iterable(inner) => Self::Item(Box::new(*inner)).simplify(),
        Self::List(inner) => *inner,
        Self::String | Self::StringLiteral(_) => Self::Char,
        s => Self::Item(Box::new(s)),
      },
      Self::Constructor(val) => Self::Constructor(Box::new(val.simplify())),
      Self::Return(val) => Self::Return(Box::new(val.simplify().into_no_ret())),
      Self::Reference(val) => Self::Reference(Box::new(val.simplify())),
      Self::Member {
        member,
        object,
        is_instance,
      } => {
        let object = object.simplify();
        let member = member.simplify();
        match (&object, &member) {
          (Self::List(inner), Self::Number) => Some(*inner.clone()),
          (Self::String | Self::StringLiteral(_), Self::Number) => Some(Self::Char),
          (Self::Mod(path), Self::StringLiteral(prop)) => {
            mod_types(path).get(prop.as_str()).cloned()
          }
          (Self::Instance { props, .. } | Self::Class { props, .. }, Self::StringLiteral(prop)) => {
            props.get(prop).cloned()
          }
          (val, Self::StringLiteral(prop)) => proto_types(&val).get(prop.as_str()).cloned(),
          _ => None,
        }
        .unwrap_or_else(|| Self::Member {
          object: Box::new(object),
          member: Box::new(member),
          is_instance: *is_instance,
        })
      }
      Self::Promise(data_type) => Self::Promise(Box::new(data_type.simplify())),
      Self::UnPromise(data_type) => match data_type.simplify() {
        Self::Promise(inner) => *inner,
        inner => Self::UnPromise(Box::new(inner)),
      },
      _ => self.clone(),
    }
  }
  fn infer(&self, params: &Vec<(&Self, Self)>) -> Self {
    match self {
      Self::Multiple(values) => {
        Self::Multiple(values.into_iter().map(|v| v.infer(params)).collect()).into_no_repeat()
      }
      Self::Param(_, _) | Self::Params(_) => {
        for (param, data_type) in params {
          if self.eq(param) {
            return data_type.clone();
          }
        }
        self.clone()
      }
      Self::Fn {
        params: values,
        ret,
      } => Self::Fn {
        params: values.into_iter().map(|v| v.infer(params)).collect(),
        ret: Box::new(ret.infer(params).into_no_ret()),
      },
      Self::ConstructorFn {
        params: values,
        ret,
      } => Self::ConstructorFn {
        params: values.into_iter().map(|v| v.infer(params)).collect(),
        ret: Box::new(ret.infer(params).into_no_ret()),
      },
      Self::List(val) => Self::List(Box::new(val.infer(params))),
      Self::Constructor(val) => Self::Constructor(Box::new(val.infer(params))),
      Self::Return(val) => Self::Return(Box::new(val.infer(params))),
      Self::Iterable(val) => Self::Iterable(Box::new(val.infer(params))),
      Self::Item(val) => Self::Item(Box::new(val.infer(params))),
      Self::Reference(val) => Self::Reference(Box::new(val.infer(params))),
      Self::Promise(val) => Self::Promise(Box::new(val.infer(params))),
      Self::UnPromise(val) => Self::UnPromise(Box::new(val.infer(params))),
      Self::Member {
        object,
        member,
        is_instance,
      } => Self::Member {
        object: Box::new(object.infer(params)),
        member: Box::new(member.infer(params)),
        is_instance: *is_instance,
      },
      Self::Call { callee, args } => Self::Call {
        args: args.iter().map(|d| d.infer(params)).collect(),
        callee: Box::new(callee.infer(params)),
      },
      Self::Unknown
      | Self::Char
      | Self::String
      | Self::StringLiteral(_)
      | Self::Number
      | Self::Byte
      | Self::Never
      | Self::Boolean
      | Self::Mod(_)
      | Self::Class { .. }
      | Self::Instance { .. }
      | Self::Identifier(_) => self.clone(),
    }
  }
  fn infer_call(&self, args: &Vec<Self>) -> Self {
    match self.simplify() {
      Self::ConstructorFn { params, .. } | Self::Fn { params, .. } => {
        let mut diff = 0;
        let mut type_params = vec![];
        for (index, param) in params.iter().enumerate() {
          if index < diff {
            type_params.push((param, Self::List(Default::default())));
            continue;
          }
          if matches!(param, Self::Params(_)) {
            let start_index = index - diff;
            let mut type_param_list = vec![];
            for i in start_index..args.len() {
              diff += 1;
              type_param_list.push(match args.get(i).cloned().unwrap_or_default() {
                Self::Iterable(inner) => Self::Item(inner),
                data_type => data_type,
              });
            }
            type_params.push((
              param,
              Self::List(Box::new(Self::Multiple(type_param_list).simplify())),
            ));
            continue;
          }
          let type_param = match args.get(index - diff).cloned().unwrap_or_default() {
            Self::Iterable(inner) => {
              diff += 1;
              Self::Item(inner)
            }
            data_type => data_type,
          };
          type_params.push((param, type_param));
        }
        self.infer(&type_params)
      }
      Self::Reference(data) => data.infer_call(args),
      Self::Multiple(data) => {
        Self::Multiple(data.into_iter().map(|v| v.infer_call(args)).collect()).into_no_repeat()
      }
      Self::Call { callee, args } => Self::Call {
        callee: Box::new(callee.infer_call(&args)),
        args: args.iter().map(|v| v.infer_call(&args)).collect(),
      },
      Self::Unknown
      | Self::Char
      | Self::String
      | Self::StringLiteral(_)
      | Self::Number
      | Self::Byte
      | Self::Never
      | Self::Boolean
      | Self::Iterable(_)
      | Self::List(_)
      | Self::Item(_)
      | Self::Promise(_)
      | Self::Mod(_)
      | Self::Class { .. }
      | Self::Instance { .. }
      | Self::Constructor(_)
      | Self::Identifier(_)
      | Self::Param(_, _)
      | Self::Params(_)
      | Self::Return(_)
      | Self::Member { .. }
      | Self::UnPromise(_) => self.clone(),
    }
  }
  fn get_params(&self) -> Vec<Self> {
    match self {
      Self::Fn { params, .. } => params.clone(),
      Self::ConstructorFn { params, .. } => params.clone(),
      Self::Call { .. }
      | Self::Unknown
      | Self::Char
      | Self::String
      | Self::StringLiteral(_)
      | Self::Number
      | Self::Byte
      | Self::Never
      | Self::Boolean
      | Self::Reference(_)
      | Self::Iterable(_)
      | Self::List(_)
      | Self::Item(_)
      | Self::Promise(_)
      | Self::Mod(_)
      | Self::Class { .. }
      | Self::Constructor(_)
      | Self::Instance { .. }
      | Self::Identifier(_)
      | Self::Param(_, _)
      | Self::Params(_)
      | Self::Return(_)
      | Self::Multiple(_)
      | Self::Member { .. }
      | Self::UnPromise(_) => vec![],
    }
  }
  fn unpromise(&self) -> Self {
    match self {
      Self::Promise(val) => *val.clone(),
      Self::Multiple(vals) => {
        let mut new_vals = vec![];
        for val in vals {
          new_vals.push(val.unpromise());
        }
        Self::Multiple(new_vals).into_no_repeat()
      }
      _ => Self::UnPromise(Box::new(self.clone())),
    }
  }
}
impl AsRef<DataType> for DataType {
  fn as_ref(&self) -> &DataType {
    self
  }
}
impl ToJSON for DataType {
  fn to_json(&self) -> String {
    match self.simplify() {
      Self::Unknown => "null".to_string(),
      Self::Char => "{\"class\":\"agal\",\"type\":\"caracter\"}".to_string(),
      Self::StringLiteral(val) => format!(
        "{{\"class\":\"agal\",\"type\":\"cadena\",\"val\":{}}}",
        val.to_json()
      ),
      Self::String => "{\"class\":\"agal\",\"type\":\"cadena\"}".to_string(),
      Self::Number => "{\"class\":\"agal\",\"type\":\"numero\"}".to_string(),
      Self::Byte => "{\"class\":\"agal\",\"type\":\"byte\"}".to_string(),
      Self::Never => "{\"class\":\"agal\",\"type\":\"nada\"}".to_string(),
      Self::Boolean => "{\"class\":\"agal\",\"type\":\"buleano\"}".to_string(),
      Self::Reference(data) => format!(
        "{{\"class\":\"agal\",\"type\":\"referencia\",\"val\":{}}}",
        data.to_json()
      ),
      Self::Iterable(data) => format!(
        "{{\"class\":\"agal\",\"type\":\"iterable\",\"val\":{}}}",
        data.to_json()
      ),
      Self::List(data) => format!(
        "{{\"class\":\"agal\",\"type\":\"lista\",\"val\":{}}}",
        data.to_json()
      ),
      Self::Item(data) => format!("{{\"class\":\"item\",\"val\":{}}}", data.to_json()),
      Self::Promise(data) => format!("{{\"class\":\"promesa\",\"val\":{}}}", data.to_json()),
      Self::Mod(path) => format!("{{\"class\":\"mod\",\"path\":{}}}", path.to_json()),
      Self::Fn { params, ret } => format!(
        "{{\"class\":\"fn\",\"params\":{},\"ret\":{}}}",
        params.to_json(),
        ret.to_json()
      ),
      Self::ConstructorFn { params, ret } => format!(
        "{{\"class\":\"constructor_fn\",\"params\":{},\"ret\":{}}}",
        params.to_json(),
        ret.to_json()
      ),
      Self::Class {
        instance_props,
        props: static_props,
        name,
      } => format!(
        "{{\"class\":\"clase\",\"instance_props\":{},\"static_props\":{},\"name\":{}}}",
        instance_props.to_json(),
        static_props.to_json(),
        name.to_json()
      ),
      Self::Constructor(val) => format!("{{\"class\":\"constructor\",\"val\":{}}}", val.to_json()),
      Self::Instance { props, name } => format!(
        "{{\"class\":\"instancia\",\"props\":{},\"name\":{}}}",
        props.to_json(),
        name.to_json()
      ),
      Self::Param(location, index) => format!(
        "{{\"class\":\"param\",\"location\":{},\"index\":{index}}}",
        location.to_json()
      ),
      Self::Params(location) => format!(
        "{{\"class\":\"params\",\"location\":{}}}",
        location.to_json()
      ),
      Self::Identifier(location) => {
        format!("{{\"class\":\"id\",\"location\":{}}}", location.to_json())
      }
      Self::Return(data) => format!("{{\"class\":\"ret\",\"val\":{}}}", data.to_json()),
      Self::Multiple(data) => format!("{{\"class\":\"multiple\",\"val\":{}}}", data.to_json()),
      Self::Member {
        object,
        member,
        is_instance,
      } => {
        format!(
          "{{\"class\":\"member\",\"object\":{},\"member\":{},\"is_instance\":{}}}",
          object.to_json(),
          member.to_json(),
          is_instance
        )
      }
      Self::UnPromise(val) => {
        format!("{{\"class\":\"en_promesa\",\"val\":{}}}", val.to_json())
      }
      Self::Call { callee, args } => {
        format!(
          "{{\"class\":\"llamada\",\"callee\":{},\"args\":{}}}",
          callee.to_json(),
          args.to_json()
        )
      }
    }
  }
}
impl Resolvable for DataType {
  fn resolve(self, tokens: &Vec<SyntaxTokenData>) -> Self {
    match self.simplify() {
      Self::Identifier(location) => tokens
        .iter()
        .find(|d| d.location == location)
        .map_or_else(|| Self::Identifier(location), |d| d.data_type.clone()),
      Self::Params(location) => Self::Params(location),
      Self::Param(location, index) => Self::Param(location, index),
      Self::Return(val) => Self::Return(Box::new(val.resolve(tokens))),
      Self::Member {
        object,
        member,
        is_instance,
      } => Self::Member {
        object: Box::new(object.resolve(tokens)),
        member: Box::new(member.resolve(tokens)),
        is_instance,
      }
      .simplify(),
      Self::Fn { params, ret } => Self::Fn {
        params: params.into_iter().map(|p| p.resolve(tokens)).collect(),
        ret: Box::new(ret.resolve(tokens)),
      },
      Self::ConstructorFn { params, ret } => Self::ConstructorFn {
        params: params.into_iter().map(|p| p.resolve(tokens)).collect(),
        ret: Box::new(ret.resolve(tokens)),
      },
      Self::Class {
        props,
        instance_props,
        name,
      } => Self::Class {
        props: RefHash::new(
          props
            .deref()
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v.resolve(tokens)))
            .collect(),
        ),
        instance_props: RefHash::new(
          instance_props
            .deref()
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v.resolve(tokens)))
            .collect(),
        ),
        name: name,
      },
      Self::Instance { props, name } => Self::Instance {
        props: RefHash::new(
          props
            .deref()
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v.resolve(tokens)))
            .collect(),
        ),
        name: name,
      },
      Self::Reference(data_type) => Self::Reference(Box::new(data_type.resolve(tokens))),
      Self::Iterable(data_type) => Self::Iterable(Box::new(data_type.resolve(tokens))),
      Self::List(data_type) => Self::List(Box::new(data_type.resolve(tokens))),
      Self::Item(data_type) => Self::Item(Box::new(data_type.resolve(tokens))),
      Self::Promise(data_type) => Self::Promise(Box::new(data_type.resolve(tokens))),
      Self::Constructor(data_type) => Self::Constructor(Box::new(data_type.resolve(tokens))),
      Self::Multiple(data_types) => {
        Self::Multiple(data_types.into_iter().map(|d| d.resolve(tokens)).collect())
      }
      Self::UnPromise(data_type) => Self::UnPromise(Box::new(data_type.resolve(tokens))),
      Self::Call { callee, args } => Self::Call {
        callee: Box::new(callee.resolve(tokens)),
        args: args.into_iter().map(|d| d.resolve(tokens)).collect(),
      },
      Self::Unknown
      | Self::Char
      | Self::String
      | Self::StringLiteral(_)
      | Self::Number
      | Self::Byte
      | Self::Never
      | Self::Boolean
      | Self::Mod(_) => self,
    }
  }
}

// Se usa la referecia de un contador de referencias para no clonar y mantener los costes al minimo
// Solo se copia cuando se va a crear un hijo o el nodo sera analizado despues
// Esto aplica en las funciones type_scope, node_callback y node_scope

fn type_scope(locals: &Rc<Scope>, node: Option<&Node>) -> DataType {
  use DataType::*;
  if node.is_none() {
    return Unknown;
  };
  match node.unwrap() {
    Node::None => Unknown,
    Node::Program(_node_program) => todo!(),
    Node::String(data) => {
      let mut string = std::string::String::new();
      for data in &data.value {
        match data {
          crate::agal_parser::StringData::Id(_) => return String,
          crate::agal_parser::StringData::Str(s) => string.push_str(&s),
        }
      }
      StringLiteral(string)
    }
    Node::Number(_) => Number,
    Node::Object(node_object) => {
      let mut data_types = Vec::new();
      for element in &node_object.properties {
        match element {
          crate::agal_parser::NodeProperty::Property(_, value)
          | crate::agal_parser::NodeProperty::Dynamic(_, value) => {
            data_types.push(type_scope(locals, Some(&value)))
          }
          crate::agal_parser::NodeProperty::Indexable(value)
          | crate::agal_parser::NodeProperty::Iterable(value) => {
            match type_scope(locals, Some(&value)) {
              List(value) => data_types.push(*value),
              _ => {}
            }
          }
        }
      }
      Multiple(data_types).into_no_repeat()
    }
    Node::Array(node_array) => {
      let mut data_types = Vec::new();
      for element in &node_array.elements {
        match element {
          crate::agal_parser::NodeProperty::Property(_, value)
          | crate::agal_parser::NodeProperty::Indexable(value)
          | crate::agal_parser::NodeProperty::Dynamic(_, value) => {
            data_types.push(type_scope(locals, Some(&value)))
          }
          crate::agal_parser::NodeProperty::Iterable(value) => {
            match type_scope(locals, Some(&value)) {
              List(value) => data_types.push(*value),
              _ => {}
            }
          }
        }
      }
      List(Box::new(Multiple(data_types)))
    }
    Node::Byte(_) => Byte,
    Node::Identifier(node_identifier)
    | Node::VarDel(node_identifier)
    | Node::Name(node_identifier) => locals.get(&node_identifier.name).map_or_else(
      || Identifier(node_identifier.location.clone()),
      |t| t.data_type,
    ),
    Node::VarDecl(node_var_decl) => type_scope(locals, node_var_decl.value.as_deref()),
    Node::Assignment(node_assignment) => type_scope(locals, Some(&node_assignment.value)),
    Node::Class(_node_class) => todo!(),
    Node::Function(node_function) => Fn {
      params: node_function
        .params
        .enumerate()
        .map(|(index, param)| {
          if param.name.starts_with('@') {
            DataType::Params(param.location.clone())
          } else {
            DataType::Param(param.location.clone(), index)
          }
        })
        .collect(),
      ret: Box::new(type_scope(locals, Some(&node_function.body.to_node())).into_no_ret()),
    },
    Node::If(node_if) => {
      let mut multiple = vec![type_scope(&locals.child(), Some(&node_if.body.to_node()))];
      if let Some(body) = &node_if.else_body {
        multiple.push(type_scope(&locals.child(), Some(&body.to_node())));
      }
      Multiple(multiple).into_no_repeat()
    }
    Node::Import(node_import) => Mod(node_import.path.clone()),
    Node::Export(node_value) => type_scope(locals, Some(&node_value.value)),
    Node::For(node_for) => type_scope(&locals.child(), Some(&node_for.body.to_node())),
    Node::While(node_while) | Node::DoWhile(node_while) => {
      type_scope(&locals.child(), Some(&node_while.body.to_node()))
    }
    Node::Try(_node_try) => todo!(),
    Node::Throw(node_value) => type_scope(locals, Some(&node_value.value)),
    Node::Block(node_block, _) => {
      let mut multiple = vec![];
      let mut last = Never;
      for node in &node_block.body {
        let data_type = type_scope(locals, Some(&node));
        match &data_type {
          Multiple(data_types) => {
            for data_type in data_types {
              if let Return(_) = data_type {
                multiple.push(data_type.clone());
              }
            }
          }
          Return(_) => multiple.push(data_type.clone()),
          _ => {}
        }
        last = data_type;
      }
      multiple.push(last.clone());
      Multiple(multiple).into_no_repeat()
    }
    Node::Await(node_expression_medicator) => {
      type_scope(locals, Some(&node_expression_medicator.expression)).unpromise()
    }
    Node::Lazy(node_expression_medicator) => {
      type_scope(locals, Some(&node_expression_medicator.expression))
    }
    Node::Console(_) => String,
    Node::UnaryFront(node_unary) => match node_unary.operator {
      NodeOperator::Approximate | NodeOperator::Minus | NodeOperator::Plus => Number,
      NodeOperator::At => Iterable(Box::new(type_scope(locals, Some(&node_unary.operand)))),
      NodeOperator::Not | NodeOperator::QuestionMark => Boolean,
      NodeOperator::BitAnd => Reference(Box::new(type_scope(locals, Some(&node_unary.operand)))),
      _ => Unknown,
    },
    Node::Binary(node_binary) => match node_binary.operator {
      NodeOperator::And
      | NodeOperator::Equal
      | NodeOperator::GreaterThan
      | NodeOperator::GreaterThanOrEqual
      | NodeOperator::LessThan
      | NodeOperator::LessThanOrEqual
      | NodeOperator::NotEqual => Boolean,
      NodeOperator::BitAnd
      | NodeOperator::BitAndEqual
      | NodeOperator::BitMoveLeft
      | NodeOperator::BitMoveLeftEqual
      | NodeOperator::BitMoveRight
      | NodeOperator::BitMoveRightEqual
      | NodeOperator::BitOr
      | NodeOperator::BitOrEqual => Byte,
      NodeOperator::Division
      | NodeOperator::DivisionEqual
      | NodeOperator::Exponential
      | NodeOperator::ExponentialEqual
      | NodeOperator::Minus
      | NodeOperator::MinusEqual
      | NodeOperator::Modulo
      | NodeOperator::ModuloEqual
      | NodeOperator::Multiply
      | NodeOperator::MultiplyEqual
      | NodeOperator::TruncDivision
      | NodeOperator::TruncDivisionEqual => Number,
      NodeOperator::Nullish | NodeOperator::NullishEqual => {
        type_scope(locals, Some(&node_binary.left))
          .or_else(|| type_scope(locals, Some(&node_binary.right)))
      }
      NodeOperator::PipeLine => type_scope(locals, Some(&node_binary.right))
        .on_call(vec![type_scope(locals, Some(&node_binary.left))]),
      NodeOperator::Plus | NodeOperator::PlusEqual => {
        match (
          type_scope(locals, Some(&node_binary.left)),
          type_scope(locals, Some(&node_binary.right)),
        ) {
          (StringLiteral(a), StringLiteral(b)) => StringLiteral(format!("{a}{b}")),
          (String, _) | (_, String) | (StringLiteral(_), _) | (_, StringLiteral(_)) => String,
          (Number, _) | (_, Number) => Number,
          (Iterable(a), Iterable(b)) => Iterable(Box::new(Multiple(vec![*a, *b]).into_no_repeat())),
          (List(a), List(b)) => List(Box::new(Multiple(vec![*a, *b]).into_no_repeat())),
          (left, right) => left.or_else(|| right),
        }
      }
      _ => Unknown,
    },
    Node::Member(node_member) => {
      let object = type_scope(locals, Some(&node_member.object));
      if node_member.computed {
        let property = type_scope(locals, Some(&node_member.member));
        match (&object, property) {
          (String | StringLiteral(_), Number) => Char,
          (List(d), Number) => *d.clone(),
          (_, member) => Member {
            object: Box::new(object),
            member: Box::new(member),
            is_instance: node_member.instance,
          },
        }
      } else {
        match &*node_member.member {
          Node::Identifier(node_identifier) => match (&object, &node_identifier.name.as_str()) {
            (Mod(path), prop) => mod_types(path).get(prop).cloned(),
            (Instance { props, .. } | Class { props, .. }, prop) => props.get(*prop).cloned(),
            (val, prop) => proto_types(val).get(prop).cloned(),
          }
          .unwrap_or_else(|| Member {
            object: Box::new(object),
            member: Box::new(StringLiteral(node_identifier.name.clone())),
            is_instance: node_member.instance,
          }),
          _ => Unknown,
        }
      }
    }
    Node::Call(node_call) => type_scope(locals, Some(&node_call.callee)).on_call(
      node_call
        .arguments
        .map_ref(|n| type_scope(locals, Some(&n)))
        .into_iter()
        .collect(),
    ),
    Node::Return(node_return) => Return(Box::new(type_scope(locals, node_return.value.as_deref()))),
    Node::LoopEdit(_) => DataType::Never,
  }
}

fn node_callback(
  locals: &Rc<Scope>,
  node: &Node,
  argument: Option<&DataType>,
) -> (Vec<SyntaxTokenData>, Vec<(Rc<Scope>, Node)>) {
  match (node, argument) {
    (
      Node::Function(node_function),
      Some(DataType::Fn { params, .. } | DataType::ConstructorFn { params, .. }),
    ) => {
      let mut tokens = vec![];
      let mut nodes = vec![];

      let func_locals = locals.child();
      let mut params_list = vec![];
      for (index, param) in node_function.params.enumerate() {
        let (data_type, token_modifier) = if let Some(argument) = params.get(index) {
          (argument.clone(), vec![SyntaxTokenModifier::Constant])
        } else if param.name.starts_with('@') {
          (
            DataType::Params(param.location.clone()),
            vec![SyntaxTokenModifier::Iterable],
          )
        } else {
          (DataType::Param(param.location.clone(), index), vec![])
        };
        params_list.push(data_type.clone());
        let token = SyntaxTokenData {
          definition: param.location.start,
          token_type: SyntaxTokenType::Parameter,
          token_modifier,
          location: param.location.clone(),
          data_type,
          is_original_decl: false,
        };
        func_locals.insert(param.name.replace('@', ""), token.clone());
        tokens.push(token);
      }
      let token = SyntaxTokenData {
        definition: node_function.name.location.start,
        token_type: SyntaxTokenType::Function,
        token_modifier: vec![],
        location: node_function.name.location.clone(),
        is_original_decl: true,
        data_type: DataType::Fn {
          params: params_list,
          ret: Box::new(
            type_scope(&func_locals, Some(&node_function.body.to_node())).into_no_ret(),
          ),
        },
      };
      locals.insert(node_function.name.name.clone(), token.clone());
      nodes.push((func_locals, node_function.body.to_node()));
      tokens.push(token);

      (tokens, nodes)
    }
    (node, _) => node_scope(locals, &node),
  }
}

fn node_scope(locals: &Rc<Scope>, node: &Node) -> (Vec<SyntaxTokenData>, Vec<(Rc<Scope>, Node)>) {
  let mut tokens = vec![];
  let mut nodes = vec![];
  match node {
    Node::Program(node_program) => {
      for node in node_program.body.iter() {
        let (scope_tokens, scope_nodes) = node_scope(locals, &node);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
    }
    Node::String(node_string) => {
      for data in node_string.value.iter() {
        match data {
          crate::agal_parser::StringData::Id(id) => tokens.push(match locals.get(&id.name) {
            Some(definition) => SyntaxTokenData {
              definition: definition.definition,
              token_type: definition.token_type,
              is_original_decl: false,
              token_modifier: definition.token_modifier,
              location: id.location.clone(),
              data_type: definition.data_type,
            },
            None => SyntaxTokenData {
              definition: id.location.start,
              token_type: SyntaxTokenType::Variable,
              token_modifier: vec![],
              location: id.location.clone(),
              data_type: DataType::Unknown,
              is_original_decl: false,
            },
          }),
          crate::agal_parser::StringData::Str(_) => {}
        }
      }
    }
    Node::Object(node_object) => {
      for element in &node_object.properties {
        match element {
          crate::agal_parser::NodeProperty::Dynamic(key, value) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &key);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
            let (scope_tokens, scope_nodes) = node_scope(locals, &value);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
          }
          crate::agal_parser::NodeProperty::Indexable(item) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &item);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
          }
          crate::agal_parser::NodeProperty::Iterable(iter) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &iter);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
          }
          crate::agal_parser::NodeProperty::Property(node_identifier, value) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &value);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
            tokens.push(match locals.get(&node_identifier.name) {
              Some(definition) => SyntaxTokenData {
                definition: definition.definition,
                token_type: definition.token_type,
                token_modifier: definition.token_modifier,
                location: node_identifier.location,
                data_type: definition.data_type,
                is_original_decl: false,
              },
              None => SyntaxTokenData {
                definition: node_identifier.location.start,
                token_type: SyntaxTokenType::Variable,
                token_modifier: vec![],
                location: node_identifier.location,
                data_type: DataType::Unknown,
                is_original_decl: false,
              },
            })
          }
        }
      }
    }
    Node::Array(node_array) => {
      for element in node_array.elements.iter() {
        match element {
          crate::agal_parser::NodeProperty::Dynamic(key, value) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &key);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
            let (scope_tokens, scope_nodes) = node_scope(locals, &value);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
          }
          crate::agal_parser::NodeProperty::Indexable(item) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &item);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
          }
          crate::agal_parser::NodeProperty::Iterable(iter) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &iter);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
          }
          crate::agal_parser::NodeProperty::Property(node_identifier, value) => {
            let (scope_tokens, scope_nodes) = node_scope(locals, &value);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
            tokens.push(match locals.get(&node_identifier.name) {
              Some(definition) => SyntaxTokenData {
                definition: definition.definition,
                token_type: definition.token_type,
                token_modifier: definition.token_modifier,
                location: node_identifier.location.clone(),
                data_type: definition.data_type,
                is_original_decl: false,
              },
              None => SyntaxTokenData {
                definition: node_identifier.location.start,
                token_type: SyntaxTokenType::Variable,
                token_modifier: vec![],
                location: node_identifier.location.clone(),
                data_type: DataType::Unknown,
                is_original_decl: false,
              },
            })
          }
        }
      }
    }
    Node::Identifier(node_identifier) => tokens.push(match locals.get(&node_identifier.name) {
      Some(definition) => SyntaxTokenData {
        definition: definition.definition,
        token_type: definition.token_type,
        token_modifier: definition.token_modifier,
        location: node_identifier.location.clone(),
        data_type: definition.data_type,
        is_original_decl: false,
      },
      None => SyntaxTokenData {
        definition: node_identifier.location.start,
        token_type: SyntaxTokenType::Variable,
        token_modifier: vec![],
        location: node_identifier.location.clone(),
        data_type: DataType::Unknown,
        is_original_decl: false,
      },
    }),
    Node::VarDecl(node_var_decl) => {
      let mut token_modifier = vec![];
      if let Some(value) = &node_var_decl.value {
        let (scope_tokens, scope_nodes) = node_scope(locals, &value);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
      if node_var_decl.is_const {
        token_modifier.push(SyntaxTokenModifier::Constant);
      }
      let data_type = type_scope(locals, node_var_decl.value.as_deref());
      let token = SyntaxTokenData {
        definition: node_var_decl.name.location.start,
        token_type: data_type.prop_type(),
        token_modifier,
        location: node_var_decl.name.location.clone(),
        data_type,
        is_original_decl: false,
      };
      locals.insert(node_var_decl.name.name.clone(), token.clone());
      tokens.push(token);
    }
    Node::VarDel(node_identifier) => tokens.push(match locals.get(&node_identifier.name) {
      Some(definition) => SyntaxTokenData {
        definition: definition.definition,
        token_type: definition.token_type,
        token_modifier: definition.token_modifier,
        location: node_identifier.location.clone(),
        data_type: definition.data_type,
        is_original_decl: false,
      },
      None => SyntaxTokenData {
        definition: node_identifier.location.start,
        token_type: SyntaxTokenType::Variable,
        token_modifier: vec![],
        location: node_identifier.location.clone(),
        data_type: DataType::Unknown,
        is_original_decl: false,
      },
    }),
    Node::Name(node_identifier) => tokens.push(match locals.get(&node_identifier.name) {
      Some(definition) => SyntaxTokenData {
        definition: definition.definition,
        token_type: definition.token_type,
        token_modifier: definition.token_modifier,
        location: node_identifier.location.clone(),
        data_type: definition.data_type,
        is_original_decl: false,
      },
      None => SyntaxTokenData {
        definition: node_identifier.location.start,
        token_type: SyntaxTokenType::Variable,
        token_modifier: vec![],
        location: node_identifier.location.clone(),
        data_type: DataType::Unknown,
        is_original_decl: false,
      },
    }),
    Node::Assignment(node_assignment) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_assignment.identifier);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_assignment.value);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
    }
    Node::Class(node_class) => {
      let mut static_props: HashMap<String, DataType> = Default::default();
      let mut instance_props: HashMap<String, DataType> = Default::default();
      for prop in &node_class.body {
        let (scope_tokens, scope_nodes) = node_scope(locals, &prop.value);
        if prop.meta & 1 == 1 {
          static_props.insert(prop.name.name, type_scope(locals, Some(&prop.value)))
        } else {
          instance_props.insert(prop.name.name, type_scope(locals, Some(&prop.value)))
        };
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
      let token = SyntaxTokenData {
        definition: node_class.name.location.start,
        token_type: SyntaxTokenType::Class,
        token_modifier: vec![],
        location: node_class.name.location.clone(),
        data_type: DataType::Class {
          props: RefHash::new(static_props),
          instance_props: RefHash::new(instance_props),
          name: node_class.name.name.clone(),
        },
        is_original_decl: false,
      };
      locals.insert(node_class.name.name.clone(), token.clone());
      tokens.push(token);
    }
    Node::Function(node_function) => {
      let func_locals = locals.child();
      let mut params = vec![];
      for (index, param) in node_function.params.enumerate() {
        let (data_type, token_modifier) = if param.name.starts_with('@') {
          (
            DataType::Params(param.location.clone()),
            vec![SyntaxTokenModifier::Iterable],
          )
        } else {
          (DataType::Param(param.location.clone(), index), vec![])
        };
        params.push(data_type.clone());
        let token = SyntaxTokenData {
          definition: param.location.start,
          token_type: SyntaxTokenType::Parameter,
          token_modifier,
          location: param.location.clone(),
          data_type,
          is_original_decl: false,
        };
        func_locals.insert(param.name.replace('@', ""), token.clone());
        tokens.push(token);
      }
      let token = SyntaxTokenData {
        definition: node_function.name.location.start,
        token_type: SyntaxTokenType::Function,
        token_modifier: vec![],
        location: node_function.name.location.clone(),
        is_original_decl: true,
        data_type: DataType::Fn {
          params,
          ret: Box::new(
            type_scope(&func_locals, Some(&node_function.body.to_node())).into_no_ret(),
          ),
        },
      };
      locals.insert(node_function.name.name.clone(), token.clone());
      nodes.push((func_locals, node_function.body.to_node()));
      tokens.push(token);
    }
    Node::If(node_if) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_if.condition);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let scope_if = locals.child();
      for node in node_if.body.iter() {
        let (scope_tokens, scope_nodes) = node_scope(&scope_if, &node);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
      if let Some(body) = &node_if.else_body {
        let scope_else = locals.child();
        for node in body.iter() {
          let (scope_tokens, scope_nodes) = node_scope(&scope_else, &node);
          tokens.extend(scope_tokens);
          nodes.extend(scope_nodes);
        }
      }
    }
    Node::Import(node_import) => {
      if let Some(identifier) = node_import.name.clone() {
        let token = SyntaxTokenData {
          definition: identifier.location.start,
          token_type: SyntaxTokenType::Module,
          token_modifier: vec![],
          location: identifier.location,
          data_type: DataType::Mod(node_import.path.clone()),
          is_original_decl: true,
        };
        locals.insert(identifier.name, token.clone());
        tokens.push(token);
      }
    }
    Node::Export(node_value) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_value.value);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
    }
    Node::For(node_for) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_for.init);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_for.condition);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_for.update);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let for_locals = locals.child();
      for node in node_for.body.iter() {
        let (scope_tokens, scope_nodes) = node_scope(&for_locals, &node);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
    }
    Node::While(node_while) | Node::DoWhile(node_while) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_while.condition);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let while_locals = locals.child();
      for node in node_while.body.iter() {
        let (scope_tokens, scope_nodes) = node_scope(&while_locals, &node);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
    }
    Node::Try(node_try) => {
      let try_locals = locals.child();
      for node in node_try.body.iter() {
        let (scope_tokens, scope_nodes) = node_scope(&try_locals, &node);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
      if let Some((_, body)) = &node_try.catch {
        let catch_locals = locals.child();
        for node in body.iter() {
          let (scope_tokens, scope_nodes) = node_scope(&catch_locals, &node);
          tokens.extend(scope_tokens);
          nodes.extend(scope_nodes);
        }
      }
      if let Some(body) = &node_try.finally {
        let finally_locals = locals.child();
        for node in body.iter() {
          let (scope_tokens, scope_nodes) = node_scope(&finally_locals, &node);
          tokens.extend(scope_tokens);
          nodes.extend(scope_nodes);
        }
      }
    }
    Node::Throw(node_value) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_value.value);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
    }
    Node::Block(node_block, _) => {
      for node in node_block.body.iter() {
        let (scope_tokens, scope_nodes) = node_scope(locals, &node);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
    }
    Node::Await(node_expression_medicator) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_expression_medicator.expression);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
    }
    Node::Lazy(node_expression_medicator) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_expression_medicator.expression);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
    }
    Node::Console(node_console) => match node_console {
      crate::agal_parser::NodeConsole::Output { value, .. } => {
        let (scope_tokens, scope_nodes) = node_scope(locals, &value);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
      crate::agal_parser::NodeConsole::Input {
        location,
        identifier,
      } => {
        let token = match locals.get(&identifier.name) {
          Some(definition) => SyntaxTokenData {
            definition: definition.definition,
            token_type: definition.token_type,
            token_modifier: definition.token_modifier,
            location: location.clone(),
            data_type: definition.data_type,
            is_original_decl: false,
          },
          None => SyntaxTokenData {
            definition: location.start,
            token_type: SyntaxTokenType::Variable,
            is_original_decl: false,
            token_modifier: vec![],
            location: location.clone(),
            data_type: DataType::String,
          },
        };
        locals.insert(identifier.name.clone(), token.clone());
        tokens.push(token);
      }
      crate::agal_parser::NodeConsole::Full {
        value,
        location,
        identifier,
      } => {
        let (scope_tokens, scope_nodes) = node_scope(locals, &value);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
        let token = match locals.get(&identifier.name) {
          Some(definition) => SyntaxTokenData {
            definition: definition.definition,
            token_type: definition.token_type,
            token_modifier: definition.token_modifier,
            location: location.clone(),
            data_type: definition.data_type,
            is_original_decl: false,
          },
          None => SyntaxTokenData {
            definition: location.start,
            token_type: SyntaxTokenType::Variable,
            token_modifier: vec![],
            location: location.clone(),
            data_type: DataType::String,
            is_original_decl: false,
          },
        };
        locals.insert(identifier.name.clone(), token.clone());
        tokens.push(token);
      }
    },
    Node::Binary(node_binary) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_binary.left);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_binary.right);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
    }
    Node::Member(node_member) => {
      let (scope_tokens, scope_nodes) = node_scope(locals, &node_member.object);
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      if node_member.computed {
        let (scope_tokens, scope_nodes) = node_scope(locals, &node_member.member);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      } else {
        match &*node_member.member {
          Node::Identifier(id) => {
            let data_type = DataType::Member {
              object: Box::new(type_scope(locals, Some(&node_member.object))),
              member: Box::new(DataType::StringLiteral(id.name.clone())),
              is_instance: node_member.instance,
            };
            tokens.push(SyntaxTokenData {
              definition: id.location.end,
              token_type: data_type.prop_type(),
              token_modifier: vec![SyntaxTokenModifier::Constant],
              location: id.location.clone(),
              data_type,
              is_original_decl: false,
            })
          }
          _ => {}
        }
      };
    }
    Node::Call(node_call) => {
      let (scope_tokens, scope_nodes, args) = match &*node_call.callee {
        Node::Member(node_member) => {
          let mut tokens = vec![];
          let mut nodes = vec![];
          let (scope_tokens, scope_nodes) = node_scope(locals, &node_member.object);
          tokens.extend(scope_tokens);
          nodes.extend(scope_nodes);
          let args = if node_member.computed {
            let (scope_tokens, scope_nodes) = node_scope(locals, &node_member.member);
            tokens.extend(scope_tokens);
            nodes.extend(scope_nodes);
            vec![]
          } else {
            match &*node_member.member {
              Node::Identifier(id) => {
                let data_type = type_scope(locals, Some(&node_call.callee)).infer_call(
                  &node_call
                    .arguments
                    .map_ref(|n| type_scope(locals, Some(n)))
                    .collect(),
                );
                let args = data_type.get_params();
                tokens.push(SyntaxTokenData {
                  definition: id.location.end,
                  token_type: data_type.prop_type(),
                  token_modifier: vec![SyntaxTokenModifier::Constant],
                  location: id.location.clone(),
                  data_type,
                  is_original_decl: false,
                });
                args
              }
              _ => vec![],
            }
          };
          (tokens, nodes, args)
        }
        Node::Identifier(node_identifier) => {
          let data_type = type_scope(locals, Some(&node_call.callee)).infer_call(
            &node_call
              .arguments
              .map_ref(|n| type_scope(locals, Some(n)))
              .collect(),
          );
          let args = data_type.get_params();
          let token = SyntaxTokenData {
            definition: locals
              .get(&node_identifier.name)
              .map_or(node_identifier.location.start, |d| d.definition),
            token_type: data_type.prop_type(),
            token_modifier: vec![],
            location: node_identifier.location.clone(),
            data_type,
            is_original_decl: false,
          };
          locals.insert(node_identifier.name.clone(), token.clone());
          (vec![token], vec![], args)
        }
        callee => {
          let (scope_tokens, scope_nodes) = node_scope(locals, &callee);
          (scope_tokens, scope_nodes, vec![])
        }
      };
      tokens.extend(scope_tokens);
      nodes.extend(scope_nodes);
      let mut diff = 0;
      for (i, node) in node_call.arguments.enumerate() {
        let (scope_tokens, scope_nodes) = node_callback(locals, node, args.get(i - diff));
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
        if let Some(DataType::Iterable(_)) = args.get(i - diff) {
          diff += 1;
        }
      }
    }
    Node::Return(node_return) => {
      tokens.push(SyntaxTokenData {
        definition: node_return.location.end,
        token_type: SyntaxTokenType::KeywordControl,
        token_modifier: vec![],
        location: node_return.location.clone(),
        data_type: DataType::Return(Box::new(type_scope(locals, node_return.value.as_deref()))),
        is_original_decl: false,
      });
      if let Some(value) = &node_return.value {
        let (scope_tokens, scope_nodes) = node_scope(locals, value);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
    }
    Node::UnaryFront(ref node_unary) => {
      if node_unary.operator == NodeOperator::At {
        match &*node_unary.operand {
          Node::Identifier(node_identifier) => {
            let location = Location {
              start: node_unary.location.start,
              end: node_identifier.location.end,
              file_name: node_identifier.location.file_name.clone(),
              length: node_identifier.location.length,
            };
            let syntax = match locals.get(&node_identifier.name) {
              Some(definition) => SyntaxTokenData {
                definition: definition.definition,
                token_type: definition.token_type,
                token_modifier: definition.token_modifier,
                location: node_identifier.location.clone(),
                data_type: definition.data_type,
                is_original_decl: false,
              },
              None => SyntaxTokenData {
                definition: node_identifier.location.start,
                token_type: SyntaxTokenType::Variable,
                token_modifier: vec![],
                location: node_identifier.location.clone(),
                data_type: type_scope(locals, Some(node)),
                is_original_decl: false,
              },
            };
            let mut token_modifier = syntax.token_modifier;
            token_modifier.push(SyntaxTokenModifier::Iterable);
            tokens.push(SyntaxTokenData {
              definition: syntax.definition,
              token_type: syntax.token_type,
              token_modifier,
              location,
              data_type: DataType::Iterable(Box::new(syntax.data_type)),
              is_original_decl: false,
            });
          }
          _ => {}
        }
      } else {
        let (scope_tokens, scope_nodes) = node_scope(locals, &*node_unary.operand);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
      }
    }
    _ => {}
  }
  (tokens, nodes)
}
fn node_export(locals: &Rc<Scope>, node: &Node) -> Vec<(String, DataType)> {
  let mut exports = vec![];
  match node {
    Node::Program(node_program) => {
      for node in node_program.body.iter() {
        exports.extend(node_export(locals, &node));
      }
    }
    Node::VarDecl(node_var_decl) => {
      let mut token_modifier = vec![];
      if node_var_decl.is_const {
        token_modifier.push(SyntaxTokenModifier::Constant);
      }
      let data_type = type_scope(locals, node_var_decl.value.as_deref());
      exports.push((node_var_decl.name.name.clone(), data_type));
    }
    Node::Name(node_identifier) => exports.push((
      node_identifier.name.clone(),
      match locals.get(&node_identifier.name) {
        Some(definition) => definition.data_type,
        None => DataType::Unknown,
      },
    )),
    Node::Class(node_class) => {
      let data_type = type_scope(locals, Some(&Node::Class(node_class.clone())));
      exports.push((node_class.name.name.clone(), data_type));
    }
    Node::Function(node_function) => {
      let data_type = type_scope(locals, Some(&Node::Function(node_function.clone())));
      exports.push((node_function.name.name.clone(), data_type));
    }
    Node::Export(node_value) => {
      exports.extend(node_export(locals, &node_value.value));
    }
    Node::Block(node_block, _) => {
      for node in node_block.body.iter() {
        exports.extend(node_export(locals, &node));
      }
    }
    _ => {}
  };
  exports
}

fn proto_types(data: &DataType) -> RefHash<HashMap<&'static str, DataType>> {
  use DataType::*;
  let mut proto_props: HashMap<&str, DataType> = Default::default();
  match data.simplify() {
    String | StringLiteral(_) => {
      proto_props.insert(
        "reemplaza",
        Fn {
          params: vec![String, String],
          ret: Box::new(String),
        },
      );
      proto_props.insert(
        "separa",
        Fn {
          params: vec![String],
          ret: Box::new(String),
        },
      );
      proto_props.insert(
        "repite",
        Fn {
          params: vec![Number],
          ret: Box::new(String),
        },
      );
      proto_props.insert(
        "bytes",
        Fn {
          params: vec![],
          ret: Box::new(List(Box::new(Byte))),
        },
      );
    }
    Constructor(_) | ConstructorFn { .. } | Fn { .. } => {
      let param_1 = Param(
        Location {
          start: Default::default(),
          end: Default::default(),
          length: 0,
          file_name: "llama".to_string(),
        },
        0,
      );
      let params = Params(Location {
        start: Default::default(),
        end: Default::default(),
        length: 0,
        file_name: "llama".to_string(),
      });
      proto_props.insert(
        "llama",
        Fn {
          params: vec![param_1.clone(), params.clone()],
          ret: Box::new(data.on_call(vec![param_1, Item(Box::new(params))])),
        },
      );
    }
    List(_) => {
      proto_props.insert("longitud", Number);
    }
    Reference(inner) => return proto_types(&inner),
    _ => {}
  }
  RefHash::new(proto_props)
}
fn mod_types(module: &str) -> RefHash<HashMap<&str, DataType>> {
  use DataType::*;
  let mut exports: HashMap<&str, DataType> = Default::default();
  match module {
    ":constructores" => {
      exports.insert("Cadena", Constructor(Box::new(String)));
      exports.insert(
        "Lista",
        ConstructorFn {
          params: vec![Params(Location {
            start: Default::default(),
            end: Default::default(),
            length: Default::default(),
            file_name: module.to_string(),
          })],
          ret: Box::new(List(Box::new(Item(Box::new(Params(Location {
            start: Default::default(),
            end: Default::default(),
            length: Default::default(),
            file_name: module.to_string(),
          })))))),
        },
      );
    }
    ":sa" => {
      exports.insert(
        "leer_archivo",
        Fn {
          params: vec![String],
          ret: Box::new(List(Box::new(Byte))),
        },
      );
      exports.insert(
        "escribir_archivo",
        Fn {
          params: vec![String, Multiple(vec![String, List(Box::new(Byte))])],
          ret: Box::new(Never),
        },
      );
      exports.insert(
        "leer_carpeta",
        Fn {
          params: vec![String],
          ret: Box::new(List(Box::new(String))),
        },
      );
      exports.insert(
        "crear_archivo",
        Fn {
          params: vec![String],
          ret: Box::new(Never),
        },
      );
      exports.insert(
        "borrar_archivo",
        Fn {
          params: vec![String],
          ret: Box::new(Never),
        },
      );
      exports.insert(
        "crear_carpeta",
        Fn {
          params: vec![String],
          ret: Box::new(Never),
        },
      );
      exports.insert(
        "borrar_carpeta",
        Fn {
          params: vec![String],
          ret: Box::new(Never),
        },
      );
      exports.insert("Ruta", {
        let mut instance_props: HashMap<std::string::String, DataType> = Default::default();
        instance_props.insert(
          "es_archivo".to_string(),
          Fn {
            params: vec![],
            ret: Box::new(Boolean),
          },
        );
        instance_props.insert(
          "es_archivo".to_string(),
          Fn {
            params: vec![],
            ret: Box::new(Boolean),
          },
        );
        instance_props.insert(
          "obtener_padre".to_string(),
          Fn {
            params: vec![],
            ret: Box::new(String),
          },
        );
        instance_props.insert(
          "obtener_nombre".to_string(),
          Fn {
            params: vec![],
            ret: Box::new(String),
          },
        );
        instance_props.insert(
          "obtener_extension".to_string(),
          Fn {
            params: vec![],
            ret: Box::new(String),
          },
        );
        Class {
          props: Default::default(),
          instance_props: RefHash::new(instance_props),
          name: "Ruta".to_string(),
        }
      });
    }
    ":consola" => {
      exports.insert(
        "pinta",
        Fn {
          params: vec![Params(Location {
            start: Default::default(),
            end: Default::default(),
            length: Default::default(),
            file_name: module.to_string(),
          })],
          ret: Box::new(Never),
        },
      );
      exports.insert(
        "inspecciona",
        Fn {
          params: vec![Param(
            Location {
              start: Default::default(),
              end: Default::default(),
              length: Default::default(),
              file_name: module.to_string(),
            },
            0,
          )],
          ret: Box::new(String),
        },
      );
    }
    ":red" => {
      let buffer = List(Box::new(Byte));
      let valid_buffer = Multiple(vec![buffer.clone(), String]);
      let servidor_tcp = {
        let mut instance_props: HashMap<std::string::String, DataType> = Default::default();
        instance_props.insert("promesa".to_string(), Promise(Box::new(Never)));
        instance_props.insert("puerto".to_string(), Number);
        instance_props.insert("ip".to_string(), String);
        Instance {
          props: RefHash::new(instance_props),
          name: "ServidorTCP".to_string(),
        }
      };
      let socket = {
        let mut instance_props: HashMap<std::string::String, DataType> = Default::default();
        instance_props.insert(
          "lee".to_string(),
          Fn {
            params: vec![],
            ret: Box::new(buffer),
          },
        );
        instance_props.insert(
          "escribe".to_string(),
          Fn {
            params: vec![valid_buffer],
            ret: Box::new(Never),
          },
        );
        instance_props.insert(
          "cierra".to_string(),
          Fn {
            params: vec![],
            ret: Box::new(Never),
          },
        );
        instance_props.insert("puerto".to_string(), Number);
        instance_props.insert("ip".to_string(), String);
        Instance {
          props: RefHash::new(instance_props),
          name: "Socket".to_string(),
        }
      };
      exports.insert(
        "ServidorTCP",
        ConstructorFn {
          params: vec![
            String,
            Fn {
              params: vec![socket],
              ret: Box::new(Never),
            },
          ],
          ret: Box::new(servidor_tcp),
        },
      );
    }
    _ => {}
  };
  RefHash::new(exports)
}
pub fn print_tokens(node: Node) {
  let scope = Default::default();
  let (mut tokens, mut nodes) = node_scope(&scope, &node);
  let mut index = 0;
  loop {
    match nodes.get(index).cloned() {
      None => break,
      Some((scope, node)) => {
        let (scope_tokens, scope_nodes) = node_scope(&scope, &node);
        tokens.extend(scope_tokens);
        nodes.extend(scope_nodes);
        index += 1;
      }
    }
  }
  let mut module: HashMap<&str, DataType> = Default::default();
  let exports = node_export(&scope, &node);
  for (key, data) in &exports {
    module.insert(key, data.clone());
  }
  println!(
    "{{\"file\":{},\"mod\":{}}}",
    tokens.clone().resolve(&tokens).to_json(),
    module.to_json()
  )
}
