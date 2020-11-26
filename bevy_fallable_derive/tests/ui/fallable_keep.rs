use bevy_fallable_derive::fallable_system;

#[fallable_system(keep)]
fn system() -> anyhow::Result<()> {
    println!("simple system");
    Ok(())
}

#[fallable_system(keep)]
fn system_with_args(s: &str, a: i32, b: i32) -> anyhow::Result<()> {
    println!("hello {}, a + b = {}", s, a + b);
    Ok(())
}

fn main() {
    let _: anyhow::Result<()> = system_fallable();
    let _: () = system();

    let _: anyhow::Result<()> = system_with_args_fallable("hello", 2, 2);
    let _: () = system_with_args("world", 15, 20);
}
