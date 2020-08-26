use crate::components::{ group, item, list, noticer };
use std::collections::{ HashMap, BTreeMap };
// use std::rc::*;
// use std::cell::*;
use std::sync::{ Arc, Mutex, Weak };
use chrono::prelude::*;
// use std::thread;
// use std::time;
use std::path::Path;
use sqlx::sqlite::*;
use rusqlite::{ params, Connection };
use futures::{Future};
use futures::executor::*;


#[derive(Debug)]
pub struct Error;

pub struct TodoLst {
    groups: HashMap<String, Arc<Mutex<group::Group>>>,
    lists: HashMap<String, Arc<Mutex<list::List>>>,
    items: HashMap<u32, Arc<Mutex<item::Item>>>,
    next_item_id: u32,
    next_list_id: u32,
    next_group_id: u32,
    noticer: noticer::Noticer,
    notice_items: Arc<Mutex<BTreeMap<NaiveDateTime, Mutex<Vec<Arc<Mutex<item::Item>>>>>>>,
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
            notice_items: Arc::new(Mutex::new(BTreeMap::new())),
            notice_comed: Arc::new(Mutex::new(Vec::new())),
        };
        // let notice_commed = val.notice_comed.clone();
        let notice_items = val.notice_items.clone();
        val.noticer.add_callback(move | datetime | {
            println!("{}", datetime);
            // let notice_commed = notice_commed.lock();
            // match notice_commed {
            //     Err(_) => (),
            //     Ok(mut notice_comed) => {
            //         notice_comed.push(datetime);
            //     }
            // }
            let notice_items = notice_items.lock().unwrap();
            for item in notice_items[&datetime].lock().unwrap().iter() {
                let item = item.lock().unwrap();
                println!("item: id {}, message {}.", item.id(), item.message())
            }
        });
        val.noticer.start();
        val
    }

    pub fn item(&self, id: u32) -> Weak<Mutex<item::Item>> {
        // Arc::downgrade(&self.items[&id])
        Arc::downgrade(&self.items[&id])
    }

    pub fn new_item(&mut self, message: &str, list: Weak<Mutex<list::List>>) -> Weak<Mutex<item::Item>> {
        let itm = item::Item::new(
            self.next_item_id, message, 0, item::ItemStyle{ marker: item::Marker(0) }, false, 
            None, None, None, None, list.clone()
        );

        self.add_item(itm)
    }

    pub fn add_item(&mut self, item: item::Item) -> Weak<Mutex<item::Item>> {
        let list = item.list().clone();
        
        let itm = Arc::new(Mutex::new(item));

        self.items.insert(self.next_item_id, itm.clone());
        list.upgrade().unwrap().lock().unwrap().add_item(itm.clone());

        let itm = self.item(self.next_item_id);

        self.next_item_id += 1;

        itm
    }

    pub fn set_item_message(&self, item: Weak<Mutex<item::Item>>, message: &str) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_message(message.to_string());
                }
            }
        }
        self
    }

    pub fn set_item_level(&self, item: &Weak<Mutex<item::Item>>,  level: i8) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_level(level);
                }
            }
        }
        self
    }

    pub fn set_item_style(&self, item: &Weak<Mutex<item::Item>>,  style: item::ItemStyle) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_style(style);
                }
            }
        }
        self
    }

    pub fn set_item_today(&self, item: &Weak<Mutex<item::Item>>,  today: bool) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_today(today);
                }
            }
        }
        self
    }

    pub fn set_item_notice(&mut self, item: &Weak<Mutex<item::Item>>,  notice: Option<NaiveDateTime>) -> &Self {
        let item = item.upgrade();
        if let Some(item_arc) = item {
            let mut item = item_arc.lock().unwrap();
            match item.notice() {
                None => {
                    if let Some(notice_datetime) = notice {
                        item.set_notice(notice);
                        self.add_to_notice(item_arc.clone(), notice_datetime);
                        self.noticer.add_notice(notice_datetime);
                    }
                }
                Some(notice_already) => {
                    match notice {
                        None => {
                            item.set_notice(notice);
                            self.remove_from_notice(&item_arc, &notice_already);
                            self.noticer.remove_notice(&notice_already);
                        }
                        Some(notice_datetime) => {
                            if notice_already != notice_datetime {
                                item.set_notice(notice);
                                self.remove_from_notice(&item_arc, &notice_already);
                                self.noticer.remove_notice(&notice_already);
                                self.add_to_notice(item_arc.clone(), notice_datetime);
                                self.noticer.add_notice(notice_datetime);
                            }
                        }
                    }
                }
            }
        }
        
        self
    }

    pub fn set_item_deadline(&self, item: &Weak<Mutex<item::Item>>,  deadline: Option<NaiveDate>) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_deadline(deadline);
                }
            }
        }
        self
    }

    pub fn set_item_plan(&self, item: &Weak<Mutex<item::Item>>,  plan: Option<NaiveDate>) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_plan(plan);
                }
            }
        }
        self
    }

    pub fn set_item_repeat(&self, item: &Weak<Mutex<item::Item>>,  repeat: Option<item::RepeatSpan>) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_repeat(repeat);
                }
            }
        }
        self
    }

    pub fn set_item_list(&self, item: &Weak<Mutex<item::Item>>,  list: Weak<Mutex<list::List>>) -> &Self {
        let item = item.upgrade();
        if let Some(item_rc) = item {
            let mut list_old = None;
            let item = item_rc.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    list_old = Some(item.list());
                    item.set_list(list);
                }
            }
            match list_old {
                None => (),
                Some(list_old) => { list_old.upgrade().unwrap().lock().unwrap().remove_item(&item_rc); }
            }
            
        }
        self
    }

    pub fn set_item_finished(&self, item: &Weak<Mutex<item::Item>>,  finished: bool) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_finished(finished);
                }
            }
        }
        self
    }

    pub fn set_item_note(&self, item: &Weak<Mutex<item::Item>>, note: &str) -> &Self {
        let item = item.upgrade();
        if let Some(item) = item {
            let item = item.lock();
            match item {
                Err(_) => (),
                Ok(mut item) => {
                    item.set_note(note.to_string());
                }
            }
        }
        self
    }

    pub fn iter_items(&self) -> ItemIntoIter {
        ItemIntoIter::new(&self.items)
    }

    fn add_to_notice(&mut self, item: Arc<Mutex<item::Item>>, datetime: NaiveDateTime) {
        let mut notice_items = self.notice_items.lock().unwrap();
        if !notice_items.contains_key(&datetime) {
            notice_items.insert(datetime, Mutex::new(Vec::new()));
        }
        let v = notice_items[&datetime].lock();
        match v {
            Err(_) => (),
            Ok(mut v) => {
                v.push(item);
            }
        }
    }

    fn remove_from_notice(&mut self, item: &Arc<Mutex<item::Item>>, datetime: &NaiveDateTime) {
        let mut notice_items = self.notice_items.lock().unwrap();
        if notice_items.contains_key(datetime) {
            let mut v = notice_items[datetime].lock().unwrap();

            let mut i_to_remove: usize = 0;
            let mut found = false;
            for (i, val) in v.iter().enumerate() {
                let val = val.lock().unwrap();
                let item = item.lock().unwrap();
                if *val == *item {
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

    pub fn list(&self, title: &str) -> Weak<Mutex<list::List>> {
        Arc::downgrade(&self.lists[title])
    }

    /// ## Return
    /// 
    /// Error if title have existed. or the new `Weak<Mutex<list::List>>`
    pub fn new_list(&mut self, title: &str) -> Result<Weak<Mutex<list::List>>, Error> {
        if self.lists.contains_key(title) {
            return Err(Error)
        }

        let lst = Arc::new(Mutex::new(list::List::new(
            self.next_list_id, title, None
        )));

        self.lists.insert(title.to_string(), lst);

        let lst = self.list(title);

        self.next_list_id += 1;

        Ok(lst)
    }

    pub fn set_list_title(&self, ori_title: &str,  title: &str) -> &Self {
        let list = self.lists[ori_title].lock();
        match list {
            Err(_) => (),
            Ok(mut list) => {
                list.set_title(title.to_string());
            }
        }
        self
    }

    pub fn set_list_group(&self, title: &str, group: Option<Weak<Mutex<group::Group>>>) -> &Self {
        let list = self.lists[title].lock();
        match list {
            Err(_) => (),
            Ok(mut list) => {
                list.set_group(group);
            }
        }
        self
    }

    pub fn iter_lists(&self) -> ListIntoIter {
        ListIntoIter::new(&self.lists)
    }

    pub fn group(&self, title: &str) -> Weak<Mutex<group::Group>> {
        Arc::downgrade(&self.groups[title])
    }

    /// ## Return
    /// 
    /// Error if title have existed. or the new `Weak<Mutex<group::Group>>`
    pub fn new_group(&mut self, title: &str) -> Result<Weak<Mutex<group::Group>>, Error> {
        if self.groups.contains_key(title) {
            return Err(Error)
        }

        let grp = Arc::new(Mutex::new(group::Group::new(
            self.next_group_id, title, None
        )));

        self.groups.insert(title.to_string(), grp);

        let grp = self.group(title);

        self.next_group_id += 1;

        Ok(grp)
    }

    pub fn set_group_title(&self, ori_title: &str,  title: &str) -> &Self {
        let group = self.groups[ori_title].lock();
        match group {
            Err(_) => (),
            Ok(mut group) => {
                group.set_title(title.to_string());
            }
        }
        self
    }

    pub fn set_group_parent(&self, title: &str, parent: Option<Weak<Mutex<group::Group>>>) -> &Self {
        let group = self.groups[title].lock();
        match group {
            Err(_) => (),
            Ok(mut group) => {
                group.set_parent(parent);
            }
        }
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
                        let notice_items = self.notice_items.lock().unwrap();
                        for item in notice_items[notice].lock().unwrap().iter() {
                            let item = item.lock().unwrap();
                            println!("item: id {}, message {}.", item.id(), item.message())
                        }
                    }
                }
                notice_comed.clear();
            }
        }
    }

    pub async fn load() -> Self {
        // let conn_opt = SqliteConnectOptions::new().filename("todolst.db").read_only(true);
        // let pool = SqlitePoolOptions::new().connect_with(conn_opt).await;
        // match pool {
        //     Err(_) => (),
        //     Ok(pool) => {
                
        //     }
        // }
        
        let db_path = Path::new("todolst.db");
        let db_exists = db_path.exists();
        
        if db_exists {
            if let Ok(conn) = Connection::open(db_path) {
                let mut mgmt = Self::new();
                let mut stmt = conn.prepare("SELECT title, parent FROM groups").unwrap();
                let gps: Vec<GroupRow> = stmt.query_map(rusqlite::NO_PARAMS, |row| {
                    Ok(GroupRow(
                        row.get(0)?,
                        row.get(1)?
                    ))
                }).unwrap().map(|itm| {itm.unwrap()}).collect();
                for gp in gps.iter() {
                    mgmt.new_list(gp.0.as_ref()).unwrap_or_default();
                }
                for gp in gps.iter() {
                    mgmt.set_group_parent(gp.0.as_ref(), if gp.1.len()>0 {Some(mgmt.group(gp.1.as_ref()))} else {None});
                }

                let mut stmt = conn.prepare("SELECT title, group FROM groups").unwrap();
                let lsts: Vec<ListRow> = stmt.query_map(rusqlite::NO_PARAMS, |row| {
                    Ok(ListRow(
                        row.get(0)?,
                        row.get(1)?
                    ))
                }).unwrap().map(|itm|{itm.unwrap()}).collect();
                for lst in lsts {
                    if let Ok(_) = mgmt.new_list(lst.0.as_ref()) {
                        mgmt.set_list_group(lst.0.as_ref(), if lst.1.len()>0 {Some(mgmt.group(lst.1.as_ref()))} else {None});
                    }
                }

                let stmt = conn.prepare("SELECT message, level, marker, color, today, notice, notice, deadline, plan, repeat, list, finished, note FROM items").unwrap();
                todo!();
                
            }
        }
        
        
        

        todo!();
    }

    pub fn save(&self) {
        todo!();
    }
}

impl Drop for TodoLst {
    fn drop(&mut self) {
        self.noticer.stop();
    }
}

struct GroupRow(String/* title */, String/* parent */);
struct ListRow(String/* title */, String/* group */);

pub struct ItemIntoIter {
    items: Vec<Weak<Mutex<item::Item>>>,
    idx: usize
}

impl ItemIntoIter {
    fn new(items: &HashMap<u32, Arc<Mutex<item::Item>>>) -> Self{
        Self {
            items: items.values().map(
                | value | {
                    Arc::downgrade(value)
                }
            ).collect(),
            idx: 0usize
        }
    }
}

impl Iterator for ItemIntoIter {
    type Item = Weak<Mutex<item::Item>>;

    fn next(&mut self) -> Option<Self::Item> {
        let itm = self.items[self.idx].clone();
        self.idx += 1;
        Some(itm)
    }
}

pub struct ListIntoIter {
    items: Vec<Weak<Mutex<list::List>>>,
    idx: usize
}

impl ListIntoIter {
    fn new(items: &HashMap<String, Arc<Mutex<list::List>>>) -> Self{
        Self {
            items: items.values().map(
                | value | {
                    Arc::downgrade(value)
                }
            ).collect(),
            idx: 0usize
        }
    }
}

impl Iterator for ListIntoIter {
    type Item = Weak<Mutex<list::List>>;

    fn next(&mut self) -> Option<Self::Item> {
        let itm = self.items[self.idx].clone();
        self.idx += 1;
        Some(itm)
    }
}

pub struct GroupIntoIter {
    items: Vec<Weak<Mutex<group::Group>>>,
    idx: usize
}

impl GroupIntoIter {
    fn new(items: &HashMap<String, Arc<Mutex<group::Group>>>) -> Self{
        Self {
            items: items.values().map(
                | value | {
                    Arc::downgrade(value)
                }
            ).collect(),
            idx: 0usize
        }
    }
}

impl Iterator for GroupIntoIter {
    type Item = Weak<Mutex<group::Group>>;

    fn next(&mut self) -> Option<Self::Item> {
        let itm = self.items[self.idx].clone();
        self.idx += 1;
        Some(itm)
    }
}