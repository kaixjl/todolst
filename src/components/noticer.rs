use std::thread;
use std::time;
use chrono::prelude::*;
// use std::cell::*;
use std::sync::*;
use std::sync::atomic::*;
use std::collections::{ BTreeSet };

pub struct Noticer {
    datetimes: Arc<Mutex<BTreeSet<NaiveDateTime>>>,
    callees: Arc<Mutex<Vec<Box<dyn Fn(NaiveDateTime) + Send>>>>,
    running: Arc<AtomicBool>,
    th: Option<thread::JoinHandle<()>>,
}

impl Noticer {
    pub fn new() -> Self {
        Self {
            datetimes: Arc::new(Mutex::new(BTreeSet::new())),
            callees: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            th: None,
        }
    }

    pub fn start(&mut self) {

        if self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(true, Ordering::Relaxed);
        
        let datetimes = self.datetimes.clone();
        let callees = self.callees.clone();
        let running = self.running.clone();
        self.th = Some(thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let now = Local::now().naive_local();
                let datetimes = datetimes.lock();
                match datetimes {
                    Err(_) => (),
                    Ok(mut datetimes) => {
                        let mut datetimes_to_remove: Vec<NaiveDateTime> = Vec::new();
                        for &datetime in datetimes.iter() {
                            if now > datetime {
                                let callees = callees.lock();
                                match callees {
                                    Err(_) => (),
                                    Ok(callees) => {
                                        for callee in callees.iter() {
                                            callee(datetime);
                                        }
                                    }
                                }
                                datetimes_to_remove.push(datetime);
                            } else {
                                break;
                            }
                        }
                        for datetime in datetimes_to_remove.into_iter() {
                            datetimes.remove(&datetime);
                        }
                    }
                }

                thread::sleep(time::Duration::from_micros(500));
            }
        }));
    }

    pub fn stop(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(false, Ordering::Relaxed);
        let th = self.th.take();
        if let Some(th) = th {
            th.join().unwrap_or_default();
        }
    }

    pub fn add_notice(&self, datetime: NaiveDateTime) -> bool {
        let datetimes = self.datetimes.lock();
        match datetimes {
            Err(_) => false,
            Ok(mut datetimes) => {
                datetimes.insert(datetime)
            }
        }
    }

    pub fn remove_notice(&self, datetime: &NaiveDateTime) -> bool {
        let datetimes = self.datetimes.lock();
        match datetimes {
            Err(_) => false,
            Ok(mut datetimes) => {
                datetimes.remove(datetime)
            }
        }
    }

    pub fn contains_notice(&self, datetime: &NaiveDateTime) -> bool {
        let datetimes = self.datetimes.lock();
        match datetimes {
            Err(_) => false,
            Ok(datetimes) => {
                datetimes.contains(datetime)
            }
        }
    }

    pub fn add_callback<T: Fn(NaiveDateTime) + Send + 'static>(&self, callback: T) {
        let callees = self.callees.lock();
        match callees {
            Err(_) => (),
            Ok(mut callees) => {
                callees.push(Box::new(callback));
            }
        }
    }
}