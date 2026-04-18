pub mod manager;
pub mod commands;
pub mod weaver;
pub mod auditor;
pub mod errors;
pub mod state;
pub mod logger;
pub mod linter;

use git2::Repository;
use crate::errors::SomnusError;

pub fn get_changed_files() -> Result<Vec<String>, SomnusError> {
    let repo = Repository::open(".")?;

    let head = repo.head()?.peel_to_tree()?;

    let parent = repo
        .find_commit(repo.head()?.target().unwrap())?
        .parent(0)?
        .tree()?;
    let diff = repo.diff_tree_to_tree(Some(&parent), Some(&head), None)?;
    let mut changed_files = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                changed_files.push(path.to_string_lossy().to_string());
            }
            true
        },
        None,
        None,
        None,
    )?;
    Ok(changed_files)
}
