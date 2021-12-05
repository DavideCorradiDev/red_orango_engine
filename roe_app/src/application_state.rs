use roe_os as os;

pub enum ApplicationStateFlow<ErrorType, CustomEventType> {
    DontChange,
    Pop,
    Push(Box<dyn ApplicationState<ErrorType, CustomEventType>>),
    Change(Box<dyn ApplicationState<ErrorType, CustomEventType>>),
    Exit,
}

pub trait ApplicationState<ErrorType, CustomEventType>
where
    ErrorType: std::fmt::Display + std::error::Error + 'static,
    CustomEventType: 'static,
{
    fn on_close_requested(&mut self, _wid: os::WindowId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_destroyed(&mut self, _wid: os::WindowId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_focus_gained(&mut self, _wid: os::WindowId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_focus_lost(&mut self, _wid: os::WindowId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_resized(
        &mut self,
        _wid: os::WindowId,
        _size: os::PhysicalSize<u32>,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_scale_factor_changed<'a>(
        &mut self,
        _wid: os::WindowId,
        _scale_factor: f64,
        _new_inner_size: &'a mut os::PhysicalSize<u32>,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_moved(
        &mut self,
        _wid: os::WindowId,
        _position: os::PhysicalPosition<i32>,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_received_character(&mut self, _wid: os::WindowId, _c: char) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_hovered_file_dropped(
        &mut self,
        _wid: os::WindowId,
        _path: std::path::PathBuf,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_hovered_file_entered(
        &mut self,
        _wid: os::WindowId,
        _path: std::path::PathBuf,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_hovered_file_left(&mut self, _wid: os::WindowId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_key_pressed(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _scan_code: os::ScanCode,
        _key_code: Option<os::KeyCode>,
        _is_synthetic: bool,
        _is_repeat: bool,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_key_released(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _scan_code: os::ScanCode,
        _key_code: Option<os::KeyCode>,
        _is_synthetic: bool,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_modifiers_changed(
        &mut self,
        _wid: os::WindowId,
        _modifiers_state: os::ModifiersState,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_cursor_moved(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _position: os::PhysicalPosition<f64>,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_cursor_entered(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_cursor_left(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_mouse_button_pressed(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _button: os::MouseButton,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_mouse_button_released(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _button: os::MouseButton,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_scroll(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _delta: os::MouseScrollDelta,
        _phase: os::TouchPhase,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_touch(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _phase: os::TouchPhase,
        _location: os::PhysicalPosition<f64>,
        _force: Option<os::TouchForce>,
        _id: u64,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_axis_moved(
        &mut self,
        _wid: os::WindowId,
        _device_id: os::DeviceId,
        _axis: os::AxisId,
        _value: f64,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_added(&mut self, _device_id: os::DeviceId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_removed(&mut self, _device_id: os::DeviceId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_cursor_moved(
        &mut self,
        _device_id: os::DeviceId,
        _position_delta: os::PhysicalPosition<f64>,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_scroll(
        &mut self,
        _device_id: os::DeviceId,
        _scroll_delta: os::MouseScrollDelta,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_axis_moved(
        &mut self,
        _device_id: os::DeviceId,
        _axis: os::AxisId,
        _value: f64,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_button_pressed(
        &mut self,
        _device_id: os::DeviceId,
        _button: os::ButtonId,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_button_released(
        &mut self,
        _device_id: os::DeviceId,
        _button: os::ButtonId,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_key_pressed(
        &mut self,
        _device_id: os::DeviceId,
        _scan_code: os::ScanCode,
        _key_code: Option<os::KeyCode>,
        _is_repeat: bool,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_key_released(
        &mut self,
        _device_id: os::DeviceId,
        _scan_code: os::ScanCode,
        _key_code: Option<os::KeyCode>,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_device_text(
        &mut self,
        _device_id: os::DeviceId,
        _codepoint: char,
    ) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_custom_event(&mut self, _event: CustomEventType) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_new_events(&mut self, _start_cause: os::EventLoopStartCause) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_main_events_cleared(&mut self) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_redraw_requested(&mut self, _wid: os::WindowId) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_redraw_events_cleared(&mut self) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_suspended(&mut self) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_resumed(&mut self) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_event_loop_destroyed(&mut self) -> Result<(), ErrorType> {
        Ok(())
    }

    fn on_fixed_update(
        &mut self,
        _dt: std::time::Duration,
    ) -> Result<ApplicationStateFlow<ErrorType, CustomEventType>, ErrorType> {
        Ok(ApplicationStateFlow::DontChange)
    }

    fn on_variable_update(&mut self, _dt: std::time::Duration) -> Result<(), ErrorType> {
        Ok(())
    }
}
