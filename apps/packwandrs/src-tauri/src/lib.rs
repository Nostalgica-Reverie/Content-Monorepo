//! Native Packwand desktop host. The webview talks only through Tauri IPC;
//! no loopback server or Go subprocess participates in the runtime path.

#![deny(unsafe_code)]

mod commands;
mod error;
mod events;
mod fsutil;
mod kernel;
mod raw_input;
mod state;

use commands::{
    api, automation, diagnostics, editor, exports, extensions, git, instances, jobs, mods,
    packeater, packs, projects, providers, settings, shell, themes, workspace,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = state::AppState::load(app.handle())?;
            app.manage(state);
            // Restore live indexing for a workspace remembered from a previous run.
            if let Some(root) = app.state::<state::AppState>().settings()?.workspace_path
                && let Err(error) = app
                    .state::<state::AppState>()
                    .restart_watch(app.handle(), std::path::Path::new(&root))
            {
                eprintln!("could not restore workspace watcher: {error}");
            }
            // Starts the bounded Rust trace drain used by the output dock.
            kernel::start(app.handle());
            raw_input::start(app.handle())?;
            if app.state::<state::AppState>().settings()?.raw_input_enabled
                && let Err(error) = raw_input::set_enabled(app.handle(), true)
            {
                eprintln!("could not restore Raw Input: {error}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            workspace::workspace_get,
            workspace::workspace_set,
            workspace::workspace_select,
            workspace::select_workspace,
            workspace::backend_url,
            workspace::workspace_sync_preview,
            workspace::workspace_sync,
            automation::automation_plan,
            automation::automation_run,
            api::api_contract,
            api::api_inspect,
            shell::shell_exec,
            shell::shell_parse,
            git::git_status,
            git::git_stage,
            git::git_unstage,
            git::git_diff,
            git::git_diff_document,
            git::git_commit,
            extensions::extension_kubejs_scripts,
            extensions::extension_kubejs_validate,
            extensions::extension_language_snapshot,
            extensions::extension_recipes,
            extensions::extension_pack_graph,
            extensions::extension_language_files,
            extensions::extension_worldgen_assets,
            extensions::extension_content_lint,
            extensions::extension_registries,
            extensions::extension_krita_assets,
            extensions::extension_krita_open,
            extensions::extension_blockbench_assets,
            extensions::extension_blockbench_open,
            settings::settings_get,
            settings::settings_update,
            themes::themes_list,
            themes::themes_save,
            themes::themes_delete,
            raw_input::raw_input_set_enabled,
            packs::packs_list,
            packs::packs_get,
            packs::packs_manifest_get,
            packs::packs_manifest_put,
            packs::packs_changelog_get,
            packs::packs_changelog_put,
            packs::packs_icon,
            projects::projects_list,
            projects::projects_get,
            projects::projects_create,
            projects::projects_manifest_update,
            projects::projects_bump,
            projects::projects_freeze,
            providers::providers_resolve,
            providers::providers_add,
            providers::providers_browse,
            providers::providers_open_page,
            providers::providers_project,
            mods::mods_list,
            mods::mods_add,
            mods::mods_remove,
            mods::mods_update,
            mods::mods_refresh,
            mods::mods_pin,
            mods::mods_side_get,
            mods::mods_side_set,
            editor::editor_tree,
            editor::editor_document_read,
            editor::editor_document_write,
            editor::editor_search,
            editor::editor_file_read,
            editor::editor_file_write,
            editor::editor_create,
            editor::editor_fs_stat,
            editor::editor_fs_read_dir,
            editor::editor_fs_read_file,
            editor::editor_fs_write_file,
            editor::editor_fs_create_dir,
            editor::editor_fs_delete,
            editor::editor_fs_rename,
            diagnostics::diagnostics_lint,
            diagnostics::diagnostics_validate,
            diagnostics::diagnostics_parity,
            diagnostics::diagnostics_content_lint,
            diagnostics::diagnostics_preflight,
            diagnostics::diagnostics_registries,
            diagnostics::diagnostics_installer_test,
            jobs::jobs_list,
            jobs::jobs_get,
            jobs::jobs_cancel,
            jobs::jobs_start_demo,
            instances::instances_list,
            instances::instances_get,
            instances::instances_icon,
            instances::instances_image,
            instances::instances_create,
            instances::instances_import,
            instances::instances_export,
            instances::instances_edit,
            instances::instances_delete,
            instances::instances_install,
            instances::instances_content_list,
            instances::instances_content_toggle,
            instances::instances_content_remove,
            instances::instances_files_list,
            instances::instances_file_read,
            instances::instances_file_write,
            instances::instances_status_list,
            instances::instances_launch,
            instances::instances_stop,
            exports::exports_publish_plan,
            exports::exports_build,
            exports::exports_publish_targets,
            exports::exports_publish_inspect,
            exports::exports_publish_build,
            exports::exports_publish_upload,
            exports::exports_publish_verify,
            packeater::packeater_markers,
            packeater::packeater_preview,
            packeater::packeater_initialize,
            packeater::packeater_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Packwand");
}
