mod types;
mod validation;
mod manager;
mod history;

pub use manager::{Mutation, ProjectStateManager};
pub use types::*;
pub use validation::{validate_project_state, ValidationIssue, ValidationReport};
pub use history::UndoRedoStack;