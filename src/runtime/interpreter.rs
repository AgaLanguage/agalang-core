use parser::{
    ast::{Node, NodeLoopEditType, NodeProperty, StringData},
    internal::ErrorNames,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    env::THIS_KEYWORD, full_eval, AgalArray, AgalBoolean, AgalByte, AgalClass, AgalClassProperty,
    AgalError, AgalFunction, AgalNumber, AgalObject, AgalString, AgalThrow, AgalValuable,
    AgalValue, Environment, RefAgalValue, Stack,
};

pub fn interpreter(node: &Node, stack: &Stack, env: Rc<RefCell<Environment>>) -> RefAgalValue {
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
                    NodeProperty::Iterable(iter) => {
                        let data = interpreter(iter, &stack, env.clone());
                        let list = data.borrow().clone().to_agal_array(&stack);
                        match list {
                            Ok(list) => {
                                for n in list.get_vec().iter() {
                                    vec.push(n.clone());
                                }
                            }
                            Err(err) => return AgalValue::Throw(err).as_ref(),
                        }
                    }
                    _ => {}
                }
            }
            AgalArray::from_vec(vec).to_value().as_ref()
        }
        Node::Assignment(assignment) => {
            let value = interpreter(&assignment.value, &stack, env.clone());
            if value.borrow().is_never() {
                return AgalValue::Throw(AgalThrow::Params {
                    type_error: ErrorNames::TypeError,
                    message: "No se puede asignar \"nada\" a una variable".to_string(),
                    stack: Box::new(stack),
                })
                .as_ref();
            }
            match assignment.identifier.as_ref() {
                Node::Identifier(identifier) => {
                    env.borrow_mut().assign(&identifier.name, value, node)
                }
                Node::Member(member) => {
                    if member.instance {
                        return AgalValue::Throw(AgalThrow::Params {
                            type_error: ErrorNames::TypeError,
                            message: "No se puede asignar una propiedad de instancia".to_string(),
                            stack: Box::new(stack),
                        })
                        .as_ref();
                    }
                    let key = if !member.computed && member.member.is_identifier(){
                        member.member.get_identifier().unwrap().name.clone()
                    } else {
                        let key = interpreter(&member.member, &stack, env.clone());
                        let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                        match key {
                            Ok(key) => key.get_string().to_string(),
                            Err(err) => return AgalValue::Throw(err).as_ref(),
                        }
                    };

                    let object = interpreter(&member.object, &stack, env.clone());
                    object.clone().borrow_mut().clone().set_object_property(
                        &stack,
                        env,
                        key,
                        value.clone(),
                    )
                }
                _ => AgalValue::Never.as_ref(),
            }
        }
        Node::Binary(binary) => {
            let left = interpreter(&binary.left, &stack, env.clone());
            let right = interpreter(&binary.right, &stack, env.clone());
            left.clone()
                .borrow()
                .binary_operation(&stack, env.clone(), &binary.operator, right)
        }
        Node::Block(block) => {
            let env = env.borrow().clone().crate_child(false).as_ref();
            for n in block.body.iter() {
                let value = interpreter(n, pre_stack, env.clone());
                if value.borrow().is_return() || value.borrow().is_throw() {
                    return value;
                }
                if value.borrow().is_stop() {
                    break;
                }
            }
            AgalValue::Never.as_ref()
        }
        Node::Byte(byte_node) => AgalByte::new(byte_node.value).to_ref_value(),
        Node::Call(call) => {
            let callee = interpreter(&call.callee, &stack, env.clone());
            let mut args = vec![];
            for arg in call.arguments.iter() {
                let arg = interpreter(arg, &stack, env.clone());
                if arg.borrow().is_throw(){return arg}
                args.push(arg);
            }
            callee
                .clone()
                .borrow()
                .clone()
                .call(&stack, env.clone(), callee, args)
        }
        Node::Class(class) => {
            let extend_of_value = if let Some(extend) = &class.extend_of {
                let value = env.borrow().get(&extend.name, node);
                let value: &AgalValue = &value.borrow();
                let value = value.clone();
                match value {
                    AgalValue::Class(class) => Some(Rc::new(RefCell::new(class))),
                    AgalValue::Throw(th) => return th.to_ref_value(),
                    _ => {
                        return AgalThrow::Params {
                            type_error: ErrorNames::TypeError,
                            message: "Solo se puede extender de otras clases".to_string(),
                            stack: Box::new(stack),
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
                    interpreter(b.as_ref(), &stack, class_env.clone())
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
            env.borrow_mut()
                .define(&stack, &class.name, class_value, true, node)
        }
        Node::DoWhile(do_while) => {
            let mut value = AgalValue::Never.as_ref();
            let mut condition = Ok(AgalBoolean::new(true));
            loop {
                match condition {
                    Ok(condition) => {
                        if !condition.to_bool() {
                            break;
                        }
                    }
                    Err(err) => return AgalValue::Throw(err).as_ref(),
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
                condition = pre_condition
                    .borrow()
                    .clone()
                    .to_agal_boolean(&stack, env.clone());
            }
            value
        }
        Node::Error(error) => AgalValue::Throw(AgalThrow::Error(AgalError::new(
            ErrorNames::SyntaxError,
            error.message.clone(),
            Box::new(stack),
        )))
        .as_ref(),
        Node::Export(export) => match export.value.as_ref() {
            Node::VarDecl(var) => {
                let value = interpreter(&var.value.clone().unwrap(), &stack, env.clone());
                env.borrow_mut()
                    .define(&stack, &var.name, value.clone(), var.is_const, node);
                AgalValue::Export(var.name.clone(), value).as_ref()
            }
            Node::Function(func) => {
                let function =
                    AgalFunction::new(func.params.clone(), func.body.clone(), env.clone())
                        .to_ref_value();
                env.borrow_mut()
                    .define(&stack, &func.name, function.clone(), true, node);
                AgalValue::Export(func.name.clone(), function).as_ref()
            }
            Node::Name(name) => {
                let value = env.borrow().get(&name.name, node);
                AgalValue::Export(name.name.clone(), value).as_ref()
            }
            Node::Class(class) => {
                let extend_of_value = if let Some(extend) = &class.extend_of {
                    let value = env.borrow().get(&extend.name, node);
                    let value: &AgalValue = &value.borrow();
                    let value = value.clone();
                    match value {
                        AgalValue::Class(class) => Some(Rc::new(RefCell::new(class))),
                        AgalValue::Throw(th) => return th.to_ref_value(),
                        _ => {
                            return AgalThrow::Params {
                                type_error: ErrorNames::TypeError,
                                message: "Solo se puede extender de otras clases".to_string(),
                                stack: Box::new(stack),
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
                        interpreter(b.as_ref(), &stack, class_env.clone())
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
                env.borrow_mut()
                    .define(&stack, &class.name, class_value.clone(), true, node);

                AgalValue::Export(class.name.clone(), class_value).as_ref()
            }
            _ => AgalThrow::Params {
                type_error: ErrorNames::SyntaxError,
                message: "Se nesesita un nombre para las exportaciones".to_string(),
                stack: Box::new(stack),
            }
            .to_value()
            .as_ref(),
        },
        Node::For(for_node) => {
            let mut value = AgalValue::Never.as_ref();
            let mut condition = Ok(AgalBoolean::new(true));
            let env = env.borrow().clone().crate_child(false).as_ref();
            interpreter(&for_node.init, &stack, env.clone()); // init value
            loop {
                match condition {
                    Ok(condition) => {
                        if !condition.to_bool() {
                            break;
                        }
                    }
                    Err(err) => return AgalValue::Throw(err).as_ref(),
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
                condition = pre_condition
                    .borrow()
                    .clone()
                    .to_agal_boolean(&stack, env.clone());
                let pre_update = interpreter(&for_node.update, &stack, env.clone());
                if pre_update.borrow().is_throw() {
                    return pre_update;
                }
            }
            value
        }
        Node::Function(func) => {
            let function = AgalFunction::new(func.params.clone(), func.body.clone(), env.clone())
                .to_ref_value();
            env.borrow_mut()
                .define(&stack, &func.name, function, true, node)
        }
        Node::Identifier(id) => env.borrow().get(&id.name, node),
        Node::If(if_node) => {
            let condition = interpreter(&if_node.condition, &stack, env.clone());
            let condition = condition
                .borrow()
                .clone()
                .to_agal_boolean(&stack, env.clone());
            match condition {
                Ok(condition) => {
                    if condition.to_bool() {
                        return interpreter(&if_node.body.clone().to_node(), &stack, env.clone());
                    }
                    if let Some(else_body) = &if_node.else_body {
                        return interpreter(&else_body.clone().to_node(), &stack, env.clone());
                    }
                    return AgalValue::Never.as_ref();
                }
                Err(err) => return AgalValue::Throw(err).as_ref(),
            }
        }
        Node::Import(import) => {
            let module = if import.path.starts_with('>') {
                crate::modules::get_module(&import.path)
            } else {full_eval(import.path.clone(), &stack, env.borrow().get_global())};
            if module.is_err() {
                return AgalValue::Never.as_ref();
            }
            let module = module.unwrap();
            if let Some(n) = import.name.clone() {
                env.borrow_mut().define(&stack, &n, module, true, node);
            }
            AgalValue::Never.as_ref()
        }
        Node::LoopEdit(loop_edit) => match loop_edit.action {
            NodeLoopEditType::Break => AgalValue::Break.as_ref(),
            NodeLoopEditType::Continue => AgalValue::Continue.as_ref(),
        },
        Node::Member(member) => {
            if member.instance {
                let object = interpreter(&member.object, &stack, env.clone());
                let key = if !member.computed && member.member.is_identifier() {
                    member.member.get_identifier().unwrap().name.clone()
                } else {
                    // No deberia ser posible llegar a este punto
                    let key = interpreter(&member.member, &stack, env.clone());
                    let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                    match key {
                        Ok(key) => key.get_string().to_string(),
                        Err(err) => return AgalValue::Throw(err).as_ref(),
                    }
                };
                object
                    .clone()
                    .borrow()
                    .clone()
                    .get_instance_property(&stack, env, key)
            } else {
                let object = interpreter(&member.object, &stack, env.clone());
                let key = if !member.computed && member.member.is_identifier(){
                    member.member.get_identifier().unwrap().name.clone()
                } else {
                    let key = interpreter(&member.member, &stack, env.clone());
                    let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                    match key {
                        Ok(key) => key.get_string().to_string(),
                        Err(err) => return AgalValue::Throw(err).as_ref(),
                    }
                };
                object
                    .clone()
                    .borrow()
                    .clone()
                    .get_object_property(&stack, env, key)
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
            AgalValue::Number(AgalNumber::new(n)).as_ref()
        }
        Node::Object(obj) => {
            let mut hasmap = HashMap::new();
            for prop in obj.properties.iter() {
                match prop {
                    NodeProperty::Property(key, value) => {
                        let value = interpreter(value, &stack, env.clone());
                        hasmap.insert(key.clone(), value);
                    }
                    NodeProperty::Dynamic(key, value) => {
                        let key = interpreter(key, &stack, env.clone());
                        let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                        match key {
                            Ok(key) => {
                                let key = key.get_string();
                                let value = interpreter(value, &stack, env.clone());
                                hasmap.insert(key.clone(), value);
                            }
                            Err(err) => return AgalValue::Throw(err).as_ref(),
                        }
                    }
                    NodeProperty::Iterable(iter) => {
                        let value = interpreter(iter, &stack, env.clone());
                        let keys = value.borrow().clone().get_keys();
                        for key in keys.iter() {
                            let value = value.borrow().clone().get_object_property(
                                &stack,
                                env.clone(),
                                key.clone(),
                            );
                            hasmap.insert(key.clone(), value);
                        }
                    }
                    _ => {}
                }
            }
            AgalObject::from_hashmap(Rc::new(RefCell::new(hasmap)))
                .to_value()
                .as_ref()
        }
        Node::Program(program) => {
            let mut module_instance = HashMap::new();
            for n in program.body.iter() {
                let value = interpreter(n, pre_stack, env.clone());
                if value.borrow().is_stop() {
                    return value;
                }
                if value.borrow().is_export() {
                    let value = value.borrow().clone().get_export();
                    if value.is_none() {
                        return AgalValue::Throw(AgalThrow::Params {
                            type_error: ErrorNames::SyntaxError,
                            message: "No se puede exportar un valor nulo".to_string(),
                            stack: Box::new(stack),
                        })
                        .as_ref();
                    }
                    let export = value.unwrap();
                    module_instance.insert(
                        export.0,
                        AgalClassProperty {
                            is_public: true,
                            is_static: false,
                            value: export.1,
                        },
                    );
                }
            }
            AgalValue::Object(AgalObject::from_prototype(Rc::new(RefCell::new(
                module_instance,
            ))))
            .as_ref()
        }
        Node::Return(ret) => {
            if ret.value.is_none() {
                return AgalValue::Return(AgalValue::Never.as_ref()).as_ref();
            }
            let ret_value = ret.value.clone().unwrap();
            let value = interpreter(&ret_value, &stack, env.clone());
            AgalValue::Return(value).as_ref()
        }
        Node::String(str) => {
            let mut string = "".to_string();
            for s in str.value.iter() {
                match s {
                    StringData::Id(id) => {
                        let data = env.borrow_mut().get(id, node);
                        let data = data.borrow().clone().to_agal_string(&stack, env.clone());
                        match data {
                            Ok(data) => string.push_str(&data.get_string()),
                            Err(err) => {
                                return AgalValue::Throw(err).as_ref();
                            }
                        }
                    }
                    StringData::Str(str) => string.push_str(str),
                }
            }
            AgalValue::String(AgalString::from_string(string)).as_ref()
        }
        Node::Throw(throw) => {
            let value = interpreter(&throw.value, &stack, env.clone());
            AgalValue::Throw(AgalThrow::from_ref_value(value, Box::new(stack), env)).as_ref()
        }
        Node::Try(try_node) => {
            let try_env = env.borrow().clone().crate_child(false).as_ref();
            let try_val = interpreter(&try_node.body.clone().to_node(), &stack, try_env);
            if try_val.borrow().is_throw() {
                let env = env.borrow().clone().crate_child(false).as_ref();
                env.borrow_mut().define(
                    &stack,
                    &try_node.catch.0,
                    try_val
                        .borrow()
                        .get_throw()
                        .unwrap()
                        .get_error()
                        .to_value()
                        .as_ref(),
                    true,
                    node,
                );
                return interpreter(&try_node.catch.1.clone().to_node(), &stack, env);
            }
            try_val
        }
        Node::UnaryBack(unary) => interpreter(&unary.operand, &stack, env.clone())
            .borrow()
            .unary_back_operator(&stack, env, &unary.operator),
        Node::UnaryFront(unary) => interpreter(&unary.operand, &stack, env.clone())
            .borrow()
            .unary_operator(&stack, env, &unary.operator),
        Node::VarDecl(var) => match &var.value {
            Some(value) => {
                let value = interpreter(value, &stack, env.clone());
                if value.borrow().is_never() {
                    return AgalValue::Throw(AgalThrow::Params {
                        type_error: ErrorNames::TypeError,
                        message: "No se puede asignar \"nada\" a una variable".to_string(),
                        stack: Box::new(stack),
                    })
                    .as_ref();
                }
                env.borrow_mut()
                    .define(&stack, &var.name, value, var.is_const, node)
            }
            None => env.borrow_mut().define(
                &stack,
                &var.name,
                AgalValue::Never.as_ref(),
                var.is_const,
                node,
            ),
        },
        Node::While(while_node) => {
            let mut value = AgalValue::Never.as_ref();
            let body = &while_node.body.clone().to_node();
            loop {
                let condition = interpreter(&while_node.condition, &stack, env.clone());
                let condition = condition
                    .borrow()
                    .clone()
                    .to_agal_boolean(&stack, env.clone());
                match condition {
                    Ok(condition) => {
                        if !condition.to_bool() {
                            break;
                        }
                    }
                    Err(err) => return AgalValue::Throw(err).as_ref(),
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
        _ => AgalValue::Never.as_ref(),
    }
}
