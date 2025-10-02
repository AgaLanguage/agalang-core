use crate::compiler::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub const LIB_NAME: &str = ":tmp";
const NOW: &str = "ahora";
const ZONE: &str = "ZONA";

#[cfg(windows)]
mod c_time {
  pub use std::os::raw::{c_int};
  #[cfg(all(target_arch = "x86", target_env = "gnu"))]
  pub type TimeT = i32;
  #[cfg(not(all(target_arch = "x86", target_env = "gnu")))]
  pub type TimeT = i64;


  #[repr(C)]
  pub struct Tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
  }

  unsafe extern "C" {
    #[link_name = "_time64"]
    pub unsafe fn time(t: *mut TimeT) -> TimeT;
    #[link_name = "_localtime64_s"]
    pub unsafe fn localtime_s(tmDest: *mut Tm, sourceTime: *const TimeT) -> c_int;
    #[link_name = "_gmtime64_s"]
    pub unsafe fn gmtime_s(destTime: *mut Tm, srcTime: *const TimeT) -> c_int;
  }
}

#[cfg(unix)]
mod c_time {

  pub use std::os::raw::{c_int};
  #[cfg(all(target_arch = "aarch64", target_pointer_width = "32"))]
  pub type TimeT = i32;
  #[cfg(not(all(target_arch = "aarch64", target_pointer_width = "32")))]
  pub type TimeT = i64;


  #[repr(C)]
  pub struct Tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
  }

  unsafe extern "C" {
    #[cfg_attr(target_os = "netbsd", link_name = "__time50")]
    #[cfg_attr(any(target_env = "musl", target_env = "ohos"), allow(deprecated))]
    #[cfg_attr(gnu_time_bits64, link_name = "__time64")]
    pub unsafe fn time(t: *mut TimeT) -> TimeT;
    #[cfg_attr(target_os = "netbsd", link_name = "__localtime_r50")]
    #[cfg_attr(any(target_env = "musl", target_env = "ohos"), allow(deprecated))]
    #[cfg_attr(gnu_time_bits64, link_name = "__localtime64_r")]
    pub fn localtime_r(time_p: *const TimeT, result: *mut Tm) -> *mut Tm;
    #[cfg_attr(target_os = "netbsd", link_name = "__gmtime_r50")]
    #[cfg_attr(any(target_env = "musl", target_env = "ohos"), allow(deprecated))]
    #[cfg_attr(gnu_time_bits64, link_name = "__gmtime64_r")]
    pub fn gmtime_r(time_p: *const TimeT, result: *mut Tm) -> *mut Tm;
  }
  pub unsafe fn localtime_s(result: *mut Tm, time_p: *const TimeT){
    localtime_r(time_p, result);
  }
  pub unsafe fn gmtime_s(result: *mut Tm, time_p: *const TimeT){
    gmtime_r(time_p, result);
  }
}

mod native_time {
  use super::c_time::{TimeT, Tm, time, localtime_s, gmtime_s};

  const SECONDS_IN_DAY: i32 = 86400;

  pub unsafe fn get_utc_offset_secs() -> i32 {
    let now: TimeT = time(std::ptr::null_mut());

    let mut gmt_tm: Tm = std::mem::zeroed();
    let mut local_tm: Tm = std::mem::zeroed();
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
        let offset = unsafe {native_time::get_utc_offset_secs()};

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