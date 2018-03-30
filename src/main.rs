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

const COUNT: &'static str = r###"
<span font='11'><tt>Tomatoros Completed: {}</tt></span>"###;

const TOTAL_TIME: &'static str = r###"
<span font='11'><tt>Total Time: {}</tt></span>"###;


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

// class Tomaty(Gtk.Window):
//     def __init__(self):
//         """init for main class Tomaty, runs tomaty app"""

//         # start button
//         self.tomatyButton = TomatyButton(tmargin=5, bmargin=5)
//         self.tomatyButton.connect("clicked", self.click_start)
//         self.timerPage.pack_start(self.tomatyButton, False, False, 0)
//         self.notebook.append_page(
//             child=self.timerPage, tab_label=Gtk.Label(label='tomatoro'))

//         # statistics page setup
//         self.statsPage = TomatyPage()
//         self.countLabel = StatsLabel(
//             label=COUNT.format(self.tomatosCompleted), smargin=10, emargin=10)

//         total = str(self.tomatoroLength * self.tomatosCompleted)
//         self.totalLabel = StatsLabel(
//             label=TOTAL_TIME.format(total),
//             emargin=25,
//             justify=Gtk.Justification.LEFT)

//         self.statsPage.pack_start(self.countLabel, False, False, 0)
//         self.statsPage.pack_start(self.totalLabel, False, False, 0)
//         self.notebook.append_page(
//             child=self.statsPage, tab_label=Gtk.Label(label="stats"))

//     def click_start(self, tomatyButton):
//         # begin counting!
//         if self.running is False:
//             self.running = True
//             self.tomatyButton.updateButton()
//             if self.breakPeriod is False:
//                 self.remTime = self.tomaTime
//                 GLib.timeout_add_seconds(1, self.countDown)
//             else:
//                 self.remTime = self.breakTime
//                 GLib.timeout_add_seconds(interval=1, function=self.countDown)
//         else:
//             self.running = False
//             self.tomatyButton.updateButton()
//             if self.breakPeriod is False:
//                 self.timerLabel.set_markup(str=TOMA_RESTART_MSG)
//                 self.remTime = self.tomaTime
//                 GLib.SOURCE_REMOVE
//             else:
//                 self.timerLabel.set_markup(str=BREAK_RESTART_MSG)
//                 self.remTime = self.breakTime
//                 GLib.SOURCE_REMOVE

//     def countDown(self):
//         # check to make sure countdown is not done if it is done, then we need
//         # to reset a lot of things before going forward
//         if self.remTime == timedelta(seconds=0):
//             alarm()
//             self.running = False
//             self.tomatyButton.updateButton()
//             if self.breakPeriod is False:
//                 self.tomatosCompleted += 1
//                 self.countLabel.set_markup(
//                     str=COUNT.format(self.tomatosCompleted))

//                 total = str(self.tomatoroLength * self.tomatosCompleted)
//                 self.totalLabel.set_markup(str=TOTAL_TIME.format(total))
//                 self.timerLabel.set_markup(str=TOMA_MSG)
//                 self.breakPeriod = True
//             else:
//                 self.timerLabel.set_markup(str=BREAK_MSG)
//                 self.breakPeriod = False

//             return GLib.SOURCE_REMOVE

//         if self.running is False:
//             return GLib.SOURCE_REMOVE

//         self.timerLabel.set_markup(str=TIMER_FRMT.format(self.tickTock()))
//         # signal to continue countdown within main loop
//         return GLib.SOURCE_CONTINUE

//     def tickTock(self):
//         # TODO: change to minutes format when done dev'ing
//         self.remTime = self.remTime - timedelta(seconds=1)

//         return str(self.remTime)[2:]

struct Tomaty {
    tomatos_completed: u64,
    running: bool,
    break_period: bool,
    toma_time: Duration,
    break_time: Duration,
    remaining_time: Duration,
    // tomatoro_length: Duration,
}

// fn current_time() -> String {
//     return format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
// }

// def click_start(self, tomatyButton):
//     # begin counting!
//     if self.running is False:
//         self.running = True
//         self.tomatyButton.updateButton()
//         if self.breakPeriod is False:
//             self.remTime = self.tomaTime
//             GLib.timeout_add_seconds(1, self.countDown)
//         else:
//             self.remTime = self.breakTime
//             GLib.timeout_add_seconds(interval=1, function=self.countDown)
//     else:
//         self.running = False
//         self.tomatyButton.updateButton()
//         if self.breakPeriod is False:
//             self.timerLabel.set_markup(str=TOMA_RESTART_MSG)
//             self.remTime = self.tomaTime
//             GLib.SOURCE_REMOVE
//         else:
//             self.timerLabel.set_markup(str=BREAK_RESTART_MSG)
//             self.remTime = self.breakTime
//             GLib.SOURCE_REMOVE

// tomaty: &Tomaty
fn click_start<'r>(button: &'r gtk::Button) {
}

fn make_window(tomaty: Rc<RefCell<Tomaty>>) -> gtk::Window {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("tomaty: gtk::Focus");
    window.set_border_width(5);
    window.set_resizable(false);
    
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    window.connect_delete_event(|_, _| {
        // panic!("lol");
        gtk::main_quit();
        Inhibit(false)
    });

    // create notebook, add as main and sole child widget of window
    let notebook = make_tomaty_notebook();
    window.add(&notebook);

    let timer_page = make_tomaty_page();
    let rem_time = format!(TIMER_FRMT!(), tomaty.borrow().remaining_time);
    let timer_label = make_label("");
    timer_label.set_markup(&rem_time);
    timer_page.pack_start(&timer_label, true, true, 0);

    let tomaty_button = make_button(5, 5);
    tomaty_button.connect_clicked(click_start);
    // tomaty_button.connect_clicked(clone!(tomaty => move |_| {
    //     // let nb = u32::from_str(counter_label.get_text()
    //     //                                    .unwrap_or("0".to_owned())
    //     //                                    .as_str()).unwrap_or(0);
    //     // if nb > 0 {
    //     //     counter_label.set_text(&format!("{}", nb - 1));
    //     // }
    //     println!("Button clicked!");
    // }));
    timer_page.pack_start(&tomaty_button, false, false, 0);

    let tab_label = make_label("tomatoro");
    notebook.append_page(&timer_page, Some(&tab_label));

    // let time = current_time();
    // let label = gtk::Label::new(None);
    // label.set_text(&time);
    // window.add(&label);

    // // we are using a closure to capture the label (else we could also use a normal function)
    // let tick = move || {
    //     let time = current_time();
    //     label.set_text(&time);
    //     // we could return gtk::Continue(false) to stop our clock after this tick
    //     gtk::Continue(true)
    // };

    // // executes the closure once every second
    // gtk::timeout_add_seconds(1, tick);

    window.show_all();
    window
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let tomaty = Rc::new(RefCell::new(Tomaty {
        tomatos_completed: 0,
        running: false,
        break_period: false,
        toma_time: Duration::minutes(20),
        break_time: Duration::minutes(5),
        remaining_time: Duration::minutes(0),
        // tomatoro_length: Duration::minutes(25),
    }));

    // let tomaty = Tomaty {
    //     tomatos_completed: 0,
    //     running: false,
    //     break_period: false,
    //     toma_time: Duration::minutes(20),
    //     break_time: Duration::minutes(5),
    //     remaining_time: Duration::minutes(0),
    //     // tomatoro_length: Duration::minutes(25),
    // };

    make_window(tomaty.clone());
    gtk::main();
}
