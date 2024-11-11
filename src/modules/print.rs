use std::rc::Rc;

use crate::runtime::{AgalNativeFunction, AgalValuable, AgalValue, RefAgalValue};

pub fn get_module() -> RefAgalValue {
    AgalNativeFunction {
        name: ">pintar".to_string(),
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
    .to_ref_value()
}
pub fn get_name(prefix: &str) -> String {
    format!("{}{}", prefix, "pintar")
}