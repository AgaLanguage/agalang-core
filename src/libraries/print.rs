use std::{cell::RefCell, rc::Rc};

use crate::runtime::{
    AgalClassProperty, AgalNativeFunction, AgalObject, AgalPrototype, AgalValuable, AgalValue,
    RefAgalValue,
};

pub fn get_module(prefix: &str) -> RefAgalValue {
    let mut module_name = get_name(prefix);

    let mut hashmap = std::collections::HashMap::new();

    hashmap.insert(
        "pintar".to_string(),
        AgalClassProperty {
            is_public: true,
            is_static: true,
            value: AgalNativeFunction {
                name: format!("{module_name}::pintar"),
                func: Rc::new(|arguments, stack, env| {
                    for arg in arguments {
                        let data = arg.borrow().clone().to_agal_console(stack, env.clone());
                        if let Ok(str) = data {
                            print!("{} ", str.get_string());
                        } else if let Err(e) = data {
                            print!("\n");
                            return e.to_ref_value();
                        }
                    }
                    print!("\n");
                    AgalValue::Never.as_ref()
                }),
            }
            .to_ref_value(),
        },
    );
    hashmap.insert(
        "limpiar".to_string(),
        AgalClassProperty {
            is_public: true,
            is_static: true,
            value: AgalNativeFunction {
                name: format!("{module_name}::limpiar"),
                func: Rc::new(|_, stack, env| {
                    print!("\x1B[0;0H");
                    AgalValue::Never.as_ref()
                }),
            }
            .to_ref_value(),
        },
    );
    let prototype = AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
    AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
    format!("{}{}", prefix, "consola")
}
