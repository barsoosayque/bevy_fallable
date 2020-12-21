bevy_fallible
=============

[![Crates.io](https://img.shields.io/crates/v/bevy_fallible)](https://crates.io/crates/bevy_fallible)
[![Docs.rs](https://docs.rs/bevy_fallible/badge.svg)](https://docs.rs/bevy_fallible)
[![License](https://img.shields.io/crates/l/bevy_fallible)](https://github.com/barsoosayque/bevy_fallible/blob/master/LICENSE)

A simple plugin to install fallible systems to bevy

## API

Library provides two main components: `#[fallible_system]` attribute macro and `SystemErrorEvent` struct.
Essentially, every *fallible_system* will generate a `SystemErrorEvent` event if it results in an error, and that's about it.

For simplier usage, there is `fallibleSystemPlugin` to register everything you'll need to recieve error events.

## Example

```rust
// Some system that might fail
#[fallible_system]
fn system(asset_server: Res<AssetServer>) -> anyhow::Result<()> {
    let handle: Handle<Texture> = asset_server.load("texture")?;
}

// Let's make another system to read every event about other
// systems failures and report !
#[derive(Default)]
struct ReportSystemState{ reader: EventReader<SystemErrorEvent> }
fn report_system(mut state: Local<ReportSystemState>, mut events: ResMut<Events<SystemErrorEvent>>) {
    for event in state.reader.iter(&mut events) {
        println!("Error in {}: {}", event.system_name, event.error); 
    }
}

fn main() {
    App::build()
        .add_plugin(fallibleSystemPlugin)
        .add_startup_system(system.system())
        .add_system(report_system.system())
        .run();
}
```
