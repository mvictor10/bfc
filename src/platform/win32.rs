//! Backend de janela Windows sem crates externas: Win32 + GDI para apresentar o framebuffer.
use bfc_engine::animation::{character_frame_dir, load_background, Sprite, SpriteSet};
use bfc_engine::config::EngineConfig;
use bfc_engine::game::Game;
use super::audio::AudioManager;
use bfc_engine::input::{FighterInput, FrameInput};
use bfc_engine::render::{render_game, Canvas, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use std::ffi::{c_void, OsStr};
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};

type Bool = i32;
type Byte = u8;
type Word = u16;
type Dword = u32;
type Uint = u32;
type Wparam = usize;
type Lparam = isize;
type Lresult = isize;
type Hwnd = *mut c_void;
type Hinstance = *mut c_void;
type Hicon = *mut c_void;
type Hcursor = *mut c_void;
type Hbrush = *mut c_void;
type Hmenu = *mut c_void;
type Hdc = *mut c_void;
type Lpcwstr = *const u16;

type WindowProc = unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult;

const WS_OVERLAPPEDWINDOW: Dword = 0x00CF_0000;
const CW_USEDEFAULT: i32 = 0x8000_0000_u32 as i32;
const SW_SHOW: i32 = 5;
const PM_REMOVE: Uint = 0x0001;
const WM_DESTROY: Uint = 0x0002;
const WM_QUIT: Uint = 0x0012;
const DIB_RGB_COLORS: Uint = 0;
const BI_RGB: Dword = 0;
const SRCCOPY: Dword = 0x00CC_0020;

#[repr(C)]
struct Point { x: i32, y: i32 }
#[repr(C)]
struct RectWin { left: i32, top: i32, right: i32, bottom: i32 }
#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: Uint,
    w_param: Wparam,
    l_param: Lparam,
    time: Dword,
    pt: Point,
    l_private: Dword,
}
#[repr(C)]
struct WndClassW {
    style: Uint,
    lpfn_wnd_proc: Option<WindowProc>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: Hinstance,
    h_icon: Hicon,
    h_cursor: Hcursor,
    hbr_background: Hbrush,
    lpsz_menu_name: Lpcwstr,
    lpsz_class_name: Lpcwstr,
}
#[repr(C)]
struct BitmapInfoHeader {
    size: Dword,
    width: i32,
    height: i32,
    planes: Word,
    bit_count: Word,
    compression: Dword,
    size_image: Dword,
    x_pels_per_meter: i32,
    y_pels_per_meter: i32,
    colors_used: Dword,
    colors_important: Dword,
}
#[repr(C)]
struct BitmapInfo { header: BitmapInfoHeader, colors: [Dword; 1] }

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> Word;
    fn CreateWindowExW(ex_style: Dword, class_name: Lpcwstr, window_name: Lpcwstr, style: Dword,
        x: i32, y: i32, width: i32, height: i32, parent: Hwnd, menu: Hmenu, instance: Hinstance, param: *mut c_void) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn ShowWindow(hwnd: Hwnd, command: i32) -> Bool;
    fn UpdateWindow(hwnd: Hwnd) -> Bool;
    fn PeekMessageW(msg: *mut Msg, hwnd: Hwnd, min: Uint, max: Uint, remove: Uint) -> Bool;
    fn TranslateMessage(msg: *const Msg) -> Bool;
    fn DispatchMessageW(msg: *const Msg) -> Lresult;
    fn PostQuitMessage(exit_code: i32);
    fn GetAsyncKeyState(vkey: i32) -> i16;
    fn GetDC(hwnd: Hwnd) -> Hdc;
    fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
    fn GetClientRect(hwnd: Hwnd, rect: *mut RectWin) -> Bool;
}
#[link(name = "kernel32")]
extern "system" { fn GetModuleHandleW(name: Lpcwstr) -> Hinstance; }
#[link(name = "gdi32")]
extern "system" {
    fn StretchDIBits(hdc: Hdc, x_dest: i32, y_dest: i32, dest_width: i32, dest_height: i32,
        x_src: i32, y_src: i32, src_width: i32, src_height: i32, bits: *const c_void,
        info: *const BitmapInfo, usage: Uint, raster_op: Dword) -> i32;
}

unsafe extern "system" fn window_proc(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult {
    if msg == WM_DESTROY {
        // O loop principal termina quando o message queue recebe WM_QUIT.
        PostQuitMessage(0);
        return 0;
    }
    DefWindowProcW(hwnd, msg, w_param, l_param)
}

pub fn run() -> Result<(), String> {
    let root = find_project_root()?;
    let config = EngineConfig::load(&root);
    let mut game = Game::new(config);
    let mut loaded_p1 = game.config.roster.get(game.selected_p1).map(|c| c.id.clone()).unwrap_or_default();
    let mut loaded_p2 = game.config.roster.get(game.selected_p2).map(|c| c.id.clone()).unwrap_or_default();
    let mut sprites1 = game.config.roster.get(game.selected_p1)
        .map(|c| SpriteSet::load(&character_frame_dir(&root, &c.frame_dir))).unwrap_or_default();
    let mut sprites2 = game.config.roster.get(game.selected_p2)
        .map(|c| SpriteSet::load(&character_frame_dir(&root, &c.frame_dir))).unwrap_or_default();
    let mut loaded_stage = usize::MAX;
    let mut background: Option<Sprite> = None;
    let mut audio = AudioManager::new(root.clone());
    let mut canvas = Canvas::new(LOGICAL_WIDTH, LOGICAL_HEIGHT);
    let mut keyboard = Keyboard::default();

    let class_name = to_wide("BFC_RUST_WINDOW");
    let title = to_wide("Brazil Fight Combat - Rust Light Engine");
    let instance = unsafe { GetModuleHandleW(ptr::null()) };
    let class = WndClassW {
        style: 0,
        lpfn_wnd_proc: Some(window_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: instance,
        h_icon: ptr::null_mut(),
        h_cursor: ptr::null_mut(),
        hbr_background: ptr::null_mut(),
        lpsz_menu_name: ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
    };
    let registered = unsafe { RegisterClassW(&class) };
    if registered == 0 { return Err("RegisterClassW falhou".into()); }

    let hwnd = unsafe { CreateWindowExW(0, class_name.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, LOGICAL_WIDTH as i32, LOGICAL_HEIGHT as i32,
        ptr::null_mut(), ptr::null_mut(), instance, ptr::null_mut()) };
    if hwnd.is_null() { return Err("CreateWindowExW falhou".into()); }
    unsafe { ShowWindow(hwnd, SW_SHOW); UpdateWindow(hwnd); }

    let frame_duration = Duration::from_nanos(1_000_000_000 / 60);
    let mut previous = Instant::now();
    let mut accumulator = Duration::ZERO;
    let mut running = true;
    while running && !game.exit_requested {
        let frame_started = Instant::now();
        let mut message: Msg = unsafe { mem::zeroed() };
        while unsafe { PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
            if message.message == WM_QUIT { running = false; break; }
            unsafe { TranslateMessage(&message); DispatchMessageW(&message); }
        }
        if !running { break; }

        let now = Instant::now();
        let elapsed = now.saturating_duration_since(previous);
        previous = now;
        accumulator += elapsed;
        // Evita simular centenas de ticks após uma pausa ou travamento do sistema.
        let max_accumulator = frame_duration * 5;
        if accumulator > max_accumulator { accumulator = max_accumulator; }
        while accumulator >= frame_duration {
            let input = keyboard.poll();
            game.update(input);
            for sound in &game.sound_events {
                if let Some(voice_event) = sound.strip_prefix("voice:") {
                    let mut parts = voice_event.splitn(2, ':');
                    if let (Some(character_id), Some(event)) = (parts.next(), parts.next()) {
                        audio.play_voice(character_id, event);
                    }
                } else {
                    audio.play_sfx(sound);
                }
            }
            accumulator -= frame_duration;
        }

        if let Some(character) = game.config.roster.get(game.selected_p1) {
            if loaded_p1 != character.id {
                sprites1 = SpriteSet::load(&character_frame_dir(&root, &character.frame_dir));
                loaded_p1 = character.id.clone();
            }
        }
        if let Some(character) = game.config.roster.get(game.selected_p2) {
            if loaded_p2 != character.id {
                sprites2 = SpriteSet::load(&character_frame_dir(&root, &character.frame_dir));
                loaded_p2 = character.id.clone();
            }
        }
        if loaded_stage != game.current_stage {
            background = game.config.stages.get(game.current_stage).and_then(|stage| {
                load_background(&root.join(&stage.image), LOGICAL_WIDTH as u32, LOGICAL_HEIGHT as u32).ok()
            });
            loaded_stage = game.current_stage;
        }
        audio.sync_music(&game.desired_music);
        render_game(&mut canvas, &game, &sprites1, &sprites2, background.as_ref());
        present(hwnd, &canvas);
        // Limite explícito de 60 FPS: evita renderização ocupada e consumo de CPU
        // desnecessário em computadores modestos.
        let remaining = frame_duration.saturating_sub(frame_started.elapsed());
        if !remaining.is_zero() { thread::sleep(remaining); }
    }
    Ok(())
}

fn present(hwnd: Hwnd, canvas: &Canvas) {
    let dc = unsafe { GetDC(hwnd) };
    if dc.is_null() { return; }
    let mut client: RectWin = unsafe { mem::zeroed() };
    if unsafe { GetClientRect(hwnd, &mut client) } != 0 {
        let width = (client.right - client.left).max(1);
        let height = (client.bottom - client.top).max(1);
        let info = BitmapInfo {
            header: BitmapInfoHeader {
                size: mem::size_of::<BitmapInfoHeader>() as Dword,
                width: canvas.width as i32,
                height: -(canvas.height as i32), // Altura negativa: pixels top-down.
                planes: 1,
                bit_count: 32,
                compression: BI_RGB,
                size_image: (canvas.width * canvas.height * 4) as Dword,
                x_pels_per_meter: 0, y_pels_per_meter: 0, colors_used: 0, colors_important: 0,
            },
            colors: [0],
        };
        unsafe {
            StretchDIBits(dc, 0, 0, width, height, 0, 0, canvas.width as i32, canvas.height as i32,
                canvas.pixels.as_ptr() as *const c_void, &info, DIB_RGB_COLORS, SRCCOPY);
        }
    }
    unsafe { ReleaseDC(hwnd, dc); }
}

struct Keyboard { previous: [bool; 256], current: [bool; 256] }

// Inicialização explícita: versões do Rust usadas por alguns toolchains Windows
// não implementam Default para arrays com mais de 32 elementos.
impl Default for Keyboard {
    fn default() -> Self {
        Self {
            previous: [false; 256],
            current: [false; 256],
        }
    }
}
impl Keyboard {
    fn poll(&mut self) -> FrameInput {
        self.previous = self.current;
        for key in 0..256 { self.current[key] = unsafe { GetAsyncKeyState(key as i32) } < 0; }
        let down = |key: usize| self.current[key];
        let pressed = |key: usize| self.current[key] && !self.previous[key];
        let p1 = FighterInput {
            left: down(b'A' as usize), right: down(b'D' as usize), up: down(b'W' as usize), down: down(b'S' as usize),
            punch: pressed(b'F' as usize), kick: pressed(b'G' as usize), special: pressed(b'H' as usize), block: down(b'R' as usize),
        };
        let p2 = FighterInput {
            left: down(0x25), right: down(0x27), up: down(0x26), down: down(0x28),
            punch: pressed(b'J' as usize), kick: pressed(b'K' as usize), special: pressed(b'L' as usize), block: down(b'I' as usize),
        };
        FrameInput {
            p1, p2, menu_up: pressed(0x26), menu_down: pressed(0x28), confirm: pressed(0x0D), escape: pressed(0x1B),
            toggle_debug: pressed(0x70), toggle_training_guard: pressed(b'B' as usize), restart_training: pressed(b'N' as usize),
            p1_char_prev: pressed(b'Q' as usize), p1_char_next: pressed(b'E' as usize),
            p2_char_prev: pressed(b'U' as usize), p2_char_next: pressed(b'O' as usize),
            stage_prev: pressed(0xDB), stage_next: pressed(0xDD),
        }
    }
}

fn to_wide(value: &str) -> Vec<u16> { OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect() }

fn find_project_root() -> Result<PathBuf, String> {
    let current = std::env::current_dir().map_err(|e| e.to_string())?;
    if let Some(root) = seek_root(&current) { return Ok(root); }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() { if let Some(root) = seek_root(parent) { return Ok(root); } }
    }
    Err("nao encontrei data/engine.ini; execute o programa a partir da pasta do projeto".into())
}
fn seek_root(start: &Path) -> Option<PathBuf> {
    for candidate in start.ancestors() {
        if candidate.join("data/engine.ini").is_file() && candidate.join("chars/chars.json").is_file() {
            return Some(candidate.to_path_buf());
        }
    }
    None
}
