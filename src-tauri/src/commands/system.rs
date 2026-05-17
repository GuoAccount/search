#[tauri::command]
pub fn play_system_sound() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("afplay")
            .args(["/System/Library/Sounds/Pop.aiff"])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_config_file(app_handle: tauri::AppHandle) -> Result<(), String> {
    use crate::config::AppConfig;
    let path = AppConfig::config_path(&app_handle);
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path.parent().unwrap_or(&path).to_string_lossy().to_string())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn play_trash_sound() {
    let possible_paths = vec![
        std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("Resources/resources/trash.aif")),
        std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("resources/trash.aif")),
    ];
    
    let resource_path = possible_paths.into_iter()
        .flatten()
        .find(|p| p.exists());
    
    if let Some(path) = resource_path {
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
}
