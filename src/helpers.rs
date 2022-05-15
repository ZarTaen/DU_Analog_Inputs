use std::fs::{File, OpenOptions};
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Release;
use std::time::SystemTime;
use humantime::{format_rfc3339, Rfc3339Timestamp};

///Opens a file based on the given path, or creates a new one if it fails.
pub fn open_file(name : &str) -> File {
    let file = match OpenOptions::new().append(true).open(name){
        Ok(t) => t,
        Err(_e) => File::create(name).unwrap()
    };
    file
}

pub fn read_file(name : &str) -> File {
    let file = match OpenOptions::new().read(true).open(name){
        Ok(t) => t,
        Err(_e) => File::create(name).unwrap()
    };
    file
}

///Generates a RFC3339 timestamp for now.
pub fn timestamp_now() -> Rfc3339Timestamp{
    format_rfc3339(SystemTime::now())
}

pub fn log_write(file: &mut File, message_type: &str, message: &str) {
    file.write(format!("|{}|{}: {}\n", timestamp_now(), message_type, message).as_ref());
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

///Input Handling for shutting down the program.
fn end_this_world(electric_atomic_seppuku:Arc<AtomicBool>){
    loop {
        let mut s= String::new();
        println!("Version: v{}", VERSION);
        print!("To end the program, tip y and confirm with enter: \n");
        let _=stdout().flush();
        match stdin().read_line(&mut s) {
            Ok(_) => {
                match s.as_bytes()[0] {
                    121 => {
                        break;
                    }
                    89 => {
                        break;
                    }
                    _ => {}
                }
            }
            Err(_) => {}
        };
    }
    electric_atomic_seppuku.store(true, Release);
}