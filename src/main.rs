#![crate_type = "bin"]

extern crate gtk;
extern crate pango;
extern crate chrono;

use chrono::Duration;
use chrono::Local;

use std::{thread, time};
use std::io::BufReader;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk::Builder;
use Continue;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

const TOMA_MINUTES: u8 = 25;
const BREAK_MINUTES: u8 = 5;

// {() => ("hello")};
macro_rules! TIMER_FRMT {() => (r###"
<span font='34'>{}</span>
"###)}

// const TIMER_FRMT: &'static str = r###"
// <span font='34'>{}</span>
// "###;

const TOMA_MSG: &'static str = r###"
<span font='16'>Tomatoro Done!\nStart Break?</span>"###;

const BREAK_MSG: &'static str = r###"
<span font='16'>Break Over!\nStart Tomatoro?</span>"###;

const TOMA_RESTART_MSG: &'static str = r###"
<span font='16'>Start Tomatoro?</span>"###;

const BREAK_RESTART_MSG: &'static str = r###"
<span font='16'>Start Break?</span>"###;

// const COUNT: &'static str = r###"
// <span font='11'><tt>Tomatoros Completed: {}</tt></span>"###;

macro_rules! COUNT {() => (r###"
<span font='11'><tt>Tomatoros Completed: {}</tt></span>"###)}

macro_rules! TOTAL_TIME {() => (r###"
<span font='11'><tt>Total Time: {}</tt></span>"###)}

// const TOTAL_TIME: &'static str = r###"
// <span font='11'><tt>Total Time: {}</tt></span>"###;


// def alarm():
//     # really need to find a cleaner, non-hack, way of getting to resources/
//     resourcePath = path.join(path.split(__file__)[0], 'resources')
//     alarmPath = path.join(path.join(resourcePath, 'audio'), 'alarm.wav')
//     wav_obj = WaveObject.from_wave_file(alarmPath)
//     wav_obj.play()


fn make_label(label: &str) -> gtk::Label {
    let new_label = gtk::Label::new(label);
    // new_label.set_markup(label);
    new_label.set_margin_start(0);
    new_label.set_margin_end(0);
    new_label.set_margin_top(0);
    new_label.set_margin_bottom(0);
    new_label.set_justify(gtk::Justification::Center);
    return new_label
}

// fn make_stats_label(label: &str) -> gtk::Label {
//     let new_label = gtk::Label::new(label);
//     // new_label.set_markup(label);
//     new_label.set_margin_start(0);
//     new_label.set_margin_end(0);
//     new_label.set_margin_top(0);
//     new_label.set_margin_bottom(0);
//     new_label.set_justify(gtk::Justification::Center);
//     return new_label
// }

fn make_tomaty_notebook() -> gtk::Notebook {
    let new_notebook = gtk::Notebook::new();
    new_notebook.set_size_request(250, 150);
    return new_notebook
}

fn make_tomaty_page() -> gtk::Box {
    let new_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    // new_box.set_orientation(gtk::Orientation::Vertical);
    // new_box.set_spacing(0);
    new_box.set_homogeneous(false);
    return new_box
}

//     def updateButton(self):
//         if self.get_label() == "start":
//             self.set_label("restart")
//         else:
//             self.set_label("start")

fn make_button(margin_top: i32, margin_bottom: i32) -> gtk::Button {
    let new_button = gtk::Button::new();
    new_button.set_label("start");
    new_button.set_margin_start(0);
    new_button.set_margin_end(0);
    new_button.set_margin_top(margin_top);
    new_button.set_margin_bottom(margin_bottom);
    new_button.set_halign(gtk::Align::Center);
    return new_button
}

struct Tomaty {
    tomatos_completed: i32,
    running: bool,
    break_period: bool,
    toma_time: Duration,
    break_time: Duration,
    remaining_time: Duration,
    tomatoro_length: Duration,
    tomaty_button: gtk::Button,
    timer_label: gtk::Label,
    count_label: gtk::Label,
    total_label: gtk::Label,
}

// fn current_time() -> String {
//     return format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
// }

// <'r> button: &'r gtk::Button
fn connect_click_start(tomaty: Rc<RefCell<Tomaty>>) {
    let outer_tomato_heaven = tomaty.clone();
    let ref button = outer_tomato_heaven.borrow().tomaty_button;

    button.connect_clicked(move |cb_button: &gtk::Button| {
        let mut tomtom = tomaty.borrow_mut();
        if tomtom.running {
            tomtom.running = false;
            update_button(&cb_button);
            if tomtom.break_period {
                tomtom.timer_label.set_markup(BREAK_RESTART_MSG);
                tomtom.remaining_time = tomtom.break_time;
                //             GLib.SOURCE_REMOVE
            } else {
                tomtom.timer_label.set_markup(TOMA_RESTART_MSG);
                tomtom.remaining_time = tomtom.toma_time;
                //             GLib.SOURCE_REMOVE
            };
        } else {
            tomtom.running = true;
            update_button(&cb_button);
            if tomtom.break_period {
                tomtom.remaining_time = tomtom.break_time;
                let timer_formatted = format!(TIMER_FRMT!(), format!("{}", tomtom.remaining_time));
                tomtom.timer_label.set_markup(&timer_formatted);

                add_timeout_countdown(tomaty.clone());
            } else {
                tomtom.remaining_time = tomtom.toma_time;
                let timer_formatted = format!(TIMER_FRMT!(), format!("{}", tomtom.remaining_time));
                tomtom.timer_label.set_markup(&timer_formatted);

                add_timeout_countdown(tomaty.clone());
            };
        };
        println!("Button clicked!");
    });

}

fn alarm() {
    println!("WAKE UP FUCKO");
}

fn tick_tock(tomaty: &mut Tomaty) -> String {
    tomaty.remaining_time = tomaty.remaining_time - Duration::seconds(1);
    return format!("{}", tomaty.remaining_time)
}

// -> gtk::Continue
fn add_timeout_countdown(tomaty: Rc<RefCell<Tomaty>>) {
    gtk::timeout_add_seconds(1, move || {
        let mut tomtom = tomaty.borrow_mut();
        if tomtom.remaining_time == Duration::seconds(0) {
            alarm();
            tomtom.running = false;
            update_button(&tomtom.tomaty_button);
            if tomtom.break_period {
                tomtom.timer_label.set_markup(BREAK_MSG);
                tomtom.break_period = false;
            } else {
                tomtom.tomatos_completed += 1;
                let count_formatted =
                    format!(COUNT!(), tomtom.tomatos_completed);
                tomtom.count_label.set_markup(&count_formatted);
                let total = tomtom.tomatoro_length * tomtom.tomatos_completed;
                let total_formatted =
                    format!(TOTAL_TIME!(), total);
                tomtom.total_label.set_markup(&total_formatted);
                tomtom.timer_label.set_markup(TOMA_MSG);
                tomtom.break_period = true;
            }
            return gtk::Continue(false)
        }
        if !tomtom.running {
            return gtk::Continue(false)
        }
        let timer_formatted = format!(TIMER_FRMT!(), tick_tock(&mut tomtom));
        tomtom.timer_label.set_markup(&timer_formatted);
        return gtk::Continue(true)
    });
}

fn update_button(button: &gtk::Button) {
    match button.get_label().as_ref().map(String::as_ref) {
        Some("start") => button.set_label("restart"),
        _ => button.set_label("start"),
    }
}

// tomaty: Rc<RefCell<Tomaty>>
fn make_window() -> gtk::Window {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("tomaty: gtk::Focus");
    window.set_border_width(5);
    window.set_resizable(false);

    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // create notebook, add as main and sole child widget of window
    let notebook = make_tomaty_notebook();
    window.add(&notebook);

    let timer_page = make_tomaty_page();
    let remaining_default = Duration::minutes(0);
    let rem_time = format!(TIMER_FRMT!(), remaining_default);
    let timer_label = make_label("");
    timer_label.set_markup(&rem_time);
    timer_page.pack_start(&timer_label, true, true, 0);

    let tomaty_button = make_button(5, 5);
    // tomaty_button.connect_clicked(click_start);
    timer_page.pack_start(&tomaty_button, false, false, 0);

    let tab_label = make_label("tomatoro");
    notebook.append_page(&timer_page, Some(&tab_label));

    let stats_page = make_tomaty_page();
    let tomatos_completed_default = 0;
    let count_label_formatted =
        format!(COUNT!(), tomatos_completed_default);
    let count_label = make_label("");
    count_label.set_markup(&count_label_formatted);
    count_label.set_margin_start(10);
    count_label.set_margin_end(10);

    let tomatoro_length_default = Duration::minutes(25);
    let total = tomatoro_length_default * tomatos_completed_default;
    let total_formatted=
        format!(TOTAL_TIME!(), total);

    let total_label = make_label("");
    total_label.set_markup(&total_formatted);
    total_label.set_margin_end(25);
    total_label.set_justify(gtk::Justification::Left);

    stats_page.pack_start(&count_label, false, false, 0);
    stats_page.pack_start(&total_label, false, false, 0);

    let stats_tab_label = make_label("stats");
    notebook.append_page(&stats_page, Some(&stats_tab_label));

    let tomaty = Rc::new(RefCell::new(Tomaty {
        tomatos_completed: tomatos_completed_default.clone(),
        running: false,
        break_period: false,
        toma_time: Duration::minutes(20),
        break_time: Duration::minutes(5),
        remaining_time: remaining_default.clone(),
        tomatoro_length: tomatoro_length_default.clone(),
        tomaty_button: tomaty_button,
        timer_label: timer_label,
        count_label: count_label,
        total_label: total_label,
    }));

    connect_click_start(tomaty.clone());
    window.show_all();
    window
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    make_window();
    gtk::main();
}
