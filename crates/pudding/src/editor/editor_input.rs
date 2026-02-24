#[derive(Debug, Clone, Copy)]
pub(super) enum InputKind {
    Name,
    Command,
}

pub(super) struct InputMode {
    pub(super) kind: InputKind,
    pub(super) buffer: String,
}

impl InputMode {
    pub(super) fn for_name() -> Self {
        Self {
            kind: InputKind::Name,
            buffer: String::new(),
        }
    }

    pub(super) fn for_command() -> Self {
        Self {
            kind: InputKind::Command,
            buffer: String::new(),
        }
    }
}
