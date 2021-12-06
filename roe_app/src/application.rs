use super::{ApplicationState, ControlFlow};

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

    fn push_state(
        &mut self,
        mut state: Box<dyn ApplicationState<ErrorType, CustomEventType>>,
    ) -> Result<(), ErrorType> {
        state.on_start()?;
        self.state_stack.push(state);
        Ok(())
    }

    fn pop_state(&mut self) -> Result<(), ErrorType> {
        if let Some(mut state) = self.state_stack.pop() {
            state.on_end()?;
        }
        Ok(())
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
        // TODO: provide information about the error.
        // TODO: remove the "Exit" control flow?
        self.push_state(initialization_fn(&event_loop).expect("Initial state creation error."))
            .expect("State initialization error.");

        let current_time = std::time::Instant::now();
        self.last_fixed_update_time = current_time;
        self.last_variable_update_time = current_time;

        event_loop.run(
            move |event, _, control_flow| match self.handle_event(event) {
                Ok(flow) => *control_flow = flow,
                Err(e) => {
                    // TODO: make the error handler return a control flow?
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
        let mut control_flow = ControlFlow::Continue;

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
                            state.on_fixed_update(self.fixed_update_period)?;
                            control_flow = state.requested_control_flow();
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
                            control_flow = ControlFlow::Exit;
                        }

                        os::WindowEvent::Destroyed => {
                            state.on_destroyed(window_id)?;
                            control_flow = ControlFlow::Exit;
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
                            let is_repeat = self.keyboard_state.update_key_state(
                                Some(window_id),
                                device_id,
                                input.virtual_keycode,
                                input.state,
                            );
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
                            let is_repeat = self.keyboard_state.update_key_state(
                                None,
                                device_id,
                                input.virtual_keycode,
                                input.state,
                            );
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
            // TODO: remove this None case.
            None => {}
        }

        match control_flow {
            ControlFlow::Exit => {
                while !self.state_stack.is_empty() {
                    self.pop_state()?;
                }
                Ok(os::ControlFlow::Exit)
            }
            ControlFlow::Continue => Ok(os::ControlFlow::Poll),
            ControlFlow::PopState => {
                self.pop_state()?;
                if self.state_stack.is_empty() {
                    Ok(os::ControlFlow::Exit)
                } else {
                    Ok(os::ControlFlow::Poll)
                }
            }
            ControlFlow::PushState(new_state) => {
                self.push_state(new_state)?;
                Ok(os::ControlFlow::Poll)
            }
            ControlFlow::PopPushState(new_state) => {
                self.pop_state()?;
                self.push_state(new_state)?;
                Ok(os::ControlFlow::Poll)
            }
        }
    }
}

fn get_key_index(key_code: os::KeyCode) -> usize {
    match key_code {
        os::KeyCode::Key0 => 0,
        os::KeyCode::Key1 => 1,
        os::KeyCode::Key2 => 2,
        os::KeyCode::Key3 => 3,
        os::KeyCode::Key4 => 4,
        os::KeyCode::Key5 => 5,
        os::KeyCode::Key6 => 6,
        os::KeyCode::Key7 => 7,
        os::KeyCode::Key8 => 8,
        os::KeyCode::Key9 => 9,
        os::KeyCode::A => 10,
        os::KeyCode::B => 11,
        os::KeyCode::C => 12,
        os::KeyCode::D => 13,
        os::KeyCode::E => 14,
        os::KeyCode::F => 15,
        os::KeyCode::G => 16,
        os::KeyCode::H => 17,
        os::KeyCode::I => 18,
        os::KeyCode::J => 19,
        os::KeyCode::K => 20,
        os::KeyCode::L => 21,
        os::KeyCode::M => 22,
        os::KeyCode::N => 23,
        os::KeyCode::O => 24,
        os::KeyCode::P => 25,
        os::KeyCode::Q => 26,
        os::KeyCode::R => 27,
        os::KeyCode::S => 28,
        os::KeyCode::T => 29,
        os::KeyCode::U => 30,
        os::KeyCode::V => 31,
        os::KeyCode::W => 32,
        os::KeyCode::X => 33,
        os::KeyCode::Y => 34,
        os::KeyCode::Z => 35,
        os::KeyCode::Escape => 36,
        os::KeyCode::F1 => 37,
        os::KeyCode::F2 => 38,
        os::KeyCode::F3 => 39,
        os::KeyCode::F4 => 40,
        os::KeyCode::F5 => 41,
        os::KeyCode::F6 => 42,
        os::KeyCode::F7 => 43,
        os::KeyCode::F8 => 44,
        os::KeyCode::F9 => 45,
        os::KeyCode::F10 => 46,
        os::KeyCode::F11 => 47,
        os::KeyCode::F12 => 48,
        os::KeyCode::F13 => 49,
        os::KeyCode::F14 => 50,
        os::KeyCode::F15 => 51,
        os::KeyCode::F16 => 52,
        os::KeyCode::F17 => 53,
        os::KeyCode::F18 => 54,
        os::KeyCode::F19 => 55,
        os::KeyCode::F20 => 56,
        os::KeyCode::F21 => 57,
        os::KeyCode::F22 => 58,
        os::KeyCode::F23 => 59,
        os::KeyCode::F24 => 60,
        os::KeyCode::Snapshot => 61,
        os::KeyCode::Scroll => 62,
        os::KeyCode::Pause => 63,
        os::KeyCode::Insert => 64,
        os::KeyCode::Home => 65,
        os::KeyCode::Delete => 66,
        os::KeyCode::End => 67,
        os::KeyCode::PageDown => 68,
        os::KeyCode::PageUp => 69,
        os::KeyCode::Left => 70,
        os::KeyCode::Up => 71,
        os::KeyCode::Right => 72,
        os::KeyCode::Down => 73,
        os::KeyCode::Back => 74,
        os::KeyCode::Return => 75,
        os::KeyCode::Space => 76,
        os::KeyCode::Compose => 77,
        os::KeyCode::Caret => 78,
        os::KeyCode::Numlock => 79,
        os::KeyCode::Numpad0 => 80,
        os::KeyCode::Numpad1 => 81,
        os::KeyCode::Numpad2 => 82,
        os::KeyCode::Numpad3 => 83,
        os::KeyCode::Numpad4 => 84,
        os::KeyCode::Numpad5 => 85,
        os::KeyCode::Numpad6 => 86,
        os::KeyCode::Numpad7 => 87,
        os::KeyCode::Numpad8 => 88,
        os::KeyCode::Numpad9 => 89,
        os::KeyCode::NumpadAdd => 90,
        os::KeyCode::NumpadDivide => 91,
        os::KeyCode::NumpadDecimal => 92,
        os::KeyCode::NumpadComma => 93,
        os::KeyCode::NumpadEnter => 94,
        os::KeyCode::NumpadEquals => 95,
        os::KeyCode::NumpadMultiply => 96,
        os::KeyCode::NumpadSubtract => 97,
        os::KeyCode::AbntC1 => 98,
        os::KeyCode::AbntC2 => 99,
        os::KeyCode::Apostrophe => 100,
        os::KeyCode::Apps => 101,
        os::KeyCode::Asterisk => 102,
        os::KeyCode::At => 103,
        os::KeyCode::Ax => 104,
        os::KeyCode::Backslash => 105,
        os::KeyCode::Calculator => 106,
        os::KeyCode::Capital => 107,
        os::KeyCode::Colon => 108,
        os::KeyCode::Comma => 109,
        os::KeyCode::Convert => 110,
        os::KeyCode::Equals => 111,
        os::KeyCode::Grave => 112,
        os::KeyCode::Kana => 113,
        os::KeyCode::Kanji => 114,
        os::KeyCode::LAlt => 115,
        os::KeyCode::LBracket => 116,
        os::KeyCode::LControl => 117,
        os::KeyCode::LShift => 118,
        os::KeyCode::LWin => 119,
        os::KeyCode::Mail => 120,
        os::KeyCode::MediaSelect => 121,
        os::KeyCode::MediaStop => 122,
        os::KeyCode::Minus => 123,
        os::KeyCode::Mute => 124,
        os::KeyCode::MyComputer => 125,
        os::KeyCode::NavigateForward => 126,
        os::KeyCode::NavigateBackward => 127,
        os::KeyCode::NextTrack => 128,
        os::KeyCode::NoConvert => 129,
        os::KeyCode::OEM102 => 130,
        os::KeyCode::Period => 131,
        os::KeyCode::PlayPause => 132,
        os::KeyCode::Plus => 133,
        os::KeyCode::Power => 134,
        os::KeyCode::PrevTrack => 135,
        os::KeyCode::RAlt => 136,
        os::KeyCode::RBracket => 137,
        os::KeyCode::RControl => 138,
        os::KeyCode::RShift => 139,
        os::KeyCode::RWin => 140,
        os::KeyCode::Semicolon => 141,
        os::KeyCode::Slash => 142,
        os::KeyCode::Sleep => 143,
        os::KeyCode::Stop => 144,
        os::KeyCode::Sysrq => 145,
        os::KeyCode::Tab => 146,
        os::KeyCode::Underline => 147,
        os::KeyCode::Unlabeled => 148,
        os::KeyCode::VolumeDown => 149,
        os::KeyCode::VolumeUp => 150,
        os::KeyCode::Wake => 151,
        os::KeyCode::WebBack => 152,
        os::KeyCode::WebFavorites => 153,
        os::KeyCode::WebForward => 154,
        os::KeyCode::WebHome => 155,
        os::KeyCode::WebRefresh => 156,
        os::KeyCode::WebSearch => 157,
        os::KeyCode::WebStop => 158,
        os::KeyCode::Yen => 159,
        os::KeyCode::Copy => 160,
        os::KeyCode::Paste => 161,
        os::KeyCode::Cut => 162,
    }
}

const KEY_COUNT: usize = 163;

#[derive(Clone)]
struct KeyboardState {
    state: BTreeMap<(Option<os::WindowId>, os::DeviceId), [os::ElementState; KEY_COUNT]>,
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
        key_code: os::KeyCode,
    ) -> &mut os::ElementState {
        let key = (window_id, device_id);
        if !self.state.contains_key(&key) {
            self.state
                .insert(key, [os::ElementState::Released; KEY_COUNT]);
        }
        // Guaranteed to succeed due to the previous lines.
        let keyboard_state = self.state.get_mut(&key).unwrap();
        &mut keyboard_state[get_key_index(key_code)]
    }

    pub fn update_key_state(
        &mut self,
        window_id: Option<os::WindowId>,
        device_id: os::DeviceId,
        key_code: Option<os::KeyCode>,
        new_state: os::ElementState,
    ) -> bool {
        let mut is_repeat = false;
        if let Some(key_code) = key_code {
            let last_key_state = self.key_state(window_id, device_id, key_code);
            is_repeat = *last_key_state == new_state;
            *last_key_state = new_state;
        }
        is_repeat
    }
}

#[cfg(test)]
mod tests {
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
    struct MyAppState {}

    impl ApplicationState<MyError, ()> for MyAppState {
        fn requested_control_flow(&mut self) -> ControlFlow<MyError, ()> {
            ControlFlow::Exit
        }
    }

    #[test]
    fn application_creation() {
        let _app = Application::<MyError, ()>::new(10, Some(10));
    }

    #[test]
    fn run() {
        Application::new(10, Some(10)).run(|_event_queue| Ok(Box::new(MyAppState {})));
    }
}
