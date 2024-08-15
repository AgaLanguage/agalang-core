use std::{cell::RefCell, rc::Rc};

use super::{
    AgalArray, AgalBoolean, AgalError, AgalHashMap, AgalNumber, AgalObject, AgalString, AgalThrow, AgalValuable, AgalValue, Enviroment, RefAgalValue, Stack};
use crate::{
    frontend::ast::{Node, NodeLoopEditType, NodeProperty, StringData},
    internal::ErrorNames,
};

pub fn interpreter(node: &Node, stack: &Stack, env: Rc<RefCell<Enviroment>>) -> RefAgalValue {
    let pre_stack = stack;
    let stack = pre_stack.next(node);
    match node {
        Node::Array(list) => {
            let mut vec = vec![];
            for n in list.elements.iter() {
                match n {
                    NodeProperty::Indexable(value) => {
                        let data = interpreter(value, &stack, env.clone());
                        vec.push(data);
                    }
                    NodeProperty::Iterable(id) => {
                        let data = interpreter(node, &stack, env.clone());
                        let list = data.borrow().clone().to_agal_array(&stack);
                        match list {
                            Ok(list) => {
                                for n in list.get_vec().iter() {
                                    vec.push(n.clone());
                                }
                            }
                            Err(err) => return AgalValue::Throw(err).to_ref(),
                        }
                    }
                    _ => {}
                }
            }
            AgalArray::from_vec(vec).to_value().to_ref()
        }
        Node::Assignment(assignment) => {
            let value = interpreter(&assignment.value, &stack, env.clone());
            if value.borrow().is_never() {
                return AgalValue::Throw(AgalThrow::Params {
                    type_error: ErrorNames::CustomError("Error Tipo".to_string()),
                    message: "No se puede asignar \"nada\" a una variable".to_string(),
                    stack: Box::new(stack),
                }).to_ref();
            }
            match assignment.identifier.as_ref() {
                Node::Identifier(identifier) => {
                    env.borrow_mut().assign(&identifier.name, value, node)
                }
                Node::Member(member) => {
                    if member.instance {
                        return AgalValue::Throw(AgalThrow::Params {
                            type_error: ErrorNames::CustomError("Error Tipo".to_string()),
                            message: "No se puede asignar una propiedad de instancia".to_string(),
                            stack: Box::new(stack),
                        }).to_ref();
                    }
                    let object = interpreter(&member.object, &stack, env.clone());
                    let key = if !member.computed {
                        if member.member.is_identifier() {
                            member.member.get_identifier().unwrap().name.clone()
                        } else {
                            // [x] No deberia ser posible llegar a este punto
                            return AgalValue::Throw(AgalThrow::Params {
                                type_error: ErrorNames::CustomError("Error Tipo".to_string()),
                                message: "No se puede asignar a un objeto no identificado"
                                    .to_string(),
                                stack: Box::new(stack),
                            }).to_ref();
                        }
                    } else {
                        let key = interpreter(&member.member, &stack, env.clone());
                        let key = key.borrow().clone().to_agal_string(&stack, &env.borrow());
                        match key {
                            Ok(key) => key.get_string().to_string(),
                            Err(err) => return AgalValue::Throw(err).to_ref(),
                        }
                    };
                    object.clone().borrow().clone().set_object_property(&stack, env.borrow().as_ref(), key, value)
                }
                _ => AgalValue::Never.to_ref(),
            }
        }
        Node::Binary(binary) => {
            let left = interpreter(&binary.left, &stack, env.clone());
            let right = interpreter(&binary.right, &stack, env.clone());
            left.clone().borrow().binary_operation(&stack, &env.borrow(), binary.operator.clone(), right)
        }
        Node::Block(block) => {
            let env = env.borrow().clone().crate_child();
            let env = Rc::new(RefCell::new(env));
            let mut value = AgalValue::Never.to_ref();
            for n in block.body.iter() {
                value = interpreter(n, pre_stack, env.clone());
                if value.borrow().is_stop() {
                    return value;
                }
            }
            value
        }
        Node::Call(call) => {
            let callee = interpreter(&call.callee, &stack, env.clone());
            let mut args = vec![];
            for arg in call.arguments.iter() {
                let arg = interpreter(arg, &stack, env.clone());
                args.push(arg);
            }
            callee.clone().borrow().clone().call(&stack, &env.borrow(), callee, args)
        }
        Node::Class(class) => AgalValue::Never.to_ref(), // TODO: Implementar
        Node::DoWhile(do_while) => {
            let mut value = AgalValue::Never.to_ref();
            let mut condition = Ok(AgalBoolean::new(true));
            loop {
                match condition {
                    Ok(condition) => {
                        if !condition.to_bool() {
                            break;
                        }
                    }
                    Err(err) => return AgalValue::Throw(err).to_ref(),
                }
                value = interpreter(&do_while.body.clone().to_node(), &stack, env.clone());
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
                let pre_condition = interpreter(&do_while.condition, &stack, env.clone());
                condition = pre_condition.borrow().clone().to_agal_boolean(&stack, &env.borrow());
            }
            value
        }
        Node::Error(error) => AgalValue::Throw(AgalThrow::Error(AgalError::new(
            ErrorNames::SyntaxError,
            error.message.clone(),
            Box::new(stack),
        ))).to_ref(),
        Node::Export(export) => AgalValue::Never.to_ref(), // TODO: Implementar
        Node::For(for_node) => {
            let mut value = AgalValue::Never.to_ref();
            let mut condition = Ok(AgalBoolean::new(true));
            let env = Rc::new(RefCell::new(env.borrow().clone().crate_child()));
            interpreter(&for_node.init, &stack, env.clone()); // init value
            loop {
                match condition {
                    Ok(condition) => {
                        if !condition.to_bool() {
                            break;
                        }
                    }
                    Err(err) => return AgalValue::Throw(err).to_ref(),
                }
                value = interpreter(&for_node.body.clone().to_node(), &stack, env.clone());
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
                let pre_condition = interpreter(&for_node.condition, &stack, env.clone());
                condition = pre_condition.borrow().clone().to_agal_boolean(&stack, &env.borrow());
                let pre_update = interpreter(&for_node.update, &stack, env.clone());
                if pre_update.borrow().is_throw() {
                    return pre_update;
                }
            }
            value
        }
        Node::Function(func) => {
            let function = AgalValue::Function(func.params.clone(), func.body.clone(), env.clone()).to_ref();
            env.borrow_mut()
                .define(&stack, &func.name, function, true, node)
        }
        Node::Identifier(id) => env.borrow().get(&id.name, node),
        Node::If(if_node) => {
            let condition = interpreter(&if_node.condition, &stack, env.clone());
            let condition = condition.borrow().clone().to_agal_boolean(&stack, &env.borrow());
            match condition {
                Ok(condition) => {
                    if condition.to_bool() {
                        return interpreter(&if_node.body.clone().to_node(), &stack, env.clone());
                    }
                    if let Some(else_body) = &if_node.else_body {
                        return interpreter(&else_body.clone().to_node(), &stack, env.clone());
                    }
                    return AgalValue::Never.to_ref();
                }
                Err(err) => return AgalValue::Throw(err).to_ref(),
            }
        }
        Node::Import(import) => AgalValue::Never.to_ref(),
        Node::LoopEdit(loop_edit) => match loop_edit.action {
            NodeLoopEditType::Break => AgalValue::Break.to_ref(),
            NodeLoopEditType::Continue => AgalValue::Continue.to_ref(),
        },
        Node::Member(member) => {
            if member.instance {
                let object = interpreter(&member.object, &stack, env.clone());
                let key = if !member.computed && member.member.is_identifier() {
                    member.member.get_identifier().unwrap().name.clone()
                } else {
                    // [x] No deberia ser posible llegar a este punto
                    return AgalValue::Throw(AgalThrow::Params {
                        type_error: ErrorNames::CustomError("Error Tipo".to_string()),
                        message: "No se puede obtener la propiedad".to_string(),
                        stack: Box::new(stack),
                    }).to_ref();
                };
                object.clone().borrow().clone().get_instance_property(&stack, env.borrow().as_ref(), key)
            } else {
                let object = interpreter(&member.object, &stack, env.clone());
                let key = if !member.computed {
                    if member.member.is_identifier() {
                        member.member.get_identifier().unwrap().name.clone()
                    } else {
                        // [x] No deberia ser posible llegar a este punto
                        return AgalValue::Throw(AgalThrow::Params {
                            type_error: ErrorNames::CustomError("Error Tipo".to_string()),
                            message: "No se puede asignar a un objeto no identificado".to_string(),
                            stack: Box::new(stack),
                        }).to_ref();
                    }
                } else {
                    let key = interpreter(&member.member, &stack, env.clone());
                    let key = key.borrow().clone().to_agal_string(&stack, &env.borrow());
                    match key {
                        Ok(key) => key.get_string().to_string(),
                        Err(err) => return AgalValue::Throw(err).to_ref(),
                    }
                };
                object.clone().borrow().clone().get_object_property(&stack, env.borrow().as_ref(), key)
            }
        }
        Node::Name(name) => env.borrow().get(&name.name, node),
        Node::Number(num) => {
            let n = if num.base == 10 {
                str::parse::<f64>(&num.value).unwrap()
            } else {
                let i = i64::from_str_radix(&num.value, num.base as u32).unwrap();
                i as f64
            };
            AgalValue::Number(AgalNumber::new(n)).to_ref()
        }
        Node::Object(obj) => {
            let mut hasmap: AgalHashMap = std::collections::HashMap::new();
            for prop in obj.properties.iter() {
                match prop {
                    NodeProperty::Property(key, value) => {
                        let value = interpreter(value, &stack, env.clone());
                        hasmap.insert(key.clone(), value);
                    }
                    NodeProperty::Dynamic(key, value) => {
                        let key = interpreter(key, &stack, env.clone());
                        let key = key.borrow().clone().to_agal_string(&stack, &env.borrow());
                        match key {
                            Ok(key) => {
                                let key = key.get_string();
                                let value = interpreter(value, &stack, env.clone());
                                hasmap.insert(key.clone(), value);
                            }
                            Err(err) => return AgalValue::Throw(err).to_ref(),
                        }
                    }
                    NodeProperty::Iterable(id) => {

                    }
                    NodeProperty::Indexable(value) => {

                    }
                }
            }
            AgalObject::from_hashmap(hasmap).to_value().to_ref()
        },
        Node::Program(program) => {
            let mut value = AgalValue::Never.to_ref();
            for n in program.body.iter() {
                value = interpreter(n, pre_stack, env.clone());
                if value.borrow().is_stop() {
                    return value;
                }
            }
            value
        }
        Node::Return(ret) => {
            if ret.value.is_none() {
                return AgalValue::Return(AgalValue::Never.to_ref()).to_ref();
            }
            let ret_value = ret.value.clone().unwrap();
            let value = interpreter(&ret_value, &stack, env.clone());
            AgalValue::Return(value).to_ref()
        }
        Node::String(str) => {
            let mut string = "".to_string();
            for s in str.value.iter() {
                match s {
                    StringData::Id(id) => {
                        let data = env.borrow_mut().get(id, node);
                        let env = env.borrow();
                        let env = env.as_ref();
                        let data = data.borrow().clone().to_agal_string(&stack, env);
                        match data {
                            Ok(data) => string.push_str(&data.get_string()),
                            Err(err) => {
                                return AgalValue::Throw(err).to_ref();
                            }
                        }
                    }
                    StringData::Str(str) => string.push_str(str),
                }
            }
            AgalValue::String(AgalString::from_string(string)).to_ref()
        }
        Node::Throw(throw) => {
            let value = interpreter(&throw.value, &stack, env.clone());
            AgalValue::Throw(AgalThrow::from_ref_value(
                value,
                Box::new(stack),
                &mut env.borrow_mut(),
            )).to_ref()
        }
        Node::Try(try_node) => {
            let try_env = env.borrow().clone().crate_child();
            let try_env = Rc::new(RefCell::new(try_env));
            let try_val = interpreter(&try_node.body.clone().to_node(), &stack, try_env);
            if try_val.borrow().is_throw() {
                let env = env.borrow().clone().crate_child();
                let env = Rc::new(RefCell::new(env));
                env.borrow_mut().define(
                    &stack,
                    &try_node.catch.0,
                    try_val.borrow().get_throw().unwrap().get_error().to_value().to_ref(),
                    true,
                    node,
                );
                return interpreter(&try_node.catch.1.clone().to_node(), &stack, env);
            }
            try_val
        }
        Node::UnaryBack(unary) => AgalValue::Never.to_ref(), // TODO: Implementar
        Node::UnaryFront(unary) => AgalValue::Never.to_ref(), // TODO: Implementar
        Node::VarDecl(var) => match &var.value {
            Some(value) => {
                let value = interpreter(value, &stack, env.clone());
                if value.borrow().is_never() {
                    return AgalValue::Throw(AgalThrow::Params {
                        type_error: ErrorNames::CustomError("Error Tipo".to_string()),
                        message: "No se puede asignar \"nada\" a una variable".to_string(),
                        stack: Box::new(stack),
                    }).to_ref();
                }
                env.borrow_mut()
                    .define(&stack, &var.name, value, var.is_const, node)
            }
            None => {
                env.borrow_mut()
                    .define(&stack, &var.name, AgalValue::Never.to_ref(), var.is_const, node)
            }
        },
        Node::While(while_node) => {
            let mut value = AgalValue::Never.to_ref();
            let body = &while_node.body.clone().to_node();
            loop {
                let condition = interpreter(&while_node.condition, &stack, env.clone());
                let condition = condition.borrow().clone().to_agal_boolean(&stack, &env.borrow());
                match condition {
                    Ok(condition) => {
                        if !condition.to_bool() {
                            break;
                        }
                    }
                    Err(err) => return AgalValue::Throw(err).to_ref(),
                }
                value = interpreter(body, &stack, env.clone());
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
        _ => AgalValue::Never.to_ref(),
    }
}
