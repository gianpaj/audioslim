use freya::prelude::*;

mod converter;
mod types;

use types::*;

// ─── Shared async batch runner ───────────────────────────────────────────────
// Called from both the direct-convert and the overwrite-confirmed paths.

async fn run_batch(
    mut files: Signal<Vec<AudioFile>>,
    mut app_error: Signal<Option<String>>,
    mut toast: Signal<Option<String>>,
    mut is_converting: Signal<bool>,
    file_list: Vec<AudioFile>,
    opts: ConversionOptions,
) {
    // Verify FFmpeg before starting
    if let Err(e) = converter::check_ffmpeg().await {
        eprintln!("[AudioSlim] FFmpeg check failed: {}", e);
        app_error.set(Some(format!("FFmpeg error: {}", e)));
        let mut current = files.read().clone();
        for f in current.iter_mut() {
            f.status = FileStatus::Error;
            f.error = Some(e.clone());
        }
        files.set(current);
        is_converting.set(false);
        return;
    }

    for file_data in &file_list {
        let path = file_data.path.clone();
        eprintln!("[AudioSlim] Converting: {}", path);

        {
            let mut current = files.read().clone();
            if let Some(f) = current.iter_mut().find(|f| f.path == path) {
                f.status = FileStatus::Converting;
            }
            files.set(current);
        }

        let result = converter::convert_audio(&path, &opts).await;

        {
            let mut current = files.read().clone();
            if let Some(f) = current.iter_mut().find(|f| f.path == path) {
                if result.success {
                    eprintln!("[AudioSlim] Done: {} → {} bytes", path, result.output_size);
                    f.status = FileStatus::Done;
                    f.output_path = Some(result.output_path);
                    f.input_size = Some(result.input_size);
                    f.output_size = Some(result.output_size);
                } else {
                    eprintln!("[AudioSlim] Error on {}: {:?}", path, result.error);
                    f.status = FileStatus::Error;
                    f.error = result.error.clone();
                    if app_error.read().is_none() {
                        app_error.set(result.error.map(|e| {
                            let short = if e.len() > 200 { e[..200].to_string() } else { e };
                            format!("Conversion failed: {}", short)
                        }));
                    }
                }
            }
            files.set(current);
        }
    }

    is_converting.set(false);
    eprintln!("[AudioSlim] Batch done");

    let final_files = files.read().clone();
    let done_count = final_files.iter().filter(|f| f.status == FileStatus::Done).count();
    let err_count = final_files.iter().filter(|f| f.status == FileStatus::Error).count();
    eprintln!("[AudioSlim] Results: {} done, {} errors", done_count, err_count);

    if done_count > 0 {
        let msg = if err_count > 0 {
            format!("{} converted  ·  {} failed", done_count, err_count)
        } else {
            format!(
                "{} file{} converted successfully",
                done_count,
                if done_count == 1 { "" } else { "s" }
            )
        };
        toast.set(Some(msg));
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
            toast.set(None);
        });
    }
}

// ─── Entry point ────────────────────────────────────────────────────────────

fn main() {
    launch_cfg(
        App,
        LaunchConfig::<()>::new()
            .with_title("AudioSlim")
            .with_size(900.0, 700.0),
    );
}

// ─── Root component ─────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    let mut files: Signal<Vec<AudioFile>> = use_signal(Vec::new);
    let mut format = use_signal(OutputFormat::default);
    let mut opts = use_signal(ConversionOptions::default);
    let mut is_converting = use_signal(|| false);
    let mut ffmpeg_version: Signal<Option<String>> = use_signal(|| None);
    let mut ffmpeg_error: Signal<Option<String>> = use_signal(|| None);
    let mut app_error: Signal<Option<String>> = use_signal(|| None);
    let mut toast: Signal<Option<String>> = use_signal(|| None);
    let mut next_id: Signal<usize> = use_signal(|| 0);
    // Overwrite confirmation: holds the output paths that already exist
    let mut overwrite_targets: Signal<Vec<String>> = use_signal(Vec::new);
    // Snapshot kept while waiting for user confirmation
    let mut pending_files: Signal<Vec<AudioFile>> = use_signal(Vec::new);
    let mut pending_opts: Signal<ConversionOptions> = use_signal(ConversionOptions::default);

    // Check FFmpeg availability on startup
    use_effect(move || {
        spawn(async move {
            eprintln!("[AudioSlim] Checking FFmpeg...");
            match converter::check_ffmpeg().await {
                Ok(v) => { eprintln!("[AudioSlim] FFmpeg OK: {v}"); ffmpeg_version.set(Some(v)); }
                Err(e) => { eprintln!("[AudioSlim] FFmpeg error: {e}"); ffmpeg_error.set(Some(e)); }
            }
        });
    });

    // File picker
    let pick_files = move |_| {
        spawn(async move {
            let handles = rfd::AsyncFileDialog::new()
                .add_filter("Audio Files", &["wav","mp3","ogg","flac","aac","m4a","mp4","wma","aiff"])
                .set_title("Select Audio Files")
                .pick_files()
                .await;
            if let Some(handles) = handles {
                let mut current = files.read().clone();
                let mut id = *next_id.read();
                for h in handles {
                    let path = h.path().to_string_lossy().to_string();
                    eprintln!("[AudioSlim] File selected: {path}");
                    if !current.iter().any(|f| f.path == path) {
                        current.push(AudioFile::new(id, path));
                        id += 1;
                    }
                }
                files.set(current);
                next_id.set(id);
            }
        });
    };

    let remove_file = move |id: usize| {
        let mut current = files.read().clone();
        current.retain(|f| f.id != id);
        files.set(current);
    };

    let clear_files = move |_| {
        app_error.set(None);
        toast.set(None);
        overwrite_targets.set(Vec::new());
        files.set(Vec::new());
    };

    let change_format = move |new_format: OutputFormat| {
        opts.set(ConversionOptions::from_format(new_format.clone()));
        format.set(new_format);
    };

    // Helper: build reset file list from current files signal
    let make_reset = |snapshot: &[AudioFile]| -> Vec<AudioFile> {
        snapshot.iter().map(|f| AudioFile {
            status: FileStatus::Pending,
            output_path: None,
            error: None,
            input_size: None,
            output_size: None,
            ..f.clone()
        }).collect()
    };

    // ── Convert button clicked ───────────────────────────────────────────────
    let on_convert = move |_| {
        let snapshot = files.read().clone();
        let current_opts = opts.read().clone();

        eprintln!("[AudioSlim] Convert clicked. {} file(s)", snapshot.len());

        if snapshot.is_empty() || *is_converting.read() { return; }

        if let Some(ref err) = *ffmpeg_error.read() {
            app_error.set(Some(format!("FFmpeg not available: {}", err)));
            return;
        }

        app_error.set(None);
        toast.set(None);

        let input_paths: Vec<String> = snapshot.iter().map(|f| f.path.clone()).collect();
        let conflicts = converter::find_overwrite_targets(&input_paths, &current_opts.format);

        if !conflicts.is_empty() {
            eprintln!("[AudioSlim] Overwrite confirmation needed: {:?}", conflicts);
            // Stash data and show dialog — don't start conversion yet
            let reset = make_reset(&snapshot);
            pending_files.set(reset);
            pending_opts.set(current_opts);
            overwrite_targets.set(conflicts);
        } else {
            // No conflicts — convert immediately
            let reset = make_reset(&snapshot);
            files.set(reset.clone());
            is_converting.set(true);
            spawn(run_batch(files, app_error, toast, is_converting, reset, current_opts));
        }
    };

    // ── User confirmed overwrite ─────────────────────────────────────────────
    let on_confirm_overwrite = move |_| {
        let file_list = pending_files.read().clone();
        let current_opts = pending_opts.read().clone();
        overwrite_targets.set(Vec::new());
        files.set(file_list.clone());
        is_converting.set(true);
        spawn(run_batch(files, app_error, toast, is_converting, file_list, current_opts));
    };

    let on_cancel_overwrite = move |_| {
        overwrite_targets.set(Vec::new());
        pending_files.set(Vec::new());
    };

    // ── Render snapshots ─────────────────────────────────────────────────────
    let file_count = files.read().len();
    let converting = *is_converting.read();
    let current_format = format.read().clone();
    let current_opts = opts.read().clone();
    let version = ffmpeg_version.read().clone();
    let ffmpeg_err = ffmpeg_error.read().clone();
    let conv_err = app_error.read().clone();
    let toast_msg = toast.read().clone();
    let overwrite_list = overwrite_targets.read().clone();
    let ffmpeg_missing = ffmpeg_err.is_some();

    rsx!(
        rect {
            width: "fill",
            height: "fill",
            background: "rgb(242, 242, 247)",
            direction: "vertical",

            AppHeader { version, ffmpeg_error: ffmpeg_err.clone() }

            if let Some(ref err) = ffmpeg_err {
                ErrorBanner {
                    message: format!("FFmpeg not found: {}  ·  brew install ffmpeg", err),
                }
            }

            ScrollView {
                width: "fill",
                height: "fill",
                direction: "vertical",
                show_scrollbar: true,

                rect {
                    width: "fill",
                    direction: "vertical",
                    padding: "20",
                    spacing: "16",
                    content: "fit",

                    DropZoneArea { on_pick: pick_files }

                    if file_count > 0 {
                        FileListArea {
                            files: files.read().clone(),
                            on_remove: remove_file,
                            on_clear: clear_files,
                        }
                    }

                    FormatSelectorArea { selected: current_format, on_change: change_format }

                    OptionsPanelArea {
                        opts: current_opts,
                        on_change: move |new_opts| opts.set(new_opts),
                    }

                    ConvertButtonArea {
                        file_count,
                        is_converting: converting,
                        ffmpeg_missing,
                        on_convert,
                    }

                    if let Some(ref err) = conv_err {
                        ErrorBanner { message: err.clone() }
                    }
                }
            }

            // ── Overlays (rendered on top via position: "global") ────────
            if let Some(ref msg) = toast_msg {
                Toast {
                    message: msg.clone(),
                    on_dismiss: move |_| toast.set(None),
                }
            }

            if !overwrite_list.is_empty() {
                OverwriteDialog {
                    paths: overwrite_list,
                    on_confirm: on_confirm_overwrite,
                    on_cancel: on_cancel_overwrite,
                }
            }
        }
    )
}

// ─── Header ─────────────────────────────────────────────────────────────────

#[component]
fn AppHeader(version: Option<String>, ffmpeg_error: Option<String>) -> Element {
    let (status_text, status_color) = match (&version, &ffmpeg_error) {
        (Some(v), _) => (v.clone(), "rgb(142, 142, 147)"),
        (_, Some(_)) => ("FFmpeg not found".to_string(), "rgb(255, 59, 48)"),
        _ => ("Checking FFmpeg...".to_string(), "rgb(174, 174, 178)"),
    };

    rsx!(
        rect {
            width: "fill",
            height: "60",
            background: "rgb(255, 255, 255)",
            shadow: "0 1 4 0 rgb(0, 0, 0, 0.08)",
            padding: "0 24",
            direction: "horizontal",
            cross_align: "center",

            // Title + status stacked vertically on the left
            rect {
                direction: "vertical",
                spacing: "2",

                label {
                    font_size: "18",
                    font_weight: "bold",
                    color: "rgb(0, 0, 0)",
                    "AudioSlim"
                }

                label {
                    font_size: "11",
                    color: "{status_color}",
                    "{status_text}"
                }
            }
        }
    )
}

#[component]
fn ErrorBanner(message: String) -> Element {
    rsx!(
        rect {
            width: "fill",
            background: "rgb(255, 59, 48, 0.1)",
            border: "1 solid rgb(255, 59, 48, 0.3)",
            corner_radius: "10",
            padding: "12 16",
            direction: "horizontal",
            spacing: "8",
            cross_align: "center",

            label {
                font_size: "14",
                "⚠"
            }

            label {
                font_size: "12",
                color: "rgb(180, 0, 0)",
                "{message}"
            }
        }
    )
}

// ─── Overwrite confirmation dialog ───────────────────────────────────────────

#[component]
fn OverwriteDialog(
    paths: Vec<String>,
    on_confirm: EventHandler<MouseEvent>,
    on_cancel: EventHandler<MouseEvent>,
) -> Element {
    rsx!(
        // Full-screen dimmed backdrop
        rect {
            position: "global",
            position_top: "0",
            position_left: "0",
            width: "fill",
            height: "fill",
            background: "rgb(0, 0, 0, 0.45)",
            main_align: "center",
            cross_align: "center",

            // Dialog card
            rect {
                width: "480",
                background: "rgb(255, 255, 255)",
                corner_radius: "16",
                padding: "28",
                direction: "vertical",
                spacing: "16",
                shadow: "0 20 60 0 rgb(0, 0, 0, 0.25)",

                // Title
                label {
                    font_size: "17",
                    font_weight: "bold",
                    color: "rgb(0, 0, 0)",
                    "Overwrite existing files?"
                }

                // Subtitle
                label {
                    font_size: "13",
                    color: "rgb(100, 100, 100)",
                    "The following output files already exist and will be replaced:"
                }

                // Path list
                rect {
                    width: "fill",
                    background: "rgb(242, 242, 247)",
                    corner_radius: "8",
                    padding: "12",
                    direction: "vertical",
                    spacing: "6",

                    for path in paths.iter() {
                        label {
                            font_size: "11",
                            color: "rgb(60, 60, 67)",
                            "• {path}"
                        }
                    }
                }

                // Buttons row
                rect {
                    width: "fill",
                    direction: "horizontal",
                    spacing: "10",
                    main_align: "end",

                    // Cancel
                    rect {
                        background: "rgb(242, 242, 247)",
                        corner_radius: "8",
                        padding: "10 20",
                        onclick: move |e| on_cancel.call(e),

                        label {
                            font_size: "14",
                            color: "rgb(60, 60, 67)",
                            "Cancel"
                        }
                    }

                    // Overwrite (destructive red)
                    rect {
                        background: "rgb(255, 59, 48)",
                        corner_radius: "8",
                        padding: "10 20",
                        onclick: move |e| on_confirm.call(e),

                        label {
                            font_size: "14",
                            font_weight: "bold",
                            color: "rgb(255, 255, 255)",
                            "Overwrite"
                        }
                    }
                }
            }
        }
    )
}

// ─── Toast notification ──────────────────────────────────────────────────────

#[component]
fn Toast(message: String, on_dismiss: EventHandler<MouseEvent>) -> Element {
    let mut hovered = use_signal(|| false);
    let close_bg = if *hovered.read() {
        "rgb(255, 255, 255, 0.35)"
    } else {
        "rgb(255, 255, 255, 0.2)"
    };

    rsx!(
        rect {
            position: "global",
            position_bottom: "30",
            position_right: "30",
            background: "rgb(52, 199, 89)",
            corner_radius: "14",
            padding: "14 18",
            shadow: "0 8 24 0 rgb(0, 0, 0, 0.18)",
            direction: "horizontal",
            cross_align: "center",
            spacing: "12",

            // Checkmark icon
            rect {
                width: "28",
                height: "28",
                corner_radius: "14",
                background: "rgb(255, 255, 255, 0.25)",
                main_align: "center",
                cross_align: "center",

                label {
                    font_size: "14",
                    font_weight: "bold",
                    color: "rgb(255, 255, 255)",
                    "✓"
                }
            }

            // Message
            label {
                font_size: "14",
                font_weight: "bold",
                color: "rgb(255, 255, 255)",
                "{message}"
            }

            // Dismiss button
            rect {
                width: "24",
                height: "24",
                corner_radius: "12",
                background: "{close_bg}",
                main_align: "center",
                cross_align: "center",
                onmouseenter: move |_| hovered.set(true),
                onmouseleave: move |_| hovered.set(false),
                onclick: move |e| on_dismiss.call(e),

                label {
                    font_size: "11",
                    color: "rgb(255, 255, 255)",
                    "✕"
                }
            }
        }
    )
}

// ─── Drop zone / file picker ─────────────────────────────────────────────────

#[component]
fn DropZoneArea(on_pick: EventHandler<MouseEvent>) -> Element {
    let mut hovered = use_signal(|| false);

    let bg = if *hovered.read() {
        "rgb(230, 240, 255)"
    } else {
        "rgb(255, 255, 255)"
    };
    let border_color = if *hovered.read() {
        "rgb(0, 122, 255)"
    } else {
        "rgb(209, 209, 214)"
    };

    rsx!(
        rect {
            width: "fill",
            height: "140",
            background: "{bg}",
            border: "2 solid {border_color}",
            corner_radius: "12",
            direction: "vertical",
            main_align: "center",
            cross_align: "center",
            spacing: "10",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),

            label {
                font_size: "30",
                "🎵"
            }

            label {
                font_size: "14",
                color: "rgb(142, 142, 147)",
                "Drop audio files here, or"
            }

            rect {
                background: "rgb(0, 122, 255)",
                corner_radius: "8",
                padding: "8 18",
                onclick: move |e| on_pick.call(e),

                label {
                    font_size: "14",
                    font_weight: "bold",
                    color: "rgb(255, 255, 255)",
                    "Select Files"
                }
            }
        }
    )
}

// ─── File list ───────────────────────────────────────────────────────────────

#[component]
fn FileListArea(
    files: Vec<AudioFile>,
    on_remove: EventHandler<usize>,
    on_clear: EventHandler<MouseEvent>,
) -> Element {
    let count = files.len();
    let header_text = format!(
        "{} file{} selected",
        count,
        if count == 1 { "" } else { "s" }
    );

    rsx!(
        rect {
            width: "fill",
            background: "rgb(255, 255, 255)",
            corner_radius: "12",
            border: "1 solid rgb(209, 209, 214)",
            direction: "vertical",
            overflow: "clip",

            // List header
            rect {
                width: "fill",
                height: "44",
                padding: "0 16",
                direction: "horizontal",
                cross_align: "center",

                label {
                    font_size: "13",
                    font_weight: "bold",
                    color: "rgb(60, 60, 67)",
                    "{header_text}"
                }

                rect { width: "fill" }

                ClearButton { on_clear }
            }

            // Separator
            rect {
                width: "fill",
                height: "1",
                background: "rgb(229, 229, 234)",
            }

            // Scrollable file rows (capped at ~5 visible rows)
            rect {
                width: "fill",
                height: "220",
                overflow: "clip",

                ScrollView {
                    width: "fill",
                    height: "fill",
                    direction: "vertical",

                    for file in files.iter() {
                        FileRow {
                            key: "{file.id}",
                            file: file.clone(),
                            on_remove,
                        }
                    }
                }
            }
        }
    )
}

#[component]
fn ClearButton(on_clear: EventHandler<MouseEvent>) -> Element {
    let mut hovered = use_signal(|| false);
    let bg = if *hovered.read() {
        "rgb(229, 229, 234)"
    } else {
        "rgb(242, 242, 247)"
    };

    rsx!(
        rect {
            padding: "5 12",
            corner_radius: "6",
            background: "{bg}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |e| on_clear.call(e),

            label {
                font_size: "12",
                color: "rgb(100, 100, 100)",
                "Clear all"
            }
        }
    )
}

#[component]
fn FileRow(file: AudioFile, on_remove: EventHandler<usize>) -> Element {
    let (status_text, status_color) = match &file.status {
        FileStatus::Pending => ("Ready".to_string(), "rgb(142, 142, 147)"),
        FileStatus::Converting => ("Converting...".to_string(), "rgb(0, 122, 255)"),
        FileStatus::Done => {
            let text = match (file.input_size, file.output_size) {
                (Some(i), Some(o)) => {
                    format!("Done  ·  {} → {}", format_file_size(i), format_file_size(o))
                }
                _ => "Done".to_string(),
            };
            (text, "rgb(52, 199, 89)")
        }
        FileStatus::Error => {
            let msg = file.error.as_deref().unwrap_or("Unknown error");
            let truncated = if msg.len() > 60 { &msg[..60] } else { msg };
            (format!("Error: {}", truncated), "rgb(255, 59, 48)")
        }
    };

    let file_id = file.id;

    rsx!(
        rect {
            width: "fill",
            height: "48",
            padding: "0 16",
            direction: "horizontal",
            cross_align: "center",
            spacing: "10",

            label {
                font_size: "18",
                "🎵"
            }

            rect {
                width: "fill",
                direction: "vertical",
                spacing: "2",

                label {
                    font_size: "13",
                    color: "rgb(0, 0, 0)",
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    "{file.name}"
                }

                label {
                    font_size: "11",
                    color: "{status_color}",
                    "{status_text}"
                }
            }

            // Remove button
            RemoveButton {
                on_remove: move |_| on_remove.call(file_id),
            }
        }
    )
}

#[component]
fn RemoveButton(on_remove: EventHandler<MouseEvent>) -> Element {
    let mut hovered = use_signal(|| false);
    let bg = if *hovered.read() {
        "rgb(255, 59, 48, 0.15)"
    } else {
        "rgb(242, 242, 247)"
    };

    rsx!(
        rect {
            width: "26",
            height: "26",
            corner_radius: "13",
            background: "{bg}",
            main_align: "center",
            cross_align: "center",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |e| on_remove.call(e),

            label {
                font_size: "13",
                color: "rgb(142, 142, 147)",
                "✕"
            }
        }
    )
}

// ─── Format selector ─────────────────────────────────────────────────────────

#[component]
fn FormatSelectorArea(selected: OutputFormat, on_change: EventHandler<OutputFormat>) -> Element {
    rsx!(
        rect {
            width: "fill",
            background: "rgb(255, 255, 255)",
            corner_radius: "12",
            border: "1 solid rgb(209, 209, 214)",
            padding: "16",
            direction: "vertical",
            spacing: "12",

            label {
                font_size: "13",
                font_weight: "bold",
                color: "rgb(60, 60, 67)",
                "Output Format"
            }

            rect {
                width: "fill",
                direction: "horizontal",
                spacing: "8",
                content: "fit",

                for fmt in OutputFormat::all().iter() {
                    FormatChip {
                        key: "{fmt.as_str()}",
                        fmt: fmt.clone(),
                        is_active: *fmt == selected,
                        on_select: on_change,
                    }
                }
            }
        }
    )
}

#[component]
fn FormatChip(fmt: OutputFormat, is_active: bool, on_select: EventHandler<OutputFormat>) -> Element {
    let bg = if is_active {
        "rgb(0, 122, 255)"
    } else {
        "rgb(242, 242, 247)"
    };
    let color = if is_active {
        "rgb(255, 255, 255)"
    } else {
        "rgb(60, 60, 67)"
    };
    let label = fmt.label();
    let fmt_clone = fmt.clone();

    rsx!(
        rect {
            background: "{bg}",
            corner_radius: "8",
            padding: "8 16",
            onclick: move |_| on_select.call(fmt_clone.clone()),

            label {
                font_size: "13",
                font_weight: "bold",
                color: "{color}",
                "{label}"
            }
        }
    )
}

// ─── Options panel ───────────────────────────────────────────────────────────

#[component]
fn OptionsPanelArea(opts: ConversionOptions, on_change: EventHandler<ConversionOptions>) -> Element {
    let config = opts.format.config();

    rsx!(
        rect {
            width: "fill",
            background: "rgb(255, 255, 255)",
            corner_radius: "12",
            border: "1 solid rgb(209, 209, 214)",
            padding: "16",
            direction: "vertical",
            spacing: "16",

            label {
                font_size: "13",
                font_weight: "bold",
                color: "rgb(60, 60, 67)",
                "Conversion Options"
            }

            // Bitrate row (format-conditional)
            if config.supports_bitrate {
                OptionRow {
                    label: "Bitrate".to_string(),
                    choices: BITRATE_OPTIONS.iter().map(|s| s.to_string()).collect(),
                    selected: opts.bitrate.clone(),
                    on_change: {
                        let opts = opts.clone();
                        move |val: String| {
                            let mut o = opts.clone();
                            o.bitrate = val;
                            on_change.call(o);
                        }
                    },
                }
            }

            // Sample rate
            OptionRow {
                label: "Sample Rate".to_string(),
                choices: SAMPLE_RATE_OPTIONS.iter().map(|(v, _)| v.to_string()).collect(),
                display_labels: SAMPLE_RATE_OPTIONS.iter().map(|(_, l)| l.to_string()).collect::<Vec<_>>(),
                selected: opts.sample_rate.clone().unwrap_or_else(|| "Default".to_string()),
                on_change: {
                    let opts = opts.clone();
                    move |val: String| {
                        let mut o = opts.clone();
                        o.sample_rate = if val == "Default" { None } else { Some(val) };
                        on_change.call(o);
                    }
                },
            }

            // Channels
            OptionRow {
                label: "Channels".to_string(),
                choices: CHANNEL_OPTIONS.iter().map(|(v, _)| v.to_string()).collect(),
                display_labels: CHANNEL_OPTIONS.iter().map(|(_, l)| l.to_string()).collect::<Vec<_>>(),
                selected: opts.channels.clone().unwrap_or_else(|| "Default".to_string()),
                on_change: {
                    let opts = opts.clone();
                    move |val: String| {
                        let mut o = opts.clone();
                        o.channels = if val == "Default" { None } else { Some(val) };
                        on_change.call(o);
                    }
                },
            }

            // Quality control (format-conditional)
            if config.supports_quality {
                QualityControl {
                    label: config.quality_label.to_string(),
                    value: opts.quality,
                    min: config.quality_min,
                    max: config.quality_max,
                    on_change: {
                        let opts = opts.clone();
                        move |val: i32| {
                            let mut o = opts.clone();
                            o.quality = val;
                            on_change.call(o);
                        }
                    },
                }
            }
        }
    )
}

#[component]
fn OptionRow(
    label: String,
    choices: Vec<String>,
    #[props(optional)] display_labels: Option<Vec<String>>,
    selected: String,
    on_change: EventHandler<String>,
) -> Element {
    // Merge choices + optional display labels into (value, display) pairs
    let items: Vec<(String, String)> = choices
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let d = display_labels
                .as_ref()
                .and_then(|l| l.get(i))
                .cloned()
                .unwrap_or(v.clone());
            (v.clone(), d)
        })
        .collect();

    rsx!(
        rect {
            width: "fill",
            direction: "vertical",
            spacing: "8",

            label {
                font_size: "12",
                font_weight: "bold",
                color: "rgb(142, 142, 147)",
                "{label}"
            }

            rect {
                width: "fill",
                direction: "horizontal",
                spacing: "6",
                content: "fit",

                for (value, display) in items.into_iter() {
                    OptionChip {
                        key: "{value}",
                        is_active: value == selected,
                        on_select: on_change,
                        value,
                        display,
                    }
                }
            }
        }
    )
}

#[component]
fn OptionChip(
    value: String,
    display: String,
    is_active: bool,
    on_select: EventHandler<String>,
) -> Element {
    let bg = if is_active {
        "rgb(0, 122, 255)"
    } else {
        "rgb(242, 242, 247)"
    };
    let color = if is_active {
        "rgb(255, 255, 255)"
    } else {
        "rgb(60, 60, 67)"
    };
    let val = value.clone();

    rsx!(
        rect {
            background: "{bg}",
            corner_radius: "6",
            padding: "6 12",
            onclick: move |_| on_select.call(val.clone()),

            label {
                font_size: "12",
                color: "{color}",
                "{display}"
            }
        }
    )
}

#[component]
fn QualityControl(
    label: String,
    value: i32,
    min: i32,
    max: i32,
    on_change: EventHandler<i32>,
) -> Element {
    rsx!(
        rect {
            width: "fill",
            direction: "vertical",
            spacing: "8",

            label {
                font_size: "12",
                font_weight: "bold",
                color: "rgb(142, 142, 147)",
                "{label}"
            }

            rect {
                direction: "horizontal",
                spacing: "10",
                cross_align: "center",
                content: "fit",

                // Decrement
                StepButton {
                    glyph: "−".to_string(),
                    disabled: value <= min,
                    on_click: move |_| {
                        if value > min {
                            on_change.call(value - 1);
                        }
                    },
                }

                // Value badge
                rect {
                    width: "44",
                    height: "32",
                    background: "rgb(242, 242, 247)",
                    corner_radius: "6",
                    main_align: "center",
                    cross_align: "center",

                    label {
                        font_size: "15",
                        font_weight: "bold",
                        color: "rgb(0, 0, 0)",
                        "{value}"
                    }
                }

                // Increment
                StepButton {
                    glyph: "+".to_string(),
                    disabled: value >= max,
                    on_click: move |_| {
                        if value < max {
                            on_change.call(value + 1);
                        }
                    },
                }

                label {
                    font_size: "11",
                    color: "rgb(174, 174, 178)",
                    "({min} – {max})"
                }
            }
        }
    )
}

#[component]
fn StepButton(glyph: String, disabled: bool, on_click: EventHandler<MouseEvent>) -> Element {
    let bg = if disabled {
        "rgb(229, 229, 234)"
    } else {
        "rgb(242, 242, 247)"
    };
    let color = if disabled {
        "rgb(199, 199, 204)"
    } else {
        "rgb(60, 60, 67)"
    };

    rsx!(
        rect {
            width: "32",
            height: "32",
            background: "{bg}",
            corner_radius: "6",
            main_align: "center",
            cross_align: "center",
            onclick: move |e| {
                if !disabled {
                    on_click.call(e);
                }
            },

            label {
                font_size: "18",
                color: "{color}",
                "{glyph}"
            }
        }
    )
}

// ─── Convert button ──────────────────────────────────────────────────────────

#[component]
fn ConvertButtonArea(
    file_count: usize,
    is_converting: bool,
    ffmpeg_missing: bool,
    on_convert: EventHandler<MouseEvent>,
) -> Element {
    let disabled = file_count == 0 || is_converting || ffmpeg_missing;

    let bg = if disabled {
        "rgb(209, 209, 214)"
    } else {
        "rgb(0, 122, 255)"
    };

    let text = if ffmpeg_missing {
        "FFmpeg not installed".to_string()
    } else if is_converting {
        "Converting...".to_string()
    } else if file_count == 0 {
        "Select files to convert".to_string()
    } else {
        format!(
            "Convert {} file{}",
            file_count,
            if file_count == 1 { "" } else { "s" }
        )
    };

    rsx!(
        rect {
            width: "fill",
            padding: "4 0 20 0",

            rect {
                width: "fill",
                height: "50",
                background: "{bg}",
                corner_radius: "12",
                main_align: "center",
                cross_align: "center",
                onclick: move |e| {
                    if !disabled {
                        on_convert.call(e);
                    }
                },

                label {
                    font_size: "16",
                    font_weight: "bold",
                    color: "rgb(255, 255, 255)",
                    "{text}"
                }
            }
        }
    )
}
