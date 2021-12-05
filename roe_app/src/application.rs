use super::{ApplicationState, ApplicationStateFlow};

use roe_os as os;

use std::{collections::BTreeMap, ops::DerefMut};

pub struct Application<ErrorType, CustomEventType>
where
    ErrorType: std::fmt::Display + std::error::Error + 'static,
    CustomEventType: 'static,
{
    keyboard_state: KeyboardState,
    fixed_update_period: std::time::Duration,
    variable_update_min_period: std::time::Duration,
    last_fixed_update_time: std::time::Instant,
    last_variable_update_time: std::time::Instant,
    state_stack: Vec<Box<dyn ApplicationState<ErrorType, CustomEventType>>>,
}

impl<ErrorType, CustomEventType> Application<ErrorType, CustomEventType>
where
    ErrorType: std::fmt::Display + std::error::Error + 'static,
    CustomEventType: 'static,
{
    pub fn new(
        fixed_update_frequency_hz: u64,
        variable_update_max_frequency_hz: Option<u64>,
    ) -> Self {
        assert!(
            fixed_update_frequency_hz > 0,
            "The fixed update frequency must be higher than 0"
        );
        if let Some(v) = variable_update_max_frequency_hz {
            assert!(v > 0, "The variable update frequency must be higher than 0");
        }

        let fixed_update_period =
            std::time::Duration::from_secs_f64(1. / fixed_update_frequency_hz as f64);
        let variable_update_min_period = match variable_update_max_frequency_hz {
            Some(v) => std::time::Duration::from_secs_f64(1. / v as f64),
            None => std::time::Duration::from_secs_f64(0.),
        };
        let current_time = std::time::Instant::now();

        Self {
            keyboard_state: KeyboardState::new(),
            fixed_update_period,
            variable_update_min_period,
            last_fixed_update_time: current_time,
            last_variable_update_time: current_time,
            state_stack: Vec::new(),
        }
    }

    #[cfg(test)]
    fn create_event_loop() -> os::EventLoop<CustomEventType> {
        use os::EventLoopAnyThread;
        os::EventLoop::<CustomEventType>::new_any_thread()
    }

    #[cfg(not(test))]
    fn create_event_loop() -> os::EventLoop<CustomEventType> {
        os::EventLoop::<CustomEventType>::with_user_event()
    }

    fn default_error_handler<E: std::fmt::Display>(error: E) {
        eprintln!("The application shut down due to an error ({})", error);
    }

    pub fn run(
        mut self,
        initialization_fn: fn(
            &os::EventLoop<CustomEventType>,
        ) -> Result<
            Box<dyn ApplicationState<ErrorType, CustomEventType>>,
            ErrorType,
        >,
    ) {
        let event_loop = Self::create_event_loop();
        self.state_stack.push(
            initialization_fn(&event_loop)
                .expect("Failed to initialize the application initial state"),
        );

        let current_time = std::time::Instant::now();
        self.last_fixed_update_time = current_time;
        self.last_variable_update_time = current_time;

        // TODO: remove the custom ControlFlow enum.
        event_loop.run(
            move |event, _, control_flow| match self.handle_event(event) {
                Ok(flow) => *control_flow = flow,
                Err(e) => {
                    match self.state_stack.last_mut() {
                        Some(state) => {
                            let state = state.deref_mut();
                            if !state.handle_error(&e) {
                                Self::default_error_handler(e);
                            }
                        }
                        None => {
                            Self::default_error_handler(e);
                        }
                    }
                    *control_flow = os::ControlFlow::Exit;
                }
            },
        );
    }

    fn handle_event(
        &mut self,
        event: os::Event<CustomEventType>,
    ) -> Result<os::ControlFlow, ErrorType> {
        let mut application_state_flow =
            ApplicationStateFlow::<ErrorType, CustomEventType>::Continue;

        match self.state_stack.last_mut() {
            Some(state) => {
                let state = state.deref_mut();
                match event {
                    os::Event::NewEvents(start_cause) => {
                        state.on_new_events(start_cause)?;
                    }

                    os::Event::UserEvent(event) => {
                        state.on_custom_event(event)?;
                    }

                    os::Event::Suspended => {
                        state.on_suspended()?;
                    }

                    os::Event::Resumed => {
                        state.on_resumed()?;
                    }

                    os::Event::MainEventsCleared => {
                        let current_time = std::time::Instant::now();

                        while current_time - self.last_fixed_update_time >= self.fixed_update_period
                        {
                            application_state_flow =
                                state.on_fixed_update(self.fixed_update_period)?;
                            self.last_fixed_update_time += self.fixed_update_period;
                        }

                        let time_since_last_variable_update =
                            current_time - self.last_variable_update_time;
                        if time_since_last_variable_update > self.variable_update_min_period {
                            state.on_variable_update(time_since_last_variable_update)?;
                            self.last_variable_update_time = current_time;
                        }

                        state.on_main_events_cleared()?;
                    }

                    os::Event::RedrawRequested(window_id) => {
                        state.on_redraw_requested(window_id)?;
                    }

                    os::Event::RedrawEventsCleared => {
                        state.on_redraw_events_cleared()?;
                    }

                    os::Event::LoopDestroyed => {
                        state.on_event_loop_destroyed()?;
                    }

                    os::Event::WindowEvent { window_id, event } => match event {
                        os::WindowEvent::CloseRequested => {
                            state.on_close_requested(window_id)?;
                            application_state_flow =
                                ApplicationStateFlow::<ErrorType, CustomEventType>::Exit;
                        }

                        os::WindowEvent::Destroyed => {
                            state.on_destroyed(window_id)?;
                            application_state_flow =
                                ApplicationStateFlow::<ErrorType, CustomEventType>::Exit;
                        }

                        os::WindowEvent::Focused(focused) => {
                            if focused {
                                state.on_focus_gained(window_id)?;
                            } else {
                                state.on_focus_lost(window_id)?;
                            }
                        }

                        os::WindowEvent::Resized(size) => {
                            state.on_resized(window_id, size)?;
                        }

                        os::WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            new_inner_size,
                        } => {
                            state.on_scale_factor_changed(
                                window_id,
                                scale_factor,
                                new_inner_size,
                            )?;
                        }

                        os::WindowEvent::Moved(pos) => {
                            state.on_moved(window_id, pos)?;
                        }

                        os::WindowEvent::ReceivedCharacter(c) => {
                            state.on_received_character(window_id, c)?;
                        }

                        os::WindowEvent::DroppedFile(path) => {
                            state.on_hovered_file_dropped(window_id, path)?;
                        }

                        os::WindowEvent::HoveredFile(path) => {
                            state.on_hovered_file_entered(window_id, path)?;
                        }

                        os::WindowEvent::HoveredFileCancelled => {
                            state.on_hovered_file_left(window_id)?;
                        }

                        os::WindowEvent::KeyboardInput {
                            device_id,
                            input,
                            is_synthetic,
                        } => {
                            let last_key_state = self.keyboard_state.key_state(
                                Some(window_id),
                                device_id,
                                input.scancode,
                            );
                            let is_repeat = *last_key_state == input.state;
                            *last_key_state = input.state;
                            match input.state {
                                os::ElementState::Pressed => state.on_key_pressed(
                                    window_id,
                                    device_id,
                                    input.scancode,
                                    input.virtual_keycode,
                                    is_synthetic,
                                    is_repeat,
                                )?,
                                os::ElementState::Released => state.on_key_released(
                                    window_id,
                                    device_id,
                                    input.scancode,
                                    input.virtual_keycode,
                                    is_synthetic,
                                )?,
                            }
                        }

                        os::WindowEvent::ModifiersChanged(mods) => {
                            state.on_modifiers_changed(window_id, mods)?;
                        }

                        os::WindowEvent::CursorMoved {
                            device_id,
                            position,
                            ..
                        } => {
                            state.on_cursor_moved(window_id, device_id, position)?;
                        }

                        os::WindowEvent::CursorEntered { device_id } => {
                            state.on_cursor_entered(window_id, device_id)?;
                        }

                        os::WindowEvent::CursorLeft { device_id } => {
                            state.on_cursor_left(window_id, device_id)?;
                        }

                        os::WindowEvent::MouseInput {
                            device_id,
                            state: element_state,
                            button,
                            ..
                        } => match element_state {
                            os::ElementState::Pressed => {
                                state.on_mouse_button_pressed(window_id, device_id, button)?;
                            }
                            os::ElementState::Released => {
                                state.on_mouse_button_released(window_id, device_id, button)?;
                            }
                        },

                        os::WindowEvent::MouseWheel {
                            device_id,
                            delta,
                            phase,
                            ..
                        } => {
                            state.on_scroll(window_id, device_id, delta, phase)?;
                        }

                        os::WindowEvent::Touch(touch) => {
                            state.on_touch(
                                window_id,
                                touch.device_id,
                                touch.phase,
                                touch.location,
                                touch.force,
                                touch.id,
                            )?;
                        }

                        os::WindowEvent::AxisMotion {
                            device_id,
                            axis,
                            value,
                        } => {
                            state.on_axis_moved(window_id, device_id, axis, value)?;
                        }

                        // Not universally supported.
                        os::WindowEvent::TouchpadPressure { .. } => {}

                        // Not universally supported.
                        os::WindowEvent::ThemeChanged(_) => {}
                    },

                    os::Event::DeviceEvent { device_id, event } => match event {
                        os::DeviceEvent::Added => {
                            state.on_device_added(device_id)?;
                        }

                        os::DeviceEvent::Removed => {
                            state.on_device_removed(device_id)?;
                        }

                        os::DeviceEvent::MouseMotion { delta } => {
                            state.on_device_cursor_moved(
                                device_id,
                                os::PhysicalPosition::new(delta.0, delta.1),
                            )?;
                        }

                        os::DeviceEvent::MouseWheel { delta } => {
                            state.on_device_scroll(device_id, delta)?;
                        }

                        os::DeviceEvent::Motion { axis, value } => {
                            state.on_device_axis_moved(device_id, axis, value)?;
                        }

                        os::DeviceEvent::Button {
                            button,
                            state: element_state,
                        } => match element_state {
                            os::ElementState::Pressed => {
                                state.on_device_button_pressed(device_id, button)?;
                            }
                            os::ElementState::Released => {
                                state.on_device_button_released(device_id, button)?;
                            }
                        },

                        os::DeviceEvent::Key(input) => {
                            let last_key_state =
                                self.keyboard_state
                                    .key_state(None, device_id, input.scancode);
                            let is_repeat = *last_key_state == input.state;
                            *last_key_state = input.state;
                            match input.state {
                                os::ElementState::Pressed => state.on_device_key_pressed(
                                    device_id,
                                    input.scancode,
                                    input.virtual_keycode,
                                    is_repeat,
                                )?,
                                os::ElementState::Released => state.on_device_key_released(
                                    device_id,
                                    input.scancode,
                                    input.virtual_keycode,
                                )?,
                            }
                        }

                        os::DeviceEvent::Text { codepoint } => {
                            state.on_device_text(device_id, codepoint)?;
                        }
                    },
                }
            }
            None => {}
        }

        match application_state_flow {
            ApplicationStateFlow::Exit => Ok(os::ControlFlow::Exit),
            ApplicationStateFlow::Continue => Ok(os::ControlFlow::Poll),
            ApplicationStateFlow::Pop => {
                self.state_stack.pop();
                if self.state_stack.is_empty() {
                    Ok(os::ControlFlow::Exit)
                } else {
                    Ok(os::ControlFlow::Poll)
                }
            }
            ApplicationStateFlow::Push(new_state) => {
                self.state_stack.push(new_state);
                Ok(os::ControlFlow::Poll)
            }
            ApplicationStateFlow::PopPush(new_state) => {
                self.state_stack.pop();
                self.state_stack.push(new_state);
                Ok(os::ControlFlow::Poll)
            }
        }
    }
}

#[derive(Clone)]
struct KeyboardState {
    state: BTreeMap<(Option<os::WindowId>, os::DeviceId), [os::ElementState; 128]>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            state: BTreeMap::new(),
        }
    }

    pub fn key_state(
        &mut self,
        window_id: Option<os::WindowId>,
        device_id: os::DeviceId,
        scan_code: os::ScanCode,
    ) -> &mut os::ElementState {
        let key = (window_id, device_id);
        if !self.state.contains_key(&key) {
            self.state.insert(key, [os::ElementState::Released; 128]);
        }
        // Guaranteed to succeed due to the previous lines.
        let keyboard_state = self.state.get_mut(&key).unwrap();
        // Assuming at most a certain number of scancodes. It should be enough.
        // Asserting just for safety.
        let key_idx = scan_code as usize;
        assert!(
            key_idx < keyboard_state.len(),
            "Invalid scan code {}",
            key_idx
        );
        &mut keyboard_state[key_idx]
    }
}

#[cfg(test)]
mod tests {
    use super::super::EventHandler;
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    enum MyError {}

    impl std::fmt::Display for MyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MyError")
        }
    }

    impl std::error::Error for MyError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            None
        }
    }

    #[derive(Debug)]
    struct MyEventHandler {}

    impl EventHandler<MyError, ()> for MyEventHandler {
        fn on_fixed_update(&mut self, _: std::time::Duration) -> Result<ControlFlow, MyError> {
            Ok(ControlFlow::Exit)
        }
    }

    impl ApplicationInitializer<MyError, ()> for MyEventHandler {
        fn new(_: &os::EventLoop<()>) -> Result<Self, MyError> {
            Ok(Self {})
        }
    }

    #[test]
    fn application_creation() {
        let _app = Application::<MyEventHandler, _, _>::new(10, Some(10));
    }

    #[test]
    fn run() {
        Application::<MyEventHandler, _, _>::new(10, Some(10)).run();
    }
}
