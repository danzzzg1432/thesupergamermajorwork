#![pyo3::pymodule(name = "pythoneer", gil_used = false)]

use bevy::hierarchy::{BuildChildren, Parent};
use bevy::prelude::{Entity, Query, Single, With, Without, Commands};
use crate::game::execution::channel::Run;
use pyo3::{pyclass, pyfunction, pymethods, PyResult, Python};
use pyo3::exceptions::PyValueError;
use crate::game::execution::run::tick;
use crate::game::level::{Character, Connection, Item};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Key(pub String);

#[pymethods]
impl Key {
    fn __repr__(&self) -> String {
        format!("Key({})", self.0)
    }
 
    fn r#use(&self, py: Python, connection: String) -> PyResult<()> {
        let name = self.0.clone();
        py.allow_threads(tick);
        Run::new(move |character: Single<&Parent, With<Character>>, mut connections: Query<&mut Connection, Without<Character>>| {
            let parent = *character;
            let mut con = connections.iter_mut()
                .find(|con| con.from == parent.get() && *con.name == *connection)
                .ok_or(PyValueError::new_err(format!("Invalid connection: {connection:?}")))?;
            if con.key.as_ref() != Some(&name) {
                return Err(PyValueError::new_err(format!("The {name} key doesn't fit connection {connection:?}")))
            }
            con.locked = !con.locked;
            Ok(())
        }).execute()
    }
}

#[pyfunction]
fn r#move(py: Python, connection: String) -> PyResult<()> {
    py.allow_threads(tick);
    Run::new(move |mut commands: Commands, character: Single<(Entity, &Parent), With<Character>>, connections: Query<&Connection, Without<Character>>| {
        let (entity, parent) = *character;
        let Connection { to, locked, .. } = connections.iter()
            .find(|Connection { name, from, .. }| *from == parent.get() && *name == *connection)
            .ok_or(PyValueError::new_err(format!("Invalid connection: {connection:?}")))?;
        if *locked {
            return Err(PyValueError::new_err(format!("Connection is locked: {connection:?}")));
        }
        commands.entity(entity).set_parent(*to);
        Ok(())
    }).execute()
}

#[pyfunction]
fn pickup(py: Python) -> Option<Item> {
    py.allow_threads(tick);
    Run::new(move |mut commands: Commands, room: Single<&Parent, With<Character>>, items: Query<(Entity, &Parent, &Item), Without<Character>>| {
        if let Some((entity, _, item)) = items.iter().find(|(_, parent, _)| parent.get() == room.get()) {
            commands.entity(entity).despawn();
            Some(item.clone())
        } else {
            None
        }
    }).execute()
}
