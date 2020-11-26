#![allow(dead_code)]
use bevy_app::{AppBuilder, Plugin};
use std::error::Error;

pub use bevy_app;
pub use bevy_ecs;

pub use bevy_fallable_derive::fallable_system;

/// Event which is sent every time a fallable_system results
/// in an error.
pub struct SystemErrorEvent {
    /// Name of the system
    pub system_name: &'static str,
    /// Error produced by the system
    pub error: Box<dyn Error + Send + Sync>
}

/// Plugin to register fallable system parts.
pub struct FallableSystemPlugin;

impl Plugin for FallableSystemPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<SystemErrorEvent>();
    }
}
