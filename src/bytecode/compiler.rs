use std::vec;

use crate::parser::{Node, NodeIdentifier};

use super::{
  chunk::{Chunk, OpCode},
  value::{Value, NEVER_NAME},
};

struct Local {
  identifier: NodeIdentifier,
  depth: u32,
}
struct Compiler {
  locals: Vec<Local>,
  scope_depth: u32,
}

fn set_constant(value: Value, line: usize, chunk: &mut Chunk) {
  let constant = chunk.make_constant(value);
  chunk.write_buffer(vec![OpCode::OpConstant as u8, constant as u8], line);
}
pub fn node_to_bytes(node: &Node, chunk: &mut Chunk) -> Result<(), String> {
  match node {
    Node::Number(n) => {
      let number = if n.base == 10u8 {
        n.value.parse::<f64>().unwrap()
      } else {
        u32::from_str_radix(&n.value, n.base as u32).unwrap() as f64
      };
      set_constant(Value::Number(number), n.location.start.line, chunk);
    }
    Node::Binary(b) => {
      let mut left = node_to_bytes(&b.left, chunk)?;
      let mut right = node_to_bytes(&b.right, chunk)?;
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
      chunk.write(operator as u8, b.location.start.line);
    }
    Node::Program(p) => {
      node_to_bytes(&p.body.clone().to_node(), chunk)?;
      chunk.write(OpCode::OpReturn as u8, p.location.end.line);
    }
    Node::Block(b, is_async) => {
      chunk.write(OpCode::OpNewLocals as u8, b.location.start.line);
      for node in &b.body {
        node_to_bytes(&node, chunk)?;
        chunk.write(OpCode::OpPop as u8, node.get_location().end.line);
      }
      chunk.write(OpCode::OpRemoveLocals as u8, b.location.start.line);
    }
    Node::UnaryFront(u) => {
      node_to_bytes(&u.operand, chunk)?;
      let operator = match &u.operator {
        crate::parser::NodeOperator::Minus => OpCode::OpNegate,
        crate::parser::NodeOperator::Not => OpCode::OpNot,
        crate::parser::NodeOperator::QuestionMark => OpCode::OpAsBoolean,
        op => {
          return Err(format!(
            "NodeOperator::{op:?}: No es un nodo valido en bytecode"
          ))
        }
      } as u8;
      chunk.write(operator, u.location.start.line);
    }
    Node::Identifier(i) => {
      let global = chunk.make_constant(Value::Object(i.name.as_str().into())) as u8;
      chunk.write_buffer(vec![OpCode::OpGetVar as u8, global], i.location.start.line);
    }
    Node::Console(c) => match c {
      crate::parser::NodeConsole::Output { value, location } => {
        node_to_bytes(&value, chunk);
        chunk.write(OpCode::OpConsoleOut as u8, location.start.line);
      }
      _ => {}
    },
    Node::String(s) => {
      let mut string = String::new();
      set_constant(Value::Object("".into()), s.location.start.line, chunk);
      for data in &s.value {
        match data {
          crate::parser::StringData::Str(val) => set_constant(
            Value::Object(val.as_str().into()),
            s.location.start.line,
            chunk,
          ),
          crate::parser::StringData::Id(id) => {
            set_constant(
              Value::Object(id.as_str().into()),
              s.location.start.line,
              chunk,
            );
          }
        }
        chunk.write(OpCode::OpAdd as u8, s.location.start.line);
      }
    }
    Node::VarDecl(v) => {
      let global = chunk.make_constant(Value::Object(v.name.as_str().into()));
      let op;
      if v.is_const {
        match &v.value {
          Some(value) => {
            node_to_bytes(&value, chunk);
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
            node_to_bytes(&value, chunk);
          }
          None => {
            set_constant(Value::Never, v.location.start.line, chunk);
          }
        };
        op = OpCode::OpVarDecl as u8;
      }
      chunk.write_buffer(vec![op, global], v.location.start.line);
    }
    Node::Assignment(a) => {
      match a.identifier.as_ref() {
        Node::Identifier(id) => {
          let name = chunk.make_constant(Value::Object(id.name.as_str().into()));
          node_to_bytes(&a.value, chunk);
          chunk.write_buffer(vec![OpCode::OpSetVar as u8, name], a.location.start.line);
        }
        _ => return Err("Se esperaba una assignacion valida".to_string()),
      };
    }
    Node::If(i)=> {
      node_to_bytes(&i.condition, chunk)?;
      let jump_if = chunk.jump(OpCode::OpJumpIfFalse);
      node_to_bytes(&i.body.clone().to_node(), chunk)?;

        let jump_else = chunk.jump(OpCode::OpJump);
      chunk.patch_jump(jump_if);

        if let Some(e) = &i.else_body {
          node_to_bytes(&e.clone().to_node(), chunk)?;
        }
        chunk.patch_jump(jump_else);
    }
    Node::While(i) => {
      let loop_start = chunk.last_code;
        node_to_bytes(&i.condition, chunk)?;
        let jump_while = chunk.jump(OpCode::OpJumpIfFalse);
          chunk.write(OpCode::OpPop as u8, loop_start);
          node_to_bytes(&i.body.clone().to_node(), chunk)?;
      chunk.add_loop(loop_start);
        chunk.patch_jump(jump_while);
    }
    Node::DoWhile(i) => {
      let jump_do = chunk.jump(OpCode::OpJump);
        let loop_start = chunk.last_code;
          node_to_bytes(&i.condition, chunk)?;
          let jump_while = chunk.jump(OpCode::OpJumpIfFalse);
            chunk.write(OpCode::OpPop as u8, loop_start);
      chunk.patch_jump(jump_do);
            node_to_bytes(&i.body.clone().to_node(), chunk)?;
        chunk.add_loop(loop_start);
          chunk.patch_jump(jump_while);
    }
    Node::For(f) => {
      chunk.write(OpCode::OpNewLocals as u8, f.location.start.line);
      node_to_bytes(&f.init, chunk)?;
        let loop_start = chunk.last_code;
          node_to_bytes(&f.condition, chunk)?;
          let jump_while = chunk.jump(OpCode::OpJumpIfFalse);
            chunk.write(OpCode::OpPop as u8, loop_start);
            node_to_bytes(&f.body.clone().to_node(), chunk)?;
            node_to_bytes(&f.update, chunk)?;
        chunk.add_loop(loop_start);
          chunk.patch_jump(jump_while);
      chunk.write(OpCode::OpRemoveLocals as u8, f.location.start.line);
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
