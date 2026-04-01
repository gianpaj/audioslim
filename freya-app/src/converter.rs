use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use crate::types::{ConversionOptions, OutputFormat};

pub async fn resolve_ffmpeg() -> Option<String> {
    // Check environment override first
    if let Ok(path) = std::env::var("AUDIOSLIM_FFMPEG_PATH") {
        eprintln!("[converter] Trying AUDIOSLIM_FFMPEG_PATH: {}", path);
        if tokio::fs::metadata(&path).await.is_ok() {
            return Some(path);
        }
    }

    // Bundled binary (relative to executable)
    if let Ok(exe) = std::env::current_exe() {
        let bundled = exe
            .parent()
            .map(|p| p.join("../Resources/bin/ffmpeg"))
            .and_then(|p| p.canonicalize().ok());
        if let Some(path) = bundled {
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    // Local vendor path (dev fallback)
    let vendor = Path::new("vendors/ffmpeg/macos-arm64/ffmpeg");
    if vendor.exists() {
        return Some(vendor.to_string_lossy().to_string());
    }

    // macOS system paths
    for candidate in &[
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/local/bin/ffmpeg",
    ] {
        if tokio::fs::metadata(candidate).await.is_ok() {
            eprintln!("[converter] Found ffmpeg at system path: {}", candidate);
            return Some(candidate.to_string());
        }
    }

    // Fallback to PATH
    if let Ok(output) = std::process::Command::new("which").arg("ffmpeg").output() {
        if output.status.success() {
            let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !p.is_empty() {
                eprintln!("[converter] Found ffmpeg via PATH: {}", p);
                return Some(p);
            }
        }
    }

    eprintln!("[converter] FFmpeg not found anywhere");
    None
}

pub async fn check_ffmpeg() -> Result<String, String> {
    let ffmpeg = resolve_ffmpeg()
        .await
        .ok_or_else(|| "FFmpeg not found. Install via Homebrew: brew install ffmpeg".to_string())?;

    let output = Command::new(&ffmpeg)
        .arg("-version")
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.lines().next().unwrap_or("ffmpeg (version unknown)");
    Ok(version.to_string())
}

pub fn build_output_path(input: &str, format: &OutputFormat) -> String {
    let p = Path::new(input);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let dir = p
        .parent()
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let ext = format.as_str();

    let input_ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let filename = if input_ext == ext {
        format!("{}_converted.{}", stem, ext)
    } else {
        format!("{}.{}", stem, ext)
    };

    dir.join(filename).to_string_lossy().to_string()
}

pub fn build_ffmpeg_args(input: &str, output: &str, opts: &ConversionOptions) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "-i".into(),
        input.into(),
        "-y".into(),
    ];

    // Bitrate
    if !opts.bitrate.is_empty() && opts.format.config().supports_bitrate {
        args.push("-b:a".into());
        args.push(opts.bitrate.clone());
    }

    // Sample rate
    if let Some(ref sr) = opts.sample_rate {
        args.push("-ar".into());
        args.push(sr.clone());
    }

    // Channels
    if let Some(ref ch) = opts.channels {
        args.push("-ac".into());
        args.push(ch.clone());
    }

    // Quality
    if opts.format.config().supports_quality {
        match opts.format {
            OutputFormat::Mp3 | OutputFormat::Ogg => {
                args.push("-q:a".into());
                args.push(opts.quality.to_string());
            }
            OutputFormat::Flac => {
                args.push("-compression_level".into());
                args.push(opts.quality.to_string());
            }
            _ => {}
        }
    }

    // Codec overrides
    match opts.format {
        OutputFormat::Aac | OutputFormat::M4a => {
            args.push("-c:a".into());
            args.push("aac".into());
        }
        OutputFormat::Mp4 => {
            args.push("-c:a".into());
            args.push("aac".into());
            args.push("-vn".into());
        }
        OutputFormat::Ogg => {
            args.push("-c:a".into());
            args.push("libvorbis".into());
        }
        _ => {}
    }

    args.push(output.into());
    args
}

/// Returns output paths that already exist on disk (would be overwritten).
pub fn find_overwrite_targets(input_paths: &[String], format: &OutputFormat) -> Vec<String> {
    input_paths
        .iter()
        .map(|p| build_output_path(p, format))
        .filter(|out| std::path::Path::new(out).exists())
        .collect()
}

pub struct ConversionResult {
    pub success: bool,
    pub output_path: String,
    pub error: Option<String>,
    pub input_size: u64,
    pub output_size: u64,
}

pub async fn convert_audio(input: &str, opts: &ConversionOptions) -> ConversionResult {
    let output_path = build_output_path(input, &opts.format);

    let input_size = tokio::fs::metadata(input)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    let ffmpeg = match resolve_ffmpeg().await {
        Some(p) => p,
        None => {
            return ConversionResult {
                success: false,
                output_path,
                error: Some("FFmpeg not found".to_string()),
                input_size,
                output_size: 0,
            }
        }
    };

    let args = build_ffmpeg_args(input, &output_path, opts);
    eprintln!("[converter] Running: {} {}", ffmpeg, args.join(" "));

    let result = Command::new(&ffmpeg)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match result {
        Ok(out) if out.status.success() => {
            let output_size = tokio::fs::metadata(&output_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0);
            ConversionResult {
                success: true,
                output_path,
                error: None,
                input_size,
                output_size,
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            ConversionResult {
                success: false,
                output_path,
                error: Some(stderr),
                input_size,
                output_size: 0,
            }
        }
        Err(e) => ConversionResult {
            success: false,
            output_path,
            error: Some(format!("Failed to spawn ffmpeg: {}", e)),
            input_size,
            output_size: 0,
        },
    }
}
