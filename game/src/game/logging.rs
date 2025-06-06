use bevy::prelude::{ResMut, Resource};
use pyo3::{pyclass, pymethods};
use crate::game::execution::channel::Run;

#[derive(Resource, Default)]
pub struct Log(pub String);

#[pyclass]
pub struct Logger;

#[pymethods]
impl Logger {
    #[allow(clippy::unused_self)]
    fn write(&self, text: String) {
        Run::new(move |mut log: ResMut<Log>| log.0.push_str(&text)).execute();
    }
}
