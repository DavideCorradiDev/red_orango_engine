use roe_examples::*;

use roe_app::{Application, ApplicationState, ControlFlow};

use roe_os as os;

use std::rc::Rc;

struct ApplicationData {
    window: os::Window,
}

impl ApplicationData {
    fn new(event_loop: &os::EventLoop<ApplicationEvent>) -> Result<Self, ApplicationError> {
        let window = os::WindowBuilder::new()
            .with_title("Application State")
            .with_inner_size(os::Size::Physical(os::PhysicalSize {
                width: 800,
                height: 600,
            }))
            .build(event_loop)?;
        Ok(Self { window })
    }
}

struct StateA {
    app_data: Rc<ApplicationData>,
    control_flow: ControlFlow<ApplicationError, ApplicationEvent>,
}

impl StateA {
    fn new(app_data: Rc<ApplicationData>) -> Self {
        Self {
            app_data,
            control_flow: ControlFlow::Continue,
        }
    }
}

// TODO: Add state init event.
// TODO: invalid scan code when pressing numpad enter?
// TODO: add docu online

impl ApplicationState<ApplicationError, ApplicationEvent> for StateA {
    fn on_start(&mut self) -> Result<(), ApplicationError> {
        println!("State A - Initialized. Right: Change to state B, Up: push state L, Down: pop.");
        Ok(())
    }

    fn on_end(&mut self) -> Result<(), ApplicationError> {
        println!("State A - Removed.");
        Ok(())
    }

    fn on_key_pressed(
        &mut self,
        wid: os::WindowId,
        _device_id: os::DeviceId,
        _scan_code: os::ScanCode,
        key_code: Option<os::KeyCode>,
        _is_synthetic: bool,
        is_repeat: bool,
    ) -> Result<(), ApplicationError> {
        if !is_repeat && wid == self.app_data.window.id() {
            if let Some(key_code) = key_code {
                match key_code {
                    os::KeyCode::Down => self.control_flow = ControlFlow::PopState,
                    _ => println!("Invalid key"),
                }
            }
        }
        Ok(())
    }

    fn requested_control_flow(&mut self) -> ControlFlow<ApplicationError, ()> {
        let mut control_flow = ControlFlow::Continue;
        std::mem::swap(&mut control_flow, &mut self.control_flow);
        control_flow
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP)).run(|event_queue| {
        Ok(Box::new(StateA::new(Rc::new(ApplicationData::new(
            event_queue,
        )?))))
    });
}
