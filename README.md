A simple plugin to install fallable systems to bevy

```rust
#[fallable_system]
fn system(asset_server: Res<AssetServer>) -> anyhow::Result<()> {
    let handle: Handle<Texture> = asset_server.load("texture")?;
}

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
