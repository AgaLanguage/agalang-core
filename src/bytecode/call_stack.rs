struct CallStackItem {
  tasks: Vec<CallStackItem>,
  is_done: bool
}

pub struct CallStack{
  macro_tasks: Vec<CallStackItem>
}