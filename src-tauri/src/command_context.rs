use crate::app_paths::AppPaths;
use crate::error::{Result, SkillMasterError};
use crate::models::AppState;
use crate::state_store::{load_or_create_state, save_state, LoadedState, StateLoadStatus};
use tauri::{AppHandle, Manager};

pub(crate) struct CommandState {
    pub paths: AppPaths,
    pub state: AppState,
    pub load_status: StateLoadStatus,
}

fn command_paths(app: &AppHandle) -> Result<AppPaths> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| SkillMasterError::InvalidPath(error.to_string()))?;
    Ok(AppPaths::from_config_dir(&config_dir))
}

pub(crate) fn load_command_state(app: &AppHandle) -> Result<CommandState> {
    let paths = command_paths(app)?;
    let LoadedState { state, load_status } =
        load_or_create_state(&paths.state_file, &paths.skill_library)?;
    Ok(CommandState {
        paths,
        state,
        load_status,
    })
}

pub(crate) fn persist(paths: &AppPaths, state: &AppState) -> Result<()> {
    save_state(&paths.state_file, state)
}
