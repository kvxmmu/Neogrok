#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitState {
    WaitingForPing,
    WaitingForRightsUpdate,
    WaitingForServer,

    Polling,
}
