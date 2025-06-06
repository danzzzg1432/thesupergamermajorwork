mod main_menu;

use bevy::app::{App, Plugin};
use bevy::prelude::OnEnter;
use crate::scenes::Scene;

pub(super) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Scene::MainMenu), main_menu::build);
    }
}
