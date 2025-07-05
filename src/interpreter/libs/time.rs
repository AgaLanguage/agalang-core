use crate::compiler::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub const LIB_NAME: &str = ":tmp";
const NOW: &str = "ahora";
const ZONE: &str = "ZONA";

#[cfg(target_os = "windows")]
mod native_time {
  use libc::{gmtime_s, localtime_s, time, time_t, tm};
  const SECONDS_IN_DAY: i32 = 86400; // 24 * 60 * 60

  pub unsafe fn get_utc_in_secs() -> i32 {
    let now: time_t = time(std::ptr::null_mut());

    let mut gmt_tm: tm = std::mem::zeroed();
    let mut local_tm: tm = std::mem::zeroed();
    gmtime_s(&mut gmt_tm, &now);
    localtime_s(&mut local_tm, &now);

    let utc_sec = gmt_tm.tm_hour * 3600 + gmt_tm.tm_min * 60 + gmt_tm.tm_sec;
    let local_sec = local_tm.tm_hour * 3600 + local_tm.tm_min * 60 + local_tm.tm_sec;

    let mut offset = local_sec - utc_sec;

    if local_tm.tm_mday != gmt_tm.tm_mday {
      offset += if local_tm.tm_mday > gmt_tm.tm_mday {
        SECONDS_IN_DAY
      } else {
        -SECONDS_IN_DAY
      };
    }
    offset
  }
}

#[cfg(target_arch = "wasm32")]
mod wasm_time {
  // Para wasm, aquí puedes usar wasm-bindgen para obtener la zona horaria
  // Pero para ejemplo sencillo, solo devolveremos 0
  pub fn get_utc_in_secs() -> i32 {
    0
  }
}

pub fn lib_value() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    NOW,
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{NOW}"),
        path: format!("<{LIB_NAME}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, _, _, _| {
          let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(e) => Err(format!("Error: {:?}", e))?,
          };
          Ok(Value::Number(nanos.into()))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );

  hashmap.set_instance_property(
    ZONE,
    Value::Object(
      {
        #[cfg(not(target_arch = "wasm32"))]
        let offset = unsafe { native_time::get_utc_in_secs() };

        #[cfg(target_arch = "wasm32")]
        let offset = wasm_time::get_utc_in_secs();

        // Convertir a horas y minutos
        let hours = offset / 3600;
        let minutes = (offset.abs() % 3600) / 60;

        vec![Value::Number(hours.into()), Value::Number(minutes.into())]
      }
      .into(),
    ),
    true,
  );

  Value::Object(crate::compiler::Object::Map(
    Default::default(),
    hashmap.into(),
  ))
}
