// Hugo Bros - Tauri Backend

mod commands;
mod config;
mod files;
mod frontmatter_config;
mod hugo;
mod markdown;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            select_project_folder,
            get_project_config,
            get_frontmatter_config,
            generate_frontmatter_config_command,
            list_posts,
            get_post,
            save_post,
            create_post,
            delete_post,
            list_pages,
            create_page,
            get_page,
            save_page,
            delete_page,
            list_drafts,
            create_draft,
            get_draft,
            save_draft,
            delete_draft,
            list_images,
            list_static_entries,
            create_static_folder,
            delete_static_entry,
            copy_image_to_project,
            delete_image,
            get_app_config,
            save_app_config,
            run_hugo_command,
            start_hugo_server,
            stop_hugo_server,
            is_hugo_server_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
