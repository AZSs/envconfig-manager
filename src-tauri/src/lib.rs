mod commands;
mod utils;

use commands::{backup, config, env_var, profile, validation};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // 配置文件管理
            config::scan_config_files,
            config::read_config_file,
            config::write_config_file,
            config::apply_config_file,
            // 环境变量管理
            env_var::get_env_variables,
            env_var::set_env_variable,
            env_var::delete_env_variable,
            env_var::search_env_variables,
            // 备份管理
            backup::create_backup,
            backup::list_backups,
            backup::restore_backup,
            backup::delete_backup,
            backup::get_backup_config,
            // 配置校验
            validation::validate_config_file,
            validation::check_syntax,
            validation::check_path_existence,
            // 配置集管理
            profile::list_profiles,
            profile::create_profile,
            profile::update_profile,
            profile::delete_profile,
            profile::apply_profile,
            profile::diff_profile,
            profile::export_profile,
            profile::import_profile,
            profile::toggle_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
