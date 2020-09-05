// use orbtk::prelude::*;
extern crate chrono;

pub use self::main_state::*;
pub use self::main_view::*;
use ::todolst::components::*;
use chrono::prelude::*;
use std::thread;
use std::time;
use futures::executor::block_on;

mod main_state;
mod main_view;


fn main() {
    // Application::from_name("todolst")
    //     .window(move |ctx| {
    //         Window::create()
    //             .title("todolst")
    //             .position((100.0, 100.0))
    //             .size(372.0, 768.0)
    //             .resizeable(true)
    //             .child(MainView::create().title("Hello OrbTk").build(ctx))
    //             .build(ctx)
    //     })
    //     .run();
    
    {
        let mut system = todolst::TodoLst::new();
        let now = Local::now();
        // let now_std = time::Instant::now();
        let list1 = system.new_list("list1").unwrap();
        let item1 = system.new_item("item1", list1);
        system.set_item_notice(&item1, Some(now.naive_local() + chrono::Duration::seconds(5))).unwrap()
            .set_item_note(&item1, "Hello!").unwrap();
        block_on(system.save()).unwrap();
        // let end_std = now_std + time::Duration::from_secs(10);
        // while time::Instant::now() < end_std {
        //     system.update();
        // }

        thread::sleep(time::Duration::from_secs(10));
    }

    // let mut system = block_on(todolst::TodoLst::load());
    // println!("Groups:");
    // for group in system.iter_groups() {
    //     let group = group.upgrade().unwrap();
    //     let group = group.lock().unwrap();
    //     println!("{:?}", group);
    // }
    // println!("Lists:");
    // for list in system.iter_lists() {
    //     let list = list.upgrade().unwrap();
    //     let list = list.lock().unwrap();
    //     println!("{:?}", list);
    // }
    // println!("items:");
    // for item in system.iter_items() {
    //     let item = item.upgrade().unwrap();
    //     let item = item.lock().unwrap();
    //     println!("{:?}", item);
    // }

    println!("Application finished.");
}
