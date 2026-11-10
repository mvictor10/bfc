#![cfg_attr(windows, windows_subsystem = "windows")]
//! BFC Studio: dashboard nativo Win32 para gerenciar personagens, mapas e trilhas.
//! Não utiliza Python. Usa as crates image/serde_json já declaradas pelo motor.

#[cfg(not(windows))]
fn main() {
    eprintln!("BFC Studio e um aplicativo Windows. Compile no Windows com cargo build --release --bin BFCStudio.");
}

#[cfg(windows)]
mod windows_app {
    use image::imageops::{self, FilterType};
    use image::{GenericImageView, Rgba, RgbaImage};
    use serde_json::{json, Value};
    use std::collections::{HashMap, VecDeque};
    use std::ffi::c_void;
    use std::fs;
    use std::mem;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::ptr;

    type Hwnd = *mut c_void;
    type Hinstance = *mut c_void;
    type Hicon = *mut c_void;
    type Hcursor = *mut c_void;
    type Hbrush = *mut c_void;
    type Hmenu = *mut c_void;
    type Uint = u32;
    type Wparam = usize;
    type Lparam = isize;
    type Lresult = isize;
    type WindowProc = unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult;
    type DialogHook = unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> usize;

    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_BORDER: u32 = 0x0080_0000;
    const WS_TABSTOP: u32 = 0x0001_0000;
    const WS_VSCROLL: u32 = 0x0020_0000;
    const ES_AUTOHSCROLL: u32 = 0x0080;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_READONLY: u32 = 0x0800;
    const SS_LEFT: u32 = 0;
    const BS_PUSHBUTTON: u32 = 0;
    const SW_SHOW: i32 = 5;
    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_COMMAND: u32 = 0x0111;
    const BN_CLICKED: usize = 0;
    const OFN_PATHMUSTEXIST: u32 = 0x0000_0800;
    const OFN_FILEMUSTEXIST: u32 = 0x0000_1000;
    const OFN_EXPLORER: u32 = 0x0008_0000;
    const SW_SHOWNORMAL: i32 = 1;

    const C_NAME: i32 = 101;
    const C_ID: i32 = 102;
    const C_SPEED: i32 = 103;
    const C_POWER: i32 = 104;
    const C_HP: i32 = 105;
    const C_ROLE: i32 = 106;
    const C_HOME_STAGE: i32 = 107;
    const C_MUSIC: i32 = 108;
    const C_PHRASE: i32 = 109;
    const C_SPRITE: i32 = 110;
    const B_SPRITE: i32 = 111;
    const C_FACE: i32 = 112;
    const B_FACE: i32 = 113;
    const C_VOICE: i32 = 114;
    const B_VOICE: i32 = 115;
    const B_SAVE_CHARACTER: i32 = 116;
    const B_LOAD_CHARACTER: i32 = 117;

    const S_ID: i32 = 201;
    const S_NAME: i32 = 202;
    const S_GROUND: i32 = 203;
    const S_MUSIC: i32 = 204;
    const S_IMAGE: i32 = 205;
    const B_STAGE_IMAGE: i32 = 206;
    const B_ADD_STAGE: i32 = 207;
    const B_LOAD_STAGE: i32 = 208;

    const M_ID: i32 = 301;
    const M_FILE: i32 = 302;
    const B_MUSIC_FILE: i32 = 303;
    const B_ADD_MUSIC: i32 = 304;

    const B_AUDIT: i32 = 401;
    const B_OPEN_FOLDER: i32 = 402;
    const B_RELOAD: i32 = 403;
    const STATUS: i32 = 500;

    #[repr(C)]
    struct Point { x: i32, y: i32 }

    #[repr(C)]
    struct Message {
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
        time: u32,
        pt: Point,
        l_private: u32,
    }

    #[repr(C)]
    struct WindowClassW {
        style: Uint,
        lpfn_wnd_proc: Option<WindowProc>,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: Hinstance,
        h_icon: Hicon,
        h_cursor: Hcursor,
        hbr_background: Hbrush,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
    }

    #[repr(C)]
    struct OpenFileNameW {
        l_struct_size: u32,
        hwnd_owner: Hwnd,
        h_instance: Hinstance,
        lpstr_filter: *const u16,
        lpstr_custom_filter: *mut u16,
        n_max_cust_filter: u32,
        n_filter_index: u32,
        lpstr_file: *mut u16,
        n_max_file: u32,
        lpstr_file_title: *mut u16,
        n_max_file_title: u32,
        lpstr_initial_dir: *const u16,
        lpstr_title: *const u16,
        flags: u32,
        n_file_offset: u16,
        n_file_extension: u16,
        lpstr_def_ext: *const u16,
        l_cust_data: isize,
        lpfn_hook: Option<DialogHook>,
        lp_template_name: *const u16,
        pv_reserved: *mut c_void,
        dw_reserved: u32,
        flags_ex: u32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassW(class: *const WindowClassW) -> u16;
        fn CreateWindowExW(ex_style: u32, class_name: *const u16, window_name: *const u16, style: u32,
            x: i32, y: i32, width: i32, height: i32, parent: Hwnd, menu: Hmenu, instance: Hinstance, param: *mut c_void) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
        fn UpdateWindow(hwnd: Hwnd) -> i32;
        fn GetMessageW(msg: *mut Message, hwnd: Hwnd, min: Uint, max: Uint) -> i32;
        fn TranslateMessage(msg: *const Message) -> i32;
        fn DispatchMessageW(msg: *const Message) -> Lresult;
        fn PostQuitMessage(exit_code: i32);
        fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max_count: i32) -> i32;
        fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
        fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" { fn GetModuleHandleW(name: *const u16) -> Hinstance; }
    #[link(name = "comdlg32")]
    extern "system" { fn GetOpenFileNameW(ofn: *mut OpenFileNameW) -> i32; }
    #[link(name = "shell32")]
    extern "system" { fn ShellExecuteW(hwnd: Hwnd, operation: *const u16, file: *const u16,
        params: *const u16, directory: *const u16, show: i32) -> isize; }

    struct AppState {
        root: PathBuf,
        controls: HashMap<i32, isize>,
    }

    static mut APP: Option<AppState> = None;

    pub fn run() -> Result<(), String> {
        let root = find_project_root().ok_or_else(|| "Nao encontrei a pasta do projeto (chars/chars.json e data/engine.ini). Execute tools/BFCStudio.exe dentro da pasta do projeto.".to_owned())?;
        unsafe { APP = Some(AppState { root: root.clone(), controls: HashMap::new() }); }

        let class_name = to_wide("BFC_STUDIO_RUST_WINDOW");
        let title = to_wide("BFC Studio - Dashboard do Brazil Fight Combat");
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        let class = WindowClassW {
            style: 0, lpfn_wnd_proc: Some(window_proc), cb_cls_extra: 0, cb_wnd_extra: 0,
            h_instance: instance, h_icon: ptr::null_mut(), h_cursor: ptr::null_mut(),
            hbr_background: 6usize as Hbrush, lpsz_menu_name: ptr::null(), lpsz_class_name: class_name.as_ptr(),
        };
        if unsafe { RegisterClassW(&class) } == 0 { return Err("RegisterClassW falhou ao criar o dashboard.".into()); }
        let hwnd = unsafe { CreateWindowExW(0, class_name.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW,
            70, 40, 1120, 730, ptr::null_mut(), ptr::null_mut(), instance, ptr::null_mut()) };
        if hwnd.is_null() { return Err("Nao foi possivel criar a janela do BFC Studio.".into()); }
        unsafe { ShowWindow(hwnd, SW_SHOW); UpdateWindow(hwnd); }

        let mut message: Message = unsafe { mem::zeroed() };
        loop {
            let result = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
            if result <= 0 { break; }
            unsafe { TranslateMessage(&message); DispatchMessageW(&message); }
        }
        Ok(())
    }

    unsafe extern "system" fn window_proc(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Lresult {
        match message {
            WM_CREATE => { create_controls(hwnd); 0 }
            WM_COMMAND if ((w_param >> 16) & 0xffff) == BN_CLICKED => {
                handle_command(hwnd, (w_param & 0xffff) as i32); 0
            }
            WM_DESTROY => { PostQuitMessage(0); 0 }
            _ => DefWindowProcW(hwnd, message, w_param, l_param),
        }
    }

    unsafe fn create_controls(parent: Hwnd) {
        add_label(parent, 0, 18, 8, 1060, 22, "BFC STUDIO  |  Gerenciador nativo de personagens, mapas, trilhas e vozes");
        let root = app_root().display().to_string();
        add_label(parent, 0, 18, 30, 1060, 20, &format!("Projeto detectado: {}", root));

        add_label(parent, 0, 20, 56, 500, 22, "1. PERSONAGEM  (edita ou cria uma pasta completa em chars/<id>)");
        add_label(parent, 0, 24, 84, 70, 20, "Nome:"); add_edit(parent, C_NAME, 88, 79, 220, 24, "Novo personagem");
        add_label(parent, 0, 324, 84, 34, 20, "ID:"); add_edit(parent, C_ID, 355, 79, 150, 24, "novo-personagem");
        add_label(parent, 0, 24, 114, 48, 20, "Veloc.:"); add_edit(parent, C_SPEED, 75, 109, 55, 24, "4.5");
        add_label(parent, 0, 143, 114, 43, 20, "Poder:"); add_edit(parent, C_POWER, 184, 109, 55, 24, "1.0");
        add_label(parent, 0, 252, 114, 28, 20, "Vida:"); add_edit(parent, C_HP, 284, 109, 58, 24, "200");
        add_label(parent, 0, 354, 114, 45, 20, "Papel:"); add_edit(parent, C_ROLE, 398, 109, 107, 24, "normal");
        add_label(parent, 0, 24, 144, 82, 20, "Mapa da casa:"); add_edit(parent, C_HOME_STAGE, 110, 139, 169, 24, "");
        add_label(parent, 0, 294, 144, 50, 20, "Música:"); add_edit(parent, C_MUSIC, 346, 139, 159, 24, "");
        add_label(parent, 0, 24, 175, 80, 20, "Frase de vitória:"); add_edit_ex(parent, C_PHRASE, 110, 170, 395, 42, "", false);
        add_label(parent, 0, 24, 223, 80, 20, "Arte principal:"); add_edit(parent, C_SPRITE, 110, 218, 300, 24, ""); add_button(parent, B_SPRITE, 416, 218, 89, 24, "Procurar...");
        add_label(parent, 0, 24, 252, 80, 20, "Rosto (face):"); add_edit(parent, C_FACE, 110, 247, 300, 24, ""); add_button(parent, B_FACE, 416, 247, 89, 24, "Procurar...");
        add_label(parent, 0, 24, 281, 80, 20, "Voz gravada:"); add_edit(parent, C_VOICE, 110, 276, 300, 24, ""); add_button(parent, B_VOICE, 416, 276, 89, 24, "Procurar...");
        add_button(parent, B_SAVE_CHARACTER, 24, 311, 232, 32, "Criar / salvar personagem");
        add_button(parent, B_LOAD_CHARACTER, 266, 311, 239, 32, "Carregar ID existente");

        add_label(parent, 0, 550, 56, 530, 22, "2. MAPAS  (imagem JPG/JPEG/BMP/PNG convertida para PNG)");
        add_label(parent, 0, 554, 84, 42, 20, "ID mapa:"); add_edit(parent, S_ID, 612, 79, 137, 24, "novo-mapa");
        add_label(parent, 0, 758, 84, 55, 20, "Nome:"); add_edit(parent, S_NAME, 807, 79, 248, 24, "Novo cenário");
        add_label(parent, 0, 554, 114, 50, 20, "Chão:"); add_edit(parent, S_GROUND, 612, 109, 95, 24, "470");
        add_label(parent, 0, 716, 114, 56, 20, "Música:"); add_edit(parent, S_MUSIC, 777, 109, 278, 24, "");
        add_label(parent, 0, 554, 144, 56, 20, "Imagem:"); add_edit(parent, S_IMAGE, 612, 139, 305, 24, ""); add_button(parent, B_STAGE_IMAGE, 923, 139, 132, 24, "Procurar...");
        add_button(parent, B_ADD_STAGE, 554, 171, 240, 32, "Adicionar / salvar mapa");
        add_button(parent, B_LOAD_STAGE, 804, 171, 251, 32, "Carregar mapa pelo ID");

        add_label(parent, 0, 550, 226, 530, 22, "3. TRILHAS SONORAS  (MP3/WAV)");
        add_label(parent, 0, 554, 254, 65, 20, "ID faixa:"); add_edit(parent, M_ID, 622, 249, 140, 24, "tema-novo");
        add_label(parent, 0, 554, 284, 56, 20, "Arquivo:"); add_edit(parent, M_FILE, 612, 279, 305, 24, ""); add_button(parent, B_MUSIC_FILE, 923, 279, 132, 24, "Procurar...");
        add_button(parent, B_ADD_MUSIC, 554, 311, 240, 32, "Importar trilha sonora");
        add_label(parent, 0, 554, 355, 500, 32, "Dica: depois de importar a trilha, o caminho será preenchido nos campos Música do personagem e do mapa. Salve o personagem/mapa para associá-la.");

        add_label(parent, 0, 18, 365, 1050, 20, "4. RELATORIO / AUDITORIA");
        add_edit_ex(parent, STATUS, 18, 388, 1060, 155, "Pronto. Escolha uma imagem/áudio e use uma das ações acima.", true);
        add_button(parent, B_AUDIT, 18, 555, 230, 32, "Auditar personagens/mapas");
        add_button(parent, B_OPEN_FOLDER, 258, 555, 210, 32, "Abrir pasta do projeto");
        add_button(parent, B_RELOAD, 478, 555, 250, 32, "Recarregar contagem do projeto");
        set_text(STATUS, &format!("BFC Studio iniciado. Roster: {} personagens | Mapas: {} | Trilhas: {}.\r\nO programa edita os arquivos usados pelo motor Rust; reinicie o jogo para recarregar o conteúdo.", roster_ids(&app_root()).len(), stage_count(&app_root()), music_count(&app_root())));
    }

    unsafe fn add_control(parent: Hwnd, id: i32, class: &str, text: &str, style: u32, x: i32, y: i32, w: i32, h: i32) {
        let class_w = to_wide(class); let text_w = to_wide(text);
        let instance = GetModuleHandleW(ptr::null());
        let hwnd = CreateWindowExW(0, class_w.as_ptr(), text_w.as_ptr(), WS_CHILD | WS_VISIBLE | style,
            x, y, w, h, parent, id as usize as Hmenu, instance, ptr::null_mut());
        if let Some(state) = APP.as_mut() { state.controls.insert(id, hwnd as isize); }
    }
    unsafe fn add_label(parent: Hwnd, id: i32, x: i32, y: i32, w: i32, h: i32, text: &str) {
        add_control(parent, id, "STATIC", text, SS_LEFT, x, y, w, h);
    }
    unsafe fn add_edit(parent: Hwnd, id: i32, x: i32, y: i32, w: i32, h: i32, text: &str) {
        add_control(parent, id, "EDIT", text, WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL, x, y, w, h);
    }
    unsafe fn add_edit_ex(parent: Hwnd, id: i32, x: i32, y: i32, w: i32, h: i32, text: &str, read_only: bool) {
        let mut style = WS_BORDER | WS_TABSTOP | ES_MULTILINE | ES_AUTOVSCROLL | WS_VSCROLL;
        if read_only { style |= ES_READONLY; } else { style |= ES_AUTOHSCROLL; }
        add_control(parent, id, "EDIT", text, style, x, y, w, h);
    }
    unsafe fn add_button(parent: Hwnd, id: i32, x: i32, y: i32, w: i32, h: i32, text: &str) {
        add_control(parent, id, "BUTTON", text, BS_PUSHBUTTON | WS_TABSTOP, x, y, w, h);
    }

    unsafe fn handle_command(parent: Hwnd, id: i32) {
        let root = app_root();
        match id {
            B_SPRITE => choose_file(parent, C_SPRITE, image_filter(), "Selecione a arte principal do personagem"),
            B_FACE => choose_file(parent, C_FACE, image_filter(), "Selecione a imagem do rosto (opcional)"),
            B_VOICE => choose_file(parent, C_VOICE, audio_filter(), "Selecione uma fala gravada MP3/WAV (opcional)"),
            B_STAGE_IMAGE => choose_file(parent, S_IMAGE, image_filter(), "Selecione a imagem do cenário"),
            B_MUSIC_FILE => choose_file(parent, M_FILE, audio_filter(), "Selecione a trilha MP3/WAV"),
            B_SAVE_CHARACTER => report(create_or_update_character(&root), "Personagem salvo"),
            B_LOAD_CHARACTER => report(load_character_into_form(&root), "Personagem carregado"),
            B_ADD_STAGE => report(add_stage(&root), "Mapa salvo"),
            B_LOAD_STAGE => report(load_stage_into_form(&root), "Mapa carregado"),
            B_ADD_MUSIC => report(add_music(&root), "Trilha importada"),
            B_AUDIT => report(audit_project(&root), "Auditoria concluida"),
            B_OPEN_FOLDER => {
                let root_w = to_wide(&root.display().to_string()); let op = to_wide("open");
                let _ = ShellExecuteW(parent, op.as_ptr(), root_w.as_ptr(), ptr::null(), ptr::null(), SW_SHOWNORMAL);
                set_text(STATUS, "Pasta do projeto aberta no Explorer.");
            }
            B_RELOAD => set_text(STATUS, &format!("Roster: {} personagens | Mapas: {} | Trilhas: {} | Audite para validar arquivos.", roster_ids(&root).len(), stage_count(&root), music_count(&root))),
            _ => {}
        }
    }

    unsafe fn choose_file(parent: Hwnd, control_id: i32, filter: Vec<u16>, title: &str) {
        let root = app_root();
        let title_w = to_wide(title);
        let initial = wide_path(&root);
        let mut file_buffer = vec![0u16; 32768];
        let mut ofn: OpenFileNameW = mem::zeroed();
        ofn.l_struct_size = mem::size_of::<OpenFileNameW>() as u32;
        ofn.hwnd_owner = parent;
        ofn.lpstr_filter = filter.as_ptr();
        ofn.n_filter_index = 1;
        ofn.lpstr_file = file_buffer.as_mut_ptr();
        ofn.n_max_file = file_buffer.len() as u32;
        ofn.lpstr_initial_dir = initial.as_ptr();
        ofn.lpstr_title = title_w.as_ptr();
        ofn.flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;
        if GetOpenFileNameW(&mut ofn) != 0 {
            let len = file_buffer.iter().position(|c| *c == 0).unwrap_or(file_buffer.len());
            let path = String::from_utf16_lossy(&file_buffer[..len]);
            set_text(control_id, &path);
        }
    }

    fn image_filter() -> Vec<u16> { to_wide("Imagens\0PNG, JPEG/JPG, BMP, WebP, GIF e TIFF\0*.png;*.jpg;*.jpeg;*.bmp;*.webp;*.gif;*.tif;*.tiff\0Todos os arquivos\0*.*\0\0") }
    fn audio_filter() -> Vec<u16> { to_wide("Áudio\0MP3 e WAV\0*.mp3;*.wav\0Todos os arquivos\0*.*\0\0") }

    unsafe fn field(id: i32) -> String {
        let handle = APP.as_ref().and_then(|s| s.controls.get(&id)).copied().unwrap_or(0) as Hwnd;
        if handle.is_null() { return String::new(); }
        let len = GetWindowTextLengthW(handle).max(1) as usize;
        let mut buf = vec![0u16; (len + 2).min(16384)];
        let copied = GetWindowTextW(handle, buf.as_mut_ptr(), buf.len() as i32).max(0) as usize;
        String::from_utf16_lossy(&buf[..copied])
    }
    unsafe fn set_text(id: i32, text: &str) {
        if let Some(handle) = APP.as_ref().and_then(|s| s.controls.get(&id)).copied() {
            let value = to_wide(text);
            SetWindowTextW(handle as Hwnd, value.as_ptr());
        }
    }
    unsafe fn report(result: Result<String, String>, prefix: &str) {
        match result {
            Ok(message) => set_text(STATUS, &format!("{}: {}", prefix, message)),
            Err(error) => set_text(STATUS, &format!("ERRO: {}", error)),
        }
    }

    fn create_or_update_character(root: &Path) -> Result<String, String> {
        let (name, id, speed, power, hp, role, home_stage, music, phrase, sprite_path, face_path, voice_path) = unsafe {
            (field(C_NAME).trim().to_owned(), normalize_id(&field(C_ID)), parse_float(&field(C_SPEED), 4.5, 1.0, 12.0),
             parse_float(&field(C_POWER), 1.0, 0.1, 5.0), field(C_HP).trim().parse::<i32>().unwrap_or(200).clamp(1, 5000),
             normalize_role(&field(C_ROLE)), normalize_optional(&field(C_HOME_STAGE)), normalize_optional(&field(C_MUSIC)),
             field(C_PHRASE).trim().to_owned(), field(C_SPRITE).trim().to_owned(), field(C_FACE).trim().to_owned(), field(C_VOICE).trim().to_owned())
        };
        if name.is_empty() { return Err("Informe o nome do personagem.".into()); }
        if id.is_empty() { return Err("O ID deve conter letras, números ou hífens.".into()); }
        if !matches!(role.as_str(), "normal" | "subboss" | "boss") { return Err("Papel inválido. Use normal, subboss ou boss.".into()); }
        let directory = root.join("chars").join(&id);
        fs::create_dir_all(&directory).map_err(|e| format!("Nao foi possivel criar a pasta do personagem: {e}"))?;
        let sprite_out = directory.join("sprite.png");
        if !sprite_path.is_empty() {
            save_character_sprite(Path::new(&sprite_path), &sprite_out).map_err(|e| format!("Arte principal: {e}"))?;
        } else if !sprite_out.is_file() {
            return Err("Escolha a arte principal para um personagem novo.".into());
        }
        let face_out = directory.join("face.png");
        if !face_path.is_empty() {
            save_face(Path::new(&face_path), &face_out, false).map_err(|e| format!("Face: {e}"))?;
        } else if !face_out.is_file() {
            let source = if !sprite_path.is_empty() { PathBuf::from(&sprite_path) } else { sprite_out.clone() };
            save_face(&source, &face_out, true).map_err(|e| format!("Nao foi possivel criar face.png: {e}"))?;
        }

        let json_path = directory.join("character.json");
        let exists = json_path.is_file();
        let mut character = if exists {
            fs::read(&json_path).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()).unwrap_or_else(|| json!({}))
        } else {
            json!({
                "stats": {}, "body": {"w": 70, "h": 230}, "anchor": {"x": 0.5, "y": 0.92},
                "chroma": {"color": "#00ff00", "tolerance": 90, "softness": 60, "maxHeight": 560},
                "size": 300, "facing": "right", "maxFrames": 24, "moves": {}, "effects": {},
                "animations": {"idle":{"prefix":"idle","fps":8,"mode":"loop"}, "walk":{"prefix":"walk","fps":10,"mode":"loop"},
                    "run":{"prefix":"run","fps":12,"mode":"loop"}, "jump":{"prefix":"jump","fps":10,"mode":"vy"},
                    "attack":{"prefix":"attack","fps":12,"mode":"progress"}, "kick":{"prefix":"kick","fps":12,"mode":"progress"}},
                "rig": {"neck": 0.24, "hip": 0.57, "split": 0.493, "flip": false, "skirt": false}
            })
        };
        character["name"] = json!(name);
        character["stats"]["speed"] = json!(speed);
        character["stats"]["power"] = json!(power);
        character["stats"]["hp"] = json!(hp);
        character["stage"] = if home_stage.is_empty() { Value::Null } else { json!(home_stage) };
        character["music"] = if music.is_empty() { Value::Null } else { json!(music.replace('\\', "/")) };
        character["voice"] = json!(id);
        character["campaignRole"] = json!(role);
        character["art"]["sprite"] = json!("sprite.png");
        character["art"]["face"] = json!("face.png");
        if phrase.is_empty() {
            if let Some(win) = character.get_mut("voices").and_then(|v| v.get_mut("win")) { if let Some(obj) = win.as_object_mut() { obj.remove("text"); } }
        } else {
            character["voices"]["win"]["text"] = json!(phrase);
        }
        if !voice_path.is_empty() {
            let source = Path::new(&voice_path);
            let extension = source.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            if extension != "mp3" && extension != "wav" { return Err("A fala gravada deve ser MP3 ou WAV.".into()); }
            let target = directory.join(format!("win.{}", extension));
            fs::copy(source, &target).map_err(|e| format!("Nao foi possivel copiar a fala: {e}"))?;
            character["voices"]["win"]["files"] = json!([format!("win.{}", extension)]);
        }
        let bytes = serde_json::to_vec_pretty(&character).map_err(|e| e.to_string())?;
        fs::write(&json_path, bytes).map_err(|e| format!("Nao foi possivel salvar character.json: {e}"))?;
        add_to_roster(root, &id)?;
        enforce_unique_role(root, &id, &role)?;
        if !exists {
            let story_dir = root.join("story").join(&id);
            fs::create_dir_all(&story_dir).map_err(|e| e.to_string())?;
            let story_path = story_dir.join("historia.txt");
            if !story_path.exists() {
                fs::write(&story_path, format!("{} venceu a campanha Brazil Fight Combat.\n\nEscreva aqui a história de final deste personagem.\n", name)).map_err(|e| e.to_string())?;
            }
            let guide_path = directory.join("README.txt");
            if !guide_path.exists() {
                fs::write(&guide_path, "Personagem criado pelo BFC Studio.\nAdicione frames PNG numerados: idle_01.png, walk_01.png, run_01.png, jump_01.png, attack_01.png, kick_01.png, special_01.png, crouch_01.png, defense_01.png, hit_01.png, ko_01.png.\nTodos os frames devem manter o mesmo canvas e a mesma posição dos pés.\n").map_err(|e| e.to_string())?;
            }
        }
        if !home_stage.is_empty() && !stage_exists(root, &home_stage) {
            // O personagem é salvo, mas o status avisa que o stage precisa existir.
            return Ok(format!("{} salvo. AVISO: o mapa da casa '{}' ainda não está em data/stages.ini.", name, home_stage));
        }
        Ok(format!("{} ({}) salvo em chars/{}. Arte e face em PNG; roster atualizado.", name, role, id))
    }

    fn load_character_into_form(root: &Path) -> Result<String, String> {
        let id = unsafe { normalize_id(&field(C_ID)) };
        if id.is_empty() { return Err("Digite o ID do personagem e tente novamente.".into()); }
        let directory = root.join("chars").join(&id);
        let json_path = directory.join("character.json");
        let bytes = fs::read(&json_path).map_err(|e| format!("Nao consegui abrir {}: {e}", json_path.display()))?;
        let c: Value = serde_json::from_slice(&bytes).map_err(|e| format!("character.json inválido: {e}"))?;
        unsafe {
            set_text(C_NAME, c.get("name").and_then(Value::as_str).unwrap_or(&id));
            set_text(C_ID, &id);
            set_text(C_SPEED, &c.pointer("/stats/speed").and_then(Value::as_f64).unwrap_or(4.5).to_string());
            set_text(C_POWER, &c.pointer("/stats/power").and_then(Value::as_f64).unwrap_or(1.0).to_string());
            set_text(C_HP, &c.pointer("/stats/hp").and_then(Value::as_i64).unwrap_or(200).to_string());
            set_text(C_ROLE, c.get("campaignRole").or_else(|| c.get("campaign_role")).and_then(Value::as_str).unwrap_or_else(|| if id == "viktor-krayller" { "boss" } else if id == "amanda" { "subboss" } else { "normal" }));
            set_text(C_HOME_STAGE, c.get("stage").and_then(Value::as_str).unwrap_or(""));
            set_text(C_MUSIC, c.get("music").and_then(Value::as_str).unwrap_or(""));
            set_text(C_PHRASE, c.pointer("/voices/win/text").and_then(Value::as_str).unwrap_or(""));
            set_text(C_SPRITE, &directory.join(c.pointer("/art/sprite").and_then(Value::as_str).unwrap_or("sprite.png")).display().to_string());
            set_text(C_FACE, &directory.join(c.pointer("/art/face").and_then(Value::as_str).unwrap_or("face.png")).display().to_string());
            let voice_file = directory.join("win.mp3");
            let voice_file = if voice_file.is_file() { voice_file } else { directory.join("win.wav") };
            // Guarde o caminho em uma variável com vida útil suficiente antes de passá-lo ao formulário.
            let voice_path = if voice_file.is_file() {
                voice_file.display().to_string()
            } else {
                String::new()
            };
            set_text(C_VOICE, &voice_path);
        }
        Ok(format!("Dados de '{}' carregados no formulário.", id))
    }

    fn add_stage(root: &Path) -> Result<String, String> {
        let (id, name, ground, music, source) = unsafe {
            (normalize_id(&field(S_ID)), safe_ini(&field(S_NAME)), parse_float(&field(S_GROUND), 470.0, 0.0, 540.0),
             normalize_optional(&field(S_MUSIC)).replace('\\', "/"), field(S_IMAGE).trim().to_owned())
        };
        if id.is_empty() || name.is_empty() { return Err("Informe o ID e o nome do mapa.".into()); }
        let existing = load_stage_records(root).into_iter().find(|(stage_id, _, _)| stage_id == &id);
        if source.is_empty() && existing.is_none() { return Err("Selecione uma imagem válida para o novo mapa.".into()); }
        let index = load_stage_records(root).iter().position(|(stage_id, _, _)| stage_id == &id).unwrap_or_else(|| stage_count(root));
        if existing.is_none() && index >= 64 { return Err("O motor aceita no máximo 64 mapas no momento.".into()); }
        let relative = if !source.is_empty() {
            let source_path = Path::new(&source);
            if !source_path.is_file() { return Err("O arquivo de imagem indicado não existe.".into()); }
            let relative = format!("assets/images/stages/stage_{}.png", id);
            let target = root.join(&relative);
            if existing.is_none() && target.exists() { return Err(format!("O arquivo {} já existe; escolha outro ID.", relative)); }
            save_stage_image(source_path, &target).map_err(|e| format!("Conversão de imagem falhou: {e}"))?;
            relative
        } else {
            existing.as_ref().map(|(_, image, _)| image.clone()).unwrap_or_default()
        };
        let music_value = if music.is_empty() || music.eq_ignore_ascii_case("null") { "null".to_owned() } else { music };
        update_stage_ini(root, index, &id, &name, &relative, &music_value, ground)?;
        unsafe { set_text(C_HOME_STAGE, &id); }
        Ok(format!("Mapa '{}' salvo como {}. {} estágios no total.", name, relative, stage_count(root)))
    }

    fn update_stage_ini(root: &Path, stage_index: usize, id: &str, name: &str, image: &str, music: &str, ground: f32) -> Result<(), String> {
        let path = root.join("data/stages.ini");
        let original = fs::read_to_string(&path).unwrap_or_default();
        let mut lines = original.lines().map(str::to_owned).collect::<Vec<_>>();
        let mut section_start: Option<usize> = None;
        let mut section_end = lines.len();
        let mut found_index = stage_index;
        for index in 0..lines.len() {
            if lines[index].trim().starts_with("[stage.") {
                let end = (index + 1..lines.len()).find(|i| lines[*i].trim().starts_with('[')).unwrap_or(lines.len());
                let section_has_id = lines[index + 1..end].iter().any(|line| line.trim() == format!("id={}", id));
                if section_has_id {
                    section_start = Some(index);
                    section_end = end;
                    if let Some(number) = lines[index].trim().strip_prefix("[stage.").and_then(|s| s.strip_suffix(']')).and_then(|s| s.parse::<usize>().ok()) { found_index = number; }
                    break;
                }
            }
        }
        let block = vec![format!("[stage.{:02}]", found_index), format!("id={}", id), format!("name={}", safe_ini(name)),
            format!("image={}", image.replace('\\', "/")), format!("music={}", music), format!("ground={:.0}", ground)];
        if let Some(start) = section_start {
            lines.splice(start..section_end, block);
        } else {
            if !lines.is_empty() && !lines.last().map(|l| l.is_empty()).unwrap_or(false) { lines.push(String::new()); }
            lines.extend(block);
        }
        let mut content = lines.join("\n"); content.push('\n');
        fs::write(path, content).map_err(|e| format!("Nao consegui atualizar stages.ini: {e}"))
    }

    fn load_stage_into_form(root: &Path) -> Result<String, String> {
        let id = unsafe { normalize_id(&field(S_ID)) };
        if id.is_empty() { return Err("Digite o ID do mapa para carregar.".into()); }
        let path = root.join("data/stages.ini");
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // Parse individual sections to find the requested ID without relying on section names.
        let mut current = HashMap::<String, String>::new();
        let mut matched = None;
        for line in content.lines().chain(std::iter::once("[end]")) {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                if current.get("id").map(String::as_str) == Some(id.as_str()) { matched = Some(current.clone()); break; }
                current.clear(); continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') { current.insert(key.trim().to_owned(), value.trim().to_owned()); }
        }
        let record = matched.ok_or_else(|| format!("Nenhum mapa com ID '{}' foi encontrado.", id))?;
        let get = |key: &str| record.get(key).map(String::as_str).unwrap_or("");
        unsafe {
            set_text(S_ID, &id); set_text(S_NAME, get("name")); set_text(S_GROUND, if get("ground").is_empty() { "470" } else { get("ground") });
            set_text(S_MUSIC, if get("music") == "null" { "" } else { get("music") });
            set_text(S_IMAGE, &root.join(get("image")).display().to_string());
        }
        Ok(format!("Mapa '{}' carregado. Edite a música ou selecione uma imagem nova e clique em Adicionar / salvar mapa.", id))
    }

    fn add_music(root: &Path) -> Result<String, String> {
        let (id, source) = unsafe { (normalize_id(&field(M_ID)), field(M_FILE).trim().to_owned()) };
        if id.is_empty() { return Err("Informe um ID para a trilha.".into()); }
        let src = Path::new(&source);
        if !src.is_file() { return Err("Selecione um arquivo MP3 ou WAV válido.".into()); }
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        if ext != "mp3" && ext != "wav" { return Err("Formato não suportado. Use MP3 ou WAV para reprodução nativa.".into()); }
        let directory = root.join("assets/audio/music");
        fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let relative = format!("assets/audio/music/{}.{}", id, ext);
        let target = root.join(&relative);
        let library_path = root.join("data/music_library.json");
        let mut library = fs::read(&library_path).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()).unwrap_or_else(|| json!({"tracks":[]}));
        if !library["tracks"].is_array() { library["tracks"] = json!([]); }
        let tracks = library["tracks"].as_array_mut().unwrap();
        let old_path = tracks.iter().find(|entry| entry.get("id").and_then(Value::as_str) == Some(id.as_str()))
            .and_then(|entry| entry.get("path")).and_then(Value::as_str).map(str::to_owned);
        if target.exists() && old_path.is_none() {
            return Err(format!("O arquivo {} já existe, mas não está catalogado. Use outro ID ou adicione-o manualmente à biblioteca antes de substituí-lo.", relative));
        }
        let same_file = src.canonicalize().ok().zip(target.canonicalize().ok()).map(|(a, b)| a == b).unwrap_or(false);
        if !same_file { fs::copy(src, &target).map_err(|e| format!("Nao consegui copiar a trilha: {e}"))?; }
        if let Some(existing) = tracks.iter_mut().find(|entry| entry.get("id").and_then(Value::as_str) == Some(id.as_str())) {
            *existing = json!({"id":id,"path":relative});
        } else {
            tracks.push(json!({"id":id,"path":relative}));
        }
        fs::write(library_path, serde_json::to_vec_pretty(&library).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let updated_references = if let Some(old) = old_path.as_deref() {
            // Mantém o arquivo antigo no disco por segurança, mas redireciona as referências
            // registradas para a nova extensão/caminho. Referências personalizadas não registradas
            // continuam funcionando porque o asset antigo não é apagado automaticamente.
            retarget_music_references(root, &id, old, &relative).unwrap_or(0)
        } else { 0 };
        unsafe { set_text(C_MUSIC, &relative); set_text(S_MUSIC, &relative); }
        Ok(format!("Trilha {}: {}. Referências existentes atualizadas: {}. O caminho também foi preenchido nos campos Música do personagem e do mapa.", if old_path.is_some() { "substituída" } else { "importada" }, relative, updated_references))
    }

    fn retarget_music_references(root: &Path, id: &str, old_path: &str, new_path: &str) -> Result<usize, String> {
        let old_stem = Path::new(old_path).file_stem().and_then(|v| v.to_str()).unwrap_or("");
        let matches_old = |value: &str| value == id || value == old_stem || value == old_path;
        let mut changed = 0usize;
        for character_id in roster_ids(root) {
            let path = root.join("chars").join(&character_id).join("character.json");
            let Ok(bytes) = fs::read(&path) else { continue; };
            let Ok(mut character) = serde_json::from_slice::<Value>(&bytes) else { continue; };
            let old = character.get("music").and_then(Value::as_str).map(str::to_owned);
            if old.as_deref().map(matches_old).unwrap_or(false) {
                character["music"] = json!(new_path);
                fs::write(&path, serde_json::to_vec_pretty(&character).map_err(|e| e.to_string())?)
                    .map_err(|e| format!("Não foi possível atualizar a música de {}: {e}", character_id))?;
                changed += 1;
            }
        }
        let stage_file = root.join("data/stages.ini");
        let content = fs::read_to_string(&stage_file).unwrap_or_default();
        let mut in_stage = false;
        let mut stage_changed = false;
        let mut lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') { in_stage = trimmed.starts_with("[stage."); }
            if in_stage {
                if let Some((key, value)) = trimmed.split_once('=') {
                    if key.trim().eq_ignore_ascii_case("music") && matches_old(value.trim()) {
                        lines.push(format!("music={}", new_path));
                        stage_changed = true;
                        changed += 1;
                        continue;
                    }
                }
            }
            lines.push(line.to_owned());
        }
        if stage_changed {
            let mut output = lines.join("\n"); output.push('\n');
            fs::write(stage_file, output).map_err(|e| format!("Não foi possível atualizar data/stages.ini: {e}"))?;
        }
        Ok(changed)
    }

    fn audit_project(root: &Path) -> Result<String, String> {
        let ids = roster_ids(root);
        let stages = load_stage_records(root);
        let mut errors = Vec::new();
        let mut valid_characters = 0usize;
        let mut frame_count = 0usize;
        for id in &ids {
            let dir = root.join("chars").join(id);
            let json_path = dir.join("character.json");
            match fs::read(&json_path).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
                Some(j) => {
                    valid_characters += 1;
                    let sprite = j.pointer("/art/sprite").and_then(Value::as_str).unwrap_or("sprite.png");
                    let face = j.pointer("/art/face").and_then(Value::as_str).unwrap_or("face.png");
                    if !dir.join(sprite).is_file() { errors.push(format!("{}: arte ausente ({})", id, sprite)); }
                    if !dir.join(face).is_file() { errors.push(format!("{}: face ausente ({})", id, face)); }
                    if let Some(music) = j.get("music").and_then(Value::as_str).filter(|v| !v.trim().is_empty() && *v != "null") {
                        let music_path = if music.contains('/') || music.ends_with(".mp3") || music.ends_with(".wav") || music.ends_with(".ogg") {
                            if music.starts_with("assets/") || music.starts_with("chars/") { root.join(music) } else { dir.join(music) }
                        } else { root.join(format!("assets/audio/music/{}.wav", music)) };
                        if !music_path.is_file() { errors.push(format!("{}: trilha não encontrada ({})", id, music)); }
                    }
                    if let Some(files) = j.pointer("/voices/win/files").and_then(Value::as_array) {
                        for voice in files.iter().filter_map(Value::as_str) {
                            let direct = if voice.starts_with("assets/") || voice.starts_with("chars/") { root.join(voice) } else { dir.join(voice) };
                            let shared = root.join("assets/audio/voice").join(id).join(voice);
                            if !direct.is_file() && !shared.is_file() {
                                errors.push(format!("{}: arquivo de voz ausente ({})", id, voice));
                            }
                        }
                    }
                }
                None => errors.push(format!("{}: character.json ausente ou inválido", id)),
            }
            if let Ok(entries) = fs::read_dir(&dir) {
                frame_count += entries.flatten().filter(|entry| {
                    let path = entry.path();
                    path.file_name().and_then(|n| n.to_str()).map(|name| {
                        let lower = name.to_ascii_lowercase();
                        lower.ends_with(".png") && ["idle_", "walk_", "run_", "jump_", "attack_", "kick_", "special_", "crouch_", "defense_", "dead_", "hit_", "ko_"].iter().any(|prefix| lower.starts_with(prefix))
                    }).unwrap_or(false)
                }).count();
            }
        }
        for (id, image, music) in &stages {
            if !root.join(image).is_file() { errors.push(format!("Mapa {}: imagem ausente ({})", id, image)); }
            if let Some(path) = music.as_deref() {
                if path != "null" && !path.is_empty() {
                    let resolved = if path.contains('/') || path.ends_with(".mp3") || path.ends_with(".wav") || path.ends_with(".ogg") {
                        root.join(path)
                    } else { root.join(format!("assets/audio/music/{}.wav", path)) };
                    if !resolved.is_file() { errors.push(format!("Mapa {}: trilha ausente ({})", id, path)); }
                }
            }
        }
        if let Ok(bytes) = fs::read(root.join("data/music_library.json")) {
            if let Ok(library) = serde_json::from_slice::<Value>(&bytes) {
                if let Some(tracks) = library.get("tracks").and_then(Value::as_array) {
                    for track in tracks {
                        if let Some(path) = track.get("path").and_then(Value::as_str) {
                            if !root.join(path).is_file() { errors.push(format!("Biblioteca de música: arquivo ausente ({})", path)); }
                        }
                    }
                }
            }
        }
        let report = format!("PERSONAGENS: {} registrados, {} JSON válidos\r\nMAPAS: {} registrados\r\nTRILHAS: {} na biblioteca\r\nFRAMES: {} quadros de animação encontrados\r\nPROBLEMAS: {}\r\n{}",
            ids.len(), valid_characters, stages.len(), music_count(root), frame_count, errors.len(), errors.iter().take(24).cloned().collect::<Vec<_>>().join("\r\n"));
        Ok(report)
    }

    fn save_character_sprite(source: &Path, target: &Path) -> Result<(), String> {
        let mut image = image::open(source).map_err(|e| format!("formato de imagem inválido: {e}"))?.thumbnail(1200, 1600).to_rgba8();
        remove_edge_background(&mut image);
        crop_to_alpha(&image).save(target).map_err(|e| e.to_string())?;
        let opened = image::open(target).map_err(|e| e.to_string())?;
        let (w, h) = opened.dimensions();
        if h > 480 || w > 480 {
            let resized = opened.resize(480, 480, FilterType::Lanczos3);
            resized.save(target).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn save_face(source: &Path, target: &Path, derive_from_full_body: bool) -> Result<(), String> {
        let mut image = image::open(source).map_err(|e| format!("formato de imagem inválido: {e}"))?.thumbnail(1024, 1024).to_rgba8();
        remove_edge_background(&mut image);
        let (w, h) = image.dimensions();
        let (x, y, side) = if derive_from_full_body {
            let side = (w.min((h as f32 * 0.38) as u32)).max(1);
            ((w.saturating_sub(side)) / 2, 0, side)
        } else {
            let side = w.min(h).max(1);
            ((w - side) / 2, (h - side) / 2, side)
        };
        let square = imageops::crop_imm(&image, x, y, side, side).to_image();
        let face = imageops::resize(&square, 140, 140, FilterType::Lanczos3);
        face.save(target).map_err(|e| e.to_string())
    }

    fn save_stage_image(source: &Path, target: &Path) -> Result<(), String> {
        let image = image::open(source).map_err(|e| format!("imagem inválida: {e}"))?;
        image.thumbnail(1920, 1080).save(target).map_err(|e| e.to_string())
    }

    fn remove_edge_background(image: &mut RgbaImage) {
        let (width, height) = image.dimensions();
        if width < 2 || height < 2 { return; }
        let mut green = 0usize; let mut white = 0usize; let mut total = 0usize;
        for x in 0..width { for y in [0, height - 1] {
            let p = image.get_pixel(x, y); total += 1;
            if is_green(p) { green += 1; } if is_white(p) { white += 1; }
        }}
        for y in 1..height.saturating_sub(1) { for x in [0, width - 1] {
            let p = image.get_pixel(x, y); total += 1;
            if is_green(p) { green += 1; } if is_white(p) { white += 1; }
        }}
        let mode = if green * 5 > total { 1 } else if white * 2 > total { 2 } else { 0 };
        if mode == 0 { return; }
        let pixels = width as usize * height as usize;
        let mut visited = vec![false; pixels];
        let mut queue = VecDeque::<usize>::new();
        // Só enfileira pixels de fundo e marca na entrada: cada pixel entra uma vez,
        // evitando crescimento excessivo da fila em imagens grandes.
        for x in 0..width {
            enqueue_background(image, mode, x as usize, &mut visited, &mut queue);
            enqueue_background(image, mode, (height as usize - 1) * width as usize + x as usize, &mut visited, &mut queue);
        }
        for y in 1..height.saturating_sub(1) {
            enqueue_background(image, mode, y as usize * width as usize, &mut visited, &mut queue);
            enqueue_background(image, mode, y as usize * width as usize + width as usize - 1, &mut visited, &mut queue);
        }
        while let Some(index) = queue.pop_front() {
            let x = index % width as usize; let y = index / width as usize;
            let pixel = *image.get_pixel(x as u32, y as u32);
            image.put_pixel(x as u32, y as u32, Rgba([pixel[0], pixel[1], pixel[2], 0]));
            if x > 0 { enqueue_background(image, mode, index - 1, &mut visited, &mut queue); }
            if x + 1 < width as usize { enqueue_background(image, mode, index + 1, &mut visited, &mut queue); }
            if y > 0 { enqueue_background(image, mode, index - width as usize, &mut visited, &mut queue); }
            if y + 1 < height as usize { enqueue_background(image, mode, index + width as usize, &mut visited, &mut queue); }
        }
    }
    fn enqueue_background(image: &RgbaImage, mode: u8, index: usize, visited: &mut [bool], queue: &mut VecDeque<usize>) {
        if visited[index] { return; }
        let width = image.width() as usize;
        let pixel = image.get_pixel((index % width) as u32, (index / width) as u32);
        let is_background = if mode == 1 { is_green(pixel) } else { is_white(pixel) };
        if is_background { visited[index] = true; queue.push_back(index); }
    }
    fn is_green(p: &Rgba<u8>) -> bool { p[1] > 150 && p[0] < 110 && p[2] < 110 && p[1] > p[0].saturating_add(55) }
    fn is_white(p: &Rgba<u8>) -> bool { p[0] > 225 && p[1] > 225 && p[2] > 225 }

    fn crop_to_alpha(image: &RgbaImage) -> image::DynamicImage {
        let (w, h) = image.dimensions();
        let mut min_x = w; let mut min_y = h; let mut max_x = 0u32; let mut max_y = 0u32;
        let mut found = false;
        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel[3] > 0 { found = true; min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x); max_y = max_y.max(y); }
        }
        if !found { return image::DynamicImage::ImageRgba8(image.clone()); }
        let width = (max_x - min_x + 1).max(1); let height = (max_y - min_y + 1).max(1);
        image::DynamicImage::ImageRgba8(imageops::crop_imm(image, min_x, min_y, width, height).to_image())
    }

    fn add_to_roster(root: &Path, id: &str) -> Result<(), String> {
        let path = root.join("chars/chars.json");
        let mut roster = fs::read(&path).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()).unwrap_or_else(|| json!({"personagens":[]}));
        if !roster["personagens"].is_array() { roster["personagens"] = json!([]); }
        let entries = roster["personagens"].as_array_mut().unwrap();
        if !entries.iter().any(|v| v.as_str() == Some(id)) { entries.push(json!(id)); }
        fs::write(path, serde_json::to_vec_pretty(&roster).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }

    fn enforce_unique_role(root: &Path, chosen_id: &str, role: &str) -> Result<(), String> {
        if role == "normal" { return Ok(()); }
        for id in roster_ids(root) {
            if id == chosen_id { continue; }
            let path = root.join("chars").join(&id).join("character.json");
            let Ok(bytes) = fs::read(&path) else { continue; };
            let Ok(mut value) = serde_json::from_slice::<Value>(&bytes) else { continue; };
            let old_role = value.get("campaignRole").or_else(|| value.get("campaign_role")).and_then(Value::as_str)
                .map(str::to_owned).unwrap_or_else(|| if id == "viktor-krayller" { "boss".into() } else if id == "amanda" { "subboss".into() } else { "normal".into() });
            if old_role.eq_ignore_ascii_case(role) {
                value["campaignRole"] = json!("normal");
                fs::write(&path, serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    fn roster_ids(root: &Path) -> Vec<String> {
        let path = root.join("chars/chars.json");
        let Ok(bytes) = fs::read(path) else { return Vec::new(); };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else { return Vec::new(); };
        value.get("personagens").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default()
    }
    fn stage_count(root: &Path) -> usize { fs::read_to_string(root.join("data/stages.ini")).unwrap_or_default().lines().filter(|l| l.trim().starts_with("[stage.")).count() }
    fn music_count(root: &Path) -> usize {
        let path = root.join("data/music_library.json");
        if let Ok(bytes) = fs::read(path) { if let Ok(value) = serde_json::from_slice::<Value>(&bytes) { return value.get("tracks").and_then(Value::as_array).map(Vec::len).unwrap_or(0); } }
        fs::read_dir(root.join("assets/audio/music")).map(|e| e.flatten().filter(|entry| entry.path().extension().and_then(|x| x.to_str()).map(|x| matches!(x.to_ascii_lowercase().as_str(), "mp3" | "wav")).unwrap_or(false)).count()).unwrap_or(0)
    }
    fn load_stage_records(root: &Path) -> Vec<(String, String, Option<String>)> {
        let text = fs::read_to_string(root.join("data/stages.ini")).unwrap_or_default();
        let mut records = Vec::new(); let mut id = String::new(); let mut image = String::new(); let mut music: Option<String> = None;
        let flush = |records: &mut Vec<(String, String, Option<String>)>, id: &mut String, image: &mut String, music: &mut Option<String>| {
            if !id.is_empty() { records.push((id.clone(), image.clone(), music.clone())); }
            id.clear(); image.clear(); *music = None;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("[stage.") { flush(&mut records, &mut id, &mut image, &mut music); continue; }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() { "id" => id = value.trim().to_owned(), "image" => image = value.trim().to_owned(), "music" => { let v = value.trim(); if !v.is_empty() && v != "null" { music = Some(v.to_owned()); } }, _ => {} }
            }
        }
        flush(&mut records, &mut id, &mut image, &mut music); records
    }
    fn stage_exists(root: &Path, id: &str) -> bool { load_stage_records(root).iter().any(|(stage_id, _, _)| stage_id == id) }
    fn normalize_optional(value: &str) -> String {
        let value = safe_ini(value.trim());
        if value.eq_ignore_ascii_case("null") { String::new() } else { value }
    }
    fn safe_ini(value: &str) -> String { value.chars().map(|ch| if ch == '\r' || ch == '\n' || ch == '=' { ' ' } else { ch }).collect::<String>().trim().to_owned() }
    fn normalize_id(value: &str) -> String {
        value.trim().to_ascii_lowercase().chars().map(|ch| if ch.is_ascii_alphanumeric() || ch == '_' { ch } else { '-' })
            .collect::<String>().split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
    }
    fn normalize_role(value: &str) -> String {
        match value.trim().to_ascii_lowercase().replace('-', "").replace('_', "").as_str() {
            "boss" => "boss".into(), "subboss" => "subboss".into(), _ => "normal".into()
        }
    }
    fn parse_float(value: &str, fallback: f32, min: f32, max: f32) -> f32 { value.trim().replace(',', ".").parse::<f32>().unwrap_or(fallback).clamp(min, max) }

    fn find_project_root() -> Option<PathBuf> {
        let mut starts = Vec::new();
        if let Ok(current) = std::env::current_dir() { starts.push(current); }
        if let Ok(exe) = std::env::current_exe() { if let Some(parent) = exe.parent() { starts.push(parent.to_path_buf()); } }
        for start in starts { for candidate in start.ancestors() {
            if candidate.join("data/engine.ini").is_file() && candidate.join("chars/chars.json").is_file() { return Some(candidate.to_path_buf()); }
        }}
        None
    }
    fn app_root() -> PathBuf { unsafe { APP.as_ref().map(|s| s.root.clone()).unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) } }
    fn to_wide(value: &str) -> Vec<u16> { value.encode_utf16().chain(std::iter::once(0)).collect() }
    fn wide_path(path: &Path) -> Vec<u16> { path.as_os_str().encode_wide().chain(std::iter::once(0)).collect() }
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_app::run() {
        eprintln!("Falha ao iniciar BFC Studio: {error}");
        std::process::exit(1);
    }
}
