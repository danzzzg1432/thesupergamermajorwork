use bevy::prelude::Update;
use bevy::app::{App, Plugin};

pub mod channel;
pub mod run;
pub mod execution_state;

pub(super) struct ExecutionPlugin;

impl Plugin for ExecutionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(channel::ChannelPlugin)
            .add_systems(Update, run::watch);
    }
}
