//! Configuração de domínio: elenco, atributos, golpes e cenários do jogo original.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CampaignRole { Normal, SubBoss, Boss }

impl CampaignRole {
    pub fn from_json(value: Option<&str>, id: &str) -> Self {
        match value.unwrap_or("").to_ascii_lowercase().as_str() {
            "boss" => Self::Boss,
            "subboss" | "sub-boss" | "sub_boss" => Self::SubBoss,
            "normal" => Self::Normal,
            _ if id == "viktor-krayller" => Self::Boss,
            _ if id == "amanda" => Self::SubBoss,
            _ => Self::Normal,
        }
    }

    pub fn tag(self) -> Option<&'static str> {
        match self { Self::Boss => Some("BOSS"), Self::SubBoss => Some("SUB-BOSS"), Self::Normal => None }
    }

    pub fn as_str(self) -> &'static str {
        match self { Self::Boss => "boss", Self::SubBoss => "subboss", Self::Normal => "normal" }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MoveData {
    pub startup: u16,
    pub active: u16,
    pub recovery: u16,
    pub damage: i32,
    pub reach: f32,
    pub y_offset: f32,
    pub stun: u16,
    pub knockback: f32,
}

impl MoveData {
    pub const fn new(startup: u16, active: u16, recovery: u16, damage: i32, reach: f32, y_offset: f32, stun: u16, knockback: f32) -> Self {
        Self { startup, active, recovery, damage, reach, y_offset, stun, knockback }
    }
    pub fn duration(self) -> u16 { self.startup.saturating_add(self.active).saturating_add(self.recovery) }
}

#[derive(Clone, Copy, Debug)]
pub struct PartialMoveData {
    pub startup: Option<u16>, pub active: Option<u16>, pub recovery: Option<u16>,
    pub damage: Option<i32>, pub reach: Option<f32>, pub y_offset: Option<f32>,
    pub stun: Option<u16>, pub knockback: Option<f32>,
}

impl PartialMoveData {
    fn apply(self, base: MoveData) -> MoveData {
        MoveData {
            startup: self.startup.unwrap_or(base.startup), active: self.active.unwrap_or(base.active),
            recovery: self.recovery.unwrap_or(base.recovery), damage: self.damage.unwrap_or(base.damage),
            reach: self.reach.unwrap_or(base.reach), y_offset: self.y_offset.unwrap_or(base.y_offset),
            stun: self.stun.unwrap_or(base.stun), knockback: self.knockback.unwrap_or(base.knockback),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MoveOverrides {
    pub punch: Option<PartialMoveData>, pub kick: Option<PartialMoveData>,
    pub special: Option<PartialMoveData>, pub sweep: Option<PartialMoveData>,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveSet { pub punch: MoveData, pub kick: MoveData, pub special: MoveData, pub sweep: MoveData }

impl Default for MoveSet {
    fn default() -> Self {
        // Valores extraídos de js/data/config.js do Brazil Fight Combat original.
        Self {
            punch: MoveData::new(5, 4, 9, 6, 75.0, 170.0, 16, 3.0),
            kick: MoveData::new(8, 5, 13, 9, 95.0, 110.0, 20, 5.0),
            sweep: MoveData::new(7, 5, 14, 8, 100.0, 35.0, 30, 4.0),
            special: MoveData::new(14, 7, 22, 16, 140.0, 160.0, 28, 9.0),
        }
    }
}

impl MoveSet {
    pub fn get(&self, kind: crate::fighter::MoveKind) -> MoveData {
        match kind { crate::fighter::MoveKind::Punch => self.punch, crate::fighter::MoveKind::Kick => self.kick,
            crate::fighter::MoveKind::Special => self.special, crate::fighter::MoveKind::Sweep => self.sweep }
    }
    pub fn for_character(self, character: &CharacterConfig) -> Self {
        let o = &character.move_overrides;
        Self { punch: o.punch.map_or(self.punch, |v| v.apply(self.punch)),
            kick: o.kick.map_or(self.kick, |v| v.apply(self.kick)),
            special: o.special.map_or(self.special, |v| v.apply(self.special)),
            sweep: o.sweep.map_or(self.sweep, |v| v.apply(self.sweep)) }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RigConfig { pub neck: f32, pub hip: f32, pub split: f32, pub flip: bool, pub skirt: bool }

#[derive(Clone, Debug)]
pub struct CharacterConfig {
    pub id: String,
    pub name: String,
    /// Pasta relativa à raiz do projeto, normalmente chars/<id>.
    pub frame_dir: String,
    pub speed: f32,
    pub power: f32,
    pub hp: i32,
    pub size: u32,
    pub body_width: f32,
    pub body_height: f32,
    pub color: [u8; 3],
    pub stage: Option<String>,
    pub music: Option<String>,
    pub voice: String,
    pub campaign_role: CampaignRole,
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub facing_left: bool,
    pub chroma_tolerance: u8,
    pub max_frame_height: u32,
    pub move_overrides: MoveOverrides,
    pub rig: Option<RigConfig>,
}

impl CharacterConfig {
    pub fn default_red() -> Self {
        Self { id: "rubro".into(), name: "Guerreiro Rubro".into(), frame_dir: "assets/characters/rubro".into(),
            speed: 4.3, power: 1.0, hp: 200, size: 215, body_width: 54.0, body_height: 150.0,
            color: [202, 52, 66], stage: None, music: None, voice: "default".into(), campaign_role: CampaignRole::Normal, anchor_x: 0.5, anchor_y: 0.92,
            facing_left: false, chroma_tolerance: 90, max_frame_height: 560, move_overrides: MoveOverrides::default(), rig: None }
    }
    pub fn default_blue() -> Self {
        Self { id: "azul".into(), name: "Combatente Azul".into(), frame_dir: "assets/characters/azul".into(),
            speed: 4.0, power: 1.08, hp: 200, size: 215, body_width: 54.0, body_height: 150.0,
            color: [47, 115, 220], stage: None, music: None, voice: "default".into(), campaign_role: CampaignRole::Normal, anchor_x: 0.5, anchor_y: 0.92,
            facing_left: false, chroma_tolerance: 90, max_frame_height: 560, move_overrides: MoveOverrides::default(), rig: None }
    }
}

#[derive(Clone, Debug)]
pub struct StageConfig { pub id: String, pub name: String, pub image: String, pub music: Option<String>, pub ground: f32 }

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub width: usize, pub height: usize, pub ground: f32, pub round_seconds: u32, pub rounds_to_win: u8, pub cpu_difficulty: u8,
    pub gravity: f32, pub jump_velocity: f32, pub stage_name: String,
    pub sky: [u8; 3], pub floor: [u8; 3], pub player1: CharacterConfig, pub player2: CharacterConfig,
    pub moves: MoveSet, pub roster: Vec<CharacterConfig>, pub stages: Vec<StageConfig>, pub arcade_opponents: usize, pub root: PathBuf,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let player1 = CharacterConfig::default_red();
        let player2 = CharacterConfig::default_blue();
        Self { width: 960, height: 540, ground: 470.0, round_seconds: 60, rounds_to_win: 2, cpu_difficulty: 2, gravity: 1.0, jump_velocity: 15.0,
            stage_name: "Centro da Cidade".into(), sky: [25, 34, 67], floor: [52, 58, 68],
            player1: player1.clone(), player2: player2.clone(), moves: MoveSet::default(),
            roster: vec![player1, player2], stages: Vec::new(), arcade_opponents: 8, root: PathBuf::from(".") }
    }
}

impl EngineConfig {
    pub fn load(root: &Path) -> Self {
        let mut cfg = Self::default();
        cfg.root = root.to_path_buf();
        let engine = read_ini(&root.join("data/engine.ini"));
        cfg.ground = get_f32(&engine, "ground", cfg.ground);
        cfg.round_seconds = get_u32(&engine, "round_seconds", cfg.round_seconds).max(1);
        cfg.rounds_to_win = get_u32(&engine, "rounds_to_win", cfg.rounds_to_win as u32).clamp(1, 5) as u8;
        cfg.cpu_difficulty = get_u32(&engine, "cpu_difficulty", cfg.cpu_difficulty as u32).min(5) as u8;
        cfg.arcade_opponents = get_u32(&engine, "arcade_opponents", cfg.arcade_opponents as u32).min(100) as usize;
        cfg.gravity = get_f32(&engine, "gravity", cfg.gravity).max(0.01);
        cfg.jump_velocity = get_f32(&engine, "jump_velocity", cfg.jump_velocity).max(1.0);
        cfg.stage_name = engine.get("stage_name").cloned().unwrap_or(cfg.stage_name);
        cfg.sky = [get_u8(&engine, "sky_red", cfg.sky[0]), get_u8(&engine, "sky_green", cfg.sky[1]), get_u8(&engine, "sky_blue", cfg.sky[2])];
        cfg.floor = [get_u8(&engine, "floor_red", cfg.floor[0]), get_u8(&engine, "floor_green", cfg.floor[1]), get_u8(&engine, "floor_blue", cfg.floor[2])];
        cfg.moves = load_moves(&root.join("data/moves.ini"));
        cfg.stages = load_stages(&root.join("data/stages.ini"));

        let roster_path = root.join("chars/chars.json");
        if let Ok(bytes) = fs::read(&roster_path) {
            if let Ok(json) = serde_json::from_slice::<Value>(&bytes) {
                if let Some(entries) = json.get("personagens").and_then(Value::as_array) {
                    let loaded: Vec<CharacterConfig> = entries.iter().filter_map(|v| v.as_str())
                        .filter_map(|id| load_character_json(root, id)).collect();
                    if !loaded.is_empty() {
                        cfg.roster = loaded;
                        let p1 = cfg.roster.iter().position(|c| c.id == "amanda").unwrap_or(0);
                        let p2 = cfg.roster.iter().position(|c| c.id == "ayana").unwrap_or_else(|| if cfg.roster.len() > 1 { 1 } else { 0 });
                        cfg.player1 = cfg.roster[p1].clone();
                        cfg.player2 = cfg.roster[p2].clone();
                    }
                }
            }
        }
        // Compatibilidade com o protótipo anterior caso a pasta chars/ original não exista.
        if cfg.roster.len() == 2 && !root.join("chars/chars.json").exists() {
            cfg.player1 = load_legacy_character(&root.join("data/chars/rubro.ini"), CharacterConfig::default_red());
            cfg.player2 = load_legacy_character(&root.join("data/chars/azul.ini"), CharacterConfig::default_blue());
            cfg.roster = vec![cfg.player1.clone(), cfg.player2.clone()];
        }
        cfg
    }

    pub fn stage_index(&self, id: &str) -> Option<usize> { self.stages.iter().position(|stage| stage.id == id) }
}

fn load_character_json(root: &Path, id: &str) -> Option<CharacterConfig> {
    let path = root.join("chars").join(id).join("character.json");
    let bytes = fs::read(&path).ok()?;
    let j: Value = serde_json::from_slice(&bytes).ok()?;
    let null = Value::Null;
    let stats = j.get("stats").unwrap_or(&null);
    let body = j.get("body").unwrap_or(&null);
    let anchor = j.get("anchor").unwrap_or(&null);
    let chroma = j.get("chroma").unwrap_or(&null);
    let rig = j.get("rig").filter(|v| v.is_object()).map(|v| RigConfig {
        neck: json_f32(v, "neck", 0.24).clamp(0.05, 0.7), hip: json_f32(v, "hip", 0.571).clamp(0.2, 0.95),
        split: json_f32(v, "split", 0.493).clamp(0.1, 0.9), flip: json_bool(v, "flip", false), skirt: json_bool(v, "skirt", false),
    });
    let color_seed = id.bytes().fold(0_u32, |acc, b| acc.wrapping_mul(33).wrapping_add(b as u32));
    let palette = [[202, 52, 66], [47, 115, 220], [59, 176, 116], [195, 107, 223], [231, 155, 59], [65, 196, 198]];
    let color = palette[(color_seed as usize) % palette.len()];
    Some(CharacterConfig {
        id: id.to_owned(), name: j.get("name").and_then(Value::as_str).unwrap_or(id).to_owned(),
        frame_dir: format!("chars/{}", id), speed: json_f32(stats, "speed", 4.5).clamp(1.0, 12.0),
        power: json_f32(stats, "power", 1.0).clamp(0.1, 5.0), hp: json_i32(stats, "hp", 200).clamp(1, 5000),
        size: json_u32(&j, "size", 300).clamp(32, 800), body_width: json_f32(body, "w", 70.0).clamp(10.0, 300.0),
        body_height: json_f32(body, "h", 230.0).clamp(20.0, 400.0), color,
        stage: j.get("stage").and_then(Value::as_str).map(str::to_owned),
        music: j.get("music").and_then(Value::as_str).map(str::to_owned),
        voice: j.get("voice").and_then(Value::as_str).unwrap_or(id).to_owned(),
        campaign_role: CampaignRole::from_json(j.get("campaignRole").or_else(|| j.get("campaign_role")).and_then(Value::as_str), id),
        anchor_x: json_f32(anchor, "x", 0.5).clamp(0.0, 1.0), anchor_y: json_f32(anchor, "y", 0.92).clamp(0.0, 1.0),
        facing_left: j.get("facing").and_then(Value::as_str) == Some("left"),
        chroma_tolerance: json_u32(chroma, "tolerance", 90).min(255) as u8,
        max_frame_height: json_u32(chroma, "maxHeight", 560).clamp(128, 1200),
        move_overrides: parse_move_overrides(j.get("moves").unwrap_or(&null)), rig,
    })
}

fn parse_move_overrides(v: &Value) -> MoveOverrides {
    MoveOverrides { punch: parse_partial(v.get("punch")), kick: parse_partial(v.get("kick")),
        special: parse_partial(v.get("special")), sweep: parse_partial(v.get("sweep")) }
}
fn parse_partial(v: Option<&Value>) -> Option<PartialMoveData> {
    let v = v.filter(|v| v.is_object())?;
    Some(PartialMoveData {
        startup: json_opt_u16(v, "up"), active: json_opt_u16(v, "act"), recovery: json_opt_u16(v, "rec"),
        damage: json_opt_i32(v, "dmg"), reach: json_opt_f32(v, "reach"), y_offset: json_opt_f32(v, "y"),
        stun: json_opt_u16(v, "stun"), knockback: json_opt_f32(v, "kb"),
    })
}

fn load_stages(path: &Path) -> Vec<StageConfig> {
    let values = read_ini(path);
    let mut stages = Vec::new();
    for index in 0..64 {
        let prefix = format!("stage.{:02}.", index);
        let Some(id) = values.get(&(prefix.clone() + "id")) else { break; };
        let image = values.get(&(prefix.clone() + "image")).cloned().unwrap_or_default();
        let music_value = values.get(&(prefix.clone() + "music")).cloned().unwrap_or_default();
        let music = if music_value.is_empty() || music_value.eq_ignore_ascii_case("null") { None } else { Some(music_value) };
        stages.push(StageConfig { id: id.clone(), name: values.get(&(prefix.clone() + "name")).cloned().unwrap_or_else(|| id.clone()),
            image, music, ground: get_f32(&values, &(prefix + "ground"), 470.0).clamp(0.0, 540.0) });
    }
    stages
}

fn load_moves(path: &Path) -> MoveSet {
    let v = read_ini(path);
    let d = MoveSet::default();
    MoveSet { punch: read_move(&v, "punch", d.punch), kick: read_move(&v, "kick", d.kick),
        special: read_move(&v, "special", d.special), sweep: read_move(&v, "sweep", d.sweep) }
}
fn read_move(v: &HashMap<String, String>, prefix: &str, d: MoveData) -> MoveData {
    MoveData { startup: get_u16(v, &format!("{}.startup", prefix), d.startup).min(120),
        active: get_u16(v, &format!("{}.active", prefix), d.active).clamp(1, 120),
        recovery: get_u16(v, &format!("{}.recovery", prefix), d.recovery).min(240),
        damage: get_i32(v, &format!("{}.damage", prefix), d.damage).clamp(1, 500),
        reach: get_f32(v, &format!("{}.reach", prefix), d.reach).clamp(10.0, 500.0),
        y_offset: get_f32(v, &format!("{}.y_offset", prefix), d.y_offset).clamp(0.0, 500.0),
        stun: get_u16(v, &format!("{}.stun", prefix), d.stun).min(240),
        knockback: get_f32(v, &format!("{}.knockback", prefix), d.knockback).clamp(0.0, 30.0) }
}
fn load_legacy_character(path: &Path, mut cfg: CharacterConfig) -> CharacterConfig {
    let v = read_ini(path);
    cfg.name = v.get("name").cloned().unwrap_or(cfg.name);
    cfg.frame_dir = v.get("frame_dir").cloned().unwrap_or(cfg.frame_dir);
    cfg.speed = get_f32(&v, "speed", cfg.speed).clamp(1.0, 12.0);
    cfg.power = get_f32(&v, "power", cfg.power).clamp(0.1, 5.0);
    cfg.hp = get_i32(&v, "hp", cfg.hp).clamp(1, 5000);
    cfg.size = get_u32(&v, "size", cfg.size).clamp(32, 800);
    cfg.body_width = get_f32(&v, "body_width", cfg.body_width).clamp(10.0, 300.0);
    cfg.body_height = get_f32(&v, "body_height", cfg.body_height).clamp(20.0, 400.0);
    cfg.color = [get_u8(&v, "red", cfg.color[0]), get_u8(&v, "green", cfg.color[1]), get_u8(&v, "blue", cfg.color[2])];
    cfg
}
fn read_ini(path: &Path) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let Ok(contents) = fs::read_to_string(path) else { return values; };
    let mut section = String::new();
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') { continue; }
        if line.starts_with('[') && line.ends_with(']') { section = line[1..line.len() - 1].trim().to_ascii_lowercase(); continue; }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_ascii_lowercase(); let value = value.trim().to_owned();
            let full_key = if section.is_empty() { key } else { format!("{}.{}", section, key) };
            values.insert(full_key, value);
        }
    }
    values
}
fn get_f32(v: &HashMap<String, String>, k: &str, default: f32) -> f32 { v.get(k).and_then(|s| s.parse().ok()).filter(|n: &f32| n.is_finite()).unwrap_or(default) }
fn get_u32(v: &HashMap<String, String>, k: &str, default: u32) -> u32 { v.get(k).and_then(|s| s.parse().ok()).unwrap_or(default) }
fn get_u16(v: &HashMap<String, String>, k: &str, default: u16) -> u16 { v.get(k).and_then(|s| s.parse().ok()).unwrap_or(default) }
fn get_i32(v: &HashMap<String, String>, k: &str, default: i32) -> i32 { v.get(k).and_then(|s| s.parse().ok()).unwrap_or(default) }
fn get_u8(v: &HashMap<String, String>, k: &str, default: u8) -> u8 { v.get(k).and_then(|s| s.parse().ok()).unwrap_or(default) }
fn json_f32(v: &Value, k: &str, default: f32) -> f32 { v.get(k).and_then(Value::as_f64).map(|n| n as f32).filter(|n| n.is_finite()).unwrap_or(default) }
fn json_i32(v: &Value, k: &str, default: i32) -> i32 { v.get(k).and_then(Value::as_i64).and_then(|n| i32::try_from(n).ok()).unwrap_or(default) }
fn json_u32(v: &Value, k: &str, default: u32) -> u32 { v.get(k).and_then(Value::as_u64).and_then(|n| u32::try_from(n).ok()).unwrap_or(default) }
fn json_bool(v: &Value, k: &str, default: bool) -> bool {
    v.get(k)
        .and_then(|value| value.as_bool().or_else(|| value.as_i64().map(|n| n != 0)).or_else(|| value.as_u64().map(|n| n != 0)))
        .unwrap_or(default)
}
fn json_opt_u16(v: &Value, k: &str) -> Option<u16> { v.get(k).and_then(Value::as_u64).and_then(|n| u16::try_from(n).ok()) }
fn json_opt_i32(v: &Value, k: &str) -> Option<i32> { v.get(k).and_then(Value::as_i64).and_then(|n| i32::try_from(n).ok()) }
fn json_opt_f32(v: &Value, k: &str) -> Option<f32> { v.get(k).and_then(Value::as_f64).map(|n| n as f32).filter(|n| n.is_finite()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn move_duration_is_bounded_and_summed() { assert_eq!(MoveData::new(5, 4, 9, 6, 82.0, 145.0, 16, 3.0).duration(), 18); }
    #[test] fn original_punch_values_match_documentation() { let m = MoveSet::default().punch; assert_eq!((m.startup, m.active, m.recovery, m.damage), (5, 4, 9, 6)); assert_eq!(m.reach, 75.0); }

    #[test]
    fn loads_the_original_roster_stages_and_character_overrides() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config = EngineConfig::load(root);
        assert_eq!(config.roster.len(), 21);
        assert_eq!(config.stages.len(), 26);
        assert!(config.roster.iter().any(|character| character.id == "viktor-krayller"));
        assert!(config.roster.iter().any(|character| character.id == "ayana"));
        assert_eq!(config.moves.punch.startup, 5);
        assert_eq!(config.moves.special.damage, 16);
        assert!(config.roster.iter().find(|character| character.id == "amanda").unwrap().rig.is_some());
    }
}
