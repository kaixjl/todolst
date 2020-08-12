use std::rc::*;
use std::cell::*;

pub struct Group {
    id: u32,
    title: String,
    parent: Option<Weak<RefCell<Group>>>,
}

impl Group {
    pub fn new(id: u32, title: &str, parent: Option<Weak<RefCell<Group>>>) -> Self {
        Self {
            id, title: String::from(title), parent
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn title(&self) -> &str { self.title.as_ref() }
    pub fn set_title(&mut self, title: String) -> &mut Self { self.title = title; self }

    pub fn parent(&self) -> Option<Weak<RefCell<Group>>> { 
        match &self.parent {
            Some(x) => Some(x.clone()),
            None => None
        }
    }
    pub fn set_parent(&mut self, parent: Option<Weak<RefCell<Group>>>) -> &mut Self { self.parent = parent; self }
    pub fn parent_ref(&self) -> &Option<Weak<RefCell<Group>>> { &self.parent }
    pub fn parent_mut(&mut self) -> &mut Option<Weak<RefCell<Group>>> { &mut self.parent }
}