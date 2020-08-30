use crate::components::list;
// use std::rc::*;
// use std::cell::*;
use chrono::prelude::*;
use std::sync::{ Mutex, Weak };




#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RepeatSpan {
    Days(i32),
    Weeks(i32),
    Months(i32),
    Years(i32),
    // PerDay,
    // PerWeeks,
    // PerMonth,
    // PerYear,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Marker(pub i32);

// pub struct Color {
//     r: u8,
//     g: u8,
//     b: u8,
//     a: u8
// }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ItemStyle {
    pub marker: Marker,
    // color: Color
}

#[derive(Debug)]
pub struct Item {
    id: u32,
    message: String,
    level: i8,
    style: ItemStyle,
    today: bool,
    notice: Option<NaiveDateTime>,
    deadline: Option<NaiveDate>,
    plan: Option<NaiveDate>,
    repeat: Option<RepeatSpan>,
    list: Option<Weak<Mutex<list::List>>>,
    finished: bool,
    note: String,
}

impl Item {
    pub fn new(id: u32, message: &str, level: i8, style: ItemStyle, today: bool, 
        notice: Option<NaiveDateTime>, deadline: Option<NaiveDate>, 
        plan: Option<NaiveDate>, repeat: Option<RepeatSpan>, list: Option<Weak<Mutex<list::List>>>) -> Self {
            Self {
                id, message: String::from(message), level, style, today, notice, deadline, plan, repeat, list, 
                finished: false, note: String::new()
            }
        }

    pub fn id(&self) -> u32 { self.id }
    
    pub fn message(&self) -> &str { self.message.as_ref() }
    pub fn set_message(&mut self, message: String) -> &mut Self { self.message = message; self }
    
    pub fn level(&self) -> i8 { self.level }
    pub fn set_level(&mut self, level: i8) -> &mut Self { self.level = level; self  }
    pub fn level_ref(&self) -> &i8 { &self.level }
    pub fn level_mut(&mut self) -> &mut i8 { &mut self.level }

    pub fn style(&self) -> ItemStyle { self.style }
    pub fn set_style(&mut self, style: ItemStyle) -> &mut Self { self.style = style; self }
    pub fn style_ref(&self) -> &ItemStyle { &self.style }
    pub fn style_mut(&mut self) -> &mut ItemStyle { &mut self.style }

    pub fn today(&self) -> bool { self.today }
    pub fn set_today(&mut self, today: bool) -> &mut Self { self.today = today; self }
    pub fn today_ref(&self) -> &bool { &self.today }
    pub fn today_mut(&mut self) -> &mut bool { &mut self.today }

    pub fn notice(&self) -> Option<NaiveDateTime> { self.notice }
    pub fn set_notice(&mut self, notice: Option<NaiveDateTime>) -> &mut Self { self.notice = notice; self }
    pub fn notice_ref(&self) -> &Option<NaiveDateTime> { &self.notice }
    pub fn notice_mut(&mut self) -> &mut Option<NaiveDateTime> { &mut self.notice }

    pub fn deadline(&self) -> Option<NaiveDate> { self.deadline }
    pub fn set_deadline(&mut self, deadline: Option<NaiveDate>) -> &mut Self { self.deadline = deadline; self }
    pub fn deadline_ref(&self) -> &Option<NaiveDate> { &self.deadline }
    pub fn deadline_mut(&mut self) -> &mut Option<NaiveDate> { &mut self.deadline }

    pub fn plan(&self) -> Option<NaiveDate> { self.plan }
    pub fn set_plan(&mut self, plan: Option<NaiveDate>) -> &mut Self { self.plan = plan; self }
    pub fn plan_ref(&self) -> &Option<NaiveDate> { &self.plan }
    pub fn plan_mut(&mut self) -> &mut Option<NaiveDate> { &mut self.plan }

    pub fn repeat(&self) -> Option<RepeatSpan> { self.repeat }
    pub fn set_repeat(&mut self, repeat: Option<RepeatSpan>) -> &mut Self { self.repeat = repeat; self }
    pub fn repeat_ref(&self) -> &Option<RepeatSpan> { &self.repeat }
    pub fn repeat_mut(&mut self) -> &mut Option<RepeatSpan> { &mut self.repeat }

    pub fn list(&self) -> Option<Weak<Mutex<list::List>>> { self.list.clone() }
    pub fn set_list(&mut self, list: Option<Weak<Mutex<list::List>>>) -> &mut Self { self.list = list; self }
    pub fn list_ref(&self) -> &Option<Weak<Mutex<list::List>>> { &self.list }
    pub fn list_mut(&mut self) -> &mut Option<Weak<Mutex<list::List>>> { &mut self.list }

    pub fn finished(&self) -> bool { self.finished }
    pub fn set_finished(&mut self, finished: bool) -> &mut Self { self.finished = finished; self }
    pub fn finished_ref(&self) -> &bool { &self.finished }
    pub fn finished_mut(&mut self) -> &mut bool { &mut self.finished }

    pub fn note(&self) -> &str { self.note.as_ref() }
    pub fn set_note(&mut self, note: String) -> &mut Self { self.note = note; self }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Item {}