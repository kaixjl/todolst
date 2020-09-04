use crate::components::{ group, item, list, noticer };
use std::collections::{ HashMap, BTreeMap };
// use std::rc::*;
// use std::cell::*;
use std::sync::{ Arc, Mutex, Weak };
use chrono::prelude::*;
// use std::thread;
// use std::time;
use std::path::Path;
// use sqlx::sqlite::*;
use rusqlite::{ params, Connection };
// use futures::{Future};
// use futures::executor::*;


#[derive(Debug)]
pub enum Error {
    RusqliteError(rusqlite::Error),
    IoError(std::io::Error),
    NonexistingList,
    NonexistingGroup,
    NonexistingItem,
    ExistingTitle,
    OtherErrorWithStr(String),
    OtherError,
}

type Result<T> = std::result::Result<T, Error>;

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::RusqliteError(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

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

    pub fn item(&self, id: u32) -> Result<Weak<Mutex<item::Item>>> {
        if !self.items.contains_key(&id) { return Err(Error::NonexistingItem); }
        Ok(Arc::downgrade(&self.items[&id]))
    }

    pub fn new_item(&mut self, message: &str, list: Weak<Mutex<list::List>>) -> Weak<Mutex<item::Item>> {
        let itm = item::Item::new(
            self.next_item_id, message, 0, item::ItemStyle{ marker: item::Marker(0) }, false, 
            None, None, None, None, Some(list.clone())
        );

        self.add_item(itm, list)
    }

    pub fn add_item(&mut self, item: item::Item, list: Weak<Mutex<list::List>>) -> Weak<Mutex<item::Item>> {
       
        let itm = Arc::new(Mutex::new(item));

        self.items.insert(self.next_item_id, itm.clone());
        list.upgrade().unwrap().lock().unwrap().add_item(itm.clone());

        let itm = self.item(self.next_item_id);

        self.next_item_id += 1;

        itm.unwrap()
    }

    pub fn remove_item(&mut self, item: Weak<Mutex<item::Item>>) {
        self.set_item_notice(&item, None);
        
        let item = item.upgrade().unwrap();
        let item = item.lock().unwrap();
        let id = item.id();
        self.items.remove(&id);
        
    }

    pub fn set_item_message(&self, item: &Weak<Mutex<item::Item>>, message: &str) -> &Self {
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
                        drop(item);
                        self.add_to_notice(item_arc.clone(), notice_datetime);
                        if Local::now().naive_local() < notice_datetime {
                            self.noticer.add_notice(notice_datetime);
                        }
                    }
                }
                Some(notice_already) => {
                    match notice {
                        None => {
                            item.set_notice(notice);
                            drop(item);
                            self.remove_from_notice(&item_arc, &notice_already);
                            self.noticer.remove_notice(&notice_already);
                        }
                        Some(notice_datetime) => {
                            if notice_already != notice_datetime {
                                item.set_notice(notice);
                                drop(item);
                                self.remove_from_notice(&item_arc, &notice_already);
                                self.noticer.remove_notice(&notice_already);
                                self.add_to_notice(item_arc.clone(), notice_datetime);
                                if Local::now().naive_local() < notice_datetime {
                                    self.noticer.add_notice(notice_datetime);
                                }
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
                    list_old = item.list();
                    item.set_list(Some(list));
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
                let val = val.lock().unwrap().id();
                let item = item.lock().unwrap().id();
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

    pub fn list(&self, title: &str) -> Result<Weak<Mutex<list::List>>> {
        if !self.lists.contains_key(title) { return Err(Error::NonexistingList); }
        Ok(Arc::downgrade(&self.lists[title]))
    }

    /// ## Return
    /// 
    /// Error if title have existed. or the new `Weak<Mutex<list::List>>`
    pub fn new_list(&mut self, title: &str) -> Result<Weak<Mutex<list::List>>> {
        if self.lists.contains_key(title) {
            return Err(Error::ExistingTitle)
        }

        let lst = Arc::new(Mutex::new(list::List::new(
            self.next_list_id, title, None
        )));

        self.lists.insert(title.to_string(), lst);

        let lst = self.list(title);

        self.next_list_id += 1;

        Ok(lst.unwrap())
    }

    pub fn remove_list(&mut self, title: &str) {
        let items_to_remove = self.items.values().filter(|i| {
            let i = i.lock().unwrap();
            let list = i.list();
            match list {
                None => false,
                Some(list) => {
                    let list = list.upgrade().unwrap();
                    let list = list.lock().unwrap();
                    list.title()==title
                }
            }
        }).map(|i| {
            Arc::downgrade(i)
        }).collect::<Vec<Weak<Mutex<item::Item>>>>();
        for i in items_to_remove {
            self.remove_item(i);
        }

        self.lists.remove(title);
    }

    pub fn set_list_title(&self, ori_title: &str,  title: &str) -> Result<&Self> {
        if !self.lists.contains_key(ori_title) { return Err(Error::NonexistingList); }
        if self.lists.contains_key(title) { return Err(Error::ExistingTitle); }
        let list = self.lists[ori_title].lock();
        match list {
            Err(_) => (),
            Ok(mut list) => {
                list.set_title(title.to_string());
            }
        }
        Ok(self)
    }

    pub fn set_list_group(&self, title: &str, group: Option<Weak<Mutex<group::Group>>>) -> Result<&Self> {
        if !self.lists.contains_key(title) { return Err(Error::NonexistingGroup); }
        let list = self.lists[title].lock();
        match list {
            Err(_) => (),
            Ok(mut list) => {
                list.set_group(group);
            }
        }
        Ok(self)
    }

    pub fn iter_lists(&self) -> ListIntoIter {
        ListIntoIter::new(&self.lists)
    }

    pub fn group(&self, title: &str) -> Result<Weak<Mutex<group::Group>>> {
        if !self.groups.contains_key(title) { return Err(Error::NonexistingGroup); }
        Ok(Arc::downgrade(&self.groups[title]))
    }

    /// ## Return
    /// 
    /// Error if title have existed. or the new `Weak<Mutex<group::Group>>`
    pub fn new_group(&mut self, title: &str) -> Result<Weak<Mutex<group::Group>>> {
        if self.groups.contains_key(title) {
            return Err(Error::ExistingTitle)
        }

        let grp = Arc::new(Mutex::new(group::Group::new(
            self.next_group_id, title, None
        )));

        self.groups.insert(title.to_string(), grp);

        let grp = self.group(title);

        self.next_group_id += 1;

        Ok(grp.unwrap())
    }

    pub fn remove_group(&mut self, title: &str) {
        let mut group_to_remove = vec![title.to_string()];
        let mut group_to_remove_really = Vec::new();
        let mut list_to_remove_really = Vec::new();
        while let Some(title_s) = group_to_remove.pop() {
            let title = &title_s[..];
            let mut lists_to_remove = self.lists.iter().filter_map(|(k, i)| {
                let i = i.lock().unwrap();
                let group = i.group();
                match group {
                    None => None,
                    Some(group) => {
                        let group = group.upgrade().unwrap();
                        let group = group.lock().unwrap();
                        if group.title()==title {
                            Some(k.clone())
                        } else { None }
                    }
                }
            }).collect::<Vec<String>>();
            list_to_remove_really.append(&mut lists_to_remove);

            let mut groups_to_add_to_remove = self.groups.iter().filter_map(|(k, i)| {
                let i = i.lock().unwrap();
                let parent = i.parent();
                match parent {
                    None => None,
                    Some(parent) => {
                        let parent = parent.upgrade().unwrap();
                        let parent = parent.lock().unwrap();
                        if parent.title()==title {
                            Some(k.clone())
                        } else { None }
                    }
                }
            }).collect::<Vec<String>>();
            group_to_remove.append(&mut groups_to_add_to_remove);
            group_to_remove_really.push(title_s);
        }
        for i in list_to_remove_really {
            self.remove_list(i.as_ref());
        }
        for title in group_to_remove_really {
            self.groups.remove(&title);
        }
    }

    pub fn set_group_title(&self, ori_title: &str,  title: &str) -> Result<&Self> {
        if !self.groups.contains_key(ori_title) { return Err(Error::NonexistingGroup); }
        if self.groups.contains_key(title) { return Err(Error::ExistingTitle); }
        let group = self.groups[ori_title].lock();
        match group {
            Err(_) => (),
            Ok(mut group) => {
                group.set_title(title.to_string());
            }
        }
        Ok(self)
    }

    pub fn set_group_parent(&self, title: &str, parent: Option<Weak<Mutex<group::Group>>>) -> Result<&Self> {
        if !self.groups.contains_key(title) { return Err(Error::NonexistingGroup); }
        let group = self.groups[title].lock();
        match group {
            Err(_) => (),
            Ok(mut group) => {
                group.set_parent(parent);
            }
        }
        Ok(self)
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

        let mut mgmt = Self::new();
        
        if db_exists {
            if let Ok(conn) = Connection::open(db_path) {
                let mut stmt = conn.prepare("SELECT title, parent FROM groups").unwrap();
                let gps: Vec<GroupRow> = stmt.query_map(rusqlite::NO_PARAMS, |row| {
                    Ok(GroupRow(
                        row.get(0)?,
                        row.get(1)?
                    ))
                }).unwrap().map(|itm| {itm.unwrap()}).collect();
                for gp in gps.iter() {
                    mgmt.new_group(gp.0.as_ref()).unwrap_or_default();
                }
                for gp in gps.iter() {
                    let parent = {
                        match gp.1 {
                            None => None,
                            Some(ref parent) => {
                                if parent.len()>0 {
                                    Some(mgmt.group(parent.as_ref()).unwrap())
                                } else { None }
                            }
                        }
                    };
                    mgmt.set_group_parent(gp.0.as_ref(), parent).unwrap();
                }

                let mut stmt = conn.prepare("SELECT title, [group] FROM lists").unwrap();
                let lsts: Vec<ListRow> = stmt.query_map(rusqlite::NO_PARAMS, |row| {
                    Ok(ListRow(
                        row.get(0)?,
                        row.get(1)?
                    ))
                }).unwrap().map(|itm|{itm.unwrap()}).collect();
                for lst in lsts {
                    if let Ok(_) = mgmt.new_list(lst.0.as_ref()) {
                        let group = {
                            match lst.1 {
                                None => None,
                                Some(ref group) => {
                                    if group.len()>0 {
                                        Some(mgmt.group(group.as_ref()).unwrap())
                                    } else { None }
                                }
                            }
                        };
                        mgmt.set_list_group(lst.0.as_ref(), group).unwrap();
                    }
                }

                let mut stmt = conn.prepare("SELECT message, level, marker, color, today, notice, deadline, [plan], repeat, repeatunit, list, finished, note FROM items").unwrap();
                let itms: Vec<ItemRow> = stmt.query_map(rusqlite::NO_PARAMS, |row| {
                    Ok(ItemRow {
                        message: row.get(0)?,
                        level: row.get(1)?,
                        style: item::ItemStyle { marker: item::Marker(row.get(2)?)},
                        today: match row.get::<usize, Option<NaiveDate>>(4)? { None=>false, Some(date)=>Local::now().naive_local().date() == date },
                        notice: row.get(5)?,
                        deadline: row.get(6)?,
                        plan: row.get(7)?,
                        repeat: {
                            let repeatspan = row.get::<usize, i32>(8)?;
                            let repeatunit = row.get::<usize, i8>(9)?;

                            if repeatspan==0 { None }
                            else {
                                match repeatunit {
                                    0=> { Some(item::RepeatSpan::Days(repeatspan)) },
                                    1=> { Some(item::RepeatSpan::Weeks(repeatspan)) },
                                    2=> { Some(item::RepeatSpan::Months(repeatspan)) },
                                    3=> { Some(item::RepeatSpan::Years(repeatspan)) },
                                    _=> { None }
                                }
                            }
                        },
                        list: row.get(10)?,
                        finished: row.get(11)?,
                        note: row.get(12)?,
                    })
                }).unwrap().map(|itm| {itm.unwrap()}).collect();
                for itm in itms {
                    let list = mgmt.list(itm.list.as_ref()).unwrap();
                    let item = mgmt.new_item(itm.message.as_ref(), list);
                    mgmt.set_item_level(&item, itm.level);
                    mgmt.set_item_style(&item, itm.style);
                    mgmt.set_item_today(&item, itm.today);
                    mgmt.set_item_notice(&item, itm.notice);
                    mgmt.set_item_deadline(&item, itm.deadline);
                    mgmt.set_item_plan(&item, itm.plan);
                    mgmt.set_item_repeat(&item, itm.repeat);
                    mgmt.set_item_finished(&item, itm.finished);
                    mgmt.set_item_note(&item, itm.note.as_ref());
                }
            }
        }
        
       mgmt
    }

    pub async fn save(&self) -> Result<()> {
        let db_path = Path::new("todolst.db");
        let db_exists = db_path.exists();
        let db_bk_path = Path::new("todolst.db.bak");
        let db_tmp_path = Path::new("todolst.db.tmp");
        // let db_bk_exists = db_bk_path.exists();
        // if db_bk_exists { std::fs::remove_file(db_bk_path)?; }
        if db_tmp_path.exists() { std::fs::remove_file(db_tmp_path)?; }

        if let Ok(mut conn) = Connection::open(db_tmp_path) {
            conn.execute(
                r#"
                CREATE TABLE groups (
                    id     INTEGER UNIQUE
                                   NOT NULL,
                    title  STRING  PRIMARY KEY,
                    parent STRING 
                );
                "#, 
                rusqlite::NO_PARAMS)?;

            conn.execute(
                r#"
                CREATE TABLE lists (
                    id      INTEGER UNIQUE
                                    NOT NULL,
                    title   STRING  PRIMARY KEY,
                    [group] STRING  REFERENCES groups (title) 
                );
                "#, 
                rusqlite::NO_PARAMS)?;

            conn.execute(
                r#"
                CREATE TABLE items (
                    id         INTEGER  PRIMARY KEY,
                    message    STRING   NOT NULL,
                    level      INTEGER  NOT NULL
                                        DEFAULT (0),
                    marker     INTEGER  NOT NULL
                                        DEFAULT (0),
                    color      INTEGER  NOT NULL
                                        DEFAULT (0),
                    today      DATETIME,
                    notice     DATETIME,
                    deadline   DATETIME,
                    [plan]     DATETIME,
                    repeat     INTEGER  DEFAULT (0) 
                                        NOT NULL,
                    repeatunit INTEGER  DEFAULT (0) 
                                        NOT NULL,
                    list       STRING   REFERENCES lists (title),
                    finished   BOOLEAN  NOT NULL
                                        DEFAULT (false),
                    note       STRING
                );
                "#, 
                rusqlite::NO_PARAMS)?;

            let transaction = conn.transaction()?;
            { // Insert groups
                let mut stmt = transaction.prepare(
                    "INSERT INTO groups(id, title, parent) VALUES (?1, ?2, ?3)"
                )?;
                for group in self.iter_groups() {
                    if let Some(group) = group.upgrade() {
                        let group = group.lock().unwrap();
                        let id = group.id();
                        let title = group.title();
                        let parent = if group.parent().is_some() { 
                            let parent = group.parent().unwrap();
                            let parent = parent.upgrade().unwrap();
                            let parent = parent.lock().unwrap();
                            Some(parent.title().to_string())
                        } else { None };
                        stmt.execute(params![id, title, parent])?;
                    }
                }
            } // Insert groups
            { // Insert lists
                let mut stmt = transaction.prepare(
                    "INSERT INTO lists(id, title, [group]) VALUES (?1, ?2, ?3)"
                )?;
                for list in self.iter_lists() {
                    if let Some(list) = list.upgrade() {
                        let list = list.lock().unwrap();
                        let id = list.id();
                        let title = list.title();
                        let group = if list.group().is_some() {
                            let group = list.group().unwrap();
                            let group = group.upgrade().unwrap();
                            let group = group.lock().unwrap();
                            Some(group.title().to_string())
                        } else { None };
                        stmt.execute(params![id, title, group])?;
                    }
                }
            } // Insert lists
            { // Insert items
                let mut stmt = transaction.prepare(
                    "INSERT INTO items(id, message, level, marker, color, today, notice, deadline, [plan], repeat, repeatunit, list, finished, note) 
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
                )?;
                for item in self.iter_items() {
                    if let Some(item) = item.upgrade() {
                        let item = item.lock().unwrap();
                        let id = item.id();
                        let message = item.message();
                        let level = item.level();
                        let marker = item.style().marker.0;
                        let color = 0i32;
                        let today = if item.today() { Some(Local::now().naive_local().date()) } else { None };
                        let notice = item.notice();
                        let deadline = item.deadline();
                        let plan = item.plan();
                        let repeat = match item.repeat() {
                            None => 0,
                            Some(r) => {
                                match r {
                                    item::RepeatSpan::Days(r) => r,
                                    item::RepeatSpan::Weeks(r) => r,
                                    item::RepeatSpan::Months(r) => r,
                                    item::RepeatSpan::Years(r) => r,
                                }
                            },
                        };
                        let repeatunit = match item.repeat() {
                            None => 0,
                            Some(r) => {
                                match r {
                                    item::RepeatSpan::Days(_) => 0,
                                    item::RepeatSpan::Weeks(_) => 1,
                                    item::RepeatSpan::Months(_) => 2,
                                    item::RepeatSpan::Years(_) => 3,
                                }
                            },
                        };
                        let list = if item.list().is_some() {
                            let list = item.list().unwrap();
                            let list = list.upgrade().unwrap();
                            let list = list.lock().unwrap();
                            Some(list.title().to_string())
                        } else { None };
                        let finished = item.finished();
                        let note = item.note();
                        stmt.execute(params![id, message, level, marker, color, today, notice, deadline, plan, repeat, repeatunit, list, finished, note])?;
                    }
                }
            } // Insert items
            transaction.commit().unwrap_or_default();
            drop(conn);
            
            if db_exists { std::fs::rename(db_path, db_bk_path).unwrap(); }
            std::fs::rename(db_tmp_path, db_path).unwrap();
            return Ok(())
        }
        Err(Error::OtherError)
    }
}

impl Drop for TodoLst {
    fn drop(&mut self) {
        self.noticer.stop();
    }
}

struct GroupRow(String/* title */, Option<String>/* parent */);
struct ListRow(String/* title */, Option<String>/* group */);
struct ItemRow {
    message: String,
    level: i8,
    style: item::ItemStyle,
    today: bool,
    notice: Option<NaiveDateTime>,
    deadline: Option<NaiveDate>,
    plan: Option<NaiveDate>,
    repeat: Option<item::RepeatSpan>,
    list: String,
    finished: bool,
    note: String,
}

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
        if self.idx >= self.items.len() { return None; }
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
        if self.idx >= self.items.len() { return None; }
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
        if self.idx >= self.items.len() { return None; }
        let itm = self.items[self.idx].clone();
        self.idx += 1;
        Some(itm)
    }
}