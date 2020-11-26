use bevy_fallable_derive::fallable_system;

#[fallable_system]
fn system() -> anyhow::Result<()> {
    println!("simple system");
    Ok(())
}

#[fallable_system]
fn system_with_args(s: &str, a: i32, b: i32) -> anyhow::Result<()> {
    println!("hello {}, a + b = {}", s, a + b);
    Ok(())
}

fn main() {
    let _: () = system();
    let _: () = system_with_args("world", 15, 20);
}
