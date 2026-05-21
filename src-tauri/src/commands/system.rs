use std::path::PathBuf;
use tauri::Manager;

fn play_sound(path: &PathBuf) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("afplay")
            .arg(path.to_string_lossy().to_string())
            .spawn();
    }
    
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("powershell")
            .args(["-c", &format!("(New-Object Media.SoundPlayer '{}').Play()", path.to_string_lossy())])
            .spawn();
    }
    
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("aplay")
            .arg(path.to_string_lossy().to_string())
            .spawn();
    }
}

#[tauri::command]
pub fn play_system_sound(app_handle: tauri::AppHandle) -> Result<(), String> {
    let resource_dir = app_handle.path().resource_dir()
        .map_err(|e| e.to_string())?;
    let sound_path = resource_dir.join("resources").join("Pop.aiff");
    if sound_path.exists() {
        play_sound(&sound_path);
    }
    Ok(())
}

#[tauri::command]
pub fn open_config_file(app_handle: tauri::AppHandle) -> Result<(), String> {
    use crate::config::AppConfig;
    use tauri_plugin_opener::OpenerExt;
    let path = AppConfig::config_path(&app_handle);
    app_handle.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn play_trash_sound(app_handle: &tauri::AppHandle) {
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        let sound_path = resource_dir.join("resources").join("trash.aif");
        if sound_path.exists() {
            play_sound(&sound_path);
        }
    }
}
