//! Native Packwand desktop host. The webview talks only through Tauri IPC;
//! local helper processes remain behind Rust command boundaries.

#![deny(unsafe_code)]

mod commands;
mod error;
mod events;
mod fsutil;
mod kernel;
mod raw_input;
mod state;

use commands::{
	accounts, api, automation, changes, collab, diagnostics, editor, exports, extensions, git,
	identity, instances, jobs, mods, packeater, packs, projects, providers, settings, shell,
	social, somnus, themes, workspace,
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
			changes::changes_enable,
			changes::changes_log,
			changes::changes_new,
			changes::changes_describe,
			changes::changes_squash,
			identity::account_login,
			identity::account_whoami,
			identity::account_logout,
			social::social_friends,
			social::social_pending_invites,
			social::social_linked_tangled_repos,
			social::social_send_invite,
			social::social_share_pack,
			social::social_share_snippet,
			social::social_share_image,
			collab::collab_host_start,
			collab::collab_host_stop,
			collab::collab_join,
			collab::collab_leave,
			collab::collab_state,
			collab::collab_set_identity,
			collab::collab_set_git_write,
			collab::collab_fs_request,
			collab::collab_git_request,
			collab::collab_document_open,
			collab::collab_document_close,
			collab::collab_document_snapshot,
			collab::collab_document_edit,
			collab::collab_document_save,
			collab::collab_presence,
			collab::collab_follow,
			collab::collab_output,
			collab::collab_problems,
			collab::collab_job_event,
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
			git::git_repository,
			git::git_init,
			git::git_clone,
			git::git_remote_add,
			git::git_set_identity,
			git::git_fetch,
			git::git_pull,
			git::git_push,
			git::git_branches,
			git::git_checkout,
			git::git_log,
			accounts::accounts_state,
			accounts::accounts_link_modrinth,
			accounts::accounts_link_curseforge,
			accounts::accounts_set_publish_token,
			accounts::accounts_unlink,
			accounts::accounts_prepare_publish,
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
			providers::providers_creator,
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
			somnus::somnus_run,
			somnus::somnus_list,
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
			instances::instances_manual_pending,
			instances::instances_manual_provide,
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
