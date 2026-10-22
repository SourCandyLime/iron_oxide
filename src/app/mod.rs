//! Application lifecycle: builders and frames in `app`, event routing in
//! `events`, window/GPU setup in `window`, and boot sequencing in `boot`.

mod app;
mod boot;
mod config;
mod events;
mod window;

pub use app::App;
pub use boot::{BootScreen, BootState};
pub use config::{AppConfig, AppIcon};

#[cfg(test)]
mod tests;
