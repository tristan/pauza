#![windows_subsystem = "windows"]

use std::thread;
use std::time::{
    Duration,
    Instant
};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::{get_idle_time, start};

use crossbeam::channel::{
    unbounded,
    Sender
};

const IDLE_PAUSE_TIME: Duration = Duration::from_secs(60);
const IDLE_RESET_TIME: Duration = Duration::from_secs(300);
const BREAK_TIME: Duration = Duration::from_secs(2700);

#[derive(Debug)]
pub enum Event {
    UpdateTime(Duration),
    NotifyBreak,
    NotifyReset
}

fn monitor_idle_time(s: Sender<Event>) {
    let mut start = Instant::now();
    let mut has_reset: bool = false;
    let mut has_break: bool = false;
    s.send(Event::UpdateTime(start.elapsed())).unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
        match get_idle_time() {
            Ok(idle_time) if idle_time > IDLE_RESET_TIME => {
                if !has_reset {
                    s.send(Event::NotifyReset).unwrap();
                    s.send(Event::UpdateTime(Duration::from_secs(0))).unwrap();
                    has_reset = true;
                }
                start = Instant::now();
            },
            Ok(idle_time) if idle_time > IDLE_PAUSE_TIME => {},
            Ok(_idle_time) => {
                if has_reset {
                    start = Instant::now();
                    has_reset = false;
                    has_break = false;
                }
                s.send(Event::UpdateTime(start.elapsed())).unwrap();
                if start.elapsed() >= BREAK_TIME {
                    if !has_break {
                        s.send(Event::NotifyBreak).unwrap();
                        has_break = true;
                    }
                }
            },
            Err(_errno) => {
            }
        }
    }
}

fn main() {

    let (s, r) = unbounded();
    thread::spawn(|| monitor_idle_time(s));
    start(r);

}
