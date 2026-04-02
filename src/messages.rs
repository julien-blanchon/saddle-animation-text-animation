use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq)]
pub enum TextAnimationAction {
    Play,
    Pause,
    Restart,
    FinishNow,
}

#[derive(Debug, Clone, Message)]
pub struct TextAnimationCommand {
    pub entity: Entity,
    pub action: TextAnimationAction,
}

#[derive(Debug, Clone, Message)]
pub struct TextAnimationStarted {
    pub entity: Entity,
}

#[derive(Debug, Clone, Message)]
pub struct TextAnimationCompleted {
    pub entity: Entity,
}

#[derive(Debug, Clone, Message)]
pub struct TextAnimationLoopFinished {
    pub entity: Entity,
    pub completed_loops: u32,
}

#[derive(Debug, Clone, Message)]
pub struct TextRevealCheckpoint {
    pub entity: Entity,
    pub revealed_units: usize,
    pub total_units: usize,
}
