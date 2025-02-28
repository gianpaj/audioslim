// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
// pub fn run() {
//     tauri::Builder::default()

//         .plugin(tauri_plugin_shell::init())
//         .invoke_handler(tauri::generate_handler![greet])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }


#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
// use tauri::api::path;

#[tauri::command]
async fn convert_to_mp3(input_path: String) -> Result<String, String> {
    let output_path = input_path.replace(".wav", ".mp3");
    
    let ffmpeg_output = Command::new("ffmpeg")
        .arg("-i")
        .arg(&input_path)
        .arg(&output_path)
        .output()
        .map_err(|e| e.to_string())?;

    if ffmpeg_output.status.success() {
        Ok(output_path)
    } else {
        Err(String::from_utf8_lossy(&ffmpeg_output.stderr).to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![convert_to_mp3])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}