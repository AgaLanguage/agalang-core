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
      path: function.file.clone(),
    };
    let mut i = function.params.len();
    while i > 0 {
      i -= 1;
      let param = function.params.get(i).unwrap();
      let _global = compiler
        .function
        .chunk()
        .make_arg(param.name.clone(), param.location.start.line);
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
      Node::Binary(b) => {
        self.node_to_bytes(&b.left)?;
        self.node_to_bytes(&b.right)?;
        let operator = match b.operator {
          crate::parser::NodeOperator::Division => OpCode::OpDivide,
          crate::parser::NodeOperator::Minus => OpCode::OpSubtract,
          crate::parser::NodeOperator::Multiply => OpCode::OpMultiply,
          crate::parser::NodeOperator::Plus => OpCode::OpAdd,
          crate::parser::NodeOperator::GreaterThan => OpCode::OpGreaterThan,
          crate::parser::NodeOperator::Equal => OpCode::OpEquals,
          crate::parser::NodeOperator::LessThan => OpCode::OpLessThan,
          crate::parser::NodeOperator::And => OpCode::OpAnd,
          crate::parser::NodeOperator::Or => OpCode::OpOr,
          a => {
            return Err(format!(
              "NodeOperator::{a:?}: No es un nodo valido en bytecode"
            ))
          }
        };
        if operator == OpCode::OpNull {
          match b.operator {
            crate::parser::NodeOperator::NotEqual => {
              self.write_buffer(
                vec![OpCode::OpEquals as u8, OpCode::OpNot as u8],
                b.location.start.line,
              );
            }
            a => {
              return Err(format!(
                "NodeOperator::{a:?}: No es un nodo valido en bytecode"
              ))
            }
          };
        } else {
          self.write(operator as u8, b.location.start.line);
        };
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
        let last_index = b.body.len();
        for (index, node) in b.body.clone().enumerate() {
          self.node_to_bytes(&node)?;
          if index < (last_index - 1) {
            self.write(OpCode::OpPop as u8, node.get_location().end.line);
          }
        }
        self.write(OpCode::OpRemoveLocals as u8, b.location.start.line);
      }
      Node::UnaryFront(u) => {
        self.node_to_bytes(&u.operand)?;
        let operator = match &u.operator {
          crate::parser::NodeOperator::Minus => OpCode::OpNegate,
          crate::parser::NodeOperator::Not => OpCode::OpNot,
          crate::parser::NodeOperator::QuestionMark => OpCode::OpAsBoolean,
          crate::parser::NodeOperator::Approximate => OpCode::OpApproximate,
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
        let global =
          self.set_constant(Value::String(v.name.as_str().into()), v.location.start.line);
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
        self.write_buffer(vec![op, global], v.location.start.line);
      }
      Node::Assignment(a) => {
        match a.identifier.as_ref() {
          Node::Identifier(id) => {
            let name = self.set_constant(
              Value::String(id.name.as_str().into()),
              id.location.start.line,
            );
            self.node_to_bytes(&a.value)?;
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
        self.write(OpCode::OpPop as u8, 0);
        self.add_loop(loop_start)?;
        self.patch_jump(jump_while)?;
      }
      Node::DoWhile(i) => {
        let jump_do = self.jump(OpCode::OpJump);
        let loop_start = self.len();
        self.node_to_bytes(&i.condition)?;
        let jump_while = self.jump(OpCode::OpJumpIfFalse);
        self.patch_jump(jump_do)?;
        self.node_to_bytes(&i.body.clone().to_node())?;
        self.write(OpCode::OpPop as u8, 0);
        self.add_loop(loop_start)?;
        self.patch_jump(jump_while)?;
      }
      Node::For(f) => {
        self.write(OpCode::OpNewLocals as u8, f.location.start.line);
        self.node_to_bytes(&f.init)?;
        let loop_start = self.len();
        self.node_to_bytes(&f.condition)?;
        let jump_while = self.jump(OpCode::OpJumpIfFalse);
        self.node_to_bytes(&f.body.clone().to_node())?;
        self.node_to_bytes(&f.update)?;
        self.write(OpCode::OpPop as u8, 0);
        self.add_loop(loop_start)?;
        self.patch_jump(jump_while)?;
        self.write(OpCode::OpRemoveLocals as u8, f.location.start.line);
      }
      Node::Function(f) => {
        let global =
          self.set_constant(Value::String(f.name.as_str().into()), f.location.start.line);

        self.set_constant(
          Value::Object(Self::parse_function(f)?.into()),
          f.location.start.line,
        );

        self.write_buffer(
          vec![OpCode::OpConstDecl as u8, global],
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
          let index_byte =
            self.set_constant(Value::String(name.to_string()), i.location.start.line);
          self.write(OpCode::OpPop as u8, i.location.start.line);
          index_byte
        } else {
          0
        };
        self.write_buffer(
          vec![OpCode::OpImport as u8, meta_byte, name_byte],
          i.location.start.line,
        );
      }
      Node::Export(e) => {
        let name = match e.value.as_ref() {
          Node::VarDecl(v) => {
            let global =
              self.set_constant(Value::String(v.name.as_str().into()), v.location.start.line);
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
            self.write_buffer(vec![op, global], v.location.start.line);
            v.name.clone()
          }
          Node::Function(f) => {
            let global =
              self.set_constant(Value::String(f.name.as_str().into()), f.location.start.line);
            
            self.set_constant(
              Value::Object(Self::parse_function(f)?.into()),
              f.location.start.line,
            );

            self.write_buffer(
              vec![OpCode::OpConstDecl as u8, global],
              f.location.start.line,
            );
            f.name.clone()
          }
          _ => todo!(),
        };
        self.write_buffer(vec![OpCode::OpExport as u8], e.location.start.line);
      }
      a => {
        return Err(format!(
          "{}: No es un nodo valido en bytecode",
          a.get_type()
        ))
      }
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
    };
    let mut compiler = Self { function, path };
    match compiler.node_to_bytes(value) {
      Err(e) => panic!("{e}"),
      _ => {}
    }
    compiler
  }
}
