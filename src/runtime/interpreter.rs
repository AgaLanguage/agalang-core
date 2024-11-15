use parser::{
    ast::{Node, NodeLoopEditType, NodeProperty, StringData},
    internal::ErrorNames,
};
use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc};

use crate::{
    path::absolute_path,
    runtime::{
        env::THIS_KEYWORD, full_eval, AgalArray, AgalBoolean, AgalByte, AgalClass,
        AgalClassProperty, AgalComplex, AgalError, AgalFunction, AgalInternal, AgalNumber,
        AgalObject, AgalPrototype, AgalString, AgalThrow, AgalValuable, AgalValuableManager,
        AgalValue, Environment, RefAgalValue, Stack,
    },
    Modules, ToResult,
};

pub fn interpreter(
    node: &Node,
    stack: &Stack,
    env: Rc<RefCell<Environment>>,
    modules_manager: &Modules,
) -> RefAgalValue {
    let pre_stack = stack;
    let stack = pre_stack.next(node);
    match node {
        Node::Array(list) => {
            let mut vec = vec![];
            for n in list.elements.iter() {
                match n {
                    NodeProperty::Indexable(value) => {
                        let data =
                            interpreter(value, &stack, env.clone(), &modules_manager.clone());
                        vec.push(data);
                    }
                    NodeProperty::Iterable(iter) => {
                        let data = interpreter(iter, &stack, env.clone(), modules_manager);
                        let list = data.borrow().clone().to_agal_array(&stack);
                        match list {
                            Ok(list) => {
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
            let value = interpreter(&assignment.value, &stack, env.clone(), modules_manager);
            if value.borrow().is_never() {
                return AgalThrow::Params {
                    type_error: ErrorNames::TypeError,
                    message: "No se puede asignar \"nada\" a una variable".to_string(),
                    stack: Box::new(stack),
                }
                .to_ref_value();
            }
            match assignment.identifier.as_ref() {
                Node::Identifier(identifier) => {
                    env.borrow_mut()
                        .assign(&stack, &identifier.name, value, node)
                }
                Node::Member(member) => {
                    if member.instance {
                        return AgalThrow::Params {
                            type_error: ErrorNames::TypeError,
                            message: "No se puede asignar una propiedad de instancia".to_string(),
                            stack: Box::new(stack),
                        }
                        .to_ref_value();
                    }
                    let key = if !member.computed && member.member.is_identifier() {
                        member.member.get_identifier().unwrap().name.clone()
                    } else {
                        let key = interpreter(&member.member, &stack, env.clone(), modules_manager);
                        let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                        match key {
                            Ok(key) => key.get_string().to_string(),
                            Err(err) => return err.to_ref_value(),
                        }
                    };

                    let object = interpreter(&member.object, &stack, env.clone(), modules_manager);
                    object.clone().borrow().clone().set_object_property(
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
            let left = interpreter(&binary.left, &stack, env.clone(), modules_manager);
            let right = interpreter(&binary.right, &stack, env.clone(), modules_manager);
            left.clone()
                .borrow()
                .binary_operation(&stack, env.clone(), &binary.operator, right)
        }
        Node::Block(block) => {
            let env = env.borrow().clone().crate_child(false).as_ref();
            for n in block.body.iter() {
                let value = interpreter(n, pre_stack, env.clone(), modules_manager);
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
            let callee = call.callee.as_ref();
            let (callee, this) = if let Node::Member(member) = callee {
                let this =
                    interpreter(member.object.as_ref(), &stack, env.clone(), modules_manager);
                let this = this.clone();
                let key = if !member.computed && member.member.is_identifier() {
                    member.member.get_identifier().unwrap().name.clone()
                } else {
                    // No deberia ser posible llegar a este punto
                    let key = interpreter(&member.member, &stack, env.clone(), modules_manager);
                    let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                    match key {
                        Ok(key) => key.get_string().to_string(),
                        Err(err) => return err.to_ref_value(),
                    }
                };
                let callee =
                    this.clone()
                        .borrow()
                        .clone()
                        .get_instance_property(&stack, env.clone(), key);
                (callee, this)
            } else {
                let callee = interpreter(&callee, &stack, env.clone(), modules_manager);
                (callee.clone(), callee)
            };

            let mut args = vec![];
            for arg in call.arguments.iter() {
                let arg = interpreter(arg, &stack, env.clone(), modules_manager);
                if arg.borrow().is_throw() {
                    return arg;
                }
                args.push(arg);
            }
            callee
                .clone()
                .borrow()
                .clone()
                .call(&stack, env.clone(), this, args, modules_manager)
        }
        Node::Class(class) => {
            let extend_of_value = if let Some(extend) = &class.extend_of {
                let value = env.borrow().get(&stack, &extend.name, node);
                let value: &AgalValue = &value.borrow();
                let value = value.clone();
                match value {
                    AgalValue::Complex(AgalComplex::Class(class)) => {
                        Some(Rc::new(RefCell::new(class)))
                    }
                    AgalValue::Internal(AgalInternal::Throw(th)) => return th.to_ref_value(),
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
                    interpreter(b.as_ref(), &stack, class_env.clone(), modules_manager)
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
                    &do_while.body.clone().to_node(),
                    &stack,
                    env.clone(),
                    modules_manager,
                );
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
                let pre_condition =
                    interpreter(&do_while.condition, &stack, env.clone(), modules_manager);
                condition = pre_condition
                    .borrow()
                    .clone()
                    .to_agal_boolean(&stack, env.clone());
            }
            value
        }
        Node::Error(error) => AgalThrow::Error(AgalError::new(
            ErrorNames::SyntaxError,
            error.message.clone(),
            Box::new(stack),
        ))
        .to_ref_value(),
        Node::Export(export) => match export.value.as_ref() {
            Node::VarDecl(var) => {
                let value = interpreter(
                    &var.value.clone().unwrap(),
                    &stack,
                    env.clone(),
                    modules_manager,
                );
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
                let value = env.borrow().get(&stack, &name.name, node);
                AgalValue::Export(name.name.clone(), value).as_ref()
            }
            Node::Class(class) => {
                let extend_of_value = if let Some(extend) = &class.extend_of {
                    let value = env.borrow().get(&stack, &extend.name, node);
                    let value: &AgalValue = &value.borrow();
                    let value = value.clone();
                    match value {
                        AgalValue::Complex(AgalComplex::Class(class)) => {
                            Some(Rc::new(RefCell::new(class)))
                        }
                        AgalValue::Internal(AgalInternal::Throw(th)) => return th.to_ref_value(),
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
                        interpreter(b.as_ref(), &stack, class_env.clone(), modules_manager)
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
            let mut condition: Result<AgalBoolean, AgalThrow> = Ok(AgalBoolean::new(true));
            let env = env.borrow().clone().crate_child(false).as_ref();
            interpreter(&for_node.init, &stack, env.clone(), modules_manager); // init value
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
                    &for_node.body.clone().to_node(),
                    &stack,
                    env.clone(),
                    modules_manager,
                );
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
                let pre_condition =
                    interpreter(&for_node.condition, &stack, env.clone(), modules_manager);
                condition = pre_condition
                    .borrow()
                    .clone()
                    .to_agal_boolean(&stack, env.clone());
                let pre_update =
                    interpreter(&for_node.update, &stack, env.clone(), modules_manager);
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
        Node::Identifier(id) => env.borrow().get(&stack, &id.name, node),
        Node::If(if_node) => {
            let condition = interpreter(&if_node.condition, &stack, env.clone(), modules_manager);
            let condition = condition
                .borrow()
                .clone()
                .to_agal_boolean(&stack, env.clone());
            match condition {
                Ok(condition) => {
                    if condition.to_bool() {
                        return interpreter(
                            &if_node.body.clone().to_node(),
                            &stack,
                            env.clone(),
                            modules_manager,
                        );
                    }
                    if let Some(else_body) = &if_node.else_body {
                        return interpreter(
                            &else_body.clone().to_node(),
                            &stack,
                            env.clone(),
                            modules_manager,
                        );
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
                crate::libraries::get_module(&import.path, &modules_manager.clone())
            } else {
                let path = absolute_path(&import.file);
                let path = Path::new(&path).parent();
                if let Some(path) = path {
                    let filename = format!("{}/{}", path.to_string_lossy(), import.path);
                    let filename = absolute_path(&filename);
                    full_eval(
                        &filename,
                        &stack,
                        env.borrow().get_global(),
                        modules_manager,
                    )
                } else {
                    Err(())
                }
            };
            if let Err(e) = module {
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
                let object = interpreter(&member.object, &stack, env.clone(), modules_manager);
                let key = if !member.computed && member.member.is_identifier() {
                    member.member.get_identifier().unwrap().name.clone()
                } else {
                    // No deberia ser posible llegar a este punto
                    let key = interpreter(&member.member, &stack, env.clone(), modules_manager);
                    let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                    match key {
                        Ok(key) => key.get_string().to_string(),
                        Err(err) => return err.to_ref_value(),
                    }
                };
                object
                    .clone()
                    .borrow()
                    .clone()
                    .get_instance_property(&stack, env, key)
            } else {
                let object = interpreter(&member.object, &stack, env.clone(), modules_manager);
                let key = if !member.computed && member.member.is_identifier() {
                    member.member.get_identifier().unwrap().name.clone()
                } else {
                    let key = interpreter(&member.member, &stack, env.clone(), modules_manager);
                    let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                    match key {
                        Ok(key) => key.get_string().to_string(),
                        Err(err) => return err.to_ref_value(),
                    }
                };
                object
                    .clone()
                    .borrow()
                    .clone()
                    .get_object_property(&stack, env, key)
            }
        }
        Node::Name(name) => env.borrow().get(&stack, &name.name, node),
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
            let mut hasmap = HashMap::new();
            for prop in obj.properties.iter() {
                match prop {
                    NodeProperty::Property(key, value) => {
                        let value = interpreter(value, &stack, env.clone(), modules_manager);
                        hasmap.insert(key.clone(), value);
                    }
                    NodeProperty::Dynamic(key, value) => {
                        let key = interpreter(key, &stack, env.clone(), modules_manager);
                        let key = key.borrow().clone().to_agal_string(&stack, env.clone());
                        match key {
                            Ok(key) => {
                                let key = key.get_string();
                                let value =
                                    interpreter(value, &stack, env.clone(), modules_manager);
                                hasmap.insert(key.clone(), value);
                            }
                            Err(err) => return err.to_ref_value(),
                        }
                    }
                    NodeProperty::Iterable(iter) => {
                        let value = interpreter(iter, &stack, env.clone(), modules_manager);
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
                let value = interpreter(n, pre_stack, env.clone(), modules_manager);
                if value.borrow().is_stop() {
                    return value;
                }
                if value.borrow().is_export() {
                    let value = value.borrow().clone().get_export();
                    if value.is_none() {
                        return AgalThrow::Params {
                            type_error: ErrorNames::SyntaxError,
                            message: "No se puede exportar un valor nulo".to_string(),
                            stack: Box::new(stack),
                        }
                        .to_ref_value();
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
            let prototype = AgalPrototype::new(Rc::new(RefCell::new(module_instance)), None);
            AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
        }
        Node::Return(ret) => {
            if ret.value.is_none() {
                return AgalValue::Return(AgalValue::Never.as_ref()).as_ref();
            }
            let ret_value = ret.value.clone().unwrap();
            let value = interpreter(&ret_value, &stack, env.clone(), modules_manager);
            AgalValue::Return(value).as_ref()
        }
        Node::String(str) => {
            let mut string = "".to_string();
            for s in str.value.iter() {
                match s {
                    StringData::Id(id) => {
                        let data = env.borrow_mut().get(&stack, id, node);
                        let data = data.borrow().clone().to_agal_string(&stack, env.clone());
                        match data {
                            Ok(data) => string.push_str(&data.get_string()),
                            Err(err) => return err.to_ref_value(),
                        }
                    }
                    StringData::Str(str) => string.push_str(str),
                }
            }
            AgalString::from_string(string).to_ref_value()
        }
        Node::Throw(throw) => {
            let value = interpreter(&throw.value, &stack, env.clone(), modules_manager);
            AgalThrow::from_ref_manager(value, &stack, env).to_ref_value()
        }
        Node::Try(try_node) => {
            let try_env = env.borrow().clone().crate_child(false).as_ref();
            let try_val = interpreter(
                &try_node.body.clone().to_node(),
                &stack,
                try_env,
                modules_manager,
            );
            if try_val.borrow().is_throw() {
                if try_node.catch == None {
                    return AgalValue::Never.to_ref_value();
                }
                let env = env.borrow().clone().crate_child(false).as_ref();
                let node_catch = try_node.catch.clone().unwrap();
                env.borrow_mut().define(
                    &stack,
                    &node_catch.0,
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
                return interpreter(
                    &node_catch.1.clone().to_node(),
                    &stack,
                    env,
                    modules_manager,
                );
            }
            try_val
        }
        Node::UnaryBack(unary) => interpreter(&unary.operand, &stack, env.clone(), modules_manager)
            .borrow()
            .unary_back_operator(&stack, env, &unary.operator),
        Node::UnaryFront(unary) => {
            interpreter(&unary.operand, &stack, env.clone(), modules_manager)
                .borrow()
                .unary_operator(&stack, env, &unary.operator)
        }
        Node::VarDecl(var) => match &var.value {
            Some(value) => {
                let value = interpreter(value, &stack, env.clone(), modules_manager);
                if value.borrow().is_never() {
                    return AgalThrow::Params {
                        type_error: ErrorNames::TypeError,
                        message: "No se puede asignar \"nada\" a una variable".to_string(),
                        stack: Box::new(stack),
                    }
                    .to_ref_value();
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
                let condition =
                    interpreter(&while_node.condition, &stack, env.clone(), modules_manager);
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
                    Err(err) => return err.to_ref_value(),
                }
                value = interpreter(body, &stack, env.clone(), modules_manager);
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
