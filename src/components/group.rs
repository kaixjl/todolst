// use std::rc::*;
// use std::cell::*;
use std::sync::{ Weak, Mutex };

#[derive(Debug)]
pub struct Group {
    id: u32,
    title: String,
    parent: Option<Weak<Mutex<Group>>>,
}

impl Group {
    pub fn new(id: u32, title: &str, parent: Option<Weak<Mutex<Group>>>) -> Self {
        Self {
            id, title: String::from(title), parent
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn title(&self) -> &str { self.title.as_str() }
    pub fn set_title(&mut self, title: String) -> &mut Self { self.title = title; self }

    pub fn parent(&self) -> Option<Weak<Mutex<Group>>> { 
        match &self.parent {
            Some(x) => Some(x.clone()),
            None => None
        }
    }
    pub fn set_parent(&mut self, parent: Option<Weak<Mutex<Group>>>) -> &mut Self { self.parent = parent; self }
    pub fn parent_ref(&self) -> &Option<Weak<Mutex<Group>>> { &self.parent }
    pub fn parent_mut(&mut self) -> &mut Option<Weak<Mutex<Group>>> { &mut self.parent }
}