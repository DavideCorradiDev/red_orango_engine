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

// TODO: add docu online

impl ApplicationState<ApplicationError, ApplicationEvent> for StateA {
    fn on_start(&mut self) -> Result<(), ApplicationError> {
        println!("State A - Initialized. Right: Change to state B, Up: push state L, Down: pop, X: exit.");
        Ok(())
    }

    fn on_end(&mut self) -> Result<(), ApplicationError> {
        println!("State A - Removed.");
        Ok(())
    }

    fn on_paused(&mut self) -> Result<(), ApplicationError> {
        println!("State A - Paused.");
        Ok(())
    }

    fn on_unpaused(&mut self) -> Result<(), ApplicationError> {
        println!("State A - Unpaused.");
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
                    os::KeyCode::X => self.control_flow = ControlFlow::Exit,
                    os::KeyCode::Down => self.control_flow = ControlFlow::PopState,
                    os::KeyCode::Up => {
                        self.control_flow =
                            ControlFlow::PushState(Box::new(StateL::new(Rc::clone(&self.app_data))))
                    }
                    os::KeyCode::Right => {
                        self.control_flow = ControlFlow::PopPushState(Box::new(StateB::new(
                            Rc::clone(&self.app_data),
                        )))
                    }
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

struct StateB {
    app_data: Rc<ApplicationData>,
    control_flow: ControlFlow<ApplicationError, ApplicationEvent>,
}

impl StateB {
    fn new(app_data: Rc<ApplicationData>) -> Self {
        Self {
            app_data,
            control_flow: ControlFlow::Continue,
        }
    }
}

impl ApplicationState<ApplicationError, ApplicationEvent> for StateB {
    fn on_start(&mut self) -> Result<(), ApplicationError> {
        println!("State B - Initialized. Left: Change to state A, Right: Change to state C, Up: push state L, Down: pop, X: exit.");
        Ok(())
    }

    fn on_end(&mut self) -> Result<(), ApplicationError> {
        println!("State B - Removed.");
        Ok(())
    }

    fn on_paused(&mut self) -> Result<(), ApplicationError> {
        println!("State B - Paused.");
        Ok(())
    }

    fn on_unpaused(&mut self) -> Result<(), ApplicationError> {
        println!("State B - Unpaused.");
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
                    os::KeyCode::X => self.control_flow = ControlFlow::Exit,
                    os::KeyCode::Down => self.control_flow = ControlFlow::PopState,
                    os::KeyCode::Up => {
                        self.control_flow =
                            ControlFlow::PushState(Box::new(StateL::new(Rc::clone(&self.app_data))))
                    }
                    os::KeyCode::Left => {
                        self.control_flow = ControlFlow::PopPushState(Box::new(StateA::new(
                            Rc::clone(&self.app_data),
                        )))
                    }
                    os::KeyCode::Right => {
                        self.control_flow = ControlFlow::PopPushState(Box::new(StateC::new(
                            Rc::clone(&self.app_data),
                        )))
                    }
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

struct StateC {
    app_data: Rc<ApplicationData>,
    control_flow: ControlFlow<ApplicationError, ApplicationEvent>,
}

impl StateC {
    fn new(app_data: Rc<ApplicationData>) -> Self {
        Self {
            app_data,
            control_flow: ControlFlow::Continue,
        }
    }
}

impl ApplicationState<ApplicationError, ApplicationEvent> for StateC {
    fn on_start(&mut self) -> Result<(), ApplicationError> {
        println!(
            "State C - Initialized. Left: Change to state B, Up: push state L, Down: pop, X: exit."
        );
        Ok(())
    }

    fn on_end(&mut self) -> Result<(), ApplicationError> {
        println!("State C - Removed.");
        Ok(())
    }

    fn on_paused(&mut self) -> Result<(), ApplicationError> {
        println!("State C - Paused.");
        Ok(())
    }

    fn on_unpaused(&mut self) -> Result<(), ApplicationError> {
        println!("State C - Unpaused.");
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
                    os::KeyCode::X => self.control_flow = ControlFlow::Exit,
                    os::KeyCode::Down => self.control_flow = ControlFlow::PopState,
                    os::KeyCode::Up => {
                        self.control_flow =
                            ControlFlow::PushState(Box::new(StateL::new(Rc::clone(&self.app_data))))
                    }
                    os::KeyCode::Left => {
                        self.control_flow = ControlFlow::PopPushState(Box::new(StateB::new(
                            Rc::clone(&self.app_data),
                        )))
                    }
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

struct StateL {
    app_data: Rc<ApplicationData>,
    control_flow: ControlFlow<ApplicationError, ApplicationEvent>,
}

impl StateL {
    fn new(app_data: Rc<ApplicationData>) -> Self {
        Self {
            app_data,
            control_flow: ControlFlow::Continue,
        }
    }
}

impl ApplicationState<ApplicationError, ApplicationEvent> for StateL {
    fn on_start(&mut self) -> Result<(), ApplicationError> {
        println!("State L - Initialized. Right: Change to state M, Down: pop, X: exit.");
        Ok(())
    }

    fn on_end(&mut self) -> Result<(), ApplicationError> {
        println!("State L - Removed.");
        Ok(())
    }

    fn on_paused(&mut self) -> Result<(), ApplicationError> {
        println!("State L - Paused.");
        Ok(())
    }

    fn on_unpaused(&mut self) -> Result<(), ApplicationError> {
        println!("State L - Unpaused.");
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
                    os::KeyCode::X => self.control_flow = ControlFlow::Exit,
                    os::KeyCode::Down => self.control_flow = ControlFlow::PopState,
                    os::KeyCode::Right => {
                        self.control_flow = ControlFlow::PopPushState(Box::new(StateM::new(
                            Rc::clone(&self.app_data),
                        )))
                    }
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

struct StateM {
    app_data: Rc<ApplicationData>,
    control_flow: ControlFlow<ApplicationError, ApplicationEvent>,
}

impl StateM {
    fn new(app_data: Rc<ApplicationData>) -> Self {
        Self {
            app_data,
            control_flow: ControlFlow::Continue,
        }
    }
}

impl ApplicationState<ApplicationError, ApplicationEvent> for StateM {
    fn on_start(&mut self) -> Result<(), ApplicationError> {
        println!("State M - Initialized. Left: Change to state L, Down: pop, X: exit.");
        Ok(())
    }

    fn on_end(&mut self) -> Result<(), ApplicationError> {
        println!("State M - Removed.");
        Ok(())
    }

    fn on_paused(&mut self) -> Result<(), ApplicationError> {
        println!("State M - Paused.");
        Ok(())
    }

    fn on_unpaused(&mut self) -> Result<(), ApplicationError> {
        println!("State M - Unpaused.");
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
                    os::KeyCode::X => self.control_flow = ControlFlow::Exit,
                    os::KeyCode::Down => self.control_flow = ControlFlow::PopState,
                    os::KeyCode::Left => {
                        self.control_flow = ControlFlow::PopPushState(Box::new(StateL::new(
                            Rc::clone(&self.app_data),
                        )))
                    }
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
