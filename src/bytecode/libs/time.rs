use libc::{gmtime_s, localtime_s, time, time_t, tm};
use std::{
  collections::HashMap,
  time::{SystemTime, UNIX_EPOCH},
};

use crate::bytecode::value::{Function, Object, Value};

pub const TIME_LIB: &str = ":tmp";
const NOW: &str = "ahora";
const ZONE: &str = "ZONA";

pub fn time_lib() -> Value {
  let mut hashmap = HashMap::new();

  hashmap.insert(
    NOW.into(),
    Value::Object(
      Function::Native {
        name: format!("<{TIME_LIB}>::{NOW}"),
        path: format!("<{TIME_LIB}>"),
        chunk: crate::bytecode::ChunkGroup::default(),
        func: |_, _, _| {
          let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(e) => return Err(format!("Error: {:?}", e)),
          };
          Ok(Value::Number(nanos.into()))
        },
      }
      .into(),
    ),
  );
  hashmap.insert(
    ZONE.into(),
    Value::Object(unsafe {
      let now: time_t = time(std::ptr::null_mut());

      // Convertir a UTC y hora local
      let mut utc_tm: tm = std::mem::zeroed();
      let mut local_tm: tm = std::mem::zeroed();
      gmtime_s(&mut utc_tm, &now);
      localtime_s(&mut local_tm, &now);

      let utc_sec = utc_tm.tm_hour * 3600 + utc_tm.tm_min * 60 + utc_tm.tm_sec;
      let local_sec = local_tm.tm_hour * 3600 + local_tm.tm_min * 60 + local_tm.tm_sec;

      let mut offset = local_sec - utc_sec;

      // Ajuste por diferencia de día (por ejemplo: UTC es 23:00, local es 01:00 del día siguiente)
      if local_tm.tm_mday != utc_tm.tm_mday {
        offset += if local_tm.tm_mday > utc_tm.tm_mday {
          86400
        } else {
          -86400
        };
      }

      // Convertir a horas y minutos
      let hours = offset / 3600;
      let minutes = (offset.abs() % 3600) / 60;

      vec![Value::Number(hours.into()), Value::Number(minutes.into())]
    }.into()),
  );
  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
