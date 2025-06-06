use bevy::app::{App, Plugin, Update};
use crate::camera::ControllableCamera2d;
use crate::game::starlark::DIALECT;
use crate::geometry::RectExt;
use bevy::asset::Assets;
use bevy::color::{Color, Srgba};
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt, Parent};
use bevy::log::debug;
use bevy::math::{Dir2, Dir3, Rect, Vec3};
use bevy::prelude::{Commands, Component, Entity, Mesh, Mesh2d, OrthographicProjection, Query, Rectangle, Res, ResMut, Resource, Segment2d, Transform, With};
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use bevy::text::{Text2d, TextColor, TextFont};
use pyo3::IntoPyObject;
use starlark::environment::{Globals, Module};
use starlark::eval::Evaluator;
use starlark::syntax::AstModule;
use starlark::values::Value;
use crate::game::logging::Log;
use crate::game::python::Key;

pub(super) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentLevel(parse(include_str!("../../levels/default.plvl"))))
            .add_systems(Update, connection_tick);
    }
}

#[derive(Debug, Component)]
pub struct LevelEntity;

#[derive(Resource, Debug)]
pub struct CurrentLevel(pub Level);

#[derive(Debug, Component)]
pub struct Character;

#[derive(Debug, Clone)]
pub struct Level {
    rooms: Vec<Room>,
    start: usize,
    initial_pan: Option<(i32, i32)>,
    initial_zoom: f32,
}

impl<'v> From<&super::starlark::level::Level::Mut<'v>> for Level {
    fn from(value: &crate::game::starlark::level::Level::Mut<'v>) -> Self {
        let mut rooms = Vec::with_capacity(value.rooms.borrow().len());
        let mut start = 0;
        for (i, v) in value.rooms.borrow().iter().enumerate() {
            let room = Room::from(super::starlark::room::Room::from_value(*v).unwrap(), &value.rooms.borrow());
            if v.ptr_eq(value.start.borrow().unwrap()) {
                start = i;
            }
            rooms.push(room);
        }
        Self {
            rooms,
            start,
            initial_pan: value.initial_pan_x.borrow().zip_with(*value.initial_pan_y.borrow(), |x, y| (x, y)),
            initial_zoom: 0.1 / value.initial_zoom.borrow().unwrap_or(10) as f32,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Room {
    pub rect: Rect,
    connections: Vec<ConnectionTemplate>,
    item: Option<Item>,
}

impl Room {
    fn from<'v>(value: &super::starlark::room::Room::Mut<'v>, others: &[Value<'v>]) -> Self {
        Self {
            rect: Rect::new(
                value.pos.borrow().0 as f32 - 0.4,
                value.pos.borrow().1 as f32 + 0.4,
                value.pos.borrow().0 as f32 + value.size.borrow().0 as f32 - 0.6,
                value.pos.borrow().1 as f32 - value.size.borrow().1 as f32 + 0.6,
            ),
            connections: value.connections.take().into_iter()
                .map(|connection| ConnectionTemplate {
                    name: connection.name.take(),
                    room: others.iter().enumerate()
                        .find(|(_, other)| connection.room.borrow().ptr_eq(**other))
                        .unwrap().0,
                    locked: *connection.locked.borrow(),
                    key: connection.key.borrow().clone(),
                }).collect(),
            item: value.item.take().map(|item| super::starlark::level::Key::from_value(item).unwrap().into()),
        }
    }
}

#[derive(Debug, Clone)]
struct ConnectionTemplate {
    name: String,
    room: usize,
    locked: bool,
    key: Option<String>,
}

#[derive(Debug, Component)]
pub struct Connection {
    pub name: String,
    pub from: Entity,
    pub to: Entity,
    pub locked: bool,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Component, IntoPyObject)]
pub enum Item {
    Key(Key),
}

impl From<&super::starlark::level::Key::Mut> for Item {
    fn from(value: &super::starlark::level::Key::Mut) -> Self {
        Self::Key(Key(value.name.take()))
    }
}

pub fn parse(code: &str) -> Level {
    let ast = AstModule::parse("level", code.to_string(), &DIALECT).unwrap();
    let globals = Globals::standard();
    let module = Module::new();
        let level = module.heap().alloc(super::starlark::level::Level::Mut::default());
    module.set("level", level);
    let mut eval = Evaluator::new(&module);
    eval.eval_module(ast, &globals).unwrap();
    super::starlark::level::Level::from_value(level).unwrap().into()
}

pub fn spawn(
    level: Res<CurrentLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bounds = level.0.rooms.iter()
        .map(|room| room.rect)
        .reduce(|a, b| a.union(b))
        .unwrap_or_default()
        .inflate(0.1);
    commands.spawn((
        ControllableCamera2d::new(bounds),
        LevelEntity,
        Transform::from_translation(level.0.initial_pan.map_or(bounds.center().extend(0.), |(x, y)| Vec3::new(x as f32, y as f32, 0.))),
        OrthographicProjection {
            scale: level.0.initial_zoom,
            ..OrthographicProjection::default_2d()
        },
    ));
    let rooms: Vec<_> = level.0.rooms.iter().map(Clone::clone).enumerate().map(|(i, room)| {
        let mut spawn = commands.spawn((
            room.clone(),
            LevelEntity,
            Mesh2d(meshes.add(Rectangle::from_corners(room.rect.min, room.rect.max))),
            MeshMaterial2d(materials.add(Color::Srgba(Srgba::rgb(0.2, 0.2, 0.2)))),
            Transform::from_translation(room.rect.center().extend(1.)),
        ));
        if let Some(item) = &room.item {
            spawn.with_child((
                item.clone(),
                LevelEntity,
                Mesh2d(meshes.add(Rectangle::new(0.1, 0.1))),
                MeshMaterial2d(materials.add(Color::Srgba(Srgba::rgb(0.75, 0.5, 0.75)))),
                Transform::from_translation((-room.rect.half_size() + 0.1).extend(50.)),
            ));
        }
        if i == level.0.start {
            spawn.with_child((
                Character,
                LevelEntity,
                Mesh2d(meshes.add(Rectangle::new(0.5, 0.5))),
                MeshMaterial2d(materials.add(Color::Srgba(Srgba::rgb(0.5, 0.5, 0.5)))),
                Transform::from_xyz(0., 0., 100.),
            ));
        }
        (spawn.id(), room)
    }).collect();
    for (from, room) in &rooms {
        for connection in &room.connections {
            let (to, to_room) = &rooms[connection.room];
            let direction = Dir2::new(to_room.rect.center() - room.rect.center()).unwrap();
            let start = direction * room.rect.radius(direction) + room.rect.center();
            let end = -direction * to_room.rect.radius(-direction) + to_room.rect.center();
            let (line, pos) = Segment2d::from_points(start, end);
            commands.spawn((
                LevelEntity,
                Connection { name: connection.name.clone(), from: *from, to: *to, locked: connection.locked, key: connection.key.clone() },
                Mesh2d(meshes.add(Rectangle::new(0.01, line.half_length * 2. - 0.075))),
                MeshMaterial2d(materials.add(Color::Srgba(Srgba::rgb(0.25, 0.25, 0.75)))),
                Transform::from_translation(pos.extend(10.)).looking_to(line.direction.extend(-10.), Dir3::Z),
            )).with_child((
                Text2d::new(connection.name.clone()),
                TextFont::from_font_size(55.),
                Transform::from_xyz(0., -line.half_length + 0.025, 1.)
                    .with_scale(Vec3::splat(0.001)),
                TextColor(Color::WHITE),
            ));
        }
    }
}

fn connection_tick(
    mut labels: Query<(&mut TextColor, &Parent)>,
    connection: Query<&Connection>,
) {
    for (mut colour, parent) in &mut labels {
        if let Ok(connection) = connection.get(parent.get()) {
            colour.0 = if connection.locked { Color::Srgba(Srgba::rgb(0.4, 0.4, 0.4)) } else { Color::WHITE };
        }
    }
}

pub fn despawn(mut commands: Commands, entities: Query<Entity, With<LevelEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn reset(
    mut commands: Commands,
    mut log: ResMut<Log>,
) {
    debug!("Reset");
    log.0.clear();
    commands.run_system_cached(despawn);
    commands.run_system_cached(spawn);
}
