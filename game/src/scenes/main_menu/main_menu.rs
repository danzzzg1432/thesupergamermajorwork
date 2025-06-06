#![allow(clippy::module_inception)]

use std::fs;
use bevy::app::AppExit;
use bevy::color::{Color, Srgba};
use bevy::hierarchy::{BuildChildren, ChildBuild};
use bevy::prelude::{Camera2d, Commands, EventWriter, NextState, ResMut, StateScoped};
use bevy::ui::{AlignItems, BackgroundColor, BoxShadow, FlexDirection, IsDefaultUiCamera, JustifyContent, Node, Val};
use crate::game::level;
use crate::game::level::CurrentLevel;
use crate::scenes::Scene;
use crate::ui::button::InteractiveButton;
use crate::ui::ChildBuilderExt;

pub(super) fn build(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera, StateScoped(Scene::MainMenu)));
    let play = commands.register_system(|mut next_state: ResMut<NextState<Scene>>, mut current_level: ResMut<CurrentLevel>| {
        *current_level = CurrentLevel(level::parse(include_str!("../../../levels/test.plvl")));
        next_state.set(Scene::Editor);
    });
    let load = commands.register_system(|mut next_state: ResMut<NextState<Scene>>, mut current_level: ResMut<CurrentLevel>| {
        let Some(file) = rfd::FileDialog::new()
            .set_title("Load Level")
            .add_filter("Pythoneer Level", &["plvl"])
            .pick_file() else { return; };
        let level = level::parse(&fs::read_to_string(file).unwrap());
        *current_level = CurrentLevel(level);
        next_state.set(Scene::Editor);
    });
    let exit = commands.register_system(|mut app_exit: EventWriter<AppExit>| { app_exit.send(AppExit::Success); });
    commands.spawn((
        Node {
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        StateScoped(Scene::MainMenu),
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(460.0),
                height: Val::Px(620.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            BackgroundColor(Color::Srgba(Srgba::rgb(1.0, 1.0, 1.0))),
            BoxShadow {
                color: Color::Srgba(Srgba::rgb(0.0, 0.0, 0.0)),
                x_offset: Val::Px(5.0),
                y_offset: Val::Px(8.0),
                spread_radius: Val::Px(18.0),
                blur_radius: Val::Px(37.0),
            }
        )).with_children(|parent| {
            parent.button("Play", InteractiveButton {
                on_click: play,
                background: BackgroundColor(Color::Srgba(Srgba::rgb(0.65, 0.74, 1.0))),
                background_hover: BackgroundColor(Color::Srgba(Srgba::rgb(0.55, 0.64, 1.0))),
                background_click: BackgroundColor(Color::Srgba(Srgba::rgb(0.45, 0.54, 1.0))),
            });
            parent.button("Load Level", InteractiveButton {
                on_click: load,
                background: BackgroundColor(Color::Srgba(Srgba::rgb(0.65, 0.74, 1.0))),
                background_hover: BackgroundColor(Color::Srgba(Srgba::rgb(0.55, 0.64, 1.0))),
                background_click: BackgroundColor(Color::Srgba(Srgba::rgb(0.45, 0.54, 1.0))),
            });
            parent.button("Quit", InteractiveButton {
                on_click: exit,
                background: BackgroundColor(Color::Srgba(Srgba::rgb(0.65, 0.74, 1.0))),
                background_hover: BackgroundColor(Color::Srgba(Srgba::rgb(0.55, 0.64, 1.0))),
                background_click: BackgroundColor(Color::Srgba(Srgba::rgb(0.45, 0.54, 1.0))),
            });
        });
    });
}
