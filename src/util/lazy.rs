use std::cell::RefCell;

pub struct Lazy<T> {
  function: fn() -> T,
  value: RefCell<Option<T>>
}
impl<T: std::fmt::Debug + Clone> Lazy<T>{
  pub const fn new(function: fn() -> T) -> Self {
    Self { function, value: RefCell::new(None) }
  }
  pub fn get(&self) -> std::cell::Ref<T> {
    if self.value.borrow().is_none() {
        let val = (self.function)();
        *self.value.borrow_mut() = Some(val);
    }
    std::cell::Ref::map(self.value.borrow(), |opt| {
        opt.as_ref().expect("Value must be initialized")
    })
}
}