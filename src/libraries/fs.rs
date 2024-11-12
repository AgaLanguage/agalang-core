use std::{cell::RefCell, io::Read, rc::Rc};

use crate::runtime::{
    AgalArray, AgalClassProperty, AgalNativeFunction, AgalObject, AgalPrototype, AgalString,
    AgalThrow, AgalValuable, AgalValue, RefAgalValue,
};

pub fn get_module(prefix: &str) -> RefAgalValue {
    let mut module_name = get_name(prefix);

    let mut hashmap = std::collections::HashMap::new();
    hashmap.insert(
        "leerArchivo".to_string(),
        AgalClassProperty {
            is_public: true,
            is_static: true,
            value: AgalNativeFunction {
                name: format!("{module_name}::leerArchivo"),
                func: Rc::new(|arguments, stack, env| {
                    let path = arguments.get(0);
                    if !path.is_some() {
                        return AgalValue::Never.to_ref_value();
                    }
                    let path = path.unwrap().borrow().clone().to_agal_string(stack, env);
                    if let Err(error) = path {
                        return error.to_ref_value();
                    }
                    let path = path.ok().unwrap();
                    let path = path.get_string();
                    let mut file = std::fs::File::open(path);
                    if let Ok(file) = &mut file {
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer);
                        return AgalArray::from_buffer(&buffer).to_ref_value();
                    }
                    AgalThrow::Params {
                        type_error: parser::internal::ErrorNames::PathError,
                        message: "No se pudo abrir el archivo".to_string(),
                        stack: Box::new(stack.clone()),
                    }
                    .to_ref_value()
                }),
            }
            .to_ref_value(),
        },
    );
    let prototype = AgalPrototype::new(Rc::new(RefCell::new(hashmap)), None);
    AgalObject::from_prototype(prototype.as_ref()).to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
    format!("{}{}", prefix, "sa")
}
