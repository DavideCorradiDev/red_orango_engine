pub enum ApplicationStateFlow<ErrorType, CustomEventType> {
    Keep,
    Pop,
    Push(Box<dyn ApplicationState<ErrorType, CustomEventType>>),
    Change(Box<dyn ApplicationState<ErrorType, CustomEventType>>),
}

pub trait ApplicationState<ErrorType, CustomEventType>
where
    ErrorType: std::fmt::Display + std::error::Error + 'static,
    CustomEventType: 'static,
{
}
