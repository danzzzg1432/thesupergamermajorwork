use bevy::app::{App, Plugin};

pub mod level;
pub mod python;
pub mod execution;
mod starlark;
pub mod logging;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<logging::Log>()
            .add_plugins(level::LevelPlugin)
            .add_plugins(execution::ExecutionPlugin);
    }
}
