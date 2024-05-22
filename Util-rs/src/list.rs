pub struct List<Item>(Vec<Item>)
where
    Item: Clone;
impl<Item: Clone> List<Item> {
    pub fn from_vec(vec: Vec<Item>) -> Self {
        List(vec)
    }
    pub fn new() -> Self {
        List(Vec::new())
    }
    pub fn push(&mut self, item: Item) {
        self.0.push(item);
    }
    pub fn iter(&self) -> std::slice::Iter<Item> {
        self.0.iter()
    }
    pub fn map<R, F>(&self, f: F) -> List<R>
    where
        F: FnMut(&Item) -> R,
        R: Clone,
    {
        List(self.0.iter().map(f).collect())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn append(&mut self, other: &mut List<Item>) {
        self.append_vec(&mut other.0);
    }
    pub fn append_vec(&mut self, other: &mut Vec<Item>) {
        self.0.append(other);
    }
}
impl<Item: Clone + std::fmt::Display> List<Item> {
    pub fn join(&self, sep: &str) -> String {
        self.map(|item| item.to_string()).0.join(&sep)
    }
}
impl<Item: Clone> Clone for List<Item> {
    fn clone(&self) -> Self {
        List(self.0.iter().map(|item| item.clone()).collect())
    }
}
impl<Item: Clone + std::fmt::Display> std::fmt::Display for List<Item> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str: Vec<String> = self.0.iter().map(|item| item.to_string()).collect();
        write!(f, "{}", str.join("\n"))
    }
}
