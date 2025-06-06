use bevy::prelude::States;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionState {
    #[default]
    Stopped,
    Running,
    Stepping,
    Stopping,
    Finished,
}

#[allow(clippy::match_same_arms)]
impl ExecutionState {
    pub fn interactive(self) -> bool {
        match self {
            Self::Stopped => true,
            Self::Running => false,
            Self::Stepping => false,
            Self::Stopping => false,
            Self::Finished => false,
        }
    }

    pub fn can_run(self) -> bool {
        match self {
            Self::Stopped => true,
            Self::Running => false,
            Self::Stepping => true,
            Self::Stopping => false,
            Self::Finished => false,
        }
    }

    pub fn can_stop(self) -> bool {
        match self {
            Self::Stopped => false,
            Self::Running => true,
            Self::Stepping => true,
            Self::Stopping => false,
            Self::Finished => true,
        }
    }

    pub fn can_exit(self) -> bool {
        match self {
            Self::Stopped => true,
            Self::Running => false,
            Self::Stepping => false,
            Self::Stopping => false,
            Self::Finished => false,
        }
    }

    pub fn show_console(self) -> bool {
        match self {
            Self::Stopped => false,
            Self::Running => true,
            Self::Stepping => true,
            Self::Stopping => true,
            Self::Finished => true,
        }
    }

    pub fn shutdown(self) -> bool {
        match self {
            Self::Stopped => true,
            Self::Running => false,
            Self::Stepping => false,
            Self::Stopping => true,
            Self::Finished => false,
        }
    }
}
