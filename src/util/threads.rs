use core::time::Duration;

use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Instant,
};

pub fn throttle<F: 'static + FnMut(Args) + Send, Args: 'static + Send>(
    mut function: F,
    min_duration: Duration,
) -> impl FnMut(Args, bool) -> bool + 'static + Send {
    let previous = Arc::new(Mutex::new(Instant::now() - min_duration));
    move |args: Args, force: bool| match previous.lock() {
        Ok(mut previous_guard) => {
            let now = Instant::now();
            let elapsed_duration = now.duration_since(*previous_guard);
            if elapsed_duration >= min_duration {
                *previous_guard = now;
                function(args);
                true
            } else if force {
                let wait_time = min_duration - elapsed_duration;
                if wait_time > Duration::ZERO {
                    sleep(wait_time);
                }
                *previous_guard = now + wait_time;
                function(args);
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
