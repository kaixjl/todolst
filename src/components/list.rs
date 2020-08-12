use crate::components::group;
use std::rc::*;

pub struct List {
    id: u32,
    title: String,
    group: Option<Weak<group::Group>>,
}

impl List {
    pub fn new(id: u32, title: &str, group: Option<Weak<group::Group>>) -> Self {
        Self {
            id, title: String::from(title), group
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn title(&self) -> &str { self.title.as_ref() }
    pub fn set_title(&mut self, title: String) -> &mut Self { self.title = title; self }

    pub fn group(&self) -> Option<Weak<group::Group>> { 
        match &self.group {
            Some(x) => Some(x.clone()),
            None => None
        }
    }
    pub fn set_group(&mut self, group: Option<Weak<group::Group>>) -> &mut Self { self.group = group; self }
    pub fn group_ref(&self) -> &Option<Weak<group::Group>> { &self.group }
    pub fn group_mut(&mut self) -> &mut Option<Weak<group::Group>> { &mut self.group }
}