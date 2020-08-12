// use orbtk::prelude::*;

pub use self::main_state::*;
pub use self::main_view::*;
use ::todolst::components::*;
use chrono::prelude::*;
// use std::thread;
use std::time;

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
    let mut system = todolst::TodoLst::new();
    let now = Local::now();
    let now_std = time::Instant::now();
    let list1 = system.new_list("list1").unwrap();
    let item1 = system.new_item("item1", list1);
    system.set_item_notice(&item1, Some(now.naive_local() + chrono::Duration::seconds(5)))
        .set_item_note(&item1, "Hello!");
    let end_std = now_std + time::Duration::from_secs(10);
    while time::Instant::now() < end_std {
        system.update();
    }
    println!("Application finished.");
}
