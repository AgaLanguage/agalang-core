use std::collections::HashMap;

pub mod binary;
mod chunk;
mod value;
pub use chunk::{ChunkGroup, OpCode};
pub use value::*;

use crate::parser::{Node, NodeFunction};
use crate::util::{OnError as _, OnSome as _};
use crate::{Decode, StructTag};

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
  fn node_value_to_bytes(&mut self, node: &Node) -> Result<(), String> {
    match node {
      Node::Function(node_function) => {
        let function = Value::Object(Self::parse_function(node_function)?.into());
        self.set_constant(function.clone(), node_function.location.start.line);
        self.write(OpCode::OpSetScope as u8, node_function.location.start.line);
        Ok(())
      }
      node => self.node_to_bytes(node),
    }
  }
  fn node_to_bytes(&mut self, node: &Node) -> Result<(), String> {
    match node {
      Node::Number(node_number) => {
        let number: Number = if node_number.base == 10u8 {
          node_number.value.parse().unwrap()
        } else {
          Number::from_str_radix(&node_number.value, node_number.base)
        };
        self.set_constant(Value::Number(number), node_number.location.start.line);
      }
      Node::Byte(node_byte) => {
        self.set_constant(Value::Byte(node_byte.value), node_byte.location.start.line);
      }
      Node::Binary(node_binary) => {
        self.node_to_bytes(&node_binary.left)?;
        self.node_to_bytes(&node_binary.right)?;
        let operator = match node_binary.operator {
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
        self.write_buffer(operator, node_binary.location.start.line);
      }
      Node::Program(node_program) => {
        if node_program.body.len() != 0 {
          self.node_to_bytes(&node_program.body.clone().to_node())?;
          self.write(OpCode::OpPop as u8, node_program.location.start.line);
        }
        self.set_constant(Value::Never, node_program.location.start.line);
        self.write(OpCode::OpReturn as u8, node_program.location.end.line);
      }
      Node::Block(node_block, _is_async) => {
        self.write(OpCode::OpNewLocals as u8, node_block.location.start.line);
        let code_len = node_block.body.len();
        for (index, node) in node_block.body.clone().enumerate() {
          self.node_to_bytes(&node)?;
          if index < (code_len - 1) {
            self.write(OpCode::OpPop as u8, node.get_location().end.line);
          }
        }
        self.write(OpCode::OpRemoveLocals as u8, node_block.location.end.line);
      }
      Node::UnaryFront(node_unary) => {
        self.node_to_bytes(&node_unary.operand)?;
        let operator = match &node_unary.operator {
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
        self.write(operator, node_unary.location.start.line);
      }
      Node::Identifier(node_identifier) => {
        self.read_var(
          node_identifier.name.clone(),
          node_identifier.location.start.line,
        );
      }
      Node::Console(node_console) => match node_console {
        crate::parser::NodeConsole::Output { value, location } => {
          self.node_to_bytes(&value)?;
          self.write(OpCode::OpConsoleOut as u8, location.start.line);
        }
        _ => {}
      },
      Node::String(node_string) => {
        for (i, data) in node_string.value.clone().enumerate() {
          match data {
            crate::parser::StringData::Str(val) => {
              self.set_constant(
                Value::String(val.as_str().into()),
                node_string.location.start.line,
              );
            }
            crate::parser::StringData::Id(id) => {
              self.read_var(id, node_string.location.start.line);
            }
          }
          if i != 0 {
            self.write(OpCode::OpAdd as u8, node_string.location.start.line);
          }
        }
      }
      Node::VarDecl(node_var_decl) => {
        let op;
        if node_var_decl.is_const {
          match &node_var_decl.value {
            Some(value) => {
              self.node_value_to_bytes(&value)?;
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
          match &node_var_decl.value {
            Some(value) => {
              self.node_value_to_bytes(&value)?;
            }
            None => {
              self.set_constant(Value::Never, node_var_decl.location.start.line);
            }
          };
          op = OpCode::OpVarDecl as u8;
        }
        let name = self.set_value(Value::String(node_var_decl.name.as_str().into()));
        self.write_buffer(vec![op, name], node_var_decl.location.start.line);
      }
      Node::Assignment(node_assignament) => {
        match node_assignament.identifier.as_ref() {
          Node::Identifier(id) => {
            self.node_value_to_bytes(&node_assignament.value)?;
            let name = self.set_value(Value::String(id.name.as_str().into()));
            self.write_buffer(
              vec![OpCode::OpSetVar as u8, name],
              node_assignament.location.start.line,
            );
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
            self.node_value_to_bytes(&node_assignament.value)?;
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
      Node::If(node_if) => {
        self.node_to_bytes(&node_if.condition)?;
        let jump_if = self.jump(OpCode::OpJumpIfFalse);
        self.node_to_bytes(&node_if.body.clone().to_node())?;

        let jump_else = self.jump(OpCode::OpJump);
        self.patch_jump(jump_if)?;

        if let Some(e) = &node_if.else_body {
          self.node_to_bytes(&e.clone().to_node())?;
        } else {
          self.set_constant(Value::Never, node_if.location.start.line);
        }
        self.patch_jump(jump_else)?;
      }
      Node::While(node_while) => {
        let loop_start = self.len();
        self.node_to_bytes(&node_while.condition)?;
        let jump_while = self.jump(OpCode::OpJumpIfFalse);
        self.node_to_bytes(&node_while.body.clone().to_node())?;
        if node_while.body.len() > 0 {
          self.write(OpCode::OpPop as u8, 0);
        }
        self.add_loop(loop_start)?;
        self.patch_jump(jump_while)?;
        self.set_constant(Value::Never, node_while.location.start.line);
      }
      Node::DoWhile(node_do_while) => {
        let jump_do = self.jump(OpCode::OpJump);
        let loop_start = self.len();
        self.node_to_bytes(&node_do_while.condition)?;
        let jump_do_while = self.jump(OpCode::OpJumpIfFalse);
        self.patch_jump(jump_do)?;
        self.node_to_bytes(&node_do_while.body.clone().to_node())?;
        if node_do_while.body.len() > 0 {
          self.write(OpCode::OpPop as u8, 0);
        }
        self.add_loop(loop_start)?;
        self.patch_jump(jump_do_while)?;
        self.set_constant(Value::Never, node_do_while.location.start.line);
      }
      Node::For(node_for) => {
        self.write(OpCode::OpNewLocals as u8, node_for.location.start.line);
        self.node_to_bytes(&node_for.init)?;
        self.write(OpCode::OpPop as u8, 0);
        let loop_start = self.len();
        self.node_to_bytes(&node_for.condition)?;
        let jump_for = self.jump(OpCode::OpJumpIfFalse);
        self.node_to_bytes(&node_for.body.clone().to_node())?;
        if node_for.body.len() > 0 {
          self.write(OpCode::OpPop as u8, 0);
        }
        self.node_to_bytes(&node_for.update)?;
        self.write(OpCode::OpPop as u8, 0);
        self.add_loop(loop_start)?;
        self.patch_jump(jump_for)?;
        self.write(OpCode::OpRemoveLocals as u8, node_for.location.start.line);
        self.set_constant(Value::Never, node_for.location.end.line);
      }
      Node::Function(node_function) => {
        let function = Value::Object(Self::parse_function(node_function)?.into());
        self.set_constant(function.clone(), node_function.location.start.line);

        let name = self.set_value(Value::String(node_function.name.as_str().into()));
        self.write_buffer(
          vec![OpCode::OpSetScope as u8, OpCode::OpConstDecl as u8, name],
          node_function.location.start.line,
        );
      }
      Node::Call(node_call) => {
        for arg in &node_call.arguments {
          self.node_value_to_bytes(&arg)?;
        }
        match node_call.callee.as_ref() {
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
            self.node_value_to_bytes(&node_call.callee)?;
            self.write(OpCode::OpCopy as u8, node.get_location().start.line)
          }
        };

        self.write_buffer(
          vec![OpCode::OpCall as u8, node_call.arguments.len() as u8],
          node_call.location.start.line,
        );
      }
      Node::Return(node_return) => {
        match &node_return.value {
          Some(value) => {
            self.node_to_bytes(&value)?;
          }
          None => {
            self.set_constant(Value::Never, node_return.location.start.line);
          }
        };
        self.write(OpCode::OpReturn as u8, node_return.location.start.line);
      }
      Node::Object(node_object) => {
        let value = Value::Object(HashMap::new().into());
        for p in node_object.properties.clone() {
          self.set_constant(value.clone(), node_object.location.start.line);
          match p {
            crate::parser::NodeProperty::Dynamic(key, value) => {
              self.node_to_bytes(&key)?;
              self.node_value_to_bytes(&value)?;
              self.write_buffer(
                vec![
                  OpCode::OpSetMember as u8,
                  OBJECT_MEMBER,
                  OpCode::OpPop as u8,
                ],
                node_object.location.start.line,
              );
            }
            crate::parser::NodeProperty::Property(key, value) => {
              self.set_constant(Value::String(key), node_object.location.start.line);
              self.node_value_to_bytes(&value)?;
              self.write_buffer(
                vec![
                  OpCode::OpSetMember as u8,
                  OBJECT_MEMBER,
                  OpCode::OpPop as u8,
                ],
                node_object.location.start.line,
              );
            }
            _ => {}
          };
        }
        self.set_constant(value, node_object.location.start.line);
      }
      Node::Member(node_member) => {
        self.node_to_bytes(&node_member.object)?;
        if node_member.computed {
          self.node_to_bytes(&node_member.member)?;
        } else {
          let name = match node_member.member.as_ref() {
            Node::Identifier(id) => id.name.as_str(),
            _ => return Err("Se esperaba un identificador como propiedad".to_string()),
          };
          self.set_constant(Value::String(name.into()), node_member.location.start.line);
        };
        let is_instance = if node_member.instance {
          INSTANCE_MEMBER
        } else {
          OBJECT_MEMBER
        };
        self.write_buffer(
          vec![OpCode::OpGetMember as u8, is_instance],
          node_member.location.start.line,
        );
      }
      Node::Array(node_array) => {
        let value = Value::Object(vec![].into());
        let mut index = 0;
        for p in node_array.elements.clone() {
          self.set_constant(value.clone(), node_array.location.start.line);
          match p {
            crate::parser::NodeProperty::Indexable(value) => {
              self.set_constant(Value::Number(index.into()), node_array.location.start.line);
              self.node_value_to_bytes(&value)?;
              self.write_buffer(
                vec![
                  OpCode::OpSetMember as u8,
                  OBJECT_MEMBER,
                  OpCode::OpPop as u8,
                ],
                node_array.location.start.line,
              );
            }
            _ => {}
          };
          index += 1;
        }
        self.set_constant(value, node_array.location.start.line);
      }
      Node::LoopEdit(node_loop_editor) => {
        let byte = match node_loop_editor.action {
          crate::parser::NodeLoopEditType::Break => OpCode::OpBreak,
          crate::parser::NodeLoopEditType::Continue => OpCode::OpContinue,
        } as u8;
        self.write(byte, node_loop_editor.location.start.line);
      }
      Node::Import(node_import) => {
        self.set_constant(
          Value::String(node_import.path.clone()),
          node_import.location.start.line,
        );
        let lazy_bit = if node_import.is_lazy { 0b10 } else { 0b00 };
        let alias_bit = if node_import.name.is_some() {
          0b01
        } else {
          0b00
        };
        let meta_byte = lazy_bit | alias_bit;
        let name_byte = if let Some(name) = &node_import.name {
          self.set_value(Value::String(name.to_string()))
        } else {
          0
        };
        self.write_buffer(
          vec![OpCode::OpImport as u8, meta_byte, name_byte],
          node_import.location.start.line,
        );
      }
      Node::Name(_) => {
        return Err(format!(
          "No se puede usar '{}' sin exportar",
          crate::parser::KeywordsType::Name
        ))
      }
      Node::Export(node_export) => {
        let name: &str = match node_export.value.as_ref() {
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
          node_export.location.start.line,
        );
      }
      Node::VarDel(node_identifier) => {
        self.read_var(
          node_identifier.name.clone(),
          node_identifier.location.start.line,
        );
        self.write(OpCode::OpDelVar as u8, node_identifier.location.start.line);
      }
      Node::Await(node_expression) => {
        self.node_value_to_bytes(&node_expression.expression)?;
        self.write_buffer(
          vec![OpCode::OpAwait as u8, OpCode::OpUnPromise as u8],
          node_expression.location.start.line,
        );
      }
      Node::Class(node_class) => {
        let class = Object::Class(Class::new(node_class.name.clone()));

        for prop in &node_class.body {
          let is_static = prop.meta & 0b01 != 0;
          let is_public = prop.meta & 0b10;

          self.set_constant(
            Value::Object(class.clone()),
            prop.value.get_location().start.line,
          );
          if !is_static {
            self.write(OpCode::OpGetInstance as u8, node_class.location.start.line);
          };

          self.set_constant(
            Value::String(prop.name),
            prop.value.get_location().start.line,
          );
          self.node_value_to_bytes(&prop.value)?;
          self.set_constant(
            Value::Object(class.clone()),
            prop.value.get_location().start.line,
          );
          self.write_buffer(
            vec![
              OpCode::OpInClass as u8,
              OpCode::OpSetMember as u8,
              INSTANCE_MEMBER | is_public,
              OpCode::OpPop as u8,
            ],
            prop.value.get_location().start.line,
          );
        }
        self.set_constant(Value::Object(class), node_class.location.start.line);
        if let Some(node_identifier) = &node_class.extend_of {
          self.read_var(
            node_identifier.name.clone(),
            node_identifier.location.start.line,
          );
          self.write(
            OpCode::OpExtendClass as u8,
            node_identifier.location.end.line,
          );
        }
        let name = self.set_value(Value::String(node_class.name.as_str().into()));
        self.write_buffer(
          vec![OpCode::OpConstDecl as u8, name],
          node_class.location.start.line,
        );
      }
      Node::Throw(node_value) => {
        self.node_value_to_bytes(node)?;
        self.write(OpCode::OpThrow as u8, node_value.location.start.line);
      }
      Node::Try(node_try) => {
        let mut try_block = Self {
          function: (&node_try.body).into(),
          path: node_try.location.file_name.clone(),
        };
        if node_try.body.len() > 0 {
          try_block.node_to_bytes(&node_try.body.clone().to_node())?;
        } else {
          try_block.set_constant(Value::Never, node_try.location.end.line);
        }
        try_block.write(OpCode::OpReturn as u8, node_try.location.end.line);
        self.set_constant(
          Value::Object(try_block.function.into()),
          node_try.location.start.line,
        );
        let mut catch_block = Self {
          function: (&node_try.body).into(),
          path: node_try.location.file_name.clone(),
        };
        match &node_try.catch {
          Some((error, block)) => {
            catch_block
              .function
              .chunk()
              .make_arg(error.clone(), block.location.start.line);
            if block.len() > 0 {
              catch_block.node_to_bytes(&block.clone().to_node())?;
            } else {
              catch_block.set_constant(Value::Never, block.location.end.line);
            }
          }
          None => {
            catch_block.set_constant(Value::Never, node_try.location.end.line);
          }
        };
        catch_block.write(OpCode::OpReturn as u8, node_try.location.end.line);
        let catch_block = catch_block.function;
        self.set_constant(
          Value::Object(catch_block.into()),
          node_try.location.start.line,
        );
        self.write(OpCode::OpTry as u8, node_try.location.start.line);
        self.set_constant(Value::Never, node_try.location.end.line);
      }
      Node::Lazy(node_expression) => {
        let mut lazy_block = Self {
          function: Function::Script {
            chunk: ChunkGroup::new(),
            path: node.get_file(),
            scope: None.into(),
          },
          path: node_expression.location.file_name.clone(),
        };
        lazy_block.node_to_bytes(&node_expression.expression)?;
        lazy_block.write(OpCode::OpReturn as u8, node_expression.location.end.line);
        self.set_constant(
          Value::Lazy(lazy_block.function.into()),
          node_expression.location.start.line,
        );
        self.write(OpCode::OpSetScope as u8, node_expression.location.end.line);
      }
      Node::None => return Err("Se encontro un error de nodos".to_string()),
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
impl crate::Encode for Compiler {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![];

    encode.push(crate::StructTag::Compile as u8);
    encode.extend(self.path.encode()?);
    encode.extend(self.function.encode()?);

    Ok(encode)
  }
}
impl Decode for Compiler {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Compile as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba un compilador".to_string())?;
    Ok(Self {
      path: String::decode(vec)?,
      function: Function::decode(vec)?,
    })
  }
}
