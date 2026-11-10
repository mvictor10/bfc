//! Carregamento de sprites do jogo original: frames por prefixo, chroma key e efeitos.
use crate::fighter::{FighterState, MoveKind};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, RgbaImage};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    /// RGB em 24 bits; zero representa pixel transparente.
    pub pixels: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimationMode {
    Loop,
    Once,
    Progress,
    Velocity,
}

#[derive(Clone, Debug)]
struct AnimationClip {
    frames: Vec<Sprite>,
    fps: u32,
    mode: AnimationMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectAnchor {
    Attacker,
    Target,
    Head,
}

#[derive(Clone, Copy, Debug)]
pub struct EffectPlacement {
    pub anchor: EffectAnchor,
    /// Deslocamento horizontal relativo à direção do lutador-âncora.
    pub offset_x: f32,
    /// Altura medida a partir dos pés, como no character.json original.
    pub offset_y: f32,
    pub scale: f32,
    pub additive: bool,
}

impl EffectPlacement {
    fn defaults_for(name: &str) -> Self {
        let (anchor, offset_x, offset_y) = match name {
            "punch" => (EffectAnchor::Attacker, 70.0, 170.0),
            "kick" => (EffectAnchor::Attacker, 80.0, 115.0),
            "special" => (EffectAnchor::Attacker, 95.0, 160.0),
            "sweep" => (EffectAnchor::Attacker, 80.0, 35.0),
            "dizzy" => (EffectAnchor::Head, 0.0, 185.0),
            "block" | "hit" => (EffectAnchor::Target, 0.0, 150.0),
            _ => (EffectAnchor::Target, 0.0, 150.0),
        };
        Self { anchor, offset_x, offset_y, scale: 1.0, additive: false }
    }
}

#[derive(Clone, Debug)]
struct EffectClip {
    clip: AnimationClip,
    placement: EffectPlacement,
}

#[derive(Default, Debug)]
pub struct SpriteSet {
    animations: HashMap<String, AnimationClip>,
    effects: HashMap<String, EffectClip>,
    pub base_art: Option<Sprite>,
    /// Indica que a arte única pode ser animada pelo rig legado do personagem.
    pub has_rig: bool,
    pub animated_frames: usize,
}

impl SpriteSet {
    /// Carrega os assets de um personagem. A pasta compartilhada de efeitos é usada como fallback.
    pub fn load(directory: &Path) -> Self {
        let project_root = directory.parent().and_then(Path::parent).unwrap_or_else(|| Path::new("."));
        Self::load_with_root(directory, project_root)
    }

    /// Variante explícita, útil para projetos em que a pasta de personagem não é `chars/<id>`.
    pub fn load_with_root(directory: &Path, project_root: &Path) -> Self {
        let mut result = Self::default();
        let character_json = directory.join("character.json");
        let json = fs::read(&character_json)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .unwrap_or(Value::Null);
        let null = Value::Null;
        result.has_rig = json.get("rig").map(Value::is_object).unwrap_or(false);
        let chroma = json.get("chroma").unwrap_or(&null);
        let tolerance = chroma.get("tolerance").and_then(Value::as_u64).unwrap_or(90).min(255) as u8;
        let max_height = chroma.get("maxHeight").and_then(Value::as_u64).unwrap_or(560).clamp(128, 1200) as u32;
        let max_frames = json.get("maxFrames").and_then(Value::as_u64).unwrap_or(24).clamp(1, 60) as usize;

        let defaults = default_animations();
        let mut animation_configs: Vec<(String, String, u32, AnimationMode, usize)> = Vec::new();
        if let Some(items) = json.get("animations").and_then(Value::as_object) {
            for (name, value) in items {
                let default = defaults.get(name.as_str()).copied().unwrap_or((8, AnimationMode::Loop));
                let prefix = value.get("prefix").and_then(Value::as_str).unwrap_or(name).to_owned();
                let fps = value.get("fps").and_then(Value::as_f64)
                    .map(|n| n.max(1.0) as u32).unwrap_or(default.0).clamp(1, 60);
                let mode = value.get("mode").and_then(Value::as_str).map(parse_mode).unwrap_or(default.1);
                let frame_limit = value.get("frames").and_then(Value::as_u64)
                    .map(|n| n as usize).unwrap_or(max_frames).clamp(1, max_frames);
                animation_configs.push((name.clone(), prefix, fps, mode, frame_limit));
            }
        } else {
            for (name, (fps, mode)) in &defaults {
                animation_configs.push(((*name).to_owned(), (*name).to_owned(), *fps, *mode, max_frames));
            }
        }

        for (name, prefix, fps, mode, frame_limit) in animation_configs {
            let frames = load_sequence(directory, &prefix, frame_limit, max_height, tolerance);
            if !frames.is_empty() {
                result.animated_frames += frames.len();
                result.animations.insert(name, AnimationClip { frames, fps, mode });
            }
        }

        // O jogo original mantém efeitos isolados por prefixo; chars/_exemplo contém os FX partilhados.
        let shared_effect_dir = project_root.join("assets/images/effects");
        if let Some(items) = json.get("effects").and_then(Value::as_object) {
            for (name, value) in items {
                let prefix = value.get("prefix").and_then(Value::as_str)
                    .map(str::to_owned).unwrap_or_else(|| format!("fx_{}", name));
                let fps = value.get("fps").and_then(Value::as_u64).unwrap_or(24).clamp(1, 60) as u32;
                let looping = value.get("loop").and_then(Value::as_bool).unwrap_or(name == "dizzy");
                let placement = parse_effect_placement(name, value);
                let mut frames = load_sequence(directory, &prefix, max_frames, max_height, tolerance);
                if frames.is_empty() {
                    let shared_prefix = shared_prefix_for(name, &prefix);
                    frames = load_sequence(&shared_effect_dir, shared_prefix, max_frames, max_height, tolerance);
                }
                if !frames.is_empty() {
                    result.effects.insert(name.clone(), EffectClip {
                        clip: AnimationClip { frames, fps, mode: if looping { AnimationMode::Loop } else { AnimationMode::Once } },
                        placement,
                    });
                }
            }
        }

        // Garante as convenções usadas pelo motor, mesmo quando algum personagem não lista os FX.
        for name in ["punch", "kick", "special", "sweep", "hit", "block", "dizzy"] {
            if result.effects.contains_key(name) { continue; }
            let requested_prefix = format!("fx_{}", name);
            let prefix = shared_prefix_for(name, &requested_prefix);
            let frames = load_sequence(&shared_effect_dir, prefix, max_frames, max_height, tolerance);
            if !frames.is_empty() {
                result.effects.insert(name.to_owned(), EffectClip {
                    clip: AnimationClip {
                        frames,
                        fps: if name == "dizzy" { 10 } else { 24 },
                        mode: if name == "dizzy" { AnimationMode::Loop } else { AnimationMode::Once },
                    },
                    placement: EffectPlacement::defaults_for(name),
                });
            }
        }

        // Para personagens sem frames próprios, usa a arte grande original ou o fallback de corpo inteiro.
        let art = json.get("art").unwrap_or(&null);
        let mut candidates = Vec::new();
        if let Some(sprite) = art.get("sprite").and_then(Value::as_str) { candidates.push(directory.join(sprite)); }
        candidates.extend([
            directory.join("sprite.png"), directory.join("sprite.jpg"), directory.join("sprite.jpeg"),
            directory.join("sprite_fallback.png"), directory.join("sprite_fallback.jpg"),
            directory.join("face.png"), directory.join("face.jpg"),
        ]);
        for path in candidates {
            if path.is_file() {
                if let Ok(sprite) = load_image(&path, max_height, tolerance) {
                    result.base_art = Some(sprite);
                    break;
                }
            }
        }
        result
    }

    pub fn frame_for(
        &self,
        state: FighterState,
        animation_ticks: u64,
        state_ticks: u64,
        attack_ticks: u64,
        attack_duration: u64,
        velocity_y: f32,
    ) -> Option<&Sprite> {
        let candidates: &[&str] = match state {
            FighterState::Idle => &["idle"],
            FighterState::Walk => &["walk", "run", "idle"],
            FighterState::Run => &["run", "walk", "idle"],
            FighterState::Jump => &["jump", "idle"],
            FighterState::Crouch => &["crouch", "idle"],
            FighterState::Block => &["defense", "block", "crouch", "idle"],
            FighterState::Hit => &["hit", "defense", "idle"],
            FighterState::Ko => &["ko", "dead", "crouch", "idle"],
            FighterState::Attack(MoveKind::Punch) => &["attack", "punch", "idle"],
            FighterState::Attack(MoveKind::Kick) => &["kick", "attack", "idle"],
            FighterState::Attack(MoveKind::Special) => &["special", "kick", "attack", "idle"],
            FighterState::Attack(MoveKind::Sweep) => &["sweep", "kick", "attack", "idle"],
        };
        for name in candidates {
            // Como no FrameAnimator.js, não use idle como fallback de estado quando
            // existe arte legada; nesse caso, o renderizador de rig anima a arte única.
            if *name == "idle" && state != FighterState::Idle && self.has_rig && self.base_art.is_some() {
                continue;
            }
            if let Some(clip) = self.animations.get(*name) {
                let elapsed = match clip.mode {
                    AnimationMode::Loop => animation_ticks,
                    AnimationMode::Once => state_ticks,
                    AnimationMode::Progress => attack_ticks,
                    AnimationMode::Velocity => state_ticks,
                };
                let raw_index = if state == FighterState::Ko {
                    // O FrameAnimator.js mantém o último quadro de KO estático.
                    clip.frames.len() - 1
                } else {
                    match clip.mode {
                        AnimationMode::Loop => ((elapsed.saturating_mul(clip.fps as u64) / 60) as usize) % clip.frames.len(),
                        AnimationMode::Once => ((elapsed.saturating_mul(clip.fps as u64) / 60) as usize).min(clip.frames.len() - 1),
                        AnimationMode::Progress => {
                            let duration = attack_duration.max(1);
                            ((attack_ticks.min(duration - 1) * clip.frames.len() as u64) / duration) as usize
                        }
                        AnimationMode::Velocity => {
                            if velocity_y < -4.0 { 0 }
                            else if velocity_y > 4.0 { clip.frames.len() - 1 }
                            else { (clip.frames.len() - 1) / 2 }
                        }
                    }
                };
                let index = raw_index.min(clip.frames.len() - 1);
                return clip.frames.get(index);
            }
        }
        None
    }

    pub fn effect_frame(&self, name: &str, tick: u64) -> Option<&Sprite> {
        let clip = &self.effects.get(name)?.clip;
        let index = match clip.mode {
            AnimationMode::Loop => ((tick.saturating_mul(clip.fps as u64) / 60) as usize) % clip.frames.len(),
            _ => ((tick.saturating_mul(clip.fps as u64) / 60) as usize).min(clip.frames.len() - 1),
        };
        clip.frames.get(index)
    }

    pub fn effect_placement(&self, name: &str) -> Option<EffectPlacement> {
        self.effects.get(name).map(|effect| effect.placement)
    }

    pub fn effect_duration_ticks(&self, name: &str) -> Option<u16> {
        let effect = self.effects.get(name)?;
        let frames = effect.clip.frames.len() as u64;
        Some(((frames * 60 + effect.clip.fps as u64 - 1) / effect.clip.fps as u64).clamp(1, u16::MAX as u64) as u16)
    }

    pub fn frame_count(&self) -> usize {
        self.animations.values().map(|clip| clip.frames.len()).sum()
    }
}

fn parse_effect_placement(name: &str, value: &Value) -> EffectPlacement {
    let mut placement = EffectPlacement::defaults_for(name);
    if let Some(anchor) = value.get("anchor").and_then(Value::as_str) {
        placement.anchor = match anchor {
            "attacker" => EffectAnchor::Attacker,
            "head" => EffectAnchor::Head,
            _ => EffectAnchor::Target,
        };
    }
    if let Some(offset) = value.get("offset").and_then(Value::as_array) {
        if let Some(x) = offset.first().and_then(Value::as_f64) { placement.offset_x = (x as f32).clamp(-500.0, 500.0); }
        if let Some(y) = offset.get(1).and_then(Value::as_f64) { placement.offset_y = (y as f32).clamp(0.0, 700.0); }
    }
    if let Some(scale) = value.get("scale").and_then(Value::as_f64) {
        placement.scale = (scale as f32).clamp(0.1, 4.0);
    }
    placement.additive = value.get("blend").and_then(Value::as_str) == Some("lighter");
    placement
}

fn shared_prefix_for<'a>(name: &str, requested: &'a str) -> &'a str {
    match name {
        "kick" | "special" | "sweep" => "fx_punch",
        "block" => "fx_hit",
        _ => requested,
    }
}

fn default_animations() -> HashMap<&'static str, (u32, AnimationMode)> {
    HashMap::from([
        ("idle", (8, AnimationMode::Loop)), ("walk", (12, AnimationMode::Loop)), ("run", (16, AnimationMode::Loop)),
        ("jump", (8, AnimationMode::Velocity)), ("crouch", (14, AnimationMode::Once)), ("defense", (14, AnimationMode::Once)),
        ("attack", (8, AnimationMode::Progress)), ("kick", (8, AnimationMode::Progress)), ("special", (8, AnimationMode::Progress)),
        ("sweep", (8, AnimationMode::Progress)), ("hit", (10, AnimationMode::Once)), ("ko", (10, AnimationMode::Once)),
        ("dead", (10, AnimationMode::Once)), ("block", (14, AnimationMode::Once)),
    ])
}

fn parse_mode(value: &str) -> AnimationMode {
    match value { "once" | "frame" => AnimationMode::Once, "progress" => AnimationMode::Progress, "vy" => AnimationMode::Velocity, _ => AnimationMode::Loop }
}

fn load_sequence(directory: &Path, prefix: &str, max_frames: usize, max_height: u32, tolerance: u8) -> Vec<Sprite> {
    let mut out = Vec::new();
    for index in 1..=max_frames {
        let base = format!("{}_{:02}", prefix, index);
        let candidates = [directory.join(format!("{}.png", base)), directory.join(format!("{}.jpg", base)), directory.join(format!("{}.jpeg", base))];
        let path = candidates.iter().find(|path| path.is_file());
        let Some(path) = path else { break; };
        match load_image(path, max_height, tolerance) { Ok(sprite) => out.push(sprite), Err(_) => break }
    }
    out
}

fn load_image(path: &Path, max_height: u32, tolerance: u8) -> Result<Sprite, String> {
    let mut image = image::open(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let (w, h) = image.dimensions();
    if w == 0 || h == 0 || w > 8192 || h > 8192 || (w as u64) * (h as u64) > 40_000_000 {
        return Err(format!("dimensões de imagem inválidas ou excessivas: {}x{}", w, h));
    }
    if h > max_height || w > 2048 {
        image = image.resize(2048, max_height, FilterType::Triangle);
    }
    Ok(sprite_from_rgba(image.to_rgba8(), tolerance))
}

/// Carrega um cenário em 960x540, limitando memória e mantendo a proporção original.
pub fn load_background(path: &Path, width: u32, height: u32) -> Result<Sprite, String> {
    let image = image::open(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let (iw, ih) = image.dimensions();
    if iw == 0 || ih == 0 || iw > 12000 || ih > 12000 { return Err("dimensões de cenário inválidas".into()); }
    let target_ratio = width as f32 / height as f32;
    let input_ratio = iw as f32 / ih as f32;
    let cropped: DynamicImage = if input_ratio > target_ratio {
        let crop_width = ((ih as f32 * target_ratio).round() as u32).clamp(1, iw);
        image.crop_imm((iw - crop_width) / 2, 0, crop_width, ih)
    } else {
        let crop_height = ((iw as f32 / target_ratio).round() as u32).clamp(1, ih);
        image.crop_imm(0, (ih - crop_height) / 2, iw, crop_height)
    };
    let rgba = cropped.resize_exact(width, height, FilterType::Triangle).to_rgba8();
    Ok(sprite_from_rgba(rgba, 0))
}

fn sprite_from_rgba(rgba: RgbaImage, tolerance: u8) -> Sprite {
    let (width, height) = rgba.dimensions();
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for px in rgba.pixels() {
        let [r, g, b, a] = px.0;
        let is_green_key = tolerance > 0 && g > 145 && (g as i16 - r as i16) > (tolerance as i16 / 2)
            && (g as i16 - b as i16) > (tolerance as i16 / 2) && r < 130 && b < 130;
        if a < 20 || is_green_key { pixels.push(0); }
        else { pixels.push(0x0100_0000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32); }
    }
    Sprite { width: width as usize, height: height as usize, pixels }
}

pub fn character_frame_dir(root: &Path, configured_path: &str) -> PathBuf { root.join(configured_path) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequential_names_keep_original_convention() {
        let path = Path::new("chars/amanda/idle_01.png");
        assert!(path.to_string_lossy().ends_with("idle_01.png"));
    }
    #[test]
    fn mode_names_cover_original_animation_contract() {
        assert_eq!(parse_mode("progress"), AnimationMode::Progress);
        assert_eq!(parse_mode("vy"), AnimationMode::Velocity);
        assert_eq!(parse_mode("loop"), AnimationMode::Loop);
    }
    #[test]
    fn shared_fallback_maps_missing_kick_and_block_effects() {
        assert_eq!(shared_prefix_for("kick", "fx_kick"), "fx_punch");
        assert_eq!(shared_prefix_for("block", "fx_block"), "fx_hit");
        assert_eq!(shared_prefix_for("hit", "fx_hit"), "fx_hit");
    }

    #[test]
    fn decodes_original_amanda_frames_art_and_shared_impact_fx() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let sprites = SpriteSet::load_with_root(&root.join("chars/amanda"), root);
        assert!(sprites.base_art.is_some(), "Amanda precisa carregar sua arte original");
        assert!(sprites.frame_count() >= 20, "as sequências originais de animação não foram carregadas");
        assert!(sprites.effect_frame("hit", 0).is_some(), "o fallback fx_hit compartilhado não foi carregado");
    }
}
