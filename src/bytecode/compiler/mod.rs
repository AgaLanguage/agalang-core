use std::collections::HashMap;

mod chunk;

use crate::parser::{Node, NodeFunction};
pub use chunk::{ChunkGroup, OpCode};

use super::value::{Function, Number, Value, NEVER_NAME};

const OBJECT_MEMBER: u8 = 0;
const INSTANCE_MEMBER: u8 = 1;

pub struct Compiler {
  pub function: Function,
  pub path: String,
}
impl Compiler {
  fn parse_function(function: &NodeFunction) -> Result<Function, String> {
    let mut compiler = Self {
      function: function.into(),
      path: function.location.file_name.clone(),
    };
    let mut has_rest = false;
    let mut rest_param = None;
    for param in &function.params {
      if has_rest {
        return Err(format!(
          "El paramatro expandido {} debe estar al final de la lista de parametros",
          rest_param.unwrap()
        ));
      }
      let name = if param.name.starts_with('@') {
        has_rest = true;
        rest_param = Some(param.name.clone());
        param.name.replace('@', "").into()
      } else {
        param.name.clone()
      };
      let _global = compiler
        .function
        .chunk()
        .make_arg(name, param.location.start.line);
    }
    compiler.function.set_rest(has_rest);
    if function.is_async {
      compiler.write(OpCode::OpPromised as u8, function.location.start.line);
    }
    if function.body.len() > 0 {
      compiler.node_to_bytes(&function.body.clone().to_node())?;
    } else {
      compiler.set_constant(Value::Never, function.location.end.line);
    }
    compiler.write(OpCode::OpReturn as u8, function.location.end.line);
    Ok(compiler.function)
  }
  fn set_constant(&mut self, value: Value, line: usize) -> u8 {
    self.function.chunk().write_constant(value, line)
  }
  fn set_value(&mut self, value: Value) -> u8 {
    self.function.chunk().add_value(value)
  }
  fn write(&mut self, byte: u8, line: usize) {
    self.function.chunk().write(byte, line);
  }
  fn write_buffer(&mut self, buffer: Vec<u8>, line: usize) {
    self.function.chunk().write_buffer(buffer, line);
  }
  fn len(&mut self) -> usize {
    self.function.chunk().len()
  }
  fn read_var(&mut self, name: String, line: usize) {
    self.function.chunk().read_var(name, line);
  }
  fn jump(&mut self, code: OpCode) -> usize {
    self.function.chunk().jump(code)
  }
  fn patch_jump(&mut self, offset: usize) -> Result<(), String> {
    self.function.chunk().patch_jump(offset)
  }
  fn add_loop(&mut self, offset: usize) -> Result<(), String> {
    self.function.chunk().add_loop(offset)
  }
  fn node_to_bytes(&mut self, node: &Node) -> Result<(), String> {
    match node {
      Node::Number(n) => {
        let number: Number = if n.base == 10u8 {
          n.value.parse().unwrap()
        } else {
          Number::from_str_radix(&n.value, n.base)
        };
        self.set_constant(Value::Number(number), n.location.start.line);
      }
      Node::Byte(n) => {
        self.set_constant(Value::Byte(n.value), n.location.start.line);
      }
      Node::Binary(b) => {
        self.node_to_bytes(&b.left)?;
        self.node_to_bytes(&b.right)?;
        let operator = match b.operator {
          crate::parser::NodeOperator::TruncDivision => {
            vec![OpCode::OpDivide as u8, OpCode::OpApproximate as u8]
          }
          crate::parser::NodeOperator::Division => vec![OpCode::OpDivide as u8],
          crate::parser::NodeOperator::Minus => vec![OpCode::OpSubtract as u8],
          crate::parser::NodeOperator::Multiply => vec![OpCode::OpMultiply as u8],
          crate::parser::NodeOperator::Plus => vec![OpCode::OpAdd as u8],
          crate::parser::NodeOperator::GreaterThan => vec![OpCode::OpGreaterThan as u8],
          crate::parser::NodeOperator::Equal => vec![OpCode::OpEquals as u8],
          crate::parser::NodeOperator::LessThan => vec![OpCode::OpLessThan as u8],
          crate::parser::NodeOperator::Modulo => vec![OpCode::OpModulo as u8],
          crate::parser::NodeOperator::And => vec![OpCode::OpAnd as u8],
          crate::parser::NodeOperator::Or => vec![OpCode::OpOr as u8],
          a => {
            return Err(format!(
              "NodeOperator::{a:?}: No es un nodo valido en bytecode"
            ))
          }
        };
        self.write_buffer(operator, b.location.start.line);
      }
      Node::Program(p) => {
        if p.body.len() != 0 {
          self.node_to_bytes(&p.body.clone().to_node())?;
          self.write(OpCode::OpPop as u8, p.location.start.line);
        }
        self.set_constant(Value::Never, p.location.start.line);
        self.write(OpCode::OpReturn as u8, p.location.end.line);
      }
      Node::Block(b, _is_async) => {
        self.write(OpCode::OpNewLocals as u8, b.location.start.line);
        let code_len = b.body.len();
        for (index, node) in b.body.clone().enumerate() {
          self.node_to_bytes(&node)?;
          if index < (code_len - 1) {
            self.write(OpCode::OpPop as u8, node.get_location().end.line);
          }
        }
        self.write(OpCode::OpRemoveLocals as u8, b.location.start.line);
      }
      Node::UnaryFront(u) => {
        self.node_to_bytes(&u.operand)?;
        let operator = match &u.operator {
          crate::parser::NodeOperator::Approximate => OpCode::OpApproximate,
          crate::parser::NodeOperator::QuestionMark => OpCode::OpAsBoolean,
          crate::parser::NodeOperator::Minus => OpCode::OpNegate,
          crate::parser::NodeOperator::BitAnd => OpCode::OpAsRef,
          crate::parser::NodeOperator::Not => OpCode::OpNot,
          crate::parser::NodeOperator::At => OpCode::OpAt,
          op => {
            return Err(format!(
              "NodeOperator::{op:?}: No es un nodo valido en bytecode"
            ))
          }
        } as u8;
        self.write(operator, u.location.start.line);
      }
      Node::Identifier(i) => {
        self.read_var(i.name.clone(), i.location.start.line);
      }
      Node::Console(c) => match c {
        crate::parser::NodeConsole::Output { value, location } => {
          self.node_to_bytes(&value)?;
          self.write(OpCode::OpConsoleOut as u8, location.start.line);
        }
        _ => {}
      },
      Node::String(s) => {
        for (i, data) in s.value.clone().enumerate() {
          match data {
            crate::parser::StringData::Str(val) => {
              self.set_constant(Value::String(val.as_str().into()), s.location.start.line);
            }
            crate::parser::StringData::Id(id) => {
              self.read_var(id, s.location.start.line);
            }
          }
          if i != 0 {
            self.write(OpCode::OpAdd as u8, s.location.start.line);
          }
        }
      }
      Node::VarDecl(v) => {
        let op;
        if v.is_const {
          match &v.value {
            Some(value) => {
              self.node_to_bytes(&value)?;
            }
            None => {
              return Err(format!(
                "No se puede asignar '{}' a una constante",
                NEVER_NAME
              ))
            }
          }
          op = OpCode::OpConstDecl as u8;
        } else {
          match &v.value {
            Some(value) => {
              self.node_to_bytes(&value)?;
            }
            None => {
              self.set_constant(Value::Never, v.location.start.line);
            }
          };
          op = OpCode::OpVarDecl as u8;
        }
        let name = self.set_value(Value::String(v.name.as_str().into()));
        self.write_buffer(vec![op, name], v.location.start.line);
      }
      Node::Assignment(a) => {
        match a.identifier.as_ref() {
          Node::Identifier(id) => {
            self.node_to_bytes(&a.value)?;
            let name = self.set_value(Value::String(id.name.as_str().into()));
            self.write_buffer(vec![OpCode::OpSetVar as u8, name], a.location.start.line);
          }
          Node::Member(m) => {
            self.node_to_bytes(&m.object)?;
            if m.computed {
              self.node_to_bytes(&m.member)?;
            } else {
              let name = match m.member.as_ref() {
                Node::Identifier(id) => id.name.as_str(),
                _ => return Err("Se esperaba un identificador como propiedad".to_string()),
              };
              self.set_constant(Value::String(name.into()), m.location.start.line);
            };
            self.node_to_bytes(&a.value)?;
            if m.instance {
              return Err("No se puede asignar a una propiedad de instancia".to_string());
            }
            self.write_buffer(
              vec![OpCode::OpSetMember as u8, OBJECT_MEMBER],
              m.location.start.line,
            );
          }
          _ => return Err("Se esperaba una assignacion valida".to_string()),
        };
      }
      Node::If(i) => {
        self.node_to_bytes(&i.condition)?;
        let jump_if = self.jump(OpCode::OpJumpIfFalse);
        self.node_to_bytes(&i.body.clone().to_node())?;

        let jump_else = self.jump(OpCode::OpJump);
        self.patch_jump(jump_if)?;

        if let Some(e) = &i.else_body {
          self.node_to_bytes(&e.clone().to_node())?;
        } else {
          self.set_constant(Value::Never, i.location.start.line);
        }
        self.patch_jump(jump_else)?;
      }
      Node::While(i) => {
        let loop_start = self.len();
        self.node_to_bytes(&i.condition)?;
        let jump_while = self.jump(OpCode::OpJumpIfFalse);
        self.node_to_bytes(&i.body.clone().to_node())?;
        if i.body.len() > 0 {
          self.write(OpCode::OpPop as u8, 0);
        }
        self.add_loop(loop_start)?;
        self.patch_jump(jump_while)?;
        self.set_constant(Value::Never, i.location.start.line);
      }
      Node::DoWhile(i) => {
        let jump_do = self.jump(OpCode::OpJump);
        let loop_start = self.len();
        self.node_to_bytes(&i.condition)?;
        let jump_do_while = self.jump(OpCode::OpJumpIfFalse);
        self.patch_jump(jump_do)?;
        self.node_to_bytes(&i.body.clone().to_node())?;
        if i.body.len() > 0 {
          self.write(OpCode::OpPop as u8, 0);
        }
        self.add_loop(loop_start)?;
        self.patch_jump(jump_do_while)?;
        self.set_constant(Value::Never, i.location.start.line);
      }
      Node::For(f) => {
        self.write(OpCode::OpNewLocals as u8, f.location.start.line);
        self.node_to_bytes(&f.init)?;
          self.write(OpCode::OpPop as u8, 0);
        let loop_start = self.len();
        self.node_to_bytes(&f.condition)?;
        let jump_for = self.jump(OpCode::OpJumpIfFalse);
        self.node_to_bytes(&f.body.clone().to_node())?;
        if f.body.len() > 0 {
          self.write(OpCode::OpPop as u8, 0);
        }
        self.node_to_bytes(&f.update)?;
        self.write(OpCode::OpPop as u8, 0);
        self.add_loop(loop_start)?;
        self.patch_jump(jump_for)?;
        self.write(OpCode::OpRemoveLocals as u8, f.location.start.line);
        self.set_constant(Value::Never, f.location.end.line);
      }
      Node::Function(f) => {
        let function = Value::Object(Self::parse_function(f)?.into());
        self.set_constant(function.clone(), f.location.start.line);

        let name = self.set_value(Value::String(f.name.as_str().into()));
        self.write_buffer(
          vec![OpCode::OpSetScope as u8, OpCode::OpConstDecl as u8, name],
          f.location.start.line,
        );
      }
      Node::Call(c) => {
        for arg in &c.arguments {
          self.node_to_bytes(&arg)?;
        }
        match c.callee.as_ref() {
          Node::Member(m) => {
            self.node_to_bytes(&m.object)?;
            self.write(OpCode::OpCopy as u8, m.object.get_location().end.line);
            if m.computed {
              self.node_to_bytes(&m.member)?;
            } else {
              let name = match m.member.as_ref() {
                Node::Identifier(id) => id.name.as_str(),
                _ => return Err("Se esperaba un identificador como propiedad".to_string()),
              };
              self.set_constant(Value::String(name.into()), m.location.start.line);
            };
            let is_instance = if m.instance {
              INSTANCE_MEMBER
            } else {
              OBJECT_MEMBER
            };
            self.write_buffer(
              vec![OpCode::OpGetMember as u8, is_instance],
              m.location.start.line,
            );
          }
          Node::Identifier(i) => {
            self.read_var(i.name.clone(), i.location.start.line);
            self.write(OpCode::OpCopy as u8, node.get_location().start.line)
          }
          node => {
            self.node_to_bytes(&c.callee)?;
            self.write(OpCode::OpCopy as u8, node.get_location().start.line)
          }
        };

        self.write_buffer(
          vec![OpCode::OpCall as u8, c.arguments.len() as u8],
          c.location.start.line,
        );
      }
      Node::Return(r) => {
        match &r.value {
          Some(value) => {
            self.node_to_bytes(&value)?;
          }
          None => {
            self.set_constant(Value::Never, r.location.start.line);
          }
        };
        self.write(OpCode::OpReturn as u8, r.location.start.line);
      }
      Node::Object(o) => {
        let value = Value::Object(HashMap::new().into());
        for p in o.properties.clone() {
          self.set_constant(value.clone(), o.location.start.line);
          match p {
            crate::parser::NodeProperty::Dynamic(key, value) => {
              self.node_to_bytes(&key)?;
              self.node_to_bytes(&value)?;
              self.write_buffer(
                vec![
                  OpCode::OpSetMember as u8,
                  OBJECT_MEMBER,
                  OpCode::OpPop as u8,
                ],
                o.location.start.line,
              );
            }
            crate::parser::NodeProperty::Property(key, value) => {
              self.set_constant(Value::String(key), o.location.start.line);
              self.node_to_bytes(&value)?;
              self.write_buffer(
                vec![
                  OpCode::OpSetMember as u8,
                  OBJECT_MEMBER,
                  OpCode::OpPop as u8,
                ],
                o.location.start.line,
              );
            }
            _ => {}
          };
        }
        self.set_constant(value, o.location.start.line);
      }
      Node::Member(m) => {
        self.node_to_bytes(&m.object)?;
        if m.computed {
          self.node_to_bytes(&m.member)?;
        } else {
          let name = match m.member.as_ref() {
            Node::Identifier(id) => id.name.as_str(),
            _ => return Err("Se esperaba un identificador como propiedad".to_string()),
          };
          self.set_constant(Value::String(name.into()), m.location.start.line);
        };
        let is_instance = if m.instance {
          INSTANCE_MEMBER
        } else {
          OBJECT_MEMBER
        };
        self.write_buffer(
          vec![OpCode::OpGetMember as u8, is_instance],
          m.location.start.line,
        );
      }
      Node::Array(a) => {
        let value = Value::Object(vec![].into());
        let mut index = 0;
        for p in a.elements.clone() {
          self.set_constant(value.clone(), a.location.start.line);
          match p {
            crate::parser::NodeProperty::Indexable(value) => {
              self.set_constant(Value::Number(index.into()), a.location.start.line);
              self.node_to_bytes(&value)?;
              self.write_buffer(
                vec![
                  OpCode::OpSetMember as u8,
                  OBJECT_MEMBER,
                  OpCode::OpPop as u8,
                ],
                a.location.start.line,
              );
            }
            _ => {}
          };
          index += 1;
        }
        self.set_constant(value, a.location.start.line);
      }
      Node::LoopEdit(e) => {
        let byte = match e.action {
          crate::parser::NodeLoopEditType::Break => OpCode::OpBreak,
          crate::parser::NodeLoopEditType::Continue => OpCode::OpContinue,
        } as u8;
        self.write(byte, e.location.start.line);
      }
      Node::Import(i) => {
        self.set_constant(Value::String(i.path.clone()), i.location.start.line);
        let lazy_bit = if i.is_lazy { 0b10 } else { 0b00 };
        let alias_bit = if i.name.is_some() { 0b01 } else { 0b00 };
        let meta_byte = lazy_bit | alias_bit;
        let name_byte = if let Some(name) = &i.name {
          self.set_value(Value::String(name.to_string()))
        } else {
          0
        };
        self.write_buffer(
          vec![OpCode::OpImport as u8, meta_byte, name_byte],
          i.location.start.line,
        );
      }
      Node::Name(_) => {
        return Err(format!(
          "No se puede usar '{}' sin exportar",
          crate::parser::KeywordsType::Name
        ))
      }
      Node::Export(e) => {
        let name: &str = match e.value.as_ref() {
          Node::Name(n) => {
            self.read_var(n.name.clone(), n.location.start.line);
            &n.name
          }
          Node::Function(f) => {
            self.set_constant(
              Value::Object(Self::parse_function(f)?.into()),
              f.location.start.line,
            );

            let name = self.set_value(Value::String(f.name.as_str().into()));
            self.write_buffer(vec![OpCode::OpConstDecl as u8, name], f.location.start.line);
            &f.name
          }
          Node::VarDecl(v) => {
            let op;
            if v.is_const {
              match &v.value {
                Some(value) => {
                  self.node_to_bytes(&value)?;
                }
                None => {
                  return Err(format!(
                    "No se puede asignar '{}' a una constante",
                    NEVER_NAME
                  ))
                }
              }
              op = OpCode::OpConstDecl as u8;
            } else {
              match &v.value {
                Some(value) => {
                  self.node_to_bytes(&value)?;
                }
                None => {
                  self.set_constant(Value::Never, v.location.start.line);
                }
              };
              op = OpCode::OpVarDecl as u8;
            }
            let name = self.set_value(Value::String(v.name.as_str().into()));
            self.write_buffer(vec![op, name], v.location.start.line);
            &v.name
          }

          _ => {
            return Err(format!(
              "No se puede obtener el nombre y el valor de {}",
              node.get_type()
            ))
          }
        };
        let name_byte = self.set_value(Value::String(name.to_string()));
        self.write_buffer(
          vec![OpCode::OpExport as u8, name_byte],
          e.location.start.line,
        );
      }
      Node::VarDel(id) => {
        self.read_var(id.name.clone(), id.location.start.line);
        self.write(OpCode::OpDelVar as u8, id.location.start.line);
      }
      Node::Await(value) => {
        self.node_to_bytes(&value.expression)?;
        self.write_buffer(
          vec![OpCode::OpAwait as u8, OpCode::OpUnPromise as u8],
          value.location.start.line,
        );
      }
      Node::Lazy(node_expression_medicator) => todo!(),
      Node::Class(node_class) => todo!(),
      Node::Throw(node_value) => todo!(),
      Node::Try(node_try) => todo!(),
      Node::None => todo!(),
    };
    Ok(())
  }
}
impl From<&Node> for Compiler {
  fn from(value: &Node) -> Self {
    let path = value.get_file();
    let chunk = ChunkGroup::new();
    let function = Function::Script {
      chunk,
      path: path.clone(),
      scope: None.into(),
    };
    let mut compiler = Self { function, path };
    match compiler.node_to_bytes(value) {
      Err(e) => panic!("{e}"),
      _ => {}
    }
    compiler
  }
}
