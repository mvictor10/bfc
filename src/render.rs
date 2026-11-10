//! Renderizador por software, independente da janela e facilmente testável.
use crate::animation::{EffectAnchor, EffectPlacement, Sprite, SpriteSet};
use crate::fighter::{Facing, Fighter, FighterState, MoveKind};
use crate::game::{Game, Scene};
use crate::geometry::Rect;

pub const LOGICAL_WIDTH: usize = 960;
pub const LOGICAL_HEIGHT: usize = 540;

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self { Self { width, height, pixels: vec![0; width * height] } }
    pub fn clear(&mut self, color: u32) { self.pixels.fill(color); }
    pub fn pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }
    pub fn fill_rect(&mut self, r: Rect, color: u32) {
        let x0 = (r.x.floor() as i32).clamp(0, self.width as i32);
        let y0 = (r.y.floor() as i32).clamp(0, self.height as i32);
        let x1 = (r.right().ceil() as i32).clamp(0, self.width as i32);
        let y1 = (r.bottom().ceil() as i32).clamp(0, self.height as i32);
        for y in y0..y1 { let start = y as usize * self.width + x0 as usize; let end = y as usize * self.width + x1 as usize; self.pixels[start..end].fill(color); }
    }
    pub fn stroke_rect(&mut self, r: Rect, color: u32) {
        self.line(r.x as i32, r.y as i32, r.right() as i32, r.y as i32, color);
        self.line(r.x as i32, r.bottom() as i32, r.right() as i32, r.bottom() as i32, color);
        self.line(r.x as i32, r.y as i32, r.x as i32, r.bottom() as i32, color);
        self.line(r.right() as i32, r.y as i32, r.right() as i32, r.bottom() as i32, color);
    }
    pub fn circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        let rr = radius * radius;
        for y in -radius..=radius { for x in -radius..=radius { if x * x + y * y <= rr { self.pixel(cx + x, cy + y, color); } } }
    }
    pub fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs(); let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs(); let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop { self.pixel(x0, y0, color); if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err; if e2 >= dy { err += dy; x0 += sx; } if e2 <= dx { err += dx; y0 += sy; }
        }
    }
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, scale: i32, color: u32) {
        let mut cursor = x;
        for ch in text.chars() {
            let glyph = glyph_for(ch.to_ascii_uppercase());
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..5 { if *bits & (1 << (4 - col)) != 0 {
                    for sy in 0..scale { for sx in 0..scale { self.pixel(cursor + col * scale + sx, y + row as i32 * scale + sy, color); } }
                } }
            }
            cursor += 6 * scale;
        }
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 { ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }

pub fn render_game(canvas: &mut Canvas, game: &Game, sprites1: &SpriteSet, sprites2: &SpriteSet, background: Option<&Sprite>) {
    match game.scene {
        Scene::Menu => draw_menu(canvas, game, sprites1, sprites2, background),
        Scene::Settings => draw_settings(canvas, game, background),
        Scene::Fight => draw_fight(canvas, game, sprites1, sprites2, background),
        Scene::Story => draw_story(canvas, game, sprites1, background),
    }
}

fn draw_menu(c: &mut Canvas, game: &Game, sprites1: &SpriteSet, sprites2: &SpriteSet, background: Option<&Sprite>) {
    draw_background(c, &game.config.sky, &game.config.floor, game.config.ground, background);
    c.fill_rect(Rect::new(48.0, 38.0, 864.0, 462.0), rgb(8, 12, 27));
    c.stroke_rect(Rect::new(48.0, 38.0, 864.0, 462.0), rgb(65, 188, 228));
    c.draw_text(155, 58, "BRAZIL FIGHT COMBAT", 4, rgb(237, 243, 255));
    c.draw_text(320, 105, "ELENCO ORIGINAL - PORT RUST", 1, rgb(73, 219, 235));
    let p1 = game.config.roster.get(game.selected_p1);
    let p2 = game.config.roster.get(game.selected_p2);
    c.draw_text(77, 135, "JOGADOR 1 [Q/E]", 1, rgb(255, 154, 154));
    c.draw_text(77, 152, &ascii_name(p1.map(|p| p.name.as_str()).unwrap_or("JOGADOR 1")), 1, rgb(255, 225, 225));
    if let Some(tag) = p1.and_then(|p| p.campaign_role.tag()) {
        c.draw_text(77, 170, tag, 1, if tag == "BOSS" { rgb(255, 86, 86) } else { rgb(255, 185, 75) });
    }
    c.draw_text(770, 135, "JOGADOR 2 [U/O]", 1, rgb(150, 194, 255));
    c.draw_text(770, 152, &ascii_name(p2.map(|p| p.name.as_str()).unwrap_or("JOGADOR 2")), 1, rgb(225, 237, 255));
    if let Some(tag) = p2.and_then(|p| p.campaign_role.tag()) {
        c.draw_text(770, 170, tag, 1, if tag == "BOSS" { rgb(255, 86, 86) } else { rgb(255, 185, 75) });
    }

    // As artes de seleção vêm das pastas originais dos personagens.
    if let Some(character) = p1 {
        draw_character_preview(c, character, sprites1, 220.0, 440.0, Facing::Right, game.tick, &game.config.moves);
    }
    if let Some(character) = p2 {
        draw_character_preview(c, character, sprites2, 740.0, 440.0, Facing::Left, game.tick, &game.config.moves);
    }

    let items = ["VERSUS 2 JOGADORES", "LUTA CONTRA CPU", "TREINAMENTO", "CAMPANHA ARCADE", "OBSERVAR IA VS IA", "CONFIGURACOES", "SAIR"];
    for (i, label) in items.iter().enumerate() {
        let y = 195 + i as i32 * 29;
        if i == game.menu_selection { c.fill_rect(Rect::new(342.0, y as f32 - 5.0, 276.0, 24.0), rgb(33, 82, 104)); }
        c.draw_text(358, y, label, 1, if i == game.menu_selection { rgb(255, 224, 113) } else { rgb(222, 230, 242) });
    }
    let stage_name = game.config.stages.get(game.current_stage).map(|s| ascii_name(&s.name)).unwrap_or_else(|| game.config.stage_name.clone());
    c.draw_text(85, 477, &format!("CENARIO [ / ]: {}", stage_name), 1, rgb(170, 198, 219));
    c.draw_text(335, 455, &format!("ARCADE: {} LUTAS NORMAIS + SUB-CHEFE + CHEFE", game.config.arcade_opponents), 1, rgb(73, 219, 235));
}

fn draw_character_preview(
    c: &mut Canvas,
    character: &crate::config::CharacterConfig,
    sprites: &SpriteSet,
    x: f32,
    feet_y: f32,
    facing: Facing,
    tick: u64,
    moves: &crate::config::MoveSet,
) {
    if let Some(sprite) = sprites.frame_for(FighterState::Idle, tick, 0, 0, 1, 0.0) {
        let orientation_mirror = (facing == Facing::Left) ^ character.facing_left;
        draw_sprite(c, sprite, x, feet_y, 205.0, character.anchor_x, character.anchor_y, orientation_mirror, 1.0);
    } else if let Some(art) = &sprites.base_art {
        if let Some(rig) = character.rig {
            let mut preview = Fighter::new(character.clone(), x, feet_y, facing);
            preview.state_ticks = (tick % u16::MAX as u64) as u16;
            draw_rig(c, &preview, art, rig, moves, tick);
        } else {
            let asset_mirror = (facing == Facing::Left) ^ character.facing_left;
            draw_sprite(c, art, x, feet_y, 205.0, character.anchor_x, character.anchor_y, asset_mirror, 1.0);
        }
    }
}

fn draw_settings(c: &mut Canvas, game: &Game, background: Option<&Sprite>) {
    draw_background(c, &game.config.sky, &game.config.floor, game.config.ground, background);
    c.fill_rect(Rect::new(135.0, 70.0, 690.0, 400.0), rgb(8, 12, 27));
    c.stroke_rect(Rect::new(135.0, 70.0, 690.0, 400.0), rgb(65, 188, 228));
    c.draw_text(325, 96, "CONFIGURACOES", 3, rgb(237, 243, 255));
    let difficulty = ["MUITO FACIL", "FACIL", "NORMAL (APRENDE)", "DIFICIL", "MUITO DIFICIL", "HARDCORE"]
        .get(game.config.cpu_difficulty as usize).copied().unwrap_or("NORMAL");
    let rows = [
        format!("DIFICULDADE CPU: {}", difficulty),
        format!("TEMPO DO ROUND: {} SEG", game.config.round_seconds),
        format!("ROUNDS PARA VENCER: {}", game.config.rounds_to_win),
        format!("ADVERSARIOS ANTES DOS CHEFES: {}", game.config.arcade_opponents),
        format!("MOSTRAR HITBOXES: {}", if game.show_hitboxes { "SIM" } else { "NAO" }),
        "VOLTAR AO MENU".to_owned(),
    ];
    for (index, row) in rows.iter().enumerate() {
        let y = 150 + index as i32 * 45;
        if index == game.settings_selection {
            c.fill_rect(Rect::new(190.0, y as f32 - 5.0, 580.0, 27.0), rgb(33, 82, 104));
        }
        c.draw_text(210, y, row, 2, if index == game.settings_selection { rgb(255, 224, 113) } else { rgb(222, 230, 242) });
    }
    c.draw_text(205, 410, "SETAS: NAVEGAR   Q/E: ALTERAR", 1, rgb(170, 198, 219));
    c.draw_text(205, 428, "ENTER: ALTERAR/VOLTAR   ESC: MENU", 1, rgb(170, 198, 219));
}

fn draw_fight(c: &mut Canvas, game: &Game, sprites1: &SpriteSet, sprites2: &SpriteSet, background: Option<&Sprite>) {
    draw_background(c, &game.config.sky, &game.config.floor, game.config.ground, background);
    draw_fighter(c, &game.p1, sprites1, game.tick, &game.config.moves);
    draw_fighter(c, &game.p2, sprites2, game.tick, &game.config.moves);
    draw_active_effects(c, game, sprites1, sprites2);
    draw_hud(c, game);
    for p in &game.particles {
        let color = if p.blocked { rgb(85, 205, 255) } else { rgb(255, 220, 82) };
        c.circle(p.x as i32, p.y as i32, if p.life > 10 { 3 } else { 2 }, color);
    }
    if game.show_hitboxes {
        c.stroke_rect(game.p1.body_rect(), rgb(80, 255, 120));
        c.stroke_rect(game.p2.body_rect(), rgb(80, 255, 120));
        let moves1 = game.config.moves.for_character(&game.p1.config);
        let moves2 = game.config.moves.for_character(&game.p2.config);
        if let Some(r) = game.p1.attack_rect(&moves1) { c.stroke_rect(r, rgb(255, 65, 65)); }
        if let Some(r) = game.p2.attack_rect(&moves2) { c.stroke_rect(r, rgb(255, 65, 65)); }
    }
    if game.round_pause_ticks > 0 {
        c.fill_rect(Rect::new(300.0, 225.0, 360.0, 90.0), rgb(13, 17, 31));
        c.draw_text(355, 240, &format!("ROUND {}", game.round_number), 3, rgb(255, 225, 120));
        c.draw_text(340, 282, "READY", 2, rgb(242, 245, 252));
    }
    if game.match_over {
        c.fill_rect(Rect::new(240.0, 205.0, 480.0, 125.0), rgb(13, 17, 31));
        c.stroke_rect(Rect::new(240.0, 205.0, 480.0, 125.0), rgb(255, 214, 88));
        let message = match game.winner { 1 => "JOGADOR 1 VENCEU", 2 => "JOGADOR 2 VENCEU", _ => "EMPATE" };
        c.draw_text(300, 232, message, 2, rgb(255, 225, 120));
        c.draw_text(330, 284, "ENTER - CONTINUAR", 2, rgb(242, 245, 252));
    }
}

fn draw_story(c: &mut Canvas, game: &Game, sprites1: &SpriteSet, background: Option<&Sprite>) {
    draw_background(c, &game.config.sky, &game.config.floor, game.config.ground, background);
    c.fill_rect(Rect::new(55.0, 45.0, 850.0, 450.0), rgb(10, 14, 29));
    c.stroke_rect(Rect::new(55.0, 45.0, 850.0, 450.0), rgb(65, 188, 228));
    c.draw_text(92, 78, &ascii_name(&game.story_title), 2, rgb(255, 224, 113));
    if let Some(art) = &sprites1.base_art {
        draw_sprite(c, art, 790.0, 425.0, 210.0, 0.5, 0.92, false, 1.0);
    }
    let page = game.story_pages.get(game.story_page).map(String::as_str).unwrap_or(&game.story_text);
    for (index, line) in page.lines().enumerate() {
        let y = 130 + index as i32 * 25;
        c.draw_text(85, y, &ascii_name(line), 1, rgb(225, 231, 242));
    }
    c.draw_text(80, 438, &format!("PAGINA {}/{}", game.story_page + 1, game.story_pages.len().max(1)), 1, rgb(170, 198, 219));
    let prompt = if game.story_page + 1 < game.story_pages.len() { "ENTER - PROXIMA PAGINA" } else { "ENTER - VOLTAR AO MENU" };
    c.draw_text(330, 458, prompt, 1, rgb(73, 219, 235));
}

fn draw_background(c: &mut Canvas, sky: &[u8; 3], floor: &[u8; 3], ground: f32, background: Option<&Sprite>) {
    if let Some(bg) = background {
        if bg.width == c.width && bg.height == c.height && bg.pixels.len() == c.pixels.len() {
            for (dst, src) in c.pixels.iter_mut().zip(bg.pixels.iter()) { *dst = *src & 0x00FF_FFFF; }
            return;
        }
    }
    for y in 0..c.height {
        let t = (y as f32 / c.height as f32).min(1.0);
        let color = rgb(mix(sky[0], floor[0], t * 0.35), mix(sky[1], floor[1], t * 0.35), mix(sky[2], floor[2], t * 0.35));
        let start = y * c.width; c.pixels[start..start + c.width].fill(color);
    }
    for i in 0..12 {
        let x = i * 87; let h = 75 + ((i * 37) % 105);
        c.fill_rect(Rect::new(x as f32, ground - h as f32, 55.0, h as f32), rgb(21, 28, 48));
        for wy in 0..(h / 22).min(6) { c.fill_rect(Rect::new((x + 9) as f32, ground - h as f32 + 10.0 + wy as f32 * 22.0, 7.0, 8.0), rgb(58, 78, 104)); }
    }
    c.fill_rect(Rect::new(0.0, ground, c.width as f32, (c.height as f32 - ground).max(0.0)), rgb(floor[0], floor[1], floor[2]));
    c.line(0, ground as i32, c.width as i32, ground as i32, rgb(178, 185, 194));
    for i in 0..18 { let x = i * 61; c.line(x, ground as i32 + 24, x + 28, ground as i32 + 24, rgb(73, 79, 89)); }
}
fn mix(a: u8, b: u8, t: f32) -> u8 { (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8 }

fn draw_fighter(c: &mut Canvas, f: &Fighter, sprites: &SpriteSet, tick: u64, base_moves: &crate::config::MoveSet) {
    let moves = base_moves.for_character(&f.config);
    let attack_duration = match f.state { FighterState::Attack(kind) => moves.get(kind).duration() as u64, _ => 1 };
    let source = sprites.frame_for(
        f.state,
        f.animation_ticks,
        f.state_ticks as u64,
        f.attack_ticks as u64,
        attack_duration,
        f.velocity_y,
    );
    let face_sign = if f.facing == Facing::Right { 1.0 } else { -1.0 };
    let sprite_sign = if f.config.facing_left { -1.0 } else { 1.0 };
    let mirror = face_sign * sprite_sign < 0.0;
    if let Some(sprite) = source {
        draw_sprite(c, sprite, f.x, f.feet_y, f.config.size as f32, f.config.anchor_x, f.config.anchor_y, mirror, 1.0);
    } else if let (Some(art), Some(rig)) = (&sprites.base_art, f.config.rig) {
        draw_rig(c, f, art, rig, &moves, tick);
    } else if let Some(art) = &sprites.base_art {
        draw_sprite(c, art, f.x, f.feet_y, f.config.size as f32, f.config.anchor_x, f.config.anchor_y, mirror, 1.0);
    } else { draw_fallback(c, f); }
    if f.state == FighterState::Ko {
        let dizzy = sprites.effect_frame("dizzy", tick);
        if let Some(effect) = dizzy {
            let placement = sprites.effect_placement("dizzy").unwrap_or(EffectPlacement {
                anchor: EffectAnchor::Head, offset_x: 0.0, offset_y: 185.0, scale: 1.0, additive: false,
            });
            draw_effect_sprite(c, effect, f, placement);
        } else {
            // Plano B equivalente ao efeito de estrelas giratórias descrito no jogo original.
            let x = f.x as i32;
            let y = (f.feet_y - f.config.size as f32 - 12.0) as i32;
            c.circle(x - 14, y, 3, rgb(255, 225, 91));
            c.circle(x, y - 5, 3, rgb(255, 225, 91));
            c.circle(x + 14, y, 3, rgb(255, 225, 91));
        }
        c.draw_text((f.x - 10.0) as i32, (f.feet_y - f.config.size as f32 - 35.0) as i32, "KO", 2, rgb(255, 225, 91));
    }
}

fn draw_active_effects(c: &mut Canvas, game: &Game, sprites1: &SpriteSet, sprites2: &SpriteSet) {
    for active in &game.active_effects {
        let (attacker, target, attacker_sprites, target_sprites) = if active.attacker == 1 {
            (&game.p1, &game.p2, sprites1, sprites2)
        } else {
            (&game.p2, &game.p1, sprites2, sprites1)
        };
        // O original configura FX de impacto/bloqueio no personagem que os recebe.
        let source = if matches!(active.name.as_str(), "hit" | "block") { target_sprites } else { attacker_sprites };
        let Some(sprite) = source.effect_frame(&active.name, active.age as u64) else { continue; };
        let placement = source.effect_placement(&active.name).unwrap_or(EffectPlacement {
            anchor: EffectAnchor::Target, offset_x: 0.0, offset_y: 150.0, scale: 1.0, additive: false,
        });
        let anchor = match placement.anchor {
            EffectAnchor::Attacker => attacker,
            EffectAnchor::Target | EffectAnchor::Head => target,
        };
        draw_effect_sprite(c, sprite, anchor, placement);
    }
}

fn draw_effect_sprite(c: &mut Canvas, sprite: &Sprite, anchor: &Fighter, placement: EffectPlacement) {
    let direction = if anchor.facing == Facing::Right { 1.0 } else { -1.0 };
    let x = anchor.x + direction * placement.offset_x;
    let y = anchor.feet_y - placement.offset_y;
    // Os FX originais compartilham a escala de exibição dos quadros do personagem.
    let frame_height = anchor.config.max_frame_height.max(1) as f32;
    let target_height = (sprite.height as f32 * anchor.config.size as f32 / frame_height * placement.scale)
        .clamp(12.0, 360.0);
    draw_sprite_blended(c, sprite, x, y, target_height, 0.5, 0.5, false, 1.0, placement.additive);
}

/// Pose para o rig de recortes, portada da lógica de RigAnimator.js do projeto original.
#[derive(Clone, Copy, Debug)]
struct RigPose {
    dx: f32, dy: f32, lean: f32, head: f32,
    left_rotation: f32, right_rotation: f32,
    left_dy: f32, right_dy: f32,
    left_scale_y: f32, right_scale_y: f32,
    torso_scale_x: f32, torso_scale_y: f32,
    crouch: f32,
}

impl Default for RigPose {
    fn default() -> Self {
        Self { dx: 0.0, dy: 0.0, lean: 0.0, head: 0.0, left_rotation: 0.0, right_rotation: 0.0,
            left_dy: 0.0, right_dy: 0.0, left_scale_y: 1.0, right_scale_y: 1.0,
            torso_scale_x: 1.0, torso_scale_y: 1.0, crouch: 0.0 }
    }
}

fn rig_pose(f: &Fighter, moves: &crate::config::MoveSet) -> RigPose {
    use FighterState::*;
    // `animation_ticks` replica o contador f.t do jogo JavaScript, que só zera
    // no início de um golpe, ao receber dano ou ao entrar em K.O.
    let t = if matches!(f.state, Attack(_)) { f.attack_ticks as f32 } else { f.animation_ticks as f32 };
    let mut p = RigPose::default();
    match f.state {
        Idle => {
            p.torso_scale_y = 1.0 + 0.012 * (t / 9.0).sin();
            p.head = 0.03 * (t / 9.0 + 1.0).sin();
            p.left_rotation = 0.04 * (t / 18.0).sin();
            p.right_rotation = -0.04 * (t / 18.0).sin();
            p.dy = 1.5 * (t / 9.0).sin();
        }
        Walk | Run => {
            let wave = (t / 4.0).sin();
            p.left_rotation = 0.6 * wave;
            p.right_rotation = -0.6 * wave;
            p.left_dy = -wave.max(0.0) * 16.0;
            p.right_dy = -(-wave).max(0.0) * 16.0;
            p.lean = 0.16;
            p.head = -0.08;
            p.dy = -wave.abs() * if f.state == Run { 14.0 } else { 10.0 };
            if f.state == Run { p.dx = wave * 2.5; }
        }
        Jump => {
            if f.velocity_y < 0.0 {
                p.left_rotation = -0.9; p.right_rotation = -0.5;
                p.left_dy = -34.0; p.right_dy = -46.0;
                p.torso_scale_y = 1.04; p.lean = 0.08;
            } else {
                p.left_rotation = -0.3; p.right_rotation = 0.35;
                p.left_dy = -14.0; p.right_dy = -14.0;
                p.torso_scale_y = 0.98; p.lean = -0.05;
            }
        }
        Crouch => { p.crouch = 0.45; p.lean = 0.1; p.left_scale_y = 0.55; p.right_scale_y = 0.55; }
        Block => { p.crouch = 0.22; p.left_scale_y = 0.78; p.right_scale_y = 0.78; p.lean = -0.14; p.torso_scale_x = 0.92; p.head = 0.1; }
        Hit => { p.lean = -0.35; p.head = -0.5; p.dx = -14.0 + (t * 2.0).sin() * 4.0; p.left_rotation = 0.3; p.right_rotation = -0.3; }
        Ko => { p.crouch = 0.45; p.left_scale_y = 0.55; p.right_scale_y = 0.55; p.lean = 0.25; p.head = 0.4 + 0.05 * (t / 12.0).sin(); }
        Attack(kind) => {
            let movement = moves.get(kind);
            let startup = movement.startup.max(1) as f32;
            let active_end = (movement.startup + movement.active) as f32;
            let recovery = movement.recovery.max(1) as f32;
            let a = if t < movement.startup as f32 { -t / startup }
                else if t < active_end { 1.0 }
                else { 1.0 - (t - active_end) / recovery };
            let s = a.max(0.0);
            let w = (-a).max(0.0);
            match kind {
                MoveKind::Punch => {
                    p.lean = 0.28 * a; p.dx = 36.0 * s - 8.0 * w;
                    p.torso_scale_x = 1.0 + 0.14 * s; p.head = -0.1 * a;
                    p.right_rotation = 0.18 * a; p.left_rotation = -0.2 * a;
                }
                MoveKind::Kick => {
                    p.lean = if a >= 0.0 { -0.3 * s } else { 0.1 * w };
                    p.right_rotation = if a >= 0.0 { -1.45 * s } else { -0.7 * w };
                    p.right_scale_y = 1.0 + 0.12 * s; p.dx = 24.0 * s;
                    p.left_rotation = 0.25 * s; p.head = 0.12 * s;
                }
                MoveKind::Sweep => {
                    p.crouch = 0.4; p.left_scale_y = 0.6; p.right_scale_y = 1.0 + 0.5 * s;
                    p.right_rotation = -1.4 * s; p.lean = 0.15 * a; p.dx = 20.0 * s;
                }
                MoveKind::Special => {
                    p.crouch = 0.3 * w; p.lean = if a >= 0.0 { -0.35 * s } else { 0.25 * w };
                    p.right_rotation = -1.5 * s; p.left_rotation = 0.5 * s;
                    p.dx = 70.0 * s; p.dy = -18.0 * s; p.torso_scale_x = 1.0 + 0.1 * s;
                }
            }
        }
    }
    p
}

/// Renderiza o sprite único em cabeça, tronco e pernas, respeitando os cortes do rig JSON.
fn draw_rig(c: &mut Canvas, f: &Fighter, sprite: &Sprite, rig: crate::config::RigConfig, moves: &crate::config::MoveSet, _tick: u64) {
    let width = sprite.width as f32;
    let height = sprite.height as f32;
    if width < 2.0 || height < 4.0 { return; }
    let scale = (f.config.size as f32 / height).clamp(0.05, 4.0);
    let neck = (rig.neck.clamp(0.05, 0.7) * height).clamp(1.0, height - 2.0);
    let hip = (rig.hip.clamp(0.2, 0.95) * height).clamp(neck + 1.0, height - 1.0);
    let split_ratio = if rig.flip { 1.0 - rig.split } else { rig.split };
    let split = (split_ratio.clamp(0.1, 0.9) * width).clamp(1.0, width - 1.0);
    let pose = rig_pose(f, moves);
    let mirror_for_facing = (f.facing == Facing::Left) ^ f.config.facing_left;
    let body_offset_y = pose.dy + pose.crouch * (height - hip);

    if rig.skirt {
        draw_rig_part(c, sprite, f.x, f.feet_y, (0.0, hip, width, height - hip), width * 0.5, hip,
            pose.dx, body_offset_y, (pose.right_rotation + pose.left_rotation) * 0.35,
            1.0, pose.left_scale_y, scale, mirror_for_facing, rig.flip);
    } else {
        let left_width = (split + 4.0).clamp(1.0, width);
        let right_x = (split - 4.0).clamp(0.0, width - 1.0);
        draw_rig_part(c, sprite, f.x, f.feet_y, (0.0, hip, left_width, height - hip), split - width * 0.05, hip,
            pose.dx, body_offset_y + pose.left_dy, pose.left_rotation,
            1.0, pose.left_scale_y, scale, mirror_for_facing, rig.flip);
        draw_rig_part(c, sprite, f.x, f.feet_y, (right_x, hip, width - right_x, height - hip), split + width * 0.05, hip,
            pose.dx, body_offset_y + pose.right_dy, pose.right_rotation,
            1.0, pose.right_scale_y * pose.left_scale_y, scale, mirror_for_facing, rig.flip);
    }

    // Tronco e braços giram no quadril; a cabeça acompanha o tronco e gira no pescoço.
    let torso_height = (hip - neck + 14.0).clamp(1.0, height - neck);
    draw_rig_part(c, sprite, f.x, f.feet_y, (0.0, neck, width, torso_height), width * 0.5, hip,
        pose.dx, body_offset_y, pose.lean, pose.torso_scale_x, pose.torso_scale_y,
        scale, mirror_for_facing, rig.flip);
    let neck_to_hip = neck - hip;
    let head_pivot_x = width * 0.5 - pose.lean.sin() * pose.torso_scale_y * neck_to_hip;
    let head_pivot_y = hip + pose.lean.cos() * pose.torso_scale_y * neck_to_hip;
    let head_height = (neck + 10.0).clamp(1.0, height);
    draw_rig_part(c, sprite, f.x, f.feet_y, (0.0, 0.0, width, head_height), head_pivot_x, head_pivot_y,
        pose.dx, body_offset_y, pose.lean + pose.head, pose.torso_scale_x, pose.torso_scale_y,
        scale, mirror_for_facing, rig.flip);
}

/// Desenha uma região girada por amostragem inversa, sem alocar bitmaps temporários por frame.
#[allow(clippy::too_many_arguments)]
fn draw_rig_part(
    c: &mut Canvas, sprite: &Sprite, fighter_x: f32, fighter_feet_y: f32,
    crop: (f32, f32, f32, f32), pivot_x: f32, pivot_y: f32,
    offset_x: f32, offset_y: f32, rotation: f32, scale_x: f32, scale_y: f32,
    global_scale: f32, mirror_world: bool, mirror_asset: bool,
) {
    let (crop_x, crop_y, crop_w, crop_h) = crop;
    if crop_w <= 0.0 || crop_h <= 0.0 { return; }
    let sx = scale_x.abs().max(0.05);
    let sy = scale_y.abs().max(0.05);
    let cos = rotation.cos();
    let sin = rotation.sin();
    let world_sign = if mirror_world { -1.0 } else { 1.0 };
    let transform = |u: f32, v: f32| -> (f32, f32) {
        let lx = (u - pivot_x) * sx;
        let ly = (v - pivot_y) * sy;
        let rx = lx * cos - ly * sin;
        let ry = lx * sin + ly * cos;
        let x = pivot_x - sprite.width as f32 * 0.5 + offset_x + rx;
        let y = pivot_y - sprite.height as f32 + offset_y + ry;
        (fighter_x + world_sign * x * global_scale, fighter_feet_y + y * global_scale)
    };
    let corners = [
        transform(crop_x, crop_y), transform(crop_x + crop_w, crop_y),
        transform(crop_x, crop_y + crop_h), transform(crop_x + crop_w, crop_y + crop_h),
    ];
    let min_x = corners.iter().map(|p| p.0).fold(f32::INFINITY, f32::min).floor().max(0.0) as i32;
    let min_y = corners.iter().map(|p| p.1).fold(f32::INFINITY, f32::min).floor().max(0.0) as i32;
    let max_x = corners.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max).ceil().min(c.width as f32) as i32;
    let max_y = corners.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max).ceil().min(c.height as f32) as i32;
    if max_x <= min_x || max_y <= min_y { return; }
    for y in min_y..max_y {
        for x in min_x..max_x {
            let image_x = (x as f32 + 0.5 - fighter_x) / (world_sign * global_scale)
                - (pivot_x - sprite.width as f32 * 0.5 + offset_x);
            let image_y = (y as f32 + 0.5 - fighter_feet_y) / global_scale
                - (pivot_y - sprite.height as f32 + offset_y);
            let local_x = image_x * cos + image_y * sin;
            let local_y = -image_x * sin + image_y * cos;
            let u = pivot_x + local_x / sx;
            let v = pivot_y + local_y / sy;
            if u < crop_x || u >= crop_x + crop_w || v < crop_y || v >= crop_y + crop_h { continue; }
            let virtual_x = u.floor().clamp(0.0, sprite.width.saturating_sub(1) as f32) as usize;
            let source_x = if mirror_asset { sprite.width - 1 - virtual_x } else { virtual_x };
            let source_y = v.floor().clamp(0.0, sprite.height.saturating_sub(1) as f32) as usize;
            let color = sprite.pixels[source_y * sprite.width + source_x];
            if color != 0 { c.pixel(x, y, color & 0x00FF_FFFF); }
        }
    }
}

fn draw_sprite(c: &mut Canvas, sprite: &Sprite, anchor_x_world: f32, anchor_y_world: f32, target_height: f32, anchor_x: f32, anchor_y: f32, mirror: bool, scale_x: f32) {
    draw_sprite_blended(c, sprite, anchor_x_world, anchor_y_world, target_height, anchor_x, anchor_y, mirror, scale_x, false);
}

fn draw_sprite_blended(c: &mut Canvas, sprite: &Sprite, anchor_x_world: f32, anchor_y_world: f32, target_height: f32, anchor_x: f32, anchor_y: f32, mirror: bool, scale_x: f32, additive: bool) {
    if sprite.width == 0 || sprite.height == 0 { return; }
    let scale = target_height / sprite.height as f32;
    let dst_w = (sprite.width as f32 * scale * scale_x).max(1.0) as usize;
    let dst_h = target_height.max(1.0) as usize;
    let left = (anchor_x_world - dst_w as f32 * anchor_x) as i32;
    let top = (anchor_y_world - dst_h as f32 * anchor_y) as i32;
    for dy in 0..dst_h {
        let sy = (dy * sprite.height / dst_h).min(sprite.height - 1);
        for dx in 0..dst_w {
            let sx = (dx * sprite.width / dst_w).min(sprite.width - 1);
            let source_x = if mirror { sprite.width - 1 - sx } else { sx };
            let color = sprite.pixels[sy * sprite.width + source_x];
            if color != 0 {
                let px = left + dx as i32;
                let py = top + dy as i32;
                if additive && px >= 0 && py >= 0 && (px as usize) < c.width && (py as usize) < c.height {
                    let index = py as usize * c.width + px as usize;
                    let dst = c.pixels[index];
                    let src = color & 0x00FF_FFFF;
                    let red = (((dst >> 16) & 0xFF) + ((src >> 16) & 0xFF)).min(255);
                    let green = (((dst >> 8) & 0xFF) + ((src >> 8) & 0xFF)).min(255);
                    let blue = ((dst & 0xFF) + (src & 0xFF)).min(255);
                    c.pixels[index] = (red << 16) | (green << 8) | blue;
                } else {
                    c.pixel(px, py, color & 0x00FF_FFFF);
                }
            }
        }
    }
}

fn draw_fallback(c: &mut Canvas, f: &Fighter) {
    let color = rgb(f.config.color[0], f.config.color[1], f.config.color[2]); let dark = rgb(24, 28, 40);
    let x = f.x as i32; let foot = f.feet_y as i32;
    let crouching = matches!(f.state, FighterState::Crouch | FighterState::Block | FighterState::Ko);
    let jumping = f.state == FighterState::Jump; let head_y = foot - if crouching { 68 } else { 126 };
    let body_top = foot - if crouching { 61 } else { 108 };
    c.circle(x, head_y, 17, rgb(221, 176, 139)); c.circle(x + if f.facing == Facing::Right { 5 } else { -5 }, head_y - 3, 3, dark);
    c.fill_rect(Rect::new((x - 17) as f32, body_top as f32, 34.0, if crouching { 43.0 } else { 53.0 }), color);
    let direction = if f.facing == Facing::Right { 1 } else { -1 };
    let (arm_reach, arm_y) = match f.state { FighterState::Attack(MoveKind::Punch) => (42, body_top + 10), FighterState::Attack(MoveKind::Special) => (54, body_top + 1), FighterState::Block => (29, body_top + 7), _ => (21, body_top + 13) };
    c.line(x + direction * 11, body_top + 12, x + direction * arm_reach, arm_y, rgb(221, 176, 139));
    c.circle(x + direction * arm_reach, arm_y, 5, rgb(221, 176, 139));
    let kick = matches!(f.state, FighterState::Attack(MoveKind::Kick | MoveKind::Sweep));
    if kick { c.line(x, foot - 25, x + direction * 44, foot - 12, color); c.line(x + direction * 44, foot - 12, x + direction * 61, foot - 12, dark); }
    else { let leg_spread = if jumping { 17 } else { 10 }; c.line(x - 7, foot - 17, x - leg_spread, foot, dark); c.line(x + 7, foot - 17, x + leg_spread, foot, dark); }
}

fn draw_hud(c: &mut Canvas, game: &Game) {
    c.fill_rect(Rect::new(25.0, 20.0, 390.0, 54.0), rgb(13, 18, 31));
    c.fill_rect(Rect::new(545.0, 20.0, 390.0, 54.0), rgb(13, 18, 31));
    c.draw_text(32, 26, &ascii_name(&game.p1.config.name), 1, rgb(247, 248, 252));
    c.draw_text(690, 26, &ascii_name(&game.p2.config.name), 1, rgb(247, 248, 252));
    draw_health(c, 35, 43, 365, game.p1.hp, game.p1.config.hp, false);
    draw_health(c, 560, 43, 365, game.p2.hp, game.p2.config.hp, true);
    c.fill_rect(Rect::new(432.0, 18.0, 96.0, 49.0), rgb(13, 18, 31));
    let secs = game.timer_seconds();
    let timer_text = if game.mode == crate::game::GameMode::Training { "INF".to_owned() } else { format!("{:02}", secs) };
    c.draw_text(452, 26, &timer_text, 3, rgb(255, 222, 91));
    c.draw_text(428, 72, &format!("R{}", game.round_number), 1, rgb(230, 235, 244));
    if game.round_wins[0] > 0 { c.draw_text(380, 78, &format!("{}", game.round_wins[0]), 1, rgb(255, 222, 91)); }
    if game.round_wins[1] > 0 { c.draw_text(560, 78, &format!("{}", game.round_wins[1]), 1, rgb(255, 222, 91)); }
    if game.p1.combo_hits > 1 && game.p1.combo_timer > 0 { c.draw_text(55, 90, &format!("{} HITS", game.p1.combo_hits), 2, rgb(255, 222, 91)); }
    if game.p2.combo_hits > 1 && game.p2.combo_timer > 0 { c.draw_text(785, 90, &format!("{} HITS", game.p2.combo_hits), 2, rgb(255, 222, 91)); }
    c.draw_text(15, 515, "F1 HITBOX - ESC MENU", 1, rgb(211, 218, 229));
    if game.mode == crate::game::GameMode::Training { c.draw_text(680, 515, "B DEFESA - N REINICIA", 1, rgb(211, 218, 229)); }
}

fn draw_health(c: &mut Canvas, x: i32, y: i32, width: i32, hp: i32, maximum: i32, reverse: bool) {
    c.fill_rect(Rect::new(x as f32, y as f32, width as f32, 13.0), rgb(63, 28, 36));
    let ratio = if maximum <= 0 { 0.0 } else { (hp as f32 / maximum as f32).clamp(0.0, 1.0) };
    let color = if ratio < 0.30 { rgb(220, 52, 61) } else { rgb(64, 206, 109) };
    let w = width as f32 * ratio;
    let bar_x = if reverse { x as f32 + width as f32 - w } else { x as f32 };
    c.fill_rect(Rect::new(bar_x, y as f32, w, 13.0), color);
}

fn ascii_name(name: &str) -> String {
    name.chars().map(|c| match c {
        'á' | 'à' | 'ã' | 'â' | 'Á' | 'À' | 'Ã' | 'Â' => 'A',
        'é' | 'ê' | 'É' | 'Ê' => 'E',
        'í' | 'Í' => 'I',
        'ó' | 'ô' | 'õ' | 'Ó' | 'Ô' | 'Õ' => 'O',
        'ú' | 'Ú' => 'U',
        'ç' | 'Ç' => 'C',
        other if other.is_ascii() => other,
        _ => ' ',
    }).collect()
}

fn glyph_for(c: char) -> [u8; 7] {
    match c {
        'A' => [14,17,17,31,17,17,17], 'B' => [30,17,17,30,17,17,30], 'C' => [14,17,16,16,16,17,14],
        'D' => [30,17,17,17,17,17,30], 'E' => [31,16,16,30,16,16,31], 'F' => [31,16,16,30,16,16,16],
        'G' => [14,17,16,23,17,17,15], 'H' => [17,17,17,31,17,17,17], 'I' => [14,4,4,4,4,4,14],
        'J' => [7,2,2,2,18,18,12], 'K' => [17,18,20,24,20,18,17], 'L' => [16,16,16,16,16,16,31],
        'M' => [17,27,21,21,17,17,17], 'N' => [17,25,21,19,17,17,17], 'O' => [14,17,17,17,17,17,14],
        'P' => [30,17,17,30,16,16,16], 'Q' => [14,17,17,17,21,18,13], 'R' => [30,17,17,30,20,18,17],
        'S' => [15,16,16,14,1,1,30], 'T' => [31,4,4,4,4,4,4], 'U' => [17,17,17,17,17,17,14],
        'V' => [17,17,17,17,17,10,4], 'W' => [17,17,17,21,21,21,10], 'X' => [17,17,10,4,10,17,17],
        'Y' => [17,17,10,4,4,4,4], 'Z' => [31,1,2,4,8,16,31],
        '0' => [14,17,19,21,25,17,14], '1' => [4,12,4,4,4,4,14], '2' => [14,17,1,2,4,8,31],
        '3' => [30,1,1,14,1,1,30], '4' => [2,6,10,18,31,2,2], '5' => [31,16,16,30,1,1,30],
        '6' => [14,16,16,30,17,17,14], '7' => [31,1,2,4,8,8,8], '8' => [14,17,17,14,17,17,14],
        '9' => [14,17,17,15,1,1,14], ':' => [0,4,4,0,4,4,0], '-' => [0,0,0,31,0,0,0],
        '!' => [4,4,4,4,4,0,4], '.' => [0,0,0,0,0,6,6], '/' => [1,2,2,4,8,8,16],
        '+' => [0,4,4,31,4,4,0], '_' => [0,0,0,0,0,0,31], '<' => [2,4,8,16,8,4,2],
        '>' => [8,4,2,1,2,4,8], _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canvas_clips_pixels_outside_bounds() {
        let mut canvas = Canvas::new(4, 4);
        canvas.pixel(-1, 0, rgb(255, 0, 0));
        canvas.pixel(3, 3, rgb(0, 255, 0));
        assert_eq!(canvas.pixels[15], rgb(0, 255, 0));
        assert_eq!(canvas.pixels[0], 0);
    }
}
