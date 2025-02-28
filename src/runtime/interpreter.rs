use std::{cell::RefCell, collections::HashMap, future::Future, path::Path, pin::Pin, rc::Rc};

use crate::{libraries, parser, path::absolute_path};

use super::{
  full_eval,
  values::{
    complex::{
      AgalArray, AgalClass, AgalClassProperty, AgalComplex, AgalFunction, AgalObject,
      AgalPromiseData,
    },
    internal::{self, AgalImmutable, AgalThrow},
    primitive,
    traits::{AgalValuable as _, ToAgalValue as _},
    AgalValue, ResultAgalValue,
  },
  RefStack, THIS_KEYWORD,
};

pub fn call_function_interpreter(
  block: parser::NodeBlock,
  stack: RefStack,
  modules: libraries::RefModules,
) -> Pin<Box<dyn Future<Output = ResultAgalValue> + Send>> {
  Box::pin(async move {
    let mut result = AgalValue::Never.as_ref();
    for statement in &block.body {
      result = async_interpreter(statement.to_box(), stack.clone(), modules.clone()).await?;
      if (result.is_return()) {
        break;
      }
    }
    Ok(result.into_return())
  })
}
pub fn async_interpreter(
  node: parser::BNode,
  stack: RefStack,
  modules: libraries::RefModules,
) -> Pin<Box<dyn Future<Output = ResultAgalValue>>> {
  let pre_stack = stack;
  let stack = pre_stack.clone().next(node.clone());
  Box::pin(async move {
    match node.clone().as_ref() {
      parser::Node::Array(list) => {
        let mut vec = vec![];
        for n in &list.elements {
          match n {
            parser::NodeProperty::Indexable(value) => {
              let data = async_interpreter(value.to_box(), stack.clone(), modules.clone()).await?;
              vec.push(data);
            }
            parser::NodeProperty::Iterable(iter) => {
              let data =
                async_interpreter(iter.clone().to_box(), stack.clone(), modules.clone()).await?;
              let list = data.to_agal_array(stack.clone(), modules.clone())?.un_ref();
              for n in list.to_vec().borrow().iter() {
                vec.push(n.clone());
              }
            }
            _ => {}
          }
        }
        Ok(AgalArray::from(vec).to_value().as_ref())
      }
      parser::Node::Assignment(assignment) => {
        let value =
          async_interpreter(assignment.value.clone(), stack.clone(), modules.clone()).await?;
        if value.borrow().is_never() {
          return Err(internal::AgalThrow::Params {
            type_error: parser::ErrorNames::TypeError,
            message: "No se puede asignar \"nada\" a una variable".to_string(),
            stack,
          });
        }
        match assignment.identifier.as_ref() {
          parser::Node::Identifier(identifier) => {
            stack
              .env()
              .assign(stack.clone(), &identifier.name, value, node.as_ref())
          }
          parser::Node::Member(member) => {
            if member.instance {
              return Err(internal::AgalThrow::Params {
                type_error: parser::ErrorNames::TypeError,
                message: "No se puede asignar una propiedad de instancia".to_string(),
                stack,
              });
            }
            let key = if !member.computed && member.member.is_identifier() {
              member.member.get_identifier().unwrap().name.clone()
            } else {
              async_interpreter(member.member.clone(), stack.clone(), modules.clone())
                .await?
                .to_agal_string(stack.clone(), modules.clone())?
                .to_string()
            };

            async_interpreter(member.object.clone(), stack.clone(), modules.clone())
              .await?
              .set_object_property(stack, &key, value)
          }
          _ => Ok(AgalValue::Never.as_ref()),
        }
      }
      parser::Node::Await(a) => {
        let value = async_interpreter(a.expression.clone(), stack.clone(), modules.clone()).await?;
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
          let agal_value = std::mem::replace(
            &mut value.data,
            AgalPromiseData::Resolved(AgalValue::Never.to_result()),
          ).resolve().await;
          value.data = AgalPromiseData::Resolved(agal_value.clone());
          return agal_value;
        }
        Ok(value)
      }
      parser::Node::Binary(binary) => {
        let operator = binary.operator;
        let left = async_interpreter(binary.left.clone(), stack.clone(), modules.clone()).await?;
        let right = async_interpreter(binary.right.clone(), stack.clone(), modules.clone()).await?;
        left.binary_operation(stack.clone(), operator, right, modules)
      }
      parser::Node::Block(block, true) => {
        let mut result = AgalValue::Never.as_ref();
        for statement in &block.body {
          result =
            async_interpreter(statement.clone().to_box(), stack.clone(), modules.clone()).await?;
          if (result.is_stop()) {
            break;
          }
        }
        Ok(result)
      }
      parser::Node::Import(import) => {
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
            None
          }
        };
        let module = module.ok_or_else(|| AgalThrow::Params {
          type_error: parser::ErrorNames::PathError,
          message: format!("No se encontro el modulo \"{}\"", import.path),
          stack: stack.clone(),
        })?;
        if let Some(n) = import.name.clone() {
          stack.env().define(stack, &n, module, true, &node);
        }
        AgalValue::Never.to_result()
      }
      parser::Node::Call(call) => {
        let callee = call.callee.clone();
        let (mut callee, this) = if let parser::Node::Member(member) = callee.as_ref() {
          let this =
            async_interpreter(member.object.clone(), stack.clone(), modules.clone()).await?;
          let this = this.clone();
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            // Ya es valido object::["key"]
            async_interpreter(member.member.clone(), stack.clone(), modules.clone())
              .await?
              .try_to_string(stack.clone(), modules.clone())?
          };
          let callee = this
            .clone()
            .get_instance_property(stack.clone(), &key, modules.clone())?;
          (callee, this)
        } else {
          let callee = async_interpreter(callee.clone(), stack.clone(), modules.clone()).await?;
          (callee.clone(), callee)
        };

        let mut args = vec![];
        for arg in &call.arguments {
          let arg = async_interpreter(arg.clone().to_box(), stack.clone(), modules.clone()).await?;
          args.push(arg);
        }
        callee
          .call(stack.clone(), this, args, modules.clone())?
          .into_return()
          .to_result()
      }
      parser::Node::Class(class) => {
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
              type_error: parser::ErrorNames::TypeError,
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
            async_interpreter(b.clone(), class_stack.clone(), modules.clone()).await?
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
      parser::Node::Console(parser::ast::NodeConsole::Full {
        identifier, value, ..
      }) => {
        let env = stack.env();
        let valid_variable =
          env._has(identifier) && !(env.is_constant(identifier) || env.is_keyword(identifier));
        if !valid_variable {
          return internal::AgalThrow::Params {
            type_error: parser::ErrorNames::EnvironmentError,
            message: format!("No se puede asignar a la variable \"{}\"", identifier),
            stack,
          }
          .to_result();
        }
        let value = async_interpreter(value.clone(), stack.clone(), modules.clone()).await?;
        let value = value.to_agal_string(stack.clone(), modules)?.to_string();
        print!("{value}");
        use std::io::Write as _;
        std::io::stdout().flush();
        let buf = &mut String::new();
        std::io::stdin().read_line(buf);
        let value = primitive::AgalString::from_string(buf.to_string()).to_ref_value();
        stack.env().assign(stack, &identifier, value, &node)
      }
      parser::Node::Console(parser::ast::NodeConsole::Output { value, .. }) => {
        let value = async_interpreter(value.clone(), stack.clone(), modules.clone()).await?;
        let value = value.to_agal_string(stack.clone(), modules)?.to_string();
        print!("{value}");
        use std::io::Write as _;
        std::io::stdout().flush();
        Ok(AgalValue::Never.as_ref())
      }
      parser::Node::DoWhile(do_while) => {
        let mut value = AgalValue::Never.as_ref();
        let mut condition = Ok(primitive::AgalBoolean::True);
        loop {
          if !condition.clone()?.as_bool() {
            break;
          }
          value = async_interpreter(
            do_while.body.clone().to_node().to_box(),
            stack.crate_child(false, node.clone()),
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
            async_interpreter(do_while.condition.clone(), stack.clone(), modules.clone()).await?;
          condition = pre_condition.to_agal_boolean(stack.clone(), modules.clone());
        }
        Ok(value)
      }
      parser::Node::Export(export) => match export.value.as_ref() {
        parser::Node::VarDecl(var) => {
          let value = async_interpreter(var.value.clone().unwrap(), stack.clone(), modules).await?;
          stack
            .env()
            .define(stack.clone(), &var.name, value.clone(), var.is_const, &node);
          AgalValue::Export(var.name.clone(), value).to_result()
        }
        parser::Node::Function(func) => {
          let (name, function) = interpreter_function(func, stack, node);
          AgalValue::Export(name, function).to_result()
        }
        parser::Node::Name(name) => {
          let value = stack.env().get(stack, &name.name, &node)?;
          AgalValue::Export(name.name.clone(), value).to_result()
        }
        parser::Node::Class(class) => {
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
                type_error: parser::ErrorNames::TypeError,
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
              async_interpreter(b.clone(), class_stack.clone(), modules.clone()).await?
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
          type_error: parser::ErrorNames::SyntaxError,
          message: "Se nesesita un nombre para las exportaciones".to_string(),
          stack,
        }
        .to_result(),
      },
      parser::Node::For(for_node) => {
        let mut value = AgalValue::Never.as_ref();
        let stack = pre_stack.crate_child(false, node);
        async_interpreter(for_node.init.clone(), stack.clone(), modules.clone()).await?; // init value: def i = 0;
        loop {
          if async_interpreter(for_node.condition.clone(), stack.clone(), modules.clone())
            .await?
            .to_agal_boolean(stack.clone(), modules.clone())?
            .not()
            .as_bool()
          // condition value: i < 10
          {
            break;
          }
          let node = for_node.body.clone().to_node().to_box();
          value = async_interpreter(
            node.clone(),
            stack.crate_child(false, node),
            modules.clone(),
          )
          .await?; // Block {..}
          let v = value.un_ref();
          if v.is_return() {
            return value.to_result();
          }
          if v.is_break() {
            break;
          }
          async_interpreter(for_node.update.clone(), stack.clone(), modules.clone()).await?;
          // advance value: i+=1
        }
        value.to_result()
      }
      parser::Node::If(if_node) => {
        let condition =
          async_interpreter(if_node.condition.clone(), stack.clone(), modules.clone())
            .await?
            .to_agal_boolean(stack.clone(), modules.clone())?;
        if condition.as_bool() {
          return async_interpreter(if_node.body.clone().to_node().to_box(), stack, modules).await;
        }
        if let Some(else_body) = &if_node.else_body {
          return async_interpreter(else_body.clone().to_node().to_box(), stack.clone(), modules)
            .await;
        }
        return AgalValue::Never.to_result();
      }
      parser::Node::Member(member) => {
        let mut object =
          async_interpreter(member.object.clone(), stack.clone(), modules.clone()).await?;
        if member.instance && !member.computed && member.member.is_identifier() {
          let key = member.member.get_identifier().unwrap().name.clone();
          object.get_instance_property(stack, &key, modules)
        } else {
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            async_interpreter(member.member.clone(), stack.clone(), modules.clone())
              .await?
              .to_agal_string(stack.clone(), modules)?
              .to_string()
          };
          object.get_object_property(stack, &key)
        }
      }
      parser::Node::Object(obj) => {
        let mut hashmap = HashMap::new();
        for prop in &obj.properties {
          match prop {
            parser::NodeProperty::Property(key, value) => {
              let value =
                async_interpreter(value.clone().to_box(), stack.clone(), modules.clone()).await?;
              hashmap.insert(key.clone(), value);
            }
            parser::NodeProperty::Dynamic(key, value) => {
              let key =
                async_interpreter(key.clone().to_box(), stack.clone(), modules.clone()).await?;
              let key = key
                .to_agal_string(stack.clone(), modules.clone())?
                .to_string();
              let value =
                async_interpreter(value.clone().to_box(), stack.clone(), modules.clone()).await?;
              hashmap.insert(key, value);
            }
            parser::NodeProperty::Iterable(iter) => {
              let mut value =
                async_interpreter(iter.clone().to_box(), stack.clone(), modules.clone()).await?;
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
      parser::Node::Program(program) => {
        async_interpreter(program.body.clone().to_node().to_box(), stack, modules).await
      }
      parser::Node::Return(ret) => {
        let value = match &ret.value {
          None => AgalValue::Never.to_ref_value(),
          Some(value) => async_interpreter(value.clone(), stack, modules).await?,
        };
        AgalValue::Return(value).to_result()
      }
      parser::Node::Throw(throw) => {
        let value = async_interpreter(throw.value.clone(), stack.clone(), modules).await?;
        internal::AgalThrow::Value(value).to_result()
      }
      parser::Node::Try(try_node) => {
        let try_box_node = try_node.body.clone().to_node().to_box();
        let try_stack = stack.crate_child(false, try_box_node.clone());
        let try_val = async_interpreter(try_box_node, try_stack.clone(), modules.clone()).await;
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
              async_interpreter(node_catch, stack.clone(), modules.clone()).await?
            }
          },
          Ok(val) => val,
        };
        if let Some(f) = &try_node.finally {
          let node_finally = f.clone().to_node().to_box();
          let stack = pre_stack.crate_child(false, node_finally.clone());
          interpreter(node_finally, stack, modules)?
        } else {
          value
        }
        .to_result()
      }
      parser::Node::UnaryBack(unary) => {
        let value = async_interpreter(unary.operand.clone(), stack.clone(), modules).await;
        if unary.operator == parser::ast::NodeOperator::QuestionMark {
          match &value {
            Ok(_) => value,
            Err(throw) => AgalValue::Null.to_result(),
          }
        } else {
          AgalThrow::Params {
            type_error: parser::ErrorNames::SyntaxError,
            message: format!(
              "No se puede usar el operador '{}' para una operacion unitaria trasera",
              unary.operator
            ),
            stack,
          }
          .to_result()
        }
      }
      parser::Node::UnaryFront(unary) => {
        let value =
          async_interpreter(unary.operand.clone(), stack.clone(), modules.clone()).await?;
        if unary.operator == parser::ast::NodeOperator::QuestionMark {
          value.to_agal_boolean(stack, modules)?.to_result()
        } else if unary.operator == parser::ast::NodeOperator::Not {
          value.to_agal_boolean(stack, modules)?.not().to_result()
        } else if unary.operator == parser::ast::NodeOperator::BitAnd {
          AgalImmutable::new(value).to_result()
        } else if unary.operator == parser::ast::NodeOperator::Plus {
          value.to_agal_number(stack, modules)?.to_result()
        } else if unary.operator == parser::ast::NodeOperator::Minus {
          value.to_agal_number(stack, modules)?.neg().to_result()
        } else if unary.operator == parser::ast::NodeOperator::Approximate {
          value.to_agal_number(stack, modules)?.floor().to_result()
        } else {
          AgalThrow::Params {
            type_error: parser::ErrorNames::SyntaxError,
            message: format!(
              "No se puede usar el operador '{}' para una operacion unitaria frontal",
              unary.operator
            ),
            stack,
          }
          .to_result()
        }
      }
      parser::Node::VarDecl(var) => match &var.value {
        Some(value) => {
          let value = async_interpreter(value.clone(), stack.clone(), modules).await?;
          if value.is_never() {
            return internal::AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
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
      parser::Node::While(while_node) => {
        let mut value = AgalValue::Never.as_ref();
        let body = &while_node.body.clone().to_node();
        loop {
          let condition =
            async_interpreter(while_node.condition.clone(), stack.clone(), modules.clone())
              .await?
              .to_agal_boolean(stack.clone(), modules.clone())?;
          if !condition.as_bool() {
            break;
          }
          let body_node = body.clone().to_box();
          let stack = stack.crate_child(false, body_node.clone());
          value = async_interpreter(body.clone().to_box(), stack, modules.clone()).await?;
          if value.is_return() {
            return Ok(value);
          }
          if value.is_break() {
            break;
          }
        }
        Ok(value)
      }
      _ => interpreter(node, stack, modules),
    }
  })
}
pub fn interpreter(
  node: parser::BNode,
  stack: RefStack,
  modules: libraries::RefModules,
) -> ResultAgalValue {
  let pre_stack = stack;
  let stack = pre_stack.clone().next(node.clone());
  match node.clone().as_ref() {
    parser::Node::Array(list) => {
      let mut vec = vec![];
      for n in &list.elements {
        match n {
          parser::NodeProperty::Indexable(value) => {
            let data = interpreter(value.to_box(), stack.clone(), modules.clone())?;
            vec.push(data);
          }
          parser::NodeProperty::Iterable(iter) => {
            let data = interpreter(iter.clone().to_box(), stack.clone(), modules.clone())?;
            let list = data.to_agal_array(stack.clone(), modules.clone())?.un_ref();
            for n in list.to_vec().borrow().iter() {
              vec.push(n.clone());
            }
          }
          _ => {}
        }
      }
      Ok(AgalArray::from(vec).to_value().as_ref())
    }
    parser::Node::Assignment(assignment) => {
      let value = interpreter(assignment.value.clone(), stack.clone(), modules.clone())?;
      if value.borrow().is_never() {
        return Err(internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: "No se puede asignar \"nada\" a una variable".to_string(),
          stack,
        });
      }
      match assignment.identifier.as_ref() {
        parser::Node::Identifier(identifier) => {
          stack
            .env()
            .assign(stack.clone(), &identifier.name, value, node.as_ref())
        }
        parser::Node::Member(member) => {
          if member.instance {
            return Err(internal::AgalThrow::Params {
              type_error: parser::ErrorNames::TypeError,
              message: "No se puede asignar una propiedad de instancia".to_string(),
              stack,
            });
          }
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            interpreter(member.member.clone(), stack.clone(), modules.clone())?
              .to_agal_string(stack.clone(), modules.clone())?
              .to_string()
          };

          interpreter(member.object.clone(), stack.clone(), modules.clone())?
            .set_object_property(stack, &key, value)
        }
        _ => Ok(AgalValue::Never.as_ref()),
      }
    }
    parser::Node::Binary(binary) => {
      let operator = binary.operator;
      let left = interpreter(binary.left.clone(), stack.clone(), modules.clone())?;
      let right = interpreter(binary.right.clone(), stack.clone(), modules.clone())?;
      left.binary_operation(stack.clone(), operator, right, modules)
    }
    parser::Node::Block(block, false) => {
      let mut result = AgalValue::Never.as_ref();
      for statement in &block.body {
        result = interpreter(statement.clone().to_box(), stack.clone(), modules.clone())?;
        if (result.is_stop()) {
          break;
        }
      }
      Ok(result)
    }
    parser::Node::Byte(byte_node) => Ok(primitive::AgalByte::new(byte_node.value).to_ref_value()),
    parser::Node::Call(call) => {
      let callee = call.callee.clone();
      let (mut callee, this) = if let parser::Node::Member(member) = callee.as_ref() {
        let this = interpreter(member.object.clone(), stack.clone(), modules.clone())?;
        let this = this.clone();
        let key = if !member.computed && member.member.is_identifier() {
          member.member.get_identifier().unwrap().name.clone()
        } else {
          // Ya es valido object::["key"]
          interpreter(member.member.clone(), stack.clone(), modules.clone())?
            .try_to_string(stack.clone(), modules.clone())?
        };
        let callee = this
          .clone()
          .get_instance_property(stack.clone(), &key, modules.clone())?;
        (callee, this)
      } else {
        let callee = interpreter(callee.clone(), stack.clone(), modules.clone())?;
        (callee.clone(), callee)
      };

      let mut args = vec![];
      for arg in &call.arguments {
        let arg = interpreter(arg.clone().to_box(), stack.clone(), modules.clone())?;
        args.push(arg);
      }
      callee.call(stack.clone(), this, args, modules.clone())
    }
    parser::Node::Class(class) => {
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
            type_error: parser::ErrorNames::TypeError,
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
          interpreter(b.clone(), class_stack.clone(), modules.clone())?
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
    parser::Node::Console(parser::ast::NodeConsole::Full {
      identifier, value, ..
    }) => {
      let env = stack.env();
      let valid_variable =
        env._has(identifier) && !(env.is_constant(identifier) || env.is_keyword(identifier));
      if !valid_variable {
        return internal::AgalThrow::Params {
          type_error: parser::ErrorNames::EnvironmentError,
          message: format!("No se puede asignar a la variable \"{}\"", identifier),
          stack,
        }
        .to_result();
      }
      let value = interpreter(value.clone(), stack.clone(), modules.clone())?;
      let value = value.to_agal_string(stack.clone(), modules)?.to_string();
      print!("{value}");
      use std::io::Write as _;
      std::io::stdout().flush();
      let buf = &mut String::new();
      std::io::stdin().read_line(buf);
      let value = primitive::AgalString::from_string(buf.to_string()).to_ref_value();
      stack.env().assign(stack, &identifier, value, &node)
    }
    parser::Node::Console(parser::ast::NodeConsole::Output { value, .. }) => {
      let value = interpreter(value.clone(), stack.clone(), modules.clone())?;
      let value = value.to_agal_string(stack.clone(), modules)?.to_string();
      print!("{value}");
      use std::io::Write as _;
      std::io::stdout().flush();
      Ok(AgalValue::Never.as_ref())
    }
    parser::Node::Console(parser::ast::NodeConsole::Input { identifier, .. }) => {
      let env = stack.env();
      let valid_variable =
        env._has(identifier) && !(env.is_constant(identifier) || env.is_keyword(identifier));
      if !valid_variable {
        return internal::AgalThrow::Params {
          type_error: parser::ErrorNames::EnvironmentError,
          message: format!("No se puede asignar a la variable \"{}\"", identifier),
          stack,
        }
        .to_result();
      }
      let buf = &mut String::new();
      std::io::stdin().read_line(buf);
      let value = primitive::AgalString::from_string(buf.to_string()).to_ref_value();
      stack.env().assign(stack, &identifier, value, &node)
    }
    parser::Node::DoWhile(do_while) => {
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
        )?;
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
          interpreter(do_while.condition.clone(), stack.clone(), modules.clone())?;
        condition = pre_condition.to_agal_boolean(stack.clone(), modules.clone());
      }
      Ok(value)
    }
    parser::Node::Export(export) => match export.value.as_ref() {
      parser::Node::VarDecl(var) => {
        let value = interpreter(var.value.clone().unwrap(), stack.clone(), modules)?;
        stack
          .env()
          .define(stack.clone(), &var.name, value.clone(), var.is_const, &node);
        AgalValue::Export(var.name.clone(), value).to_result()
      }
      parser::Node::Function(func) => {
        let (name, function) = interpreter_function(func, stack, node);
        AgalValue::Export(name, function).to_result()
      }
      parser::Node::Name(name) => {
        let value = stack.env().get(stack, &name.name, &node)?;
        AgalValue::Export(name.name.clone(), value).to_result()
      }
      parser::Node::Class(class) => {
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
              type_error: parser::ErrorNames::TypeError,
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
            interpreter(b.clone(), class_stack.clone(), modules.clone())?
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
        type_error: parser::ErrorNames::SyntaxError,
        message: "Se nesesita un nombre para las exportaciones".to_string(),
        stack,
      }
      .to_result(),
    },
    parser::Node::For(for_node) => {
      let mut value = AgalValue::Never.as_ref();
      let mut condition = Ok(primitive::AgalBoolean::True);
      let stack = pre_stack.crate_child(false, node);
      interpreter(for_node.init.clone(), stack.clone(), modules.clone())?; // init value
      loop {
        if !condition?.as_bool() {
          break;
        }
        let node = for_node.body.clone().to_node().to_box();
        value = interpreter(
          node.clone(),
          stack.crate_child(false, node),
          modules.clone(),
        )?;
        let v = value.un_ref();
        if v.is_return() {
          return value.to_result();
        }
        if v.is_break() {
          break;
        }
        let pre_condition =
          interpreter(for_node.condition.clone(), stack.clone(), modules.clone())?;
        condition = pre_condition.to_agal_boolean(stack.clone(), modules.clone());
        let pre_update = interpreter(for_node.update.clone(), stack.clone(), modules.clone())?;
      }
      value.to_result()
    }
    parser::Node::Function(func) => Ok(interpreter_function(func, stack, node).1),
    parser::Node::Identifier(id) => stack.env().get(stack, &id.name, &node),
    parser::Node::If(if_node) => {
      let condition = interpreter(if_node.condition.clone(), stack.clone(), modules.clone())?
        .to_agal_boolean(stack.clone(), modules.clone())?;
      if condition.as_bool() {
        return interpreter(if_node.body.clone().to_node().to_box(), stack, modules);
      }
      if let Some(else_body) = &if_node.else_body {
        return interpreter(else_body.clone().to_node().to_box(), stack.clone(), modules);
      }
      return AgalValue::Never.to_result();
    }
    parser::Node::Lazy(node) => internal::AgalLazy::new(node.clone(), stack, modules).to_result(),
    parser::Node::LoopEdit(loop_edit) => match loop_edit.action {
      parser::NodeLoopEditType::Break => AgalValue::Break,
      parser::NodeLoopEditType::Continue => AgalValue::Continue,
    }
    .to_result(),
    parser::Node::Member(member) => {
      let mut object = interpreter(member.object.clone(), stack.clone(), modules.clone())?;
      if member.instance && !member.computed && member.member.is_identifier() {
        let key = member.member.get_identifier().unwrap().name.clone();
        object.get_instance_property(stack, &key, modules)
      } else {
        let key = if !member.computed && member.member.is_identifier() {
          member.member.get_identifier().unwrap().name.clone()
        } else {
          interpreter(member.member.clone(), stack.clone(), modules.clone())?
            .to_agal_string(stack.clone(), modules)?
            .to_string()
        };
        object.get_object_property(stack, &key)
      }
    }
    parser::Node::Name(name) => stack.env().get(stack, &name.name, &node),
    parser::Node::None => AgalValue::Never.to_result(),
    parser::Node::Number(num) => if num.base == 10 && num.value.contains('.') {
      let d = str::parse::<f32>(&num.value).unwrap();
      primitive::AgalNumber::Decimal(d)
    } else {
      let i = i32::from_str_radix(&num.value, num.base as u32).unwrap();
      primitive::AgalNumber::Integer(i)
    }
    .to_result(),
    parser::Node::Object(obj) => {
      let mut hashmap = HashMap::new();
      for prop in &obj.properties {
        match prop {
          parser::NodeProperty::Property(key, value) => {
            let value = interpreter(value.clone().to_box(), stack.clone(), modules.clone())?;
            hashmap.insert(key.clone(), value);
          }
          parser::NodeProperty::Dynamic(key, value) => {
            let key = interpreter(key.clone().to_box(), stack.clone(), modules.clone())?;
            let key = key
              .to_agal_string(stack.clone(), modules.clone())?
              .to_string();
            let value = interpreter(value.clone().to_box(), stack.clone(), modules.clone())?;
            hashmap.insert(key, value);
          }
          parser::NodeProperty::Iterable(iter) => {
            let mut value = interpreter(iter.clone().to_box(), stack.clone(), modules.clone())?;
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
    parser::Node::Return(ret) => {
      if ret.value.is_none() {
        return AgalValue::Return(AgalValue::Never.as_ref()).to_result();
      }
      let ret_value = ret.value.clone().unwrap();
      let value = interpreter(ret_value, stack, modules)?;
      AgalValue::Return(value).to_result()
    }
    parser::Node::String(str) => {
      let mut string = String::new();
      for s in &str.value {
        match &s {
          parser::StringData::Id(id) => {
            let data = stack
              .clone()
              .env()
              .get(stack.clone(), id, &node)?
              .to_agal_string(stack.clone(), modules.clone())?
              .to_string();
            string.push_str(&data)
          }
          parser::StringData::Str(str) => string.push_str(str),
        }
      }
      primitive::AgalString::from_string(string).to_result()
    }
    parser::Node::Throw(throw) => {
      let value = interpreter(throw.value.clone(), stack.clone(), modules)?;
      internal::AgalThrow::Value(value).to_result()
    }
    parser::Node::Try(try_node) => {
      let try_box_node = try_node.body.clone().to_node().to_box();
      let try_stack = stack.crate_child(false, try_box_node.clone());
      let try_val = interpreter(try_box_node, try_stack.clone(), modules.clone());
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
            interpreter(node_catch, stack.clone(), modules.clone())?
          }
        },
        Ok(val) => val,
      };
      if let Some(f) = &try_node.finally {
        let node_finally = f.clone().to_node().to_box();
        let stack = pre_stack.crate_child(false, node_finally.clone());
        interpreter(node_finally, stack, modules)?
      } else {
        value
      }
      .to_result()
    }
    parser::Node::UnaryBack(unary) => {
      let value = interpreter(unary.operand.clone(), stack.clone(), modules);
      if unary.operator == parser::ast::NodeOperator::QuestionMark {
        match &value {
          Ok(_) => value,
          Err(throw) => AgalValue::Null.to_result(),
        }
      } else {
        AgalThrow::Params {
          type_error: parser::ErrorNames::SyntaxError,
          message: format!(
            "No se puede usar el operador '{}' para una operacion unitaria trasera",
            unary.operator
          ),
          stack,
        }
        .to_result()
      }
    }
    parser::Node::UnaryFront(unary) => {
      let value = interpreter(unary.operand.clone(), stack.clone(), modules.clone())?;
      if unary.operator == parser::ast::NodeOperator::QuestionMark {
        value.to_agal_boolean(stack, modules)?.to_result()
      } else if unary.operator == parser::ast::NodeOperator::Not {
        value.to_agal_boolean(stack, modules)?.not().to_result()
      } else if unary.operator == parser::ast::NodeOperator::BitAnd {
        AgalImmutable::new(value).to_result()
      } else if unary.operator == parser::ast::NodeOperator::Plus {
        value.to_agal_number(stack, modules)?.to_result()
      } else if unary.operator == parser::ast::NodeOperator::Minus {
        value.to_agal_number(stack, modules)?.neg().to_result()
      } else if unary.operator == parser::ast::NodeOperator::Approximate {
        value.to_agal_number(stack, modules)?.floor().to_result()
      } else {
        AgalThrow::Params {
          type_error: parser::ErrorNames::SyntaxError,
          message: format!(
            "No se puede usar el operador '{}' para una operacion unitaria frontal",
            unary.operator
          ),
          stack,
        }
        .to_result()
      }
    }
    parser::Node::VarDecl(var) => match &var.value {
      Some(value) => {
        let value = interpreter(value.clone(), stack.clone(), modules)?;
        if value.is_never() {
          return internal::AgalThrow::Params {
            type_error: parser::ErrorNames::TypeError,
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
    parser::Node::While(while_node) => {
      let mut value = AgalValue::Never.as_ref();
      let body = &while_node.body.clone().to_node();
      loop {
        let condition = interpreter(while_node.condition.clone(), stack.clone(), modules.clone())?
          .to_agal_boolean(stack.clone(), modules.clone())?;
        if !condition.as_bool() {
          break;
        }
        let body_node = body.clone().to_box();
        let stack = stack.crate_child(false, body_node.clone());
        value = interpreter(body.clone().to_box(), stack, modules.clone())?;
        if value.is_return() {
          return Ok(value);
        }
        if value.is_break() {
          break;
        }
      }
      Ok(value)
    }
    parser::Node::VarDel(node_identifier) => {
      let with_variable = stack
        .env()
        .resolve(&node_identifier.name, node.as_ref())
        .delete(&node_identifier.name);
      AgalValue::default().to_result()
    }
    parser::Node::Import(_)
    | parser::Node::Await(_)
    | parser::Node::Block(_, true)
    | parser::Node::Program(_) => Err(AgalThrow::Params {
      type_error: parser::ErrorNames::SyntaxError,
      message: "Se intento ejecutar codigo asincrono en un bloque sincrono".into(),
      stack,
    }),
  }
}

fn interpreter_function(
  func: &parser::NodeFunction,
  stack: RefStack,
  node: parser::BNode,
) -> (String, super::values::RefAgalValue<AgalValue>) {
  let function = AgalFunction::new(
    func.name.clone(),
    func.is_async,
    func.params.clone(),
    func.body.clone(),
    stack.env(),
  )
  .to_ref_value();
  if "" != func.name {
    stack
      .env()
      .define(stack, &func.name, function.clone(), true, &node);
  }
  (func.name.clone(), function)
}
