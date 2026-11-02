use sddf::{Instant, arm_after};
use core::time::Duration;
use sel4_microkit::{Channel, ChannelSet, Handler, Infallible};

pub use super::network::SharedSddfRuntime;

fn timer_channel() -> Channel {
    Channel::new(sddf::timer_channel())
}

pub fn init() {
    assert_eq!(sddf::net_init(), 0);
    assert_eq!(sddf::timer_init(), 0);
    sddf::init(timer_channel().index());
}

pub trait Application {
    fn poll(&mut self);
}

pub struct EventLoop<A> {
    application: A,
    timer_deadline: Option<Instant>,
}

impl<A: Application> EventLoop<A> {
    pub fn new(application: A) -> Self {
        let mut event_loop = Self { application, timer_deadline: None };
        event_loop.drive();
        event_loop
    }

    fn drive(&mut self) {
        crate::runtime::timer::global_advance_clock(Instant::now());
        loop {
            self.application.poll();
            if !sddf::net_arm_rx() {
                break;
            }
        }
        if let Some(deadline) = crate::runtime::timer::global_next_deadline() {
            if self.timer_deadline.is_none_or(|armed| deadline < armed) {
                let delay = deadline
                    .checked_duration_since(Instant::now())
                    .unwrap_or(Duration::from_nanos(1));
                arm_after(delay.max(Duration::from_nanos(1)));
                self.timer_deadline = Some(deadline);
            }
        }
    }
}

impl<A: Application> Handler for EventLoop<A> {
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        if channels.contains(timer_channel()) {
            self.timer_deadline = None;
        }
        self.drive();
        Ok(())
    }
}
