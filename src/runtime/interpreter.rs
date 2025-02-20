use std::{cell::RefCell, collections::HashMap, future::Future, path::Path, pin::Pin, rc::Rc};

use parser::{
  ast::{BNode, Node, NodeLoopEditType, NodeProperty, StringData},
  internal::ErrorNames,
  util::RefValue,
};

use crate::{path::absolute_path, Modules};

use super::{
  full_eval,
  values::{
    complex::{
      AgalArray, AgalClass, AgalClassProperty, AgalComplex, AgalFunction, AgalObject,
      AgalPromiseData,
    },
    internal::{self, AgalThrow},
    primitive,
    traits::{AgalValuable as _, ToAgalValue as _},
    AgalValue, ResultAgalValue,
  },
  RefStack, THIS_KEYWORD,
};

pub fn interpreter(
  node: BNode,
  stack: RefStack,
  modules: RefValue<Modules>,
) -> Pin<Box<dyn Future<Output = ResultAgalValue>>> {
  Box::pin(async move {
    let pre_stack = stack;
    let stack = pre_stack.clone().next(node.clone());
    match node.clone().as_ref() {
      Node::Array(list) => {
        let mut vec = vec![];
        for n in &list.elements {
          match n {
            NodeProperty::Indexable(value) => {
              let data = interpreter(value.to_box(), stack.clone(), modules.clone()).await?;
              vec.push(data);
            }
            NodeProperty::Iterable(iter) => {
              let data = interpreter(iter.clone().to_box(), stack.clone(), modules.clone()).await?;
              let list = data.to_agal_array(stack.clone())?.un_ref();
              for n in list.to_vec().borrow().iter() {
                vec.push(n.clone());
              }
            }
            _ => {}
          }
        }
        Ok(AgalArray::from(vec).to_value().as_ref())
      }
      Node::Assignment(assignment) => {
        let value = interpreter(assignment.value.clone(), stack.clone(), modules.clone()).await?;
        if value.borrow().is_never() {
          return Err(internal::AgalThrow::Params {
            type_error: ErrorNames::TypeError,
            message: "No se puede asignar \"nada\" a una variable".to_string(),
            stack,
          });
        }
        match assignment.identifier.as_ref() {
          Node::Identifier(identifier) => {
            stack
              .env()
              .assign(stack.clone(), &identifier.name, value, node.as_ref())
          }
          Node::Member(member) => {
            if member.instance {
              return Err(internal::AgalThrow::Params {
                type_error: ErrorNames::TypeError,
                message: "No se puede asignar una propiedad de instancia".to_string(),
                stack,
              });
            }
            let key = if !member.computed && member.member.is_identifier() {
              member.member.get_identifier().unwrap().name.clone()
            } else {
              interpreter(member.member.clone(), stack.clone(), modules.clone())
                .await?
                .to_agal_string(stack.clone())?
                .to_string()
            };

            interpreter(member.object.clone(), stack.clone(), modules.clone())
              .await?
              .set_object_property(stack, &key, value)
          }
          _ => Ok(AgalValue::Never.as_ref()),
        }
      }
      Node::Await(a) => {
        let value = interpreter(a.expression.clone(), stack.clone(), modules.clone()).await?;
        if let AgalComplex::Promise(a) = {
          if let AgalValue::Complex(c) = value.un_ref() {
            c.un_ref()
          } else {
            return Ok(value);
          }
        } {
          let mut value = a.borrow_mut();
          if let AgalPromiseData::Resolved(r) = &value.data {
            return r.clone();
          }
          if let AgalPromiseData::Unresolved(future) = std::mem::replace(
            &mut value.data,
            AgalPromiseData::Resolved(AgalValue::Never.to_result()),
          ) {
            let agal_value = future.await;
            value.data = AgalPromiseData::Resolved(agal_value.clone());
            return agal_value;
          }
        }
        Ok(value)
      }
      Node::Binary(binary) => {
        let left = interpreter(binary.left.clone(), stack.clone(), modules.clone()).await?;
        let right = interpreter(binary.right.clone(), stack.clone(), modules.clone()).await?;
        left.binary_operation(stack.clone(), &binary.operator, right)
      }
      Node::Block(block) => {
        let mut result = AgalValue::Never.as_ref();
        for statement in &block.body {
          result = interpreter(statement.clone().to_box(), stack.clone(), modules.clone()).await?;
          if (result.is_stop()) {
            break;
          }
        }
        Ok(result)
      }
      Node::Byte(byte_node) => Ok(primitive::AgalByte::new(byte_node.value).to_ref_value()),
      Node::Call(call) => {
        let callee = call.callee.clone();
        let (mut callee, this) = if let Node::Member(member) = callee.as_ref() {
          let this = interpreter(member.object.clone(), stack.clone(), modules.clone()).await?;
          let this = this.clone();
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            // No deberia ser posible llegar a este punto
            interpreter(member.member.clone(), stack.clone(), modules.clone())
              .await?
              .try_to_string(stack.clone())?
          };
          let callee = this.clone().get_instance_property(stack.clone(), &key)?;
          (callee, this)
        } else {
          let callee = interpreter(callee.clone(), stack.clone(), modules.clone()).await?;
          (callee.clone(), callee)
        };

        let mut args = vec![];
        for arg in &call.arguments {
          let arg = interpreter(arg.clone().to_box(), stack.clone(), modules.clone()).await?;
          args.push(arg);
        }
        let ret = callee
          .call(stack.clone(), this, args, modules.clone())
          .await?;
        if ret.is_return() {
          ret.into_return().unwrap_or(AgalValue::Never.to_ref_value())
        } else if ret.is_stop() {
          AgalValue::Never.to_ref_value()
        } else {
          ret
        }
        .to_result()
      }
      Node::Class(class) => {
        let extend_of_value = if let AgalComplex::Class(class) = {
          if let AgalValue::Complex(c) = {
            if let Some(extend) = &class.extend_of {
              stack
                .env()
                .get(stack.clone(), &extend.name, &node)?
                .un_ref()
            } else {
              AgalValue::Never
            }
          } {
            c.un_ref()
          } else {
            return internal::AgalThrow::Params {
              type_error: ErrorNames::TypeError,
              message: "Solo se puede extender de otras clases".to_string(),
              stack,
            }
            .to_result();
          }
        } {
          Some(class)
        } else {
          None
        };
        let mut properties = Vec::new();
        let class_stack = pre_stack.crate_child(true, node.clone());
        for property in &class.body {
          let is_static = (property.meta & 1) != 0;
          let is_public = (property.meta & 2) != 0;

          let value = if let Some(b) = &property.value {
            interpreter(b.clone(), class_stack.clone(), modules.clone()).await?
          } else {
            AgalValue::Never.to_ref_value()
          };

          properties.push((
            property.name.clone(),
            AgalClassProperty {
              is_public,
              is_static,
              value,
            },
          ));
        }

        let class_value =
          AgalClass::new(class.name.clone(), properties, extend_of_value).to_ref_value();
        class_stack.env().set(THIS_KEYWORD, class_value.clone());
        stack
          .env()
          .define(class_stack, &class.name, class_value, true, &node)
      }
      Node::DoWhile(do_while) => {
        let mut value = AgalValue::Never.as_ref();
        let mut condition: Result<primitive::AgalBoolean, internal::AgalThrow> =
          Ok(primitive::AgalBoolean::True);
        loop {
          if !condition.clone()?.as_bool() {
            break;
          }
          value = interpreter(
            do_while.body.clone().to_node().to_box(),
            stack.clone(),
            modules.clone(),
          )
          .await?;
          let v = value.un_ref();
          if v.is_return() {
            return Ok(value);
          }
          if v.is_break() {
            break;
          }
          if v.is_continue() {
            continue;
          }
          let pre_condition =
            interpreter(do_while.condition.clone(), stack.clone(), modules.clone()).await?;
          condition = pre_condition.to_agal_boolean(stack.clone());
        }
        Ok(value)
      }
      Node::Export(export) => match export.value.as_ref() {
        Node::VarDecl(var) => {
          let value = interpreter(var.value.clone().unwrap(), stack.clone(), modules).await?;
          stack
            .env()
            .define(stack.clone(), &var.name, value.clone(), var.is_const, &node);
          AgalValue::Export(var.name.clone(), value).to_result()
        }
        Node::Function(func) => {
          let function = AgalFunction::new(
            func.name.clone(),
            func.is_async,
            func.params.clone(),
            func.body.clone(),
            stack.env(),
          )
          .to_ref_value();
          stack
            .env()
            .define(stack, &func.name, function.clone(), true, &node);
          AgalValue::Export(func.name.clone(), function).to_result()
        }
        Node::Name(name) => {
          let value = stack.env().get(stack, &name.name, &node)?;
          AgalValue::Export(name.name.clone(), value).to_result()
        }
        Node::Class(class) => {
          let extend_of_value = if let AgalComplex::Class(class) = {
            if let AgalValue::Complex(c) = {
              if let Some(extend) = &class.extend_of {
                stack
                  .env()
                  .get(stack.clone(), &extend.name, &node)?
                  .un_ref()
              } else {
                AgalValue::Never
              }
            } {
              c.un_ref()
            } else {
              return internal::AgalThrow::Params {
                type_error: ErrorNames::TypeError,
                message: "Solo se puede extender de otras clases".to_string(),
                stack,
              }
              .to_result();
            }
          } {
            Some(class)
          } else {
            None
          };
          let mut properties = Vec::new();
          let class_stack = pre_stack.crate_child(true, node.clone());
          for property in &class.body {
            let is_static = (property.meta & 1) != 0;
            let is_public = (property.meta & 2) != 0;

            let value = if let Some(b) = &property.value {
              interpreter(b.clone(), class_stack.clone(), modules.clone()).await?
            } else {
              AgalValue::Never.to_ref_value()
            };

            properties.push((
              property.name.clone(),
              AgalClassProperty {
                is_public,
                is_static,
                value,
              },
            ));
          }

          let class_value =
            AgalClass::new(class.name.clone(), properties, extend_of_value).to_ref_value();
          class_stack.env().set(THIS_KEYWORD, class_value.clone());
          stack
            .env()
            .define(stack, &class.name, class_value.clone(), true, &node);

          AgalValue::Export(class.name.clone(), class_value).to_result()
        }
        _ => internal::AgalThrow::Params {
          type_error: ErrorNames::SyntaxError,
          message: "Se nesesita un nombre para las exportaciones".to_string(),
          stack,
        }
        .to_result(),
      },
      Node::For(for_node) => {
        let mut value = AgalValue::Never.as_ref();
        let mut condition = Ok(primitive::AgalBoolean::True);
        let stack = pre_stack.crate_child(false, node);
        interpreter(for_node.init.clone(), stack.clone(), modules.clone()).await?; // init value
        loop {
          if !condition?.as_bool() {
            break;
          }
          let node = for_node.body.clone().to_node().to_box();
          value = interpreter(
            node.clone(),
            stack.crate_child(false, node),
            modules.clone(),
          )
          .await?;
          let v = value.un_ref();
          if v.is_return() {
            return value.to_result();
          }
          if v.is_break() {
            break;
          }
          let pre_condition =
            interpreter(for_node.condition.clone(), stack.clone(), modules.clone()).await?;
          condition = pre_condition.to_agal_boolean(stack.clone());
          let pre_update =
            interpreter(for_node.update.clone(), stack.clone(), modules.clone()).await?;
        }
        value.to_result()
      }
      Node::Function(func) => {
        let function = AgalFunction::new(
          func.name.clone(),
          func.is_async,
          func.params.clone(),
          func.body.clone(),
          stack.env().crate_child(false),
        )
        .to_ref_value();
        stack.env().define(stack, &func.name, function, true, &node)
      }
      Node::Identifier(id) => stack.env().get(stack, &id.name, &node),
      Node::If(if_node) => {
        let condition = interpreter(if_node.condition.clone(), stack.clone(), modules.clone())
          .await?
          .to_agal_boolean(stack.clone())?;
        if condition.as_bool() {
          return interpreter(if_node.body.clone().to_node().to_box(), stack, modules).await;
        }
        if let Some(else_body) = &if_node.else_body {
          return interpreter(else_body.clone().to_node().to_box(), stack.clone(), modules).await;
        }
        return AgalValue::Never.to_result();
      }
      Node::Import(import) => {
        let module = if import
          .path
          .starts_with(crate::libraries::PREFIX_NATIVE_MODULES)
        {
          crate::libraries::get_module(&import.path, modules.clone())
        } else {
          let path = absolute_path(&import.file);
          let path = Path::new(&path).parent();
          if let Some(path) = path {
            let filename = format!("{}/{}", path.to_string_lossy(), import.path);
            let filename = absolute_path(&filename);
            full_eval(filename, stack.get_global(), modules).await
          } else {
            Err(())
          }
        };
        if let Err(e) = module {
          return AgalThrow::Params {
            type_error: ErrorNames::PathError,
            message: format!("No se encontro el modulo \"{}\"", import.path),
            stack,
          }
          .throw();
        }
        let module = module.unwrap();
        if let Some(n) = import.name.clone() {
          stack.env().define(stack, &n, module, true, &node);
        }
        AgalValue::Never.to_result()
      }
      Node::Lazy(node) => internal::AgalLazy::new(node.clone(), stack, modules).to_result(),
      Node::LoopEdit(loop_edit) => match loop_edit.action {
        NodeLoopEditType::Break => AgalValue::Break,
        NodeLoopEditType::Continue => AgalValue::Continue,
      }
      .to_result(),
      Node::Member(member) => {
        let mut object = interpreter(member.object.clone(), stack.clone(), modules.clone()).await?;
        if member.instance && !member.computed && member.member.is_identifier() {
          let key = member.member.get_identifier().unwrap().name.clone();
          object.get_instance_property(stack, &key)
        } else {
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            interpreter(member.member.clone(), stack.clone(), modules.clone())
              .await?
              .to_agal_string(stack.clone())?
              .to_string()
          };
          object.get_object_property(stack, &key)
        }
      }
      Node::Name(name) => stack.env().get(stack, &name.name, &node),
      Node::None => AgalValue::Never.to_result(),
      Node::Number(num) => if num.base == 10 {
        let d = str::parse::<f32>(&num.value).unwrap();
        primitive::AgalNumber::Decimal(d)
      } else {
        let i = i32::from_str_radix(&num.value, num.base as u32).unwrap();
        primitive::AgalNumber::Integer(i)
      }
      .to_result(),
      Node::Object(obj) => {
        let mut hashmap = HashMap::new();
        for prop in &obj.properties {
          match prop {
            NodeProperty::Property(key, value) => {
              let value =
                interpreter(value.clone().to_box(), stack.clone(), modules.clone()).await?;
              hashmap.insert(key.clone(), value);
            }
            NodeProperty::Dynamic(key, value) => {
              let key = interpreter(key.clone().to_box(), stack.clone(), modules.clone()).await?;
              let key = key.to_agal_string(stack.clone())?.to_string();
              let value =
                interpreter(value.clone().to_box(), stack.clone(), modules.clone()).await?;
              hashmap.insert(key, value);
            }
            NodeProperty::Iterable(iter) => {
              let mut value =
                interpreter(iter.clone().to_box(), stack.clone(), modules.clone()).await?;
              let keys = value.get_keys();
              for key in keys.iter() {
                let value = value.get_object_property(stack.clone(), &key)?;
                hashmap.insert(key.clone(), value);
              }
            }
            _ => {}
          }
        }
        AgalObject::from_hashmap(Rc::new(RefCell::new(hashmap))).to_result()
      }
      Node::Program(program) => {
        interpreter(program.body.clone().to_node().to_box(), stack, modules).await
      }
      Node::Return(ret) => {
        if ret.value.is_none() {
          return internal::AgalInternal::Return(AgalValue::Never.as_ref()).to_result();
        }
        let ret_value = ret.value.clone().unwrap();
        let value = interpreter(ret_value, stack, modules).await?;
        internal::AgalInternal::Return(value).to_result()
      }
      Node::String(str) => {
        let mut string = String::new();
        for s in &str.value {
          match &s {
            StringData::Id(id) => {
              let data = stack
                .clone()
                .env()
                .get(stack.clone(), id, &node)?
                .to_agal_string(stack.clone())?
                .to_string();
              string.push_str(&data)
            }
            StringData::Str(str) => string.push_str(str),
          }
        }
        primitive::AgalString::from_string(string).to_result()
      }
      Node::Throw(throw) => {
        let value = interpreter(throw.value.clone(), stack.clone(), modules).await?;
        internal::AgalThrow::Value(value).to_result()
      }
      Node::Try(try_node) => {
        let try_box_node = try_node.body.clone().to_node().to_box();
        let try_stack = stack.crate_child(false, try_box_node.clone());
        let try_val = interpreter(try_box_node, try_stack.clone(), modules.clone()).await;
        let value = match try_val {
          Err(throw) => match try_node.clone().catch {
            None => AgalValue::Never.to_ref_value(),
            Some((error_name, catch_block)) => {
              let node_catch = catch_block.to_node().to_box();

              let stack = stack.crate_child(false, node_catch.clone());
              stack.env().define(
                stack.clone(),
                &error_name,
                throw.to_error().to_ref_value(),
                true,
                &node,
              );
              interpreter(node_catch, stack.clone(), modules.clone()).await?
            }
          },
          Ok(val) => val,
        };
        if let Some(f) = &try_node.finally {
          let node_finally = f.clone().to_node().to_box();
          let stack = pre_stack.crate_child(false, node_finally.clone());
          interpreter(node_finally, stack, modules).await?
        } else {
          value
        }
        .to_result()
      }
      Node::UnaryBack(unary) => interpreter(unary.operand.clone(), stack.clone(), modules)
        .await?
        .unary_back_operator(stack, &unary.operator),
      Node::UnaryFront(unary) => interpreter(unary.operand.clone(), stack.clone(), modules)
        .await?
        .unary_operator(stack, &unary.operator),
      Node::VarDecl(var) => match &var.value {
        Some(value) => {
          let value = interpreter(value.clone(), stack.clone(), modules).await?;
          if value.is_never() {
            return internal::AgalThrow::Params {
              type_error: ErrorNames::TypeError,
              message: "No se puede asignar \"nada\" a una variable".to_string(),
              stack: stack.clone(),
            }
            .to_result();
          }
          stack
            .env()
            .define(stack, &var.name, value, var.is_const, &node)
        }
        None => stack.env().define(
          stack.clone(),
          &var.name,
          AgalValue::Never.as_ref(),
          var.is_const,
          &node,
        ),
      },
      Node::While(while_node) => {
        let mut value = AgalValue::Never.as_ref();
        let body = &while_node.body.clone().to_node();
        loop {
          let condition = interpreter(while_node.condition.clone(), stack.clone(), modules.clone())
            .await?
            .to_agal_boolean(stack.clone())?;
          if !condition.as_bool() {
            break;
          }
          let body_node = body.clone().to_box();
          let stack = stack.crate_child(false, body_node.clone());
          value = interpreter(body.clone().to_box(), stack, modules.clone()).await?;
          if value.is_return() {
            return Ok(value);
          }
          if value.is_break() {
            break;
          }
        }
        Ok(value)
      }
    }?
    .to_result()
  })
}
