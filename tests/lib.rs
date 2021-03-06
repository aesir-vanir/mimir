#![feature(try_from)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
mod macros;
#[macro_use]
extern crate slog;

extern crate chrono;
extern crate mimir;
extern crate rand;
extern crate slog_async;
extern crate slog_term;

mod connection;
mod context;
mod dequeue;
mod enqueue;
mod lob;
mod message;
#[cfg(any(target_arch = "linux", target_arch = "windows"))]
mod objecttype;
mod pool;
mod statement;
mod variable;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[cfg(test)]
lazy_static! {
    pub static ref CREDS: Vec<String> = {
        let mut creds = Vec::new();
        if let Ok(file) = File::open(".creds/oic-test") {
            let buf_reader = BufReader::new(file);

            for line_res in buf_reader.lines() {
                if let Ok(line) = line_res {
                    let parts = line
                        .split(':')
                        .map(|x| x.trim_right().to_string())
                        .collect::<Vec<String>>();
                    creds.extend(parts);
                }
            }
        } else {
            let username = env::var("MIMIR_USERNAME").expect("invalid username");
            let password = env::var("MIMIR_PASSWORD").expect("invalid password");
            creds.push(username);
            creds.push(password);

            let odpic_username = env::var("ODPIC_USERNAME").expect("invalid username");
            let odpic_password = env::var("ODPIC_PASSWORD").expect("invalid password");
            creds.push(odpic_username);
            creds.push(odpic_password);
        }
        creds
    };
}
