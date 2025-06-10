use super::{MultiRefHash, Value};

pub const PROMISE_TYPE: &str = "promesa";

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub enum PromiseStatus {
  #[default]
  Pending,
  Done,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PromiseData {
  Pending,
  Ok(MultiRefHash<Value>),
  Err(String),
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Promise {
  value: MultiRefHash<Option<MultiRefHash<Value>>>,
  err: MultiRefHash<Option<String>>,
  status: MultiRefHash<PromiseStatus>,
}
impl Promise {
  pub fn new() -> Self {
    Self::default()
  }
  pub fn set_value(&self, value: Value) {
    *self.status.borrow_mut() = PromiseStatus::Done;
    *self.value.borrow_mut() = Some(value.into());
  }
  pub fn set_err(&self, err: String) {
    *self.status.borrow_mut() = PromiseStatus::Done;
    *self.err.borrow_mut() = Some(err);
  }
  pub fn get_value_str(&self) -> String {
    if self.value.borrow().is_some() {
      self.value.borrow().clone().unwrap().borrow().as_string()
    } else if self.err.borrow().is_some() {
      self.err.borrow().clone().unwrap()
    } else {
      "Desconocido".to_string()
    }
  }
  pub fn get_data(&self) -> PromiseData {
    if matches!(self.status.borrow().clone(), PromiseStatus::Pending) {
      return PromiseData::Pending;
    }
    if let Some(v) = self.value.borrow().clone() {
      return PromiseData::Ok(v);
    }
    if let Some(v) = self.err.borrow().clone() {
      return PromiseData::Err(v);
    }
    return PromiseData::Pending;
  }
}
impl ToString for Promise {
  fn to_string(&self) -> String {
    format!("{PROMISE_TYPE}<{}>", self.get_value_str())
  }
}
impl From<Value> for Promise {
  fn from(value: Value) -> Self {
    Self {
      err: Default::default(),
      status: PromiseStatus::Done.into(),
      value: Some(value.into()).into(),
    }
  }
}
