use super::EventHandler;

use roe_os as os;

pub trait ApplicationImpl<ErrorType, CustomEventType> 
where
    Self: std::marker::Sized + EventHandler<ErrorType, CustomEventType>,
    ErrorType: std::fmt::Display + std::error::Error + 'static,
    CustomEventType: 'static,
{
    fn new(event_loop: &os::EventLoop<CustomEventType>) -> Result<Self, ErrorType>;
}