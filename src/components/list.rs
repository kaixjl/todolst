use crate::components::group;
use crate::components::item;
// use std::rc::*;
use std::sync::{ Weak, Mutex, Arc };

#[derive(Debug)]
pub struct List {
    id: u32,
    title: String,
    group: Option<Weak<Mutex<group::Group>>>,
    items: Vec<Arc<Mutex<item::Item>>>,
}

impl List {
    pub fn new(id: u32, title: &str, group: Option<Weak<Mutex<group::Group>>>) -> Self {
        Self {
            id, title: String::from(title), group, items: Vec::new(),
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn title(&self) -> &str { self.title.as_str() }
    pub fn set_title(&mut self, title: String) -> &mut Self { self.title = title; self }

    pub fn group(&self) -> Option<Weak<Mutex<group::Group>>> { 
        match &self.group {
            Some(x) => Some(x.clone()),
            None => None
        }
    }
    pub fn set_group(&mut self, group: Option<Weak<Mutex<group::Group>>>) -> &mut Self { self.group = group; self }
    pub fn group_ref(&self) -> &Option<Weak<Mutex<group::Group>>> { &self.group }
    pub fn group_mut(&mut self) -> &mut Option<Weak<Mutex<group::Group>>> { &mut self.group }

    pub fn add_item(&mut self, item: Arc<Mutex<item::Item>>) {
        self.items.push(item)
    }

    pub fn iter_items(&self) -> core::slice::Iter<'_, Arc<Mutex<item::Item>>> {
        self.items.iter()
    }

    pub fn remove_item_by_index(&mut self, index: usize) -> Arc<Mutex<item::Item>> {
        self.items.remove(index)
    }

    pub fn remove_item(&mut self, item: &Arc<Mutex<item::Item>>) -> Option<Arc<Mutex<item::Item>>> {
        let mut idx = None;
        let item = item.lock().unwrap();
        let id_input = item.id();
        drop(item);
        for (i, item) in self.items.iter().enumerate() {
            let item = item.lock().unwrap();
            let id = item.id();
            drop(item);
            if id == id_input {
                idx = Some(i);
            }
        }
        match idx {
            None => None,
            Some(index) => Some(self.items.remove(index))
        }

    }

    pub fn get_item(&self, index: usize) -> Arc<Mutex<item::Item>> {
        self.items[index].clone()
    }
}