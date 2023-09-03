use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::object::Object;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    store: HashMap<String, Rc<Object>>,
    // outer: Option<Rc<RefCell<Environment>>>,
}

pub type Env = Rc<RefCell<Environment>>;

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            // outer: None,
        }
    }

    pub fn get(&self, name: &str) -> Option<Rc<Object>> {
        match self.store.get(name) {
            Some(obj) => Some(obj.clone()),
            None => None,
            //match &self.outer {
            //     Some(outer) => outer.borrow().get(name),
            //     None => None,
            // },
        }
    }

    pub fn set(&mut self, name: String, val: Rc<Object>) {
        self.store.insert(name, val);
    }
}
