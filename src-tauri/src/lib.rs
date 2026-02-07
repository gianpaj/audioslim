use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
    Mp3,
    Wav,
    Aac,
    Ogg,
    Flac,
    M4a,
    Mp4,
}

impl OutputFormat {
    fn extension(&self) -> &str {
        match self {
            OutputFormat::Mp3 => "mp3",
            OutputFormat::Wav => "wav",
            OutputFormat::Aac => "aac",
            OutputFormat::Ogg => "ogg",
            OutputFormat::Flac => "flac",
            OutputFormat::M4a => "m4a",
            OutputFormat::Mp4 => "mp4",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConversionOptions {
    format: OutputFormat,
    bitrate: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u8>,
    quality: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConversionProgress {
    file_path: String,
    status: String,
    output_path: Option<String>,
    error: Option<String>,
    index: usize,
    total: usize,
    input_size: Option<u64>,
    output_size: Option<u64>,
}

fn build_ffmpeg_args(input: &str, output: &str, options: &ConversionOptions) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "-i".to_string(),
        input.to_string(),
        "-y".to_string(),
    ];

    if let Some(ref bitrate) = options.bitrate {
        args.push("-b:a".to_string());
        args.push(bitrate.clone());
    }

    if let Some(sample_rate) = options.sample_rate {
        args.push("-ar".to_string());
        args.push(sample_rate.to_string());
    }

    if let Some(channels) = options.channels {
        args.push("-ac".to_string());
        args.push(channels.to_string());
    }

    if let Some(ref quality) = options.quality {
        match options.format {
            OutputFormat::Mp3 => {
                args.push("-q:a".to_string());
                args.push(quality.clone());
            }
            OutputFormat::Flac => {
                args.push("-compression_level".to_string());
                args.push(quality.clone());
            }
            OutputFormat::Ogg => {
                args.push("-q:a".to_string());
                args.push(quality.clone());
            }
            _ => {}
        }
    }

    match options.format {
        OutputFormat::Aac => {
            args.push("-c:a".to_string());
            args.push("aac".to_string());
        }
        OutputFormat::M4a => {
            args.push("-c:a".to_string());
            args.push("aac".to_string());
        }
        OutputFormat::Mp4 => {
            args.push("-c:a".to_string());
            args.push("aac".to_string());
            args.push("-vn".to_string());
        }
        OutputFormat::Ogg => {
            args.push("-c:a".to_string());
            args.push("libvorbis".to_string());
        }
        _ => {}
    }

    args.push(output.to_string());
    args
}

#[tauri::command]
fn check_ffmpeg() -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| {
            "ffmpeg is not installed or not found in PATH. Please install ffmpeg to use AudioSlim."
                .to_string()
        })?;

    if output.status.success() {
        let version_string = String::from_utf8_lossy(&output.stdout);
        let first_line = version_string
            .lines()
            .next()
            .unwrap_or("ffmpeg found")
            .to_string();
        Ok(first_line)
    } else {
        Err("ffmpeg was found but returned an error.".to_string())
    }
}

#[tauri::command]
async fn convert_audio(
    app: AppHandle,
    input_paths: Vec<String>,
    options: ConversionOptions,
) -> Result<Vec<String>, String> {
    let total = input_paths.len();
    let mut output_paths: Vec<String> = Vec::with_capacity(total);

    for (index, input_path) in input_paths.iter().enumerate() {
        let input = Path::new(input_path);

        // Get input file size
        let input_size = fs::metadata(input).ok().map(|m| m.len());

        // Emit "converting" status
        let _ = app.emit(
            "conversion-progress",
            ConversionProgress {
                file_path: input_path.clone(),
                status: "converting".to_string(),
                output_path: None,
                error: None,
                index,
                total,
                input_size,
                output_size: None,
            },
        );

        // Validate input file exists
        if !input.exists() {
            let _ = app.emit(
                "conversion-progress",
                ConversionProgress {
                    file_path: input_path.clone(),
                    status: "error".to_string(),
                    output_path: None,
                    error: Some(format!("File not found: {}", input_path)),
                    index,
                    total,
                    input_size: None,
                    output_size: None,
                },
            );
            continue;
        }

        // Determine output path
        let stem = input
            .file_stem()
            .ok_or_else(|| format!("Invalid file path: {}", input_path))?
            .to_string_lossy();

        let out_dir = input
            .parent()
            .ok_or_else(|| format!("Cannot determine parent directory for: {}", input_path))?
            .to_path_buf();

        let mut output_file = out_dir.join(format!("{}.{}", stem, options.format.extension()));

        // Handle same-format collision (input == output)
        if output_file == input {
            output_file = out_dir.join(format!("{}_converted.{}", stem, options.format.extension()));
        }

        let output_str = output_file.to_string_lossy().to_string();
        let args = build_ffmpeg_args(input_path, &output_str, &options);

        let result = tauri::async_runtime::spawn_blocking(move || {
            Command::new("ffmpeg").args(&args).output()
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

        if result.status.success() {
            let output_size = fs::metadata(&output_file).ok().map(|m| m.len());
            let _ = app.emit(
                "conversion-progress",
                ConversionProgress {
                    file_path: input_path.clone(),
                    status: "done".to_string(),
                    output_path: Some(output_str.clone()),
                    error: None,
                    index,
                    total,
                    input_size,
                    output_size,
                },
            );
            output_paths.push(output_str);
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            let _ = app.emit(
                "conversion-progress",
                ConversionProgress {
                    file_path: input_path.clone(),
                    status: "error".to_string(),
                    output_path: None,
                    error: Some(stderr.clone()),
                    index,
                    total,
                    input_size,
                    output_size: None,
                },
            );
        }
    }

    Ok(output_paths)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![check_ffmpeg, convert_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
