//! State-level undo/redo — snapshots of ProjectState (Rule #10).

use std::collections::VecDeque;

use crate::project_state::types::ProjectState;

pub const MAX_HISTORY: usize = 50;

pub struct UndoRedoStack {
    undo: VecDeque<ProjectState>,
    redo: VecDeque<ProjectState>,
}

impl UndoRedoStack {
    pub fn new() -> Self {
        Self {
            undo: VecDeque::new(),
            redo: VecDeque::new(),
        }
    }

    pub fn push(&mut self, state: &ProjectState) {
        self.undo.push_back(state.clone());
        if self.undo.len() > MAX_HISTORY {
            self.undo.pop_front();
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, current: &ProjectState) -> Option<ProjectState> {
        let previous = self.undo.pop_back()?;
        self.redo.push_back(current.clone());
        Some(previous)
    }

    pub fn redo(&mut self, current: &ProjectState) -> Option<ProjectState> {
        let next = self.redo.pop_back()?;
        self.undo.push_back(current.clone());
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

impl Default for UndoRedoStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::default_project_settings;

    #[test]
    fn undo_redo_roundtrip() {
        let mut stack = UndoRedoStack::new();
        let mut s1 = ProjectState::new("A", "/p", default_project_settings());
        let mut s2 = s1.clone();
        s2.metadata.name = "B".into();

        stack.push(&s1);
        let restored = stack.undo(&s2).unwrap();
        assert_eq!(restored.metadata.name, "A");

        let redone = stack.redo(&s1).unwrap();
        assert_eq!(redone.metadata.name, "B");
    }
}
