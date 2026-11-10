//! Adaptador de áudio WinMM: MCI para música e PlaySound para SFX WAV leves.
use std::ffi::c_void;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::Value;

#[link(name = "winmm")]
extern "system" {
    fn mciSendStringW(command: *const u16, return_string: *mut u16, return_size: u32, callback: *mut c_void) -> u32;
    fn PlaySoundW(sound: *const u16, module: *mut c_void, flags: u32) -> i32;
}

const SND_ASYNC: u32 = 0x0001;
const SND_NODEFAULT: u32 = 0x0002;
const SND_FILENAME: u32 = 0x0002_0000;

pub struct AudioManager {
    root: PathBuf,
    current_music: String,
    voice_open: bool,
}

impl AudioManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root, current_music: String::new(), voice_open: false }
    }

    pub fn sync_music(&mut self, relative_path: &str) {
        if self.current_music == relative_path { return; }
        self.stop_music();
        let path = self.root.join(relative_path);
        if !path.is_file() { self.current_music = relative_path.to_owned(); return; }
        let media_type = if path.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("wav")).unwrap_or(false) { "waveaudio" } else { "mpegvideo" };
        let command = format!("open \"{}\" type {} alias bfc_music", mci_path(&path), media_type);
        let wide = to_wide(&command);
        let result = unsafe { mciSendStringW(wide.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut()) };
        if result == 0 {
            let play = to_wide("play bfc_music repeat");
            unsafe { mciSendStringW(play.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut()); }
        }
        self.current_music = relative_path.to_owned();
    }

    /// Toca a fala win/fight/lose do personagem. Primeiro tenta gravações cadastradas
    /// no JSON; sem áudio, usa a frase de texto pelo SAPI do Windows.
    pub fn play_voice(&mut self, character_id: &str, event: &str) {
        self.stop_voice();
        if let Some(path) = self.find_voice_file(character_id, event) {
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            if extension == "mp3" || extension == "wav" {
                let media_type = if extension == "wav" { "waveaudio" } else { "mpegvideo" };
                let command = format!("open \"{}\" type {} alias bfc_voice", mci_path(&path), media_type);
                let wide = to_wide(&command);
                let opened = unsafe { mciSendStringW(wide.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut()) } == 0;
                if opened {
                    let play = to_wide("play bfc_voice");
                    unsafe { mciSendStringW(play.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut()); }
                    self.voice_open = true;
                    return;
                }
            }
        }
        if let Some(text) = self.read_voice_text(character_id, event) {
            self.speak_text(&text);
        }
    }

    fn find_voice_file(&self, character_id: &str, event: &str) -> Option<PathBuf> {
        let json_path = self.root.join("chars").join(character_id).join("character.json");
        if let Ok(bytes) = fs::read(&json_path) {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                if let Some(files) = value.get("voices").and_then(|v| v.get(event)).and_then(|v| v.get("files")).and_then(Value::as_array) {
                    for file in files.iter().filter_map(Value::as_str) {
                        let candidate = if Path::new(file).is_absolute() { PathBuf::from(file) }
                            else if file.starts_with("assets/") || file.starts_with("chars/") { self.root.join(file) }
                            else { self.root.join("chars").join(character_id).join(file) };
                        if candidate.is_file() { return Some(candidate); }
                    }
                }
            }
        }
        for extension in ["mp3", "wav"] {
            let candidate = self.root.join("assets/audio/voice").join(character_id).join(format!("{}.{}", event, extension));
            if candidate.is_file() { return Some(candidate); }
            let candidate = self.root.join("chars").join(character_id).join(format!("{}.{}", event, extension));
            if candidate.is_file() { return Some(candidate); }
        }
        None
    }

    fn read_voice_text(&self, character_id: &str, event: &str) -> Option<String> {
        let path = self.root.join("chars").join(character_id).join("character.json");
        let bytes = fs::read(path).ok()?;
        let value: Value = serde_json::from_slice(&bytes).ok()?;
        value.get("voices")?.get(event)?.get("text")?.as_str()
            .map(str::trim).filter(|text| !text.is_empty()).map(str::to_owned)
    }

    fn speak_text(&self, text: &str) {
        // PowerShell/System.Speech faz parte das instalações comuns do Windows. Ele só é
        // iniciado como fallback no evento de fala, não durante o loop de renderização.
        let escaped = text.replace("'", "''").replace('\r', " ").replace('\n', " ");
        let script = format!(
            "Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Speak('{}'); $s.Dispose()",
            escaped
        );
        let _ = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-Command", &script])
            .creation_flags(0x0800_0000)
            .spawn();
    }

    pub fn play_sfx(&self, id: &str) {
        let filename = match id {
            "punch" => "punch.wav", "kick" => "kick.wav", "special" => "special.wav", "block" => "block.wav",
            "jump" => "jump.wav", "ko" => "ko.wav", "select" => "select.wav", "confirm" => "confirm.wav", _ => return,
        };
        let path = self.root.join("assets/audio/sfx/Wav").join(filename);
        if !path.is_file() { return; }
        let wide = path_to_wide(&path);
        unsafe { PlaySoundW(wide.as_ptr(), std::ptr::null_mut(), SND_ASYNC | SND_NODEFAULT | SND_FILENAME); }
    }

    fn stop_music(&mut self) {
        if self.current_music.is_empty() { return; }
        let stop = to_wide("stop bfc_music");
        let close = to_wide("close bfc_music");
        unsafe {
            mciSendStringW(stop.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut());
            mciSendStringW(close.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut());
        }
        self.current_music.clear();
    }

    fn stop_voice(&mut self) {
        if !self.voice_open { return; }
        let stop = to_wide("stop bfc_voice");
        let close = to_wide("close bfc_voice");
        unsafe {
            mciSendStringW(stop.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut());
            mciSendStringW(close.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut());
        }
        self.voice_open = false;
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.stop_music();
        self.stop_voice();
    }
}

fn to_wide(value: &str) -> Vec<u16> { value.encode_utf16().chain(std::iter::once(0)).collect() }
fn path_to_wide(path: &Path) -> Vec<u16> { path.as_os_str().encode_wide().chain(std::iter::once(0)).collect() }
fn mci_path(path: &Path) -> String { path.to_string_lossy().replace('"', "\"\"") }
