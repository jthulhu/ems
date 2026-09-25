#![feature(try_blocks)]

use std::error::Error;

use ui::App;

mod config;
mod db;
mod error;
mod push_storage;
mod ui;
mod uring;

fn main() -> Result<(), Box<dyn Error>> {
    iced::application(App::new, App::update, App::view)
        // .theme(App::theme)
        .run()?;
    Ok(())
}
