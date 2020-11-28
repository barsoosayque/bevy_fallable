bevy_fallable
=============

[![Crates.io](https://img.shields.io/crates/v/bevy_fallable)](https://crates.io/crates/bevy_fallable)
[![Docs.rs](https://docs.rs/bevy_fallable/badge.svg)](https://docs.rs/bevy_fallable)
[![License](https://img.shields.io/crates/l/bevy_fallable)](https://github.com/barsoosayque/bevy_fallable/blob/master/LICENSE)

A simple plugin to install fallable systems to bevy

## API

Library provides two main components: `#[fallable_system]` attribute macro and `SystemErrorEvent` struct.
Essentially, every *fallable_system* will generate a `SystemErrorEvent` event if it results in an error, and that's about it.

For simplier usage, there is `FallableSystemPlugin` to register everything you'll need to recieve error events.

## Example

```rust
// Some system that might fail
#[fallable_system]
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
        .add_plugin(FallableSystemPlugin)
        .add_startup_system(system.system())
        .add_system(report_system.system())
        .run();
}
```
