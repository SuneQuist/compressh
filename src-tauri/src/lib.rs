use std::{borrow::Cow};
use std::process::{Command, Stdio};
use std::path::Path;
use tauri::{AppHandle, Emitter};

// Video ffmpeg -i input.mp4 -vcodec libx264 -crf 23 -preset medium -acodec copy output.mp4\
// Audio ffmpeg -i input.mp3 -b:a 128k output.mp3
// (flac) ffmpeg -i input.flac -compression_level 8 output.flac
// (jpg, jpeg) ffmpeg -i input.png -q:v 2 output.jpg
// (png, webp) ffmpeg -i input.png -compression_level 100 output.png

#[cfg(target_os = "windows")]
const FFMPEG_PATH: &str = "bin\\ffmpeg.exe";
const GS_PATH: &str = "bin\\gs10.05.1\\bin\\gswin64c.exe";
use std::os::windows::process::CommandExt;

#[tauri::command]
fn run_ffmpeg(app: AppHandle, path: String) {
    std::thread::spawn(move || {
        let (extension, input, output) = get_path(&path);

        let mut args: Vec<String> = Vec::new(); 

        args.push("-i".to_string());
        args.push(input);

        let ext = extension.to_lowercase();
        match ext.as_ref() {
            "png"|"webp" => args.extend("-compression_level 100".split_whitespace().map(String::from)),
            "jpg"|"jpeg" => args.extend("-q:v 2".split_whitespace().map(String::from)),
            "flac" => args.extend("-compression_level 8".split_whitespace().map(String::from)),
            "mp3"|"aac"|"wav" => args.extend("-b:a 128k".split_whitespace().map(String::from)),
            "mp4" | "mov"|"mkv"|"avi" => args.extend("-vcodec libx264 -crf 23 -preset medium -acodec copy".split_whitespace().map(String::from)),
            _ => return
        }

        args.push(output);

        let mut cmd = Command::new(FFMPEG_PATH);

        cmd
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let result = cmd.output();

        match result {
            Ok(_) => { app.emit("process-done", "done").unwrap(); },
            Err(_) => { app.emit("process-done", "error").unwrap(); }
        }
    });
}

#[tauri::command]
fn run_gs(app: AppHandle, path: String, cmd: String) {
    std::thread::spawn(move || {
        let (_extension, input, output) = get_path(&path);

        let default_command = "-sDEVICE=pdfwrite -dPDFSETTINGS=/ebook -dDownsampleColorImages=true -dColorImageResolution=180 -dColorImageDownSampleType=/Bicubic -dNOPAUSE -dQUEIT -dBATCH";

        let mut args: Vec<String> = Vec::new();

        let command_str = if cmd.trim().is_empty() { default_command } else { &cmd };
        args.extend(command_str.split_whitespace().map(String::from));

        args.push(format!("-sOutputFile={}", output));
        args.push(input);

        let mut cmd = Command::new(GS_PATH);

        cmd
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let result = cmd.output();

        match result {
            Ok(_) => { app.emit("process-done", "done").unwrap(); },
            Err(_) => { app.emit("process-done", "error").unwrap(); }
        }
    });
}

// Get Input- & Output-file from path, incl. extension.
fn get_path(path: &str) -> (Cow<str>, String, String) {
    let path = Path::new(path); // Create Path Struct

    // Get Path Attributes
    let file: Cow<str> = path.file_stem().unwrap().to_string_lossy();
    let extension: Cow<str> = path.extension().unwrap().to_string_lossy();
    let folder: Cow<str> = path.parent().unwrap().to_string_lossy();

    // Create paths for input and output
    let input = format!("{}/{}.{}", folder, file, extension);
    let output = format!("{}/{}_compressed.{}", folder, file, extension);

    return (extension, input, output);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_ffmpeg, run_gs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
