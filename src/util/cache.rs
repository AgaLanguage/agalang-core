#[macro_export]
macro_rules! cache_fn {
  (fn $function:ident ($name:ident : $key:ty ) -> $val:ty $init:block) => {
    const CACHE: std::sync::LazyLock<
      std::rc::Rc<std::cell::RefCell<std::collections::HashMap<$key, $val>>>,
    > = std::sync::LazyLock::new(|| {
      std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()))
    });

    fn $function($name: $key) -> $val
    where
      $key: std::cmp::Eq + std::hash::Hash + Clone,
      $val: Clone,
    {
      if let Some(value) = CACHE.borrow().get(&$name) {
        return value.clone();
      } else {
        let value: $val = { |$name: $key| $init }($name.clone());
        CACHE.borrow_mut().insert($name, value.clone());
        return value;
      }
    }
  };

  (pub fn $function:ident ($name:ident : $key:ty ) -> $val:ty $init:block) => {
    const CACHE: std::sync::LazyLock<
      std::rc::Rc<std::cell::RefCell<std::collections::HashMap<$key, $val>>>,
    > = std::sync::LazyLock::new(|| {
      std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()))
    });

    pub fn $function($name: $key) -> $val
    where
      $key: std::cmp::Eq + std::hash::Hash + Clone,
      $val: Clone,
    {
      if let Some(value) = CACHE.borrow().get(&$name) {
        return value.clone();
      } else {
        let value: $val = { |$name: $key| $init }($name.clone());
        CACHE.borrow_mut().insert($name, value.clone());
        return value;
      }
    }
  };
}
