use super::ApplicationState;

use roe_os as os;

// TODO: rename file.
// TODO: Maybe replace with lambda?
pub trait ApplicationInitializer<ErrorType, CustomEventType>
where
    ErrorType: std::fmt::Display + std::error::Error + 'static,
    CustomEventType: 'static,
{
    fn create_initial_state(
        event_loop: &os::EventLoop<CustomEventType>,
    ) -> Result<Box<dyn ApplicationState<ErrorType, CustomEventType>>, ErrorType>;
}
