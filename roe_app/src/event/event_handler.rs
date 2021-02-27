use super::{
    controller,
    event::{EventLoopStartCause, ScrollDelta},
    keyboard, mouse, touch, DeviceId, EventLoop,
};
use crate::window::{PhysicalPosition, PhysicalSize, WindowId};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ControlFlow {
    Continue,
    Exit,
}

pub trait EventHandler<ErrorType, CustomEventType>
where
    Self: std::marker::Sized,
    ErrorType: std::fmt::Display + std::error::Error + 'static,
    CustomEventType: 'static,
{
    type Error: std::fmt::Display + std::error::Error + 'static;
    type CustomEvent: 'static;

    fn new(event_loop: &EventLoop<Self::CustomEvent>) -> Result<Self, Self::Error>;

    fn on_close_requested(&mut self, _wid: WindowId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Exit)
    }

    fn on_destroyed(&mut self, _wid: WindowId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Exit)
    }

    fn on_focus_gained(&mut self, _wid: WindowId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_focus_lost(&mut self, _wid: WindowId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_resized(
        &mut self,
        _wid: WindowId,
        _size: PhysicalSize<u32>,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_scale_factor_changed<'a>(
        &self,
        _wid: WindowId,
        _scale_factor: f64,
        _new_inner_size: &'a mut PhysicalSize<u32>,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_moved(
        &mut self,
        _wid: WindowId,
        _position: PhysicalPosition<i32>,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_received_character(
        &mut self,
        _wid: WindowId,
        _c: char,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_hovered_file_dropped(
        &mut self,
        _wid: WindowId,
        _path: std::path::PathBuf,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_hovered_file_entered(
        &mut self,
        _wid: WindowId,
        _path: std::path::PathBuf,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_hovered_file_left(&mut self, _wid: WindowId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_key_pressed(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _scan_code: keyboard::ScanCode,
        _key_code: Option<keyboard::KeyCode>,
        _is_synthetic: bool,
        _is_repeat: bool,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_key_released(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _scan_code: keyboard::ScanCode,
        _key_code: Option<keyboard::KeyCode>,
        _is_synthetic: bool,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_modifiers_changed(
        &mut self,
        _wid: WindowId,
        _modifiers_state: keyboard::ModifiersState,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_cursor_moved(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _position: PhysicalPosition<f64>,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_cursor_entered(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_cursor_left(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_mouse_button_pressed(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _button: mouse::Button,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_mouse_button_released(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _button: mouse::Button,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_scroll(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _delta: ScrollDelta,
        _phase: touch::TouchPhase,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_touch(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _phase: touch::TouchPhase,
        _location: PhysicalPosition<f64>,
        _force: Option<touch::Force>,
        _id: u64,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_axis_moved(
        &mut self,
        _wid: WindowId,
        _device_id: DeviceId,
        _axis: controller::AxisId,
        _value: f64,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_added(&mut self, _device_id: DeviceId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_removed(&mut self, _device_id: DeviceId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_cursor_moved(
        &mut self,
        _device_id: DeviceId,
        _position_delta: PhysicalPosition<f64>,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_scroll(
        &mut self,
        _device_id: DeviceId,
        _scroll_delta: ScrollDelta,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_axis_moved(
        &mut self,
        _device_id: DeviceId,
        _axis: controller::AxisId,
        _value: f64,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_button_pressed(
        &mut self,
        _device_id: DeviceId,
        _button: controller::ButtonId,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_button_released(
        &mut self,
        _device_id: DeviceId,
        _button: controller::ButtonId,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_key_pressed(
        &mut self,
        _device_id: DeviceId,
        _scan_code: keyboard::ScanCode,
        _key_code: Option<keyboard::KeyCode>,
        _is_repeat: bool,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_key_released(
        &mut self,
        _device_id: DeviceId,
        _scan_code: keyboard::ScanCode,
        _key_code: Option<keyboard::KeyCode>,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_device_text(
        &mut self,
        _device_id: DeviceId,
        _codepoint: char,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_custom_event(&mut self, _event: Self::CustomEvent) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_new_events(
        &mut self,
        _start_cause: EventLoopStartCause,
    ) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_main_events_cleared(&mut self) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_redraw_requested(&mut self, _wid: WindowId) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_redraw_events_cleared(&mut self) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_suspended(&mut self) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_resumed(&mut self) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_event_loop_destroyed(&mut self) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Exit)
    }

    fn on_fixed_update(&mut self, _dt: std::time::Duration) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_variable_update(&mut self, _dt: std::time::Duration) -> Result<ControlFlow, Self::Error> {
        Ok(ControlFlow::Continue)
    }

    fn on_error(&mut self, error: Self::Error) {
        eprintln!("The application shut down due to an error ({})", error);
    }
}
