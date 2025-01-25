use parser::{
  ast::{Node, NodeLoopEditType, NodeProperty, StringData},
  internal::ErrorNames,
  util::RefValue,
};
use std::{
  cell::RefCell,
  collections::HashMap,
  future::{Future, IntoFuture},
  path::Path,
  pin::Pin,
  rc::Rc,
};

use crate::{
  path::absolute_path,
  runtime::{
    env::THIS_KEYWORD, full_eval, AgalArray, AgalBoolean, AgalByte, AgalClass, AgalClassProperty,
    AgalComplex, AgalError, AgalFunction, AgalInternal, AgalNumber, AgalObject, AgalPrototype,
    AgalString, AgalThrow, AgalValuable, AgalValuableManager, AgalValue, Environment, RefAgalValue,
    Stack,
  },
  Modules, ToResult,
};

pub fn interpreter<'a>(
  node: Box<Node>,
  stack: RefValue<Stack>,
  env: Rc<RefCell<Environment<'a>>>,
  modules_manager: RefValue<Modules<'a>>,
) -> Pin<Box<dyn Future<Output = RefAgalValue<'a>>>> {
  Box::pin(async move {
    let pre_stack = stack;
    let stack = pre_stack.borrow().next(&node).to_ref();
    match node.as_ref() {
      Node::Array(list) => {
        let mut vec = vec![];
        for n in list.elements.iter() {
          match n {
            NodeProperty::Indexable(value) => {
              let data = interpreter(
                value.clone().to_box(),
                stack.clone(),
                env.clone(),
                modules_manager.clone(),
              )
              .await;
              vec.push(data);
            }
            NodeProperty::Iterable(iter) => {
              let data = interpreter(
                iter.clone().to_box(),
                stack.clone(),
                env.clone(),
                modules_manager.clone(),
              )
              .await;
              let list = data.borrow().to_agal_array(&stack.borrow());
              match list {
                Ok(list) => {
                  let list = &*list.borrow();
                  for n in list.get_vec().borrow().iter() {
                    vec.push(n.clone());
                  }
                }
                Err(err) => return err.to_ref_value(),
              }
            }
            _ => {}
          }
        }
        AgalArray::from_vec(vec).to_value().as_ref()
      }
      Node::Assignment(assignment) => {
        let value = interpreter(
          assignment.value.clone(),
          stack.clone(),
          env.clone(),
          modules_manager.clone(),
        )
        .await;
        if value.borrow().is_never() {
          return AgalThrow::Params {
            type_error: ErrorNames::TypeError,
            message: "No se puede asignar \"nada\" a una variable",
            stack: Box::new(stack.borrow().clone()),
          }
          .to_ref_value();
        }
        match assignment.identifier.as_ref() {
          Node::Identifier(identifier) => {
            env
              .borrow_mut()
              .assign(&stack.borrow(), &identifier.name, value, node.as_ref())
          }
          Node::Member(member) => {
            if member.instance {
              return AgalThrow::Params {
                type_error: ErrorNames::TypeError,
                message: "No se puede asignar una propiedad de instancia",
                stack: Box::new(stack.borrow().clone()),
              }
              .to_ref_value();
            }
            let key = if !member.computed && member.member.is_identifier() {
              member.member.get_identifier().unwrap().name.clone()
            } else {
              let key = interpreter(
                member.member.clone(),
                stack.clone(),
                env.clone(),
                modules_manager.clone(),
              )
              .await;
              let key = key.borrow().to_agal_string(&stack.borrow(), env.clone());
              match key {
                Ok(key) => key.get_string().to_string(),
                Err(err) => return err.to_ref_value(),
              }
            };

            let object = interpreter(
              member.object.clone(),
              stack.clone(),
              env.clone(),
              modules_manager,
            )
            .await;
            object
              .clone()
              .borrow()
              .set_object_property(&stack.borrow(), env, key, value.clone())
          }
          _ => AgalValue::Never.as_ref(),
        }
      }
      Node::Await(a) => {
        let value: Rc<RefCell<AgalValue<'a>>> = interpreter(
          a.expression.clone(),
          stack.clone(),
          env.clone(),
          modules_manager,
        )
        .await;
        let m_value = value.clone();
        let m_value = *m_value.borrow();

        match m_value {
          AgalValue::Complex(AgalComplex::Promise(a)) => {
            let a = a.clone();
            match &a.into_future().await {
              Ok(v) => v.clone(),
              Err(e) => AgalThrow::from_ref_manager(
                AgalValue::Never.to_ref_value(),
                &stack.borrow(),
                env.clone(),
              )
              .to_ref_value(),
            }
          }
          _ => value,
        }
      }
      Node::Binary(binary) => {
        let left = interpreter(
          binary.left.clone(),
          stack.clone(),
          env.clone(),
          modules_manager.clone(),
        )
        .await
        .await;
        let right = interpreter(
          binary.right.clone(),
          stack.clone(),
          env.clone(),
          modules_manager.clone(),
        )
        .await
        .await;
        left.clone().borrow().binary_operation(
          &stack.borrow(),
          env.clone(),
          &binary.operator,
          right,
        )
      }
      Node::Block(block) => {
        let env = env.borrow().clone().crate_child(false).as_ref();
        let mut returned_value = AgalValue::Never.as_ref();
        for n in block.body.iter() {
          let value = interpreter(
            n.clone().to_box(),
            pre_stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          if value.borrow().is_return() || value.borrow().is_throw() {
            returned_value = value;
          }
          if value.borrow().is_stop() {
            break;
          }
        }
        returned_value
      }
      Node::Byte(byte_node) => AgalByte::new(byte_node.value).to_ref_value(),
      Node::Call(call) => {
        let callee = call.callee.as_ref();
        let (callee, this) = if let Node::Member(member) = callee {
          let this = interpreter(
            member.object.clone(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          let this = this.clone();
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            // No deberia ser posible llegar a este punto
            let key = interpreter(
              member.member.clone(),
              stack.clone(),
              env.clone(),
              modules_manager.clone(),
            )
            .await
            .await;
            let key = key.borrow().to_agal_string(&stack.borrow(), env.clone());
            match key {
              Ok(key) => key.get_string().to_string(),
              Err(err) => return err.to_ref_value(),
            }
          };
          let callee =
            this
              .clone()
              .borrow()
              .get_instance_property(&stack.borrow(), env.clone(), key);
          (callee, this)
        } else {
          let callee = interpreter(
            callee.clone().to_box(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          (callee.clone(), callee)
        };

        let mut args = vec![];
        for arg in call.arguments.iter() {
          let arg = interpreter(
            arg.clone().to_box(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          if arg.borrow().is_throw() {
            return arg;
          }
          args.push(arg);
        }
        callee
          .clone()
          .borrow()
          .call(
            &stack.borrow(),
            env.clone(),
            this,
            args,
            &modules_manager.borrow(),
          )
          .await
      }
      Node::Class(class) => {
        let extend_of_value = if let Some(extend) = &class.extend_of {
          let value = env.borrow().get(&stack.borrow(), &extend.name, &node);
          let value: &AgalValue = &value.borrow();
          let value = value.clone();
          match value {
            AgalValue::Complex(AgalComplex::Class(class)) => {
              Some(Rc::new(RefCell::new(class.clone())))
            }
            AgalValue::Internal(AgalInternal::Throw(th)) => return th.to_ref_value(),
            _ => {
              return AgalThrow::Params {
                type_error: ErrorNames::TypeError,
                message: "Solo se puede extender de otras clases".to_string(),
                stack: Box::new(stack.borrow().clone()),
              }
              .to_ref_value()
            }
          }
        } else {
          None
        };
        let mut properties = Vec::new();
        let mut _class_env = env.borrow().clone().crate_child(true);
        let class_env = _class_env.clone().as_ref();
        for property in class.body.iter() {
          let is_static = (property.meta & 1) != 0;
          let is_public = (property.meta & 2) != 0;

          let value = if let Some(b) = &property.value {
            interpreter(
              b.clone(),
              stack.clone(),
              class_env.clone(),
              modules_manager.clone(),
            )
            .await
            .await
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
        _class_env.set(THIS_KEYWORD, class_value.clone());
        env
          .borrow_mut()
          .define(&stack.borrow(), &class.name, class_value, true, &node)
      }
      Node::DoWhile(do_while) => {
        let mut value = AgalValue::Never.as_ref();
        let mut condition: Result<AgalBoolean, AgalThrow> = Ok(AgalBoolean::new(true));
        loop {
          match condition {
            Ok(condition) => {
              if !condition.to_bool() {
                break;
              }
            }
            Err(err) => return err.to_ref_value(),
          }
          value = interpreter(
            do_while.body.clone().to_node().to_box(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          let cv = value.clone();
          let v = cv.borrow();
          if v.is_return() {
            return value;
          }
          if v.is_throw() {
            return value;
          }
          if v.is_break() {
            break;
          }
          let pre_condition = interpreter(
            do_while.condition.clone(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          condition = pre_condition
            .borrow()
            .to_agal_boolean(&stack.borrow(), env.clone());
        }
        value
      }
      Node::Error(error) => AgalThrow::Error(AgalError::new(
        ErrorNames::SyntaxError,
        error.message.clone(),
        Box::new(stack.borrow().clone()),
      ))
      .to_ref_value(),
      Node::Export(export) => match export.value.as_ref() {
        Node::VarDecl(var) => {
          let value = interpreter(
            var.value.clone().unwrap(),
            stack.clone(),
            env.clone(),
            modules_manager,
          )
          .await
          .await;
          env.borrow_mut().define(
            &stack.borrow(),
            &var.name,
            value.clone(),
            var.is_const,
            &node,
          );
          AgalValue::Export(var.name.clone(), value).as_ref()
        }
        Node::Function(func) => {
          let function =
            AgalFunction::new(func.params.clone(), func.body.clone(), env.clone()).to_ref_value();
          env
            .borrow_mut()
            .define(&stack.borrow(), &func.name, function.clone(), true, &node);
          AgalValue::Export(func.name.clone(), function).as_ref()
        }
        Node::Name(name) => {
          let value = env.borrow().get(&stack.borrow(), &name.name, &node);
          AgalValue::Export(name.name.clone(), value).as_ref()
        }
        Node::Class(class) => {
          let extend_of_value = if let Some(extend) = &class.extend_of {
            let value = env.borrow().get(&stack.borrow(), &extend.name, &node);
            let value: &AgalValue = &value.borrow();
            let value = value.clone();
            match value {
              AgalValue::Complex(AgalComplex::Class(class)) => {
                Some(Rc::new(RefCell::new(class.clone())))
              }
              AgalValue::Internal(AgalInternal::Throw(th)) => return th.to_ref_value(),
              _ => {
                return AgalThrow::Params {
                  type_error: ErrorNames::TypeError,
                  message: "Solo se puede extender de otras clases".to_string(),
                  stack: Box::new(stack.borrow().clone()),
                }
                .to_ref_value()
              }
            }
          } else {
            None
          };
          let mut properties = Vec::new();
          let mut _class_env = env.borrow().clone().crate_child(true);
          let class_env = _class_env.clone().as_ref();
          for property in class.body.iter() {
            let is_static = (property.meta & 1) != 0;
            let is_public = (property.meta & 2) != 0;

            let value = if let Some(b) = &property.value {
              interpreter(
                b.clone(),
                stack.clone(),
                class_env.clone(),
                modules_manager.clone(),
              )
              .await
              .await
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
          _class_env.set(THIS_KEYWORD, class_value.clone());
          env.borrow_mut().define(
            &stack.borrow(),
            &class.name,
            class_value.clone(),
            true,
            &node,
          );

          AgalValue::Export(class.name.clone(), class_value).as_ref()
        }
        _ => AgalThrow::Params {
          type_error: ErrorNames::SyntaxError,
          message: "Se nesesita un nombre para las exportaciones".to_string(),
          stack: Box::new(stack.borrow().clone()),
        }
        .to_value()
        .as_ref(),
      },
      Node::For(for_node) => {
        let mut value = AgalValue::Never.as_ref();
        let mut condition: Result<AgalBoolean, AgalThrow> = Ok(AgalBoolean::new(true));
        let env = env.borrow().clone().crate_child(false).as_ref();
        interpreter(
          for_node.init.clone(),
          stack.clone(),
          env.clone(),
          modules_manager.clone(),
        ); // init value
        loop {
          match condition {
            Ok(condition) => {
              if !condition.to_bool() {
                break;
              }
            }
            Err(err) => return err.to_ref_value(),
          }
          value = interpreter(
            for_node.body.clone().to_node().to_box(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          let cv = value.clone();
          let v = cv.borrow();
          if v.is_return() {
            return value;
          }
          if v.is_throw() {
            return value;
          }
          if v.is_break() {
            break;
          }
          let pre_condition = interpreter(
            for_node.condition.clone(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          condition = pre_condition
            .borrow()
            .to_agal_boolean(&stack.borrow(), env.clone());
          let pre_update = interpreter(
            for_node.update.clone(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          if pre_update.borrow().is_throw() {
            return pre_update;
          }
        }
        value
      }
      Node::Function(func) => {
        let function =
          AgalFunction::new(func.params.clone(), func.body.clone(), env.clone()).to_ref_value();
        env
          .borrow_mut()
          .define(&stack.borrow(), &func.name, function, true, &node)
      }
      Node::Identifier(id) => env.borrow().get(&stack.borrow(), &id.name, &node),
      Node::If(if_node) => {
        let condition = interpreter(
          if_node.condition.clone(),
          stack.clone(),
          env.clone(),
          modules_manager.clone(),
        )
        .await;
        let condition = condition
          .await
          .borrow()
          .to_agal_boolean(&stack.borrow(), env.clone());
        match condition {
          Ok(condition) => {
            if condition.to_bool() {
              return interpreter(
                if_node.body.clone().to_node().to_box(),
                stack,
                env.clone(),
                modules_manager,
              )
              .await
              .await;
            }
            if let Some(else_body) = &if_node.else_body {
              return interpreter(
                else_body.clone().to_node().to_box(),
                stack.clone(),
                env.clone(),
                modules_manager,
              )
              .await
              .await;
            }
            return AgalValue::Never.as_ref();
          }
          Err(err) => return err.to_ref_value(),
        }
      }
      Node::Import(import) => {
        let module = if import
          .path
          .starts_with(crate::libraries::PREFIX_NATIVE_MODULES)
        {
          crate::libraries::get_module(&import.path, &modules_manager.borrow())
        } else {
          let path = absolute_path(&import.file);
          let path = Path::new(&path).parent();
          if let Some(path) = path {
            let filename = format!("{}/{}", path.to_string_lossy(), import.path);
            let filename = absolute_path(&filename);
            full_eval(
              &filename,
              &stack.borrow(),
              env.borrow().get_global(),
              &modules_manager.borrow(),
            )
            .await
          } else {
            Err(())
          }
        };
        if let Err(e) = module {
          return AgalValue::Never.as_ref();
        }
        let module = module.unwrap();
        if let Some(n) = import.name.clone() {
          env
            .borrow_mut()
            .define(&stack.borrow(), &n, module, true, &node);
        }
        AgalValue::Never.as_ref()
      }
      Node::LoopEdit(loop_edit) => match loop_edit.action {
        NodeLoopEditType::Break => AgalValue::Break.as_ref(),
        NodeLoopEditType::Continue => AgalValue::Continue.as_ref(),
      },
      Node::Member(member) => {
        if member.instance {
          let object = interpreter(
            member.object.clone(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            // No deberia ser posible llegar a este punto
            let key = interpreter(
              member.member.clone(),
              stack.clone(),
              env.clone(),
              modules_manager.clone(),
            )
            .await
            .await;
            let key = key.borrow().to_agal_string(&stack.borrow(), env.clone());
            match key {
              Ok(key) => key.get_string().to_string(),
              Err(err) => return err.to_ref_value(),
            }
          };
          object
            .clone()
            .borrow()
            .get_instance_property(&stack.borrow(), env, key)
        } else {
          let object = interpreter(
            member.object.clone(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          let key = if !member.computed && member.member.is_identifier() {
            member.member.get_identifier().unwrap().name.clone()
          } else {
            let key = interpreter(
              member.member.clone(),
              stack.clone(),
              env.clone(),
              modules_manager.clone(),
            )
            .await
            .await;
            let key = key.borrow().to_agal_string(&stack.borrow(), env.clone());
            match key {
              Ok(key) => key.get_string().to_string(),
              Err(err) => return err.to_ref_value(),
            }
          };
          object
            .clone()
            .borrow()
            .get_object_property(&stack.borrow(), env, key)
        }
      }
      Node::Name(name) => env.borrow().get(&stack.borrow(), &name.name, &node),
      Node::Number(num) => {
        let n = if num.base == 10 {
          str::parse::<f64>(&num.value).unwrap()
        } else {
          let i = i64::from_str_radix(&num.value, num.base as u32).unwrap();
          i as f64
        };
        AgalNumber::new(n).to_ref_value()
      }
      Node::Object(obj) => {
        let mut hashmap = HashMap::new();
        for prop in obj.properties.iter() {
          match prop {
            NodeProperty::Property(key, value) => {
              let value = interpreter(
                value.clone().to_box(),
                stack.clone(),
                env.clone(),
                modules_manager.clone(),
              )
              .await
              .await;
              hashmap.insert(key.clone(), value);
            }
            NodeProperty::Dynamic(key, value) => {
              let key = interpreter(
                key.clone().to_box(),
                stack.clone(),
                env.clone(),
                modules_manager.clone(),
              )
              .await
              .await;
              let key = key.borrow().to_agal_string(&stack.borrow(), env.clone());
              match key {
                Ok(key) => {
                  let key = key.get_string();
                  let value = interpreter(
                    value.clone().to_box(),
                    stack.clone(),
                    env.clone(),
                    modules_manager.clone(),
                  )
                  .await
                  .await;
                  hashmap.insert(key.to_string(), value);
                }
                Err(err) => return err.to_ref_value(),
              }
            }
            NodeProperty::Iterable(iter) => {
              let value = interpreter(
                iter.clone().to_box(),
                stack.clone(),
                env.clone(),
                modules_manager.clone(),
              )
              .await
              .await;
              let keys = value.borrow().get_keys();
              for key in keys.iter() {
                let value =
                  value
                    .borrow()
                    .get_object_property(&stack.borrow(), env.clone(), key.clone());
                hashmap.insert(key.clone(), value);
              }
            }
            _ => {}
          }
        }
        AgalObject::from_hashmap(Rc::new(RefCell::new(hashmap)))
          .to_value()
          .as_ref()
      }
      Node::Program(program) => {
        let mut module_instance = HashMap::new();
        for n in program.body.iter() {
          let value = interpreter(
            n.clone().to_box(),
            pre_stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          if value.borrow().is_stop() {
            return value;
          }
          if value.borrow().is_export() {
            let value = value.borrow().get_export();
            if value.is_none() {
              return AgalThrow::Params {
                type_error: ErrorNames::SyntaxError,
                message: "No se puede exportar un valor nulo".to_string(),
                stack: Box::new(stack.borrow().clone()),
              }
              .to_ref_value();
            }
            let export = value.unwrap();
            module_instance.insert(
              export.0.clone(),
              AgalClassProperty {
                is_public: true,
                is_static: false,
                value: export.1.clone(),
              },
            );
          }
        }
        let prototype = AgalPrototype::new(Rc::new(RefCell::new(module_instance)), None);
        AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
      }
      Node::Return(ret) => {
        if ret.value.is_none() {
          return AgalValue::Return(AgalValue::Never.as_ref()).as_ref();
        }
        let ret_value = ret.value.clone().unwrap();
        let value = interpreter(ret_value, stack, env.clone(), modules_manager)
          .await
          .await;
        AgalValue::Return(value).as_ref()
      }
      Node::String(str) => {
        let mut string = String::new();
        for s in str.value.iter() {
          match s {
            StringData::Id(id) => {
              let data = env.borrow_mut().get(&stack.borrow(), id, &node);
              let data = data.borrow().to_agal_string(&stack.borrow(), env.clone());
              match data {
                Ok(data) => string.push_str(&data.get_string()),
                Err(err) => return err.to_ref_value(),
              }
            }
            StringData::Str(str) => string.push_str(str),
          }
        }
        AgalString::from_string(&string).to_ref_value()
      }
      Node::Throw(throw) => {
        let value = interpreter(
          throw.value.clone(),
          stack.clone(),
          env.clone(),
          modules_manager,
        )
        .await
        .await;
        AgalThrow::from_ref_manager(value, &stack.borrow(), env).to_ref_value()
      }
      Node::Try(try_node) => {
        let try_env = env.borrow().clone().crate_child(false).as_ref();
        let try_val = interpreter(
          try_node.body.clone().to_node().to_box(),
          stack.clone(),
          try_env,
          modules_manager.clone(),
        )
        .await
        .await;
        if try_val.borrow().is_throw() {
          if try_node.catch == None {
            return AgalValue::Never.to_ref_value();
          }
          let env = env.borrow().clone().crate_child(false).as_ref();
          let node_catch = try_node.catch.clone().unwrap();
          env.borrow_mut().define(
            &stack.borrow(),
            &node_catch.0,
            try_val
              .borrow()
              .get_throw()
              .unwrap()
              .get_error()
              .to_value()
              .as_ref(),
            true,
            &node,
          );
          return interpreter(
            node_catch.1.clone().to_node().to_box(),
            stack,
            env,
            modules_manager,
          )
          .await
          .await;
        }
        try_val
      }
      Node::UnaryBack(unary) => interpreter(
        unary.operand.clone(),
        stack.clone(),
        env.clone(),
        modules_manager,
      )
      .await
      .await
      .borrow()
      .unary_back_operator(&stack.borrow(), env, &unary.operator),
      Node::UnaryFront(unary) => interpreter(
        unary.operand.clone(),
        stack.clone(),
        env.clone(),
        modules_manager,
      )
      .await
      .await
      .borrow()
      .unary_operator(&stack.borrow(), env, &unary.operator),
      Node::VarDecl(var) => match &var.value {
        Some(value) => {
          let value = interpreter(value.clone(), stack.clone(), env.clone(), modules_manager)
            .await
            .await;
          if value.borrow().is_never() {
            return AgalThrow::Params {
              type_error: ErrorNames::TypeError,
              message: "No se puede asignar \"nada\" a una variable".to_string(),
              stack: Box::new(stack.borrow().clone()),
            }
            .to_ref_value();
          }
          env
            .borrow_mut()
            .define(&stack.borrow(), &var.name, value, var.is_const, &node)
        }
        None => env.borrow_mut().define(
          &stack.borrow(),
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
          let condition = interpreter(
            while_node.condition.clone(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          let condition = condition
            .borrow()
            .to_agal_boolean(&stack.borrow(), env.clone());
          match condition {
            Ok(condition) => {
              if !condition.to_bool() {
                break;
              }
            }
            Err(err) => return err.to_ref_value(),
          }
          value = interpreter(
            body.clone().to_box(),
            stack.clone(),
            env.clone(),
            modules_manager.clone(),
          )
          .await
          .await;
          let vc = value.clone();
          let v = vc.borrow();
          if v.is_return() {
            return value;
          }
          if v.is_throw() {
            return value;
          }
          if v.is_break() {
            break;
          }
        }
        value
      }
      _ => AgalValue::Never.as_ref(),
    }
  })
}
