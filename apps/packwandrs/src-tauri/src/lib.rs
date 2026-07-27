//! Native Packwand desktop host. The webview talks only through Tauri IPC;
//! no loopback server or Go subprocess participates in the runtime path.

#![forbid(unsafe_code)]

mod commands;
mod error;
mod events;
mod fsutil;
mod kernel;
mod state;

use commands::{
    api, automation, diagnostics, editor, exports, extensions, git, instances, jobs, mods,
    packeater, packs, projects, providers, settings, shell, workspace,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = state::AppState::load(app.handle())?;
            app.manage(state);
            // Brings up the packwandc native core and starts draining its
            // trace ring into the output dock. Deliberately infallible: the
            // workbench is useful without the C layer, so a failed native boot
            // degrades rather than aborting startup (see kernel.rs).
            kernel::start(app.handle());
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
            mods::mods_list,
            mods::mods_add,
            mods::mods_remove,
            mods::mods_update,
            mods::mods_refresh,
            mods::mods_pin,
            mods::mods_side_get,
            mods::mods_side_set,
            editor::editor_tree,
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
