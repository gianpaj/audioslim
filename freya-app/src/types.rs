pub const AUDIO_EXTENSIONS: &[&str] = &[
    "wav", "mp3", "ogg", "flac", "aac", "m4a", "mp4", "wma", "aiff",
];

pub const BITRATE_OPTIONS: &[&str] = &["64k", "128k", "192k", "256k", "320k"];

pub const SAMPLE_RATE_OPTIONS: &[(&str, &str)] = &[
    ("Default", "Default"),
    ("22050", "22050 Hz"),
    ("44100", "44100 Hz"),
    ("48000", "48000 Hz"),
    ("96000", "96000 Hz"),
];

pub const CHANNEL_OPTIONS: &[(&str, &str)] = &[
    ("Default", "Default"),
    ("1", "Mono"),
    ("2", "Stereo"),
];

#[derive(Debug, Clone, PartialEq, Default)]
pub enum OutputFormat {
    #[default]
    Mp3,
    Wav,
    Aac,
    Ogg,
    Flac,
    M4a,
    Mp4,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Aac => "aac",
            Self::Ogg => "ogg",
            Self::Flac => "flac",
            Self::M4a => "m4a",
            Self::Mp4 => "mp4",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::Wav => "WAV",
            Self::Aac => "AAC",
            Self::Ogg => "OGG",
            Self::Flac => "FLAC",
            Self::M4a => "M4A",
            Self::Mp4 => "MP4",
        }
    }

    pub fn all() -> &'static [OutputFormat] {
        &[
            OutputFormat::Mp3,
            OutputFormat::Wav,
            OutputFormat::Aac,
            OutputFormat::Ogg,
            OutputFormat::Flac,
            OutputFormat::M4a,
            OutputFormat::Mp4,
        ]
    }

    pub fn config(&self) -> FormatConfig {
        match self {
            Self::Mp3 => FormatConfig {
                supports_bitrate: true,
                supports_quality: true,
                quality_label: "VBR Quality (0=best, 9=worst)",
                quality_min: 0,
                quality_max: 9,
                default_quality: 2,
                default_bitrate: "192k",
            },
            Self::Wav => FormatConfig {
                supports_bitrate: false,
                supports_quality: false,
                quality_label: "",
                quality_min: 0,
                quality_max: 0,
                default_quality: 0,
                default_bitrate: "",
            },
            Self::Aac => FormatConfig {
                supports_bitrate: true,
                supports_quality: false,
                quality_label: "",
                quality_min: 0,
                quality_max: 0,
                default_quality: 0,
                default_bitrate: "192k",
            },
            Self::Ogg => FormatConfig {
                supports_bitrate: true,
                supports_quality: true,
                quality_label: "Quality (-1 to 10, higher=better)",
                quality_min: -1,
                quality_max: 10,
                default_quality: 5,
                default_bitrate: "192k",
            },
            Self::Flac => FormatConfig {
                supports_bitrate: false,
                supports_quality: true,
                quality_label: "Compression Level (0=fast, 8=small)",
                quality_min: 0,
                quality_max: 8,
                default_quality: 5,
                default_bitrate: "",
            },
            Self::M4a => FormatConfig {
                supports_bitrate: true,
                supports_quality: false,
                quality_label: "",
                quality_min: 0,
                quality_max: 0,
                default_quality: 0,
                default_bitrate: "192k",
            },
            Self::Mp4 => FormatConfig {
                supports_bitrate: true,
                supports_quality: false,
                quality_label: "",
                quality_min: 0,
                quality_max: 0,
                default_quality: 0,
                default_bitrate: "192k",
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub supports_bitrate: bool,
    pub supports_quality: bool,
    pub quality_label: &'static str,
    pub quality_min: i32,
    pub quality_max: i32,
    pub default_quality: i32,
    pub default_bitrate: &'static str,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileStatus {
    #[default]
    Pending,
    Converting,
    Done,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioFile {
    pub id: usize,
    pub path: String,
    pub name: String,
    pub status: FileStatus,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub input_size: Option<u64>,
    pub output_size: Option<u64>,
}

impl AudioFile {
    pub fn new(id: usize, path: String) -> Self {
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();
        Self {
            id,
            path,
            name,
            status: FileStatus::Pending,
            output_path: None,
            error: None,
            input_size: None,
            output_size: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionOptions {
    pub format: OutputFormat,
    pub bitrate: String,
    pub sample_rate: Option<String>,
    pub channels: Option<String>,
    pub quality: i32,
}

impl ConversionOptions {
    pub fn from_format(format: OutputFormat) -> Self {
        let config = format.config();
        Self {
            bitrate: config.default_bitrate.to_string(),
            quality: config.default_quality,
            format,
            sample_rate: None,
            channels: None,
        }
    }
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self::from_format(OutputFormat::Mp3)
    }
}

pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
