#[tauri::command]
pub fn play_system_sound() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("afplay")
            .arg("/System/Library/Sounds/Ping.aiff")
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
