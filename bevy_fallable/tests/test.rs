use bevy::prelude::*;
use bevy_fallable::*;
use anyhow::{anyhow, Result};

struct ShouldFail(bool);
#[derive(Clone)]
struct FailCounter(u32);

#[fallable_system]
fn fallable_system(should_fail: Res<ShouldFail>) -> Result<()> {
    if should_fail.0 {
        Err(anyhow!("failed"))
    } else {
        Ok(())
    }
}

#[derive(Default)]
struct CounterSystemState {
    reader: EventReader<SystemErrorEvent>
}
fn counter_system(mut state: Local<CounterSystemState>, mut events: ResMut<Events<SystemErrorEvent>>, mut counter: ResMut<FailCounter>) {
    counter.0 += state.reader.iter(&mut events).count() as u32;
}

fn run<F>(should_fail: bool, iterations: u32, run_asserts: F)
where F: 'static + Fn(App) -> () {
    App::build()
        .add_plugin(FallableSystemPlugin)
        .add_resource(ShouldFail(should_fail))
        .add_resource(FailCounter(0))
        .add_system(fallable_system.system())
        .add_system(counter_system.system())
        .set_runner(move |mut app| {
            app.initialize();
            for _ in 0..iterations {
                app.update();
            }
            run_asserts(app);
        })
        .run();
}


#[test]
fn fail_count_errors() {
    run(true, 5, |app| {
        assert!(app.resources.contains::<FailCounter>());
        assert_eq!(app.resources.get::<FailCounter>().unwrap().0, 5);
    });
}

#[test]
fn pass_count_errors() {
    run(false, 5, |app| {
        assert!(app.resources.contains::<FailCounter>());
        assert_eq!(app.resources.get::<FailCounter>().unwrap().0, 0);
    });
}
