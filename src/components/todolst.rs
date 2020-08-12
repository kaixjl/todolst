use crate::components::{ group, item, list, noticer };
use std::collections::{ HashMap, BTreeMap };
use std::rc::*;
use std::cell::*;
use std::sync::{ Arc, Mutex };
use chrono::prelude::*;
// use std::thread;
// use std::time;

#[derive(Debug)]
pub struct Error;

pub struct TodoLst {
    groups: HashMap<String, Rc<RefCell<group::Group>>>,
    lists: HashMap<String, Rc<RefCell<list::List>>>,
    items: HashMap<u32, Rc<RefCell<item::Item>>>,
    next_item_id: u32,
    next_list_id: u32,
    next_group_id: u32,
    noticer: noticer::Noticer,
    notice_items: BTreeMap<NaiveDateTime, RefCell<Vec<Rc<RefCell<item::Item>>>>>,
    notice_comed: Arc<Mutex<Vec<NaiveDateTime>>>,
}

impl TodoLst {
    pub fn new() -> Self {
        let mut val = Self {
            groups: HashMap::new(),
            lists: HashMap::new(),
            items: HashMap::new(),
            next_item_id: 0,
            next_list_id: 0,
            next_group_id: 0,
            noticer: noticer::Noticer::new(),
            notice_items: BTreeMap::new(),
            notice_comed: Arc::new(Mutex::new(Vec::new())),
        };
        let notice_commed = val.notice_comed.clone();
        val.noticer.add_callback(move | datetime | {
            println!("{}", datetime);
            let notice_commed = notice_commed.lock();
            match notice_commed {
                Err(_) => (),
                Ok(mut notice_comed) => {
                    notice_comed.push(datetime);
                }
            }
        });
        val.noticer.start();
        val
    }

    pub fn item(&self, id: u32) -> Weak<RefCell<item::Item>> {
        // Rc::downgrade(&self.items[&id])
        Rc::downgrade(&self.items[&id])
    }

    pub fn new_item(&mut self, message: &str, list: Weak<RefCell<list::List>>) -> Weak<RefCell<item::Item>> {
        let itm = Rc::new(RefCell::new(item::Item::new(
            self.next_item_id, message, 0, item::ItemStyle{ marker: item::Marker(0) }, false, 
            None, None, None, None, list
        )));

        self.items.insert(self.next_item_id, itm);

        let itm = self.item(self.next_item_id);

        self.next_item_id += 1;

        itm
    }

    pub fn set_item_message(&self, item: Weak<RefCell<item::Item>>, message: &str) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_message(message.to_string());
        }
        self
    }

    pub fn set_item_level(&self, item: &Weak<RefCell<item::Item>>,  level: i8) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_level(level);
        }
        self
    }

    pub fn set_item_style(&self, item: &Weak<RefCell<item::Item>>,  style: item::ItemStyle) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_style(style);
        }
        self
    }

    pub fn set_item_today(&self, item: &Weak<RefCell<item::Item>>,  today: bool) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_today(today);
        }
        self
    }

    pub fn set_item_notice(&mut self, item: &Weak<RefCell<item::Item>>,  notice: Option<NaiveDateTime>) -> &Self {
        let item = item.upgrade();
        if let Some(item_rc) = item {
            let mut item = item_rc.borrow_mut();
            match item.notice() {
                None => {
                    if let Some(notice_datetime) = notice {
                        item.set_notice(notice);
                        self.add_to_notice(item_rc.clone(), notice_datetime);
                        self.noticer.add_notice(notice_datetime);
                    }
                }
                Some(notice_already) => {
                    match notice {
                        None => {
                            item.set_notice(notice);
                            self.remove_from_notice(&item_rc, &notice_already);
                            self.noticer.remove_notice(&notice_already);
                        }
                        Some(notice_datetime) => {
                            if notice_already != notice_datetime {
                                item.set_notice(notice);
                                self.remove_from_notice(&item_rc, &notice_already);
                                self.noticer.remove_notice(&notice_already);
                                self.add_to_notice(item_rc.clone(), notice_datetime);
                                self.noticer.add_notice(notice_datetime);
                            }
                        }
                    }
                }
            }
        }
        
        self
    }

    pub fn set_item_deadline(&self, item: &Weak<RefCell<item::Item>>,  deadline: Option<NaiveDate>) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_deadline(deadline);
        }
        self
    }

    pub fn set_item_plan(&self, item: &Weak<RefCell<item::Item>>,  plan: Option<NaiveDate>) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_plan(plan);
        }
        self
    }

    pub fn set_item_repeat(&self, item: &Weak<RefCell<item::Item>>,  repeat: Option<item::RepeatSpan>) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_repeat(repeat);
        }
        self
    }

    pub fn set_item_list(&self, item: &Weak<RefCell<item::Item>>,  list: Weak<RefCell<list::List>>) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_list(list);
        }
        self
    }

    pub fn set_item_finished(&self, item: &Weak<RefCell<item::Item>>,  finished: bool) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_finished(finished);
        }
        self
    }

    pub fn set_item_note(&self, item: &Weak<RefCell<item::Item>>, note: &str) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            item.borrow_mut().set_note(note.to_string());
        }
        self
    }

    pub fn iter_items(&self) -> ItemIntoIter {
        ItemIntoIter::new(&self.items)
    }

    fn add_to_notice(&mut self, item: Rc<RefCell<item::Item>>, datetime: NaiveDateTime) {
        let notice_items = &mut self.notice_items;
        if !notice_items.contains_key(&datetime) {
            notice_items.insert(datetime, RefCell::new(Vec::new()));
        }
        notice_items[&datetime].borrow_mut().push(item);
    }

    fn remove_from_notice(&mut self, item: &Rc<RefCell<item::Item>>, datetime: &NaiveDateTime) {
        let notice_items = &mut self.notice_items;
        if notice_items.contains_key(datetime) {
            let mut v = notice_items[datetime].borrow_mut();
            let mut i_to_remove: usize = 0;
            let mut found = false;
            for (i, val) in v.iter().enumerate() {
                if val == item {
                    i_to_remove = i;
                    found = true;
                    break;
                }
            }
            if found {
                v.swap_remove(i_to_remove);
            }
            let len = v.len();
            drop(v);
            if len == 0 {
                notice_items.remove(&datetime);
            }
        }
    }

    pub fn list(&self, title: &str) -> Weak<RefCell<list::List>> {
        Rc::downgrade(&self.lists[title])
    }

    /// ## Return
    /// 
    /// Error if title have existed. or the new `Weak<RefCell<list::List>>`
    pub fn new_list(&mut self, title: &str) -> Result<Weak<RefCell<list::List>>, Error> {
        if self.lists.contains_key(title) {
            return Err(Error)
        }

        let lst = Rc::new(RefCell::new(list::List::new(
            self.next_list_id, title, None
        )));

        self.lists.insert(title.to_string(), lst);

        let lst = self.list(title);

        self.next_list_id += 1;

        Ok(lst)
    }

    pub fn set_list_title(&self, ori_title: &str,  title: &str) -> &Self {
        self.lists[ori_title].borrow_mut().set_title(title.to_string());
        self
    }

    pub fn iter_lists(&self) -> ListIntoIter {
        ListIntoIter::new(&self.lists)
    }

    pub fn group(&self, title: &str) -> Weak<RefCell<group::Group>> {
        Rc::downgrade(&self.groups[title])
    }

    /// ## Return
    /// 
    /// Error if title have existed. or the new `Weak<RefCell<group::Group>>`
    pub fn new_group(&mut self, title: &str) -> Result<Weak<RefCell<group::Group>>, Error> {
        if self.groups.contains_key(title) {
            return Err(Error)
        }

        let grp = Rc::new(RefCell::new(group::Group::new(
            self.next_group_id, title, None
        )));

        self.groups.insert(title.to_string(), grp);

        let grp = self.group(title);

        self.next_group_id += 1;

        Ok(grp)
    }

    pub fn set_group_title(&self, ori_title: &str,  title: &str) -> &Self {
        self.groups[ori_title].borrow_mut().set_title(title.to_string());
        self
    }

    pub fn iter_groups(&self) -> GroupIntoIter {
        GroupIntoIter::new(&self.groups)
    }

    pub fn update(&self) {
        let notice_comed = self.notice_comed.lock();
        match notice_comed {
            Err(_) => (),
            Ok(mut notice_comed) => {
                if notice_comed.len()>0 {
                    for notice in notice_comed.iter() {
                        for item in self.notice_items[notice].borrow().iter() {
                            let item = item.borrow();
                            println!("item: id {}, message {}.", item.id(), item.message())
                        }
                    }
                }
                notice_comed.clear();
            }
        }
    }
}

impl Drop for TodoLst {
    fn drop(&mut self) {
        self.noticer.stop();
    }
}

pub struct ItemIntoIter {
    items: Vec<Weak<RefCell<item::Item>>>,
    idx: usize
}

impl ItemIntoIter {
    fn new(items: &HashMap<u32, Rc<RefCell<item::Item>>>) -> Self{
        Self {
            items: items.values().map(
                | value | {
                    Rc::downgrade(value)
                }
            ).collect(),
            idx: 0usize
        }
    }
}

impl Iterator for ItemIntoIter {
    type Item = Weak<RefCell<item::Item>>;

    fn next(&mut self) -> Option<Self::Item> {
        let itm = self.items[self.idx].clone();
        self.idx += 1;
        Some(itm)
    }
}

pub struct ListIntoIter {
    items: Vec<Weak<RefCell<list::List>>>,
    idx: usize
}

impl ListIntoIter {
    fn new(items: &HashMap<String, Rc<RefCell<list::List>>>) -> Self{
        Self {
            items: items.values().map(
                | value | {
                    Rc::downgrade(value)
                }
            ).collect(),
            idx: 0usize
        }
    }
}

impl Iterator for ListIntoIter {
    type Item = Weak<RefCell<list::List>>;

    fn next(&mut self) -> Option<Self::Item> {
        let itm = self.items[self.idx].clone();
        self.idx += 1;
        Some(itm)
    }
}

pub struct GroupIntoIter {
    items: Vec<Weak<RefCell<group::Group>>>,
    idx: usize
}

impl GroupIntoIter {
    fn new(items: &HashMap<String, Rc<RefCell<group::Group>>>) -> Self{
        Self {
            items: items.values().map(
                | value | {
                    Rc::downgrade(value)
                }
            ).collect(),
            idx: 0usize
        }
    }
}

impl Iterator for GroupIntoIter {
    type Item = Weak<RefCell<group::Group>>;

    fn next(&mut self) -> Option<Self::Item> {
        let itm = self.items[self.idx].clone();
        self.idx += 1;
        Some(itm)
    }
}