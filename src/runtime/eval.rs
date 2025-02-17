use std::{borrow::Borrow, path::Path, rc::Rc};

use parser::ast::{Node, NodeBlock, NodeIdentifier, NodeProperty};
use parser::internal;
use parser::util::List;
use parser::{self as frontend, util::RefValue};

use crate::{
  runtime::{self, env::RefEnvironment},
  Modules, ToResult,
};

use super::interpreter;
use super::{
  stack::RefStack,
  values::{internal::AgalInternal, AgalValue, DefaultRefAgalValue},
};

type EvalResult = Result<DefaultRefAgalValue, ()>;

fn code(path: &str) -> Option<String> {
  let contents = std::fs::read_to_string(path);
  match contents {
    Ok(contents) => Some(contents),
    Err(err) => {
      let ref type_err = internal::ErrorNames::PathError;
      let err = internal::ErrorTypes::IoError(err);
      internal::show_error(type_err, err);
      None
    }
  }
}

pub fn full_eval(
  path: String,
  stack: RefStack,
  modules: RefValue<Modules>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = EvalResult>>> {
  Box::pin(async move {
    let modules_manager = &*modules.as_ref().borrow();
    if modules_manager.has(&path) {
      return Ok(modules_manager.get(&path));
    }
    let contents = code(&path).to_result()?;

    let value = eval(contents, &path, stack, &modules_manager.clone()).await?;
    modules_manager.add(&path, value.clone());
    Ok(value)
  })
}

async fn eval(code: String, path: &str, stack: RefStack, modules_manager: &Modules) -> EvalResult {
  let program = {
    let mut parser = frontend::Parser::new(code, &path);
    parser.produce_ast()
  };
  let program = match program {
    Ok(value) => value,
    Err(err) => {
      let type_err = internal::errors::ErrorNames::SyntaxError;
      let data = internal::errors::error_to_string(&type_err, parser::node_error(&err));
      internal::print_error(data);
      return Err(());
    }
  };
  //println!("{}", json(&program));
  let box_node = program.to_box();
  let new_stack = stack.crate_child(false, box_node.clone());
  let value = interpreter(
    box_node,
    new_stack.clone(),
    modules_manager.clone().to_ref(),
  )
  .await;
  let value = &*value.borrow();
  match value {
    Err(throw) => {
      let (type_err, err) = throw.get_data(new_stack);
      let data = internal::errors::error_to_string(&type_err, err);
      internal::print_error(data);
      Err(())
    }
    Ok(value) => Ok(value.clone()),
  }
}
/*
fn json(node: &Node) -> String {
  match node {
        Node::Array(a) => format!(
            "{{\"kind\":\"Array\",\"column\":{},\"line\":{},\"file\":\"{}\",\"body\":[{}]}}",
            a.location.start.column,
            a.location.start.line,
            json_str(&a.file),
            list_property(&a.elements),
        ),
        Node::Assignment(a) => format!(
              "{{\"kind\":\"Assignment\",\"column\":{},\"line\":{},\"file\":\"{}\",\"identifier\":{},\"value\":{}}}",
              a.location.start.column,
              a.location.start.line,
              json_str(&a.file),
              json(a.identifier.as_ref()),
              json(a.value.as_ref())
            ),
        Node::Binary(b)=>format!(
          "{{\"kind\":\"Binary\",\"column\":{},\"line\":{},\"file\":\"{}\",\"left\":{},\"right\":{},\"operator\":\"{}\"}}",
          b.location.start.column,
          b.location.start.line,
          json_str(&b.file),
          json(b.left.as_ref()),
          json(b.right.as_ref()),
          json_str(&b.operator)
        ),
        Node::Block(b) => json_b(b),
        Node::Byte(b) => format!("{{\"kind\":\"Byte\",\"column\":{},\"line\":{},\"file\":\"{}\",\"value\":{}}}",b.location.start.column, b.location.start.line, b.location.file_name, b.value),
        Node::Call(c) => format!("{{\"kind\":\"Call\",\"column\":{},\"line\":{},\"file\":\"{}\",\"args\":[{}],\"call\":{}}}",c.location.start.column, c.location.start.line, c.location.file_name, c.arguments,json(&c.callee)),
        Node::Function(f) => format!("{{{},\"name\":\"{}\",\"is_async\":{},\"params\":[{}],\"body\":[{}]}}",json_basic(node), f.name, f.is_async,f.params.map(|id|json_op_id(Some(id))), json_b(&f.body)),
        Node::Identifier(i) => json_op_id(Some(i)),
        Node::If(f) => format!("{{{},\"condition\":{},\"body\":{},\"else_body\":{}}}", json_basic(node), json(&f.condition),json_b(&f.body),json_b_op(f.else_body.as_ref())),
        Node::Import(i) => format!()
        Node::LoopEdit(_)|
        Node::Member(_)|
        Node::Name(_)|
        Node::None|
        Node::Number(_)|
        Node::Object(_)=> "null".to_string(),
        Node::Program(p)=>format!(
          "{{\"kind\":\"Program\",\"column\":{},\"line\":{},\"file\":\"{}\",\"body\":[{}]}}",
          p.location.start.column,
          p.location.start.line,
          json_str(&p.file),
          json_b(&p.body),
      ),
      Node::Return(_)|
      Node::String(_)|
      Node::Throw(_)|
      Node::Try(_)|
      Node::UnaryBack(_)|
      Node::UnaryFront(_)|
      Node::VarDecl(_)|
      Node::While(_)|_ => "null".to_string()
    }
}
fn json_str(str: &str) -> String {
  str.replace("\n", "\\n").replace("\"", "\\\"")
}
fn json_b(b: &NodeBlock) -> String {
  format!(
    "{{\"kind\":\"Block\",\"body\":[{}],\"in_function\":{},\"in_loop\":{}}}",
    b.body.map(|n| json(n)).join(","),
    b.in_function,
    b.in_loop
  )
}
fn json_b_op(b: Option<&NodeBlock>) -> String {
  match b {
      Some(b) => json_b(b),
      None => "null".to_string(),
  }
}
fn json_op_id(op: Option<&NodeIdentifier>) -> String {
  match op {
    Some(op) => format!("{{{},\"name\":\"{}\"", json_basic(&Node::Identifier(op.clone())), op.name),
    None => "null".to_string(),
  }
}
fn json_p(node_p: &NodeProperty) -> String {
  match node_p {
    NodeProperty::Dynamic(key, value) => format!(
      "{{\"kind\":\"PropertyDynamic\",\"key\":{},\"value\":{}}}",
      json(key),
      json(value)
    ),
    NodeProperty::Indexable(val) => {
      format!("{{\"kind\":\"PropertyIndexable\",\"value\":{}}}", json(val))
    }
    NodeProperty::Iterable(val) => {
      format!("{{\"kind\":\"PropertyIterable\",\"value\":{}}}", json(val))
    }
    NodeProperty::Property(key, value) => format!(
      "{{\"kind\":\"PropertyProperty\",\"key\":\"{}\",\"value\":{}}}",
      json_str(key),
      json(value)
    ),
  }
}
fn list_property(list: &List<NodeProperty>) -> String {
  list.map(|n| json_p(n)).join(",")
}
fn json_basic(n: &Node) -> String {
  format!("\"kind\":\"{}\",\"column\":{},\"line\":{},\"file\":\"{}\"", n.get_type(), n.get_location().start.column, n.get_location().start.line, n.get_location().file_name)
}
*/
