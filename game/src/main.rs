#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
// #![warn(clippy::unwrap_used)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![feature(custom_inner_attributes)]
#![feature(proc_macro_hygiene)]
#![feature(never_type)]
#![feature(lock_value_accessors)]
#![feature(adt_const_params)]
#![feature(option_zip)]
#![feature(thread_sleep_until)]
#![feature(let_chains)]
#![feature(thread_id_value)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod scenes;
mod ui;
mod game;
mod camera;
#[cfg(debug_assertions)]
mod debug;
mod geometry;

use crate::game::python;
use bevy::app::App;
use bevy::prelude::AppExtStates;
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use pyo3::types::PyAnyMethods;
use pyo3::{append_to_inittab, prepare_freethreaded_python, Python};
use game::execution::execution_state;

fn main() {
    append_to_inittab!(python);
    prepare_freethreaded_python();
    Python::with_gil(|py| {
        let sys = py.import("sys").unwrap();
        sys.setattr("stdout", game::logging::Logger).unwrap();
        sys.setattr("stderr", game::logging::Logger).unwrap();
        sys.setattr("stdin", Option::<()>::None).unwrap();
    });
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(ui::UiPlugins)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(scenes::ScenesPlugin)
        .init_state::<execution_state::ExecutionState>();
    #[cfg(debug_assertions)]
    app.add_plugins(debug::DebugPlugin);
    app.run();
}
