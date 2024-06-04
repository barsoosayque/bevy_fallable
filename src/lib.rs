#![allow(dead_code)]
use bevy_app::{App, Plugin};
use std::error::Error;

pub use bevy_app;
pub use bevy_ecs::prelude::*;

pub use bevy_fallible_derive::fallible_system;

/// Event which is sent every time a fallible_system results
/// in an error.
#[derive(Debug, Event)]
pub struct SystemErrorEvent {
    /// Name of the system
    pub system_name: &'static str,
    /// Error produced by the system
    pub error: Box<dyn Error + Send + Sync + 'static>,
}

/// Plugin to register fallible system parts.
pub struct FallibleSystemPlugin;

impl Plugin for FallibleSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SystemErrorEvent>();
    }
}
