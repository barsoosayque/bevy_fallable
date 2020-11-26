use bevy_fallable_derive::fallable_system;
use bevy_fallable::FallableSystemPlugin;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

struct CustomRes(String);

#[fallable_system]
fn system() -> anyhow::Result<()> {
    println!("simple system");
    Ok(())
}

#[fallable_system]
fn system_with_args(res: Res<CustomRes>) -> anyhow::Result<()> {
    println!("res: {}", res.0);
    Ok(())
}

#[fallable_system]
fn system_with_commands(mut _commands: Commands, res: Res<CustomRes>) -> anyhow::Result<()> {
    println!("res with commands: {}", res.0);
    Ok(())
}

#[test]
fn fallable_system() {
    App::build()
        .add_plugin(FallableSystemPlugin)
        .add_resource::<CustomRes>(CustomRes("hello world".to_owned()))
        .add_system(system.system())
        .add_system(system_with_args.system())
        .add_system(system_with_commands.system())
        .run();
}
