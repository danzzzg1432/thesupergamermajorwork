use bevy::prelude::IntoSystem;
use std::sync::{Arc, RwLock};
use std::sync::{mpsc, Mutex, OnceLock};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Resource, System, World};

static TX: OnceLock<mpsc::Sender<Arc<dyn Runnable + Send + Sync>>> = OnceLock::new();

#[derive(Resource)]
struct Rx(Arc<Mutex<mpsc::Receiver<Arc<dyn Runnable + Send + Sync>>>>);

pub(super) struct ChannelPlugin;

impl Plugin for ChannelPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = mpsc::channel();
        TX.set(tx).unwrap();
        app.insert_resource(Rx(Arc::new(Mutex::new(rx))))
            .add_systems(Update, tick);
    }
}

pub trait Runnable {
    fn run(&self, world: &mut World);
}

#[must_use]
pub struct Run<T: Send + Sync + 'static, S: System<In = (), Out = T>> {
    system: RwLock<Option<S>>,
    result: RwLock<OnceLock<T>>,
}

impl<T: Send + Sync + 'static, S: System<In = (), Out = T>> Run<T, S> {
    pub fn new<M: 'static>(system: impl IntoSystem<(), T, M, System = S>) -> Self {
        Self {
            system: RwLock::new(Some(IntoSystem::into_system(system))),
            result: RwLock::default(),
        }
    }

    pub fn execute(self) -> T {
        let arc = Arc::new(self);
        TX.get().unwrap().send(arc.clone()).unwrap();
        arc.result.read().unwrap().wait();
        let result = arc.result.write().unwrap().take().unwrap();
        result
    }
}

impl<T: Send + Sync + 'static, S: System<In = (), Out = T>> Runnable for Run<T, S> {
    fn run(&self, world: &mut World) {
        let system = world.register_system(self.system.write().unwrap().take().unwrap());
        self.result.read().unwrap().set(world.run_system(system).unwrap()).map_err(|_| "Already run").unwrap();
    }
}

pub fn tick(world: &mut World) {
    let rx = world.get_resource::<Rx>().unwrap().0.clone();
    for run in rx.lock().unwrap().try_iter() {
        run.run(world);
    }
}
