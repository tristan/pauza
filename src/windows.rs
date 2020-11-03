use winapi::um::winuser::{
    GetLastInputInfo,
    LASTINPUTINFO,
};
use winapi::um::sysinfoapi::GetTickCount;
use std::mem::{zeroed, size_of};
use std::time::Duration;

pub fn get_idle_time() -> Result<Duration, i32> {
    let mut info: LASTINPUTINFO = unsafe { zeroed() };
    info.cbSize = size_of::<LASTINPUTINFO>() as u32;
    let result = unsafe { GetLastInputInfo(&mut info) };
    if result == 0 {
        Err(result)
    } else {
        let tick_count = unsafe { GetTickCount() };
        let elapsed_millis = tick_count - info.dwTime;
        let duration = Duration::from_millis(elapsed_millis as _);
        Ok(duration)
    }
}

// WINDOWS GUI

use nwg::NativeUi;

#[derive(Default)]
pub struct BasicApp {
    window: nwg::Window,
    label: nwg::Label,
    icon: nwg::Icon,
    tray: nwg::TrayNotification,
}

impl BasicApp {
    fn reset_notification(&self) {
        let flags = nwg::TrayNotificationFlags::USER_ICON
            | nwg::TrayNotificationFlags::LARGE_ICON;
        self.tray.show("Back to work!",
                       Some("Get back to work"),
                       Some(flags),
                       Some(&self.icon));
    }

    fn break_notification(&self) {
        let flags = nwg::TrayNotificationFlags::USER_ICON
            | nwg::TrayNotificationFlags::LARGE_ICON;
        self.tray.show("Break Time!",
                       Some("Time to take a break!"),
                       Some(flags),
                       Some(&self.icon));
    }
}

mod basic_app_ui {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::ops::Deref;

    pub struct BasicAppUi {
        inner: Rc<BasicApp>,
        default_handler: RefCell<Option<nwg::EventHandler>>
    }

    impl nwg::NativeUi<BasicAppUi> for BasicApp {
        fn build_ui(mut data: BasicApp) -> Result<BasicAppUi, nwg::NwgError> {
            use nwg::Event as E;

            nwg::Icon::builder()
                .source_file(Some("./pauza.ico"))
                .build(&mut data.icon)?;

            // Controls
            nwg::Window::builder()
                .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
                .size((300, 115))
                .position((300, 300))
                .title("Pauza")
                .build(&mut data.window)?;

            nwg::Label::builder()
                .text("Starting...")
                //.font(Some(font))
                .parent(&data.window)
                .build(&mut data.label)?;

            nwg::TrayNotification::builder()
                .parent(&data.window)
                .icon(Some(&data.icon))
                .tip(Some("Pauza"))
                .build(&mut data.tray)?;

            // Wrap-up
            let ui = BasicAppUi {
                inner: Rc::new(data),
                default_handler: Default::default(),
            };

            // Events
            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnWindowClose => if &handle == &ui.window {
                            nwg::stop_thread_dispatch();
                        },
                        _ => {}
                    }
                }
            };

            *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(&ui.window.handle, handle_events));

            return Ok(ui);
        }
    }

    impl Drop for BasicAppUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let handler = self.default_handler.borrow();
            if handler.is_some() {
                nwg::unbind_event_handler(handler.as_ref().unwrap());
            }
        }
    }

    impl Deref for BasicAppUi {
        type Target = BasicApp;

        fn deref(&self) -> &BasicApp {
            &self.inner
        }
    }
}

pub fn start(r: crossbeam::channel::Receiver<crate::Event>) {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let ui = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events_with_callback(move || {
        match r.try_recv() {
            Ok(event) => match event {
                crate::Event::UpdateTime(duration) => {
                    let text = format!("{:?}", duration);
                    ui.label.set_text(&text);
                },
                crate::Event::NotifyReset => {
                    ui.reset_notification();
                },
                crate::Event::NotifyBreak => {
                    ui.break_notification();
                }
            },
            Err(crossbeam::channel::TryRecvError::Empty) => {},
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
    });
}
