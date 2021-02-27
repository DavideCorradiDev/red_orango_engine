extern crate winit;

pub use winit::{
    event::{
        DeviceEvent, DeviceId, ElementState, Event, KeyboardInput, MouseScrollDelta as ScrollDelta,
        StartCause as EventLoopStartCause, WindowEvent,
    },
    event_loop::{EventLoop, EventLoopClosed, EventLoopProxy, EventLoopWindowTarget},
};
