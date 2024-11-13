use std::{cell::RefCell, io::Read, rc::Rc};

use parser::internal::ErrorNames;

use crate::{
    runtime::{
        env::RefEnvironment, AgalArray, AgalBoolean, AgalClass, AgalClassProperty,
        AgalNativeFunction, AgalObject, AgalPrototype, AgalString, AgalThrow, AgalValuable,
        AgalValue, RefAgalValue, Stack,
    },
    Modules,
};
fn get_path(
    stack: &Stack,
    env: RefEnvironment,
    this: RefAgalValue,
) -> Result<AgalString, AgalThrow> {
    let string = this
        .borrow()
        .clone()
        .get_object_property(stack, env.clone(), "@ruta".to_string());
    let s = string.borrow().clone().to_agal_string(stack, env);
    s
}

pub fn get_module(prefix: &str) -> RefAgalValue {
    let mut module_name = get_name(prefix);

    let path_name = format!("{module_name}::Ruta");

    let path = AgalClass::new(
        path_name.clone(),
        vec![
            (
                "es_archivo".to_string(),
                AgalClassProperty {
                    is_public: true,
                    is_static: false,
                    value: AgalNativeFunction {
                        name: format!("{path_name}::es_archivo"),
                        func: Rc::new(|_, stack, env, _, this| {
                            let string = get_path(stack, env, this);
                            if let Ok(string) = string {
                                let path = std::path::Path::new(string.get_string());
                                AgalBoolean::new(path.is_file()).to_ref_value()
                            } else if let Err(e) = string {
                                e.to_ref_value()
                            } else {
                                AgalThrow::Params {
                                    type_error: ErrorNames::PathError,
                                    stack: Box::new(stack.clone()),
                                    message: "La ruta no es valida".to_string(),
                                }
                                .to_ref_value()
                            }
                        }),
                    }
                    .to_ref_value(),
                },
            ),
            (
                "es_carpeta".to_string(),
                AgalClassProperty {
                    is_public: true,
                    is_static: false,
                    value: AgalNativeFunction {
                        name: format!("{path_name}::es_carpeta"),
                        func: Rc::new(|_, stack, env, _, this| {
                            let string = get_path(stack, env, this);
                            if let Ok(string) = string {
                                let path = std::path::Path::new(string.get_string());
                                AgalBoolean::new(path.is_dir()).to_ref_value()
                            } else if let Err(e) = string {
                                e.to_ref_value()
                            } else {
                                AgalThrow::Params {
                                    type_error: ErrorNames::PathError,
                                    stack: Box::new(stack.clone()),
                                    message: "La ruta no es valida".to_string(),
                                }
                                .to_ref_value()
                            }
                        }),
                    }
                    .to_ref_value(),
                },
            ),
            (
                "nombre".to_string(),
                AgalClassProperty {
                    is_public: true,
                    is_static: false,
                    value: AgalNativeFunction {
                        name: format!("{path_name}::nombre"),
                        func: Rc::new(|_, stack, env, _, this| {
                            let string = get_path(stack, env, this);
                            if let Ok(string) = string {
                                string.to_ref_value()
                            } else if let Err(e) = string {
                                e.to_ref_value()
                            } else {
                                AgalThrow::Params {
                                    type_error: ErrorNames::PathError,
                                    stack: Box::new(stack.clone()),
                                    message: "La ruta no es valida".to_string(),
                                }
                                .to_ref_value()
                            }
                        }),
                    }
                    .to_ref_value(),
                },
            ),
        ],
        None,
    )
    .to_value();

    let mut hashmap = std::collections::HashMap::new();
    hashmap.insert(
        "leerArchivo".to_string(),
        AgalClassProperty {
            is_public: true,
            is_static: true,
            value: AgalNativeFunction {
                name: format!("{module_name}::leerArchivo"),
                func: Rc::new(|arguments, stack, env, modules_manager, this| {
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
    hashmap.insert(
        "leerCarpeta".to_string(),
        AgalClassProperty {
            is_public: true,
            is_static: true,
            value: AgalNativeFunction {
                name: format!("{module_name}::leerCarpeta"),
                func: Rc::new(|arguments, stack, env, modules_manager, this| {
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
                    let mut dir = std::fs::read_dir(path);
                    if let Ok(dir) = &mut dir {
                        let mut files = Vec::new();
                        for i in dir {
                            if let Ok(entry) = i {
                                files.push(
                                    AgalString::from_string(
                                        entry.file_name().to_string_lossy().to_string(),
                                    )
                                    .to_ref_value(),
                                );
                            }
                        }
                        return AgalArray::from_vec(files).to_ref_value();
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
    hashmap.insert(
        "obtener_ruta".to_string(),
        AgalClassProperty {
            is_public: true,
            is_static: true,
            value: AgalNativeFunction {
                name: format!("{module_name}::obtener_ruta"),
                func: {
                    let function = move |arguments: Vec<RefAgalValue>,
                                         stack: &Stack,
                                         env: RefEnvironment,
                                         modules_manager: &Modules,
                                         this: RefAgalValue| {
                        let p = arguments.get(0);
                        if !p.is_some() {
                            return AgalValue::Never.to_ref_value();
                        }
                        let p = p
                            .unwrap()
                            .borrow()
                            .clone()
                            .to_agal_string(stack, env.clone());
                        if let Ok(value) = p {
                            let path = &path;
                            let path = path.clone().call(
                                stack,
                                env.clone(),
                                this,
                                vec![],
                                modules_manager,
                            );
                            let v = path.clone();
                            let v = v.borrow();
                            //let p = value.get_string();
                            //let p = crate::path::absolute_path(p);
                            //let value = AgalString::from_string(p);
                            v.clone().set_object_property(
                                stack,
                                env,
                                "@ruta".to_string(),
                                value.to_ref_value(),
                            );
                            path
                        } else if let Err(error) = p {
                            error.to_ref_value()
                        } else {
                            AgalThrow::Params {
                                type_error: ErrorNames::PathError,
                                message: "La ruta no es valida".to_string(),
                                stack: Box::new(stack.clone()),
                            }
                            .to_ref_value()
                        }
                    };
                    Rc::new(function)
                },
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
