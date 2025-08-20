use std::{panic, process};

pub fn ignore_sigpipe() {
    panic::set_hook(Box::new(|info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            Some(*s)
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            Some(s.as_str())
        } else {
            None
        };
        if let Some(message) = message {
            if message.contains("failed printing to stdout: Broken pipe") {
                process::exit(0);
            }
        }
        eprintln!("panic: {info}");
        process::exit(1);
    }))
}
