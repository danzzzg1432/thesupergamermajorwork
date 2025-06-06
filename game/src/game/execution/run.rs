use pyo3::Bound;
use pyo3::types::PyString;
use bevy::prelude::{error, NextState, ResMut, Resource, State};
use pyo3::types::PyDict;
use bevy::prelude::Res;
use bevy::prelude::{Commands, In};
use pyo3::types::PyAnyMethods;
use pyo3::Python;
use std::ffi::{c_int, c_ulong, CString};
use std::sync::{Arc, Condvar, Mutex, OnceLock, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use bevy::log::debug;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use pyo3::ffi::{PyExc_KeyboardInterrupt, PyObject};
use crate::game::execution::channel::Run;
use crate::game::execution::execution_state::ExecutionState;

const TICK_SPEED: Duration = Duration::from_millis(500);
static NEXT_TICK: RwLock<Option<Instant>> = RwLock::new(None);
pub static STEPPER: Stepper = Stepper::new();

extern "C" {
    fn PyThread_get_thread_ident() -> c_ulong;
    fn PyThreadState_SetAsyncExc(id: c_ulong, exc: *mut PyObject) -> c_int;
}

pub struct Stepper {
    waiting: Mutex<bool>,
    condvar: Condvar,
    skip: AtomicBool,
}

impl Stepper {
    pub const fn new() -> Self {
        Stepper {
            waiting: Mutex::new(false),
            condvar: Condvar::new(),
            skip: AtomicBool::new(false),
        }
    }

    pub fn is_waiting(&self) -> bool {
        *self.waiting.lock().unwrap()
    }

    pub fn wake(&self) {
        self.waiting.set(false).unwrap();
        self.condvar.notify_all();
    }

    pub fn skip(&self) {
        self.skip.store(true, Ordering::Relaxed);
    }

    pub fn wait(&self) {
        if self.skip.swap(false, Ordering::Relaxed) {
            return;
        }
        self.waiting.set(true).unwrap();
        drop(self.condvar.wait_while(self.waiting.lock().unwrap(), |waiting| *waiting).unwrap());
    }
}

#[derive(Debug, Resource)]
pub struct PythonTask {
    task: Task<()>,
    thread: u64,
}

pub fn run(
    code: In<String>,
    mut commands: Commands,
    task: Option<Res<PythonTask>>,
) {
    assert!(task.is_none());
    let code = CString::new(&**code).unwrap();
    let thread = Arc::new(OnceLock::new());
    let thread2 = thread.clone();
    let task = AsyncComputeTaskPool::get().spawn(async move {
        Python::with_gil(move |py| {
            let pythoneer = py.import("pythoneer").unwrap();
            let globals = PyDict::new(py);
            for item in pythoneer.getattr("__all__").unwrap().extract::<Vec<Bound<PyString>>>().unwrap() {
                globals.set_item(&item, pythoneer.getattr(&item).unwrap()).unwrap();
            }
            thread2.set(unsafe { PyThread_get_thread_ident() }).unwrap();
            if let Err(e) = py.run(&code, Some(&globals), None) {
                error!("{e}");
                e.display(py);
            }
        });
    });
    thread.wait();
    commands.insert_resource(PythonTask { task, thread: *thread.get().unwrap() });
}

pub fn watch(
    mut commands: Commands,
    task: Option<Res<PythonTask>>,
    execution: Res<State<ExecutionState>>,
    mut next_execution: ResMut<NextState<ExecutionState>>,
) {
    let Some(task) = task else { return; };
    if !task.task.is_finished() {
        if execution.shutdown() {
            debug!("Sending KeyboardInterrupt");
            Python::with_gil(|_py| {
                unsafe {
                    PyThreadState_SetAsyncExc(task.thread, PyExc_KeyboardInterrupt);
                }
            });
            STEPPER.wake();
        }
        return;
    }
    debug!("Python exited");
    commands.remove_resource::<PythonTask>();
    if execution.shutdown() {
        next_execution.set(ExecutionState::Stopped);
    } else {
        next_execution.set(ExecutionState::Finished);
    }
}

pub fn tick() {
    if Run::new(|state: Res<State<ExecutionState>>| **state).execute() == ExecutionState::Stepping {
        STEPPER.wait();
    } else {
        if let Some(next_tick) = *NEXT_TICK.read().unwrap() && next_tick > Instant::now() {
            thread::sleep_until(next_tick);
        }
        reset_tick();
    }
}

pub fn reset_tick() {
    *NEXT_TICK.write().unwrap() = Some(Instant::now() + TICK_SPEED);
}
