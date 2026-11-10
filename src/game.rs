//! Orquestra os modos, cenas e sistemas. A simulação avança em ticks fixos de 60 Hz.
use crate::ai::AiController;
use crate::combat::{CombatEvent, CombatSystem};
use crate::config::{CampaignRole, CharacterConfig, EngineConfig, MoveSet};
use crate::controller::{Controller, ControllerContext, HumanController};
use crate::fighter::{Facing, Fighter, FighterState, MoveKind};
use crate::input::{FighterInput, FrameInput};
use std::fs;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scene { Menu, Settings, Fight, Story }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameMode { Versus, Cpu, Training, Arcade, AiVsAi }

#[derive(Clone, Copy, Debug)]
pub struct Particle { pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub life: u8, pub blocked: bool }

/// Efeito temporário criado por um golpe. A arte e o posicionamento vêm do character.json.
#[derive(Clone, Debug)]
pub struct ActiveEffect {
    pub name: String,
    pub attacker: u8,
    pub target: u8,
    pub age: u16,
    pub lifetime: u16,
}

pub struct Game {
    pub config: EngineConfig,
    pub p1: Fighter,
    pub p2: Fighter,
    pub scene: Scene,
    pub mode: GameMode,
    pub menu_selection: usize,
    pub settings_selection: usize,
    pub tick: u64,
    pub timer_ticks: u32,
    pub match_over: bool,
    pub winner: u8,
    pub show_hitboxes: bool,
    pub training_guard: bool,
    pub particles: Vec<Particle>,
    pub active_effects: Vec<ActiveEffect>,
    pub exit_requested: bool,
    pub selected_p1: usize,
    pub selected_p2: usize,
    pub current_stage: usize,
    pub stage_manually_selected: bool,
    pub desired_music: String,
    pub sound_events: Vec<String>,
    pub round_wins: [u8; 2],
    pub round_number: u8,
    pub round_pause_ticks: u16,
    pub last_round_winner: u8,
    pub campaign_order: Vec<usize>,
    pub campaign_position: usize,
    pub story_title: String,
    pub story_text: String,
    pub story_pages: Vec<String>,
    pub story_page: usize,
    human_controller: HumanController,
    ai_controller: AiController,
    ai_controller_p1: AiController,
    combat: CombatSystem,
}

impl Game {
    pub fn new(mut config: EngineConfig) -> Self {
        if config.roster.is_empty() { config.roster = vec![config.player1.clone(), config.player2.clone()]; }
        let selected_p1 = config.roster.iter().position(|c| c.id == config.player1.id).unwrap_or(0);
        let selected_p2 = config.roster.iter().position(|c| c.id == config.player2.id)
            .unwrap_or_else(|| if config.roster.len() > 1 { 1 } else { 0 });
        let player1 = config.roster[selected_p1].clone();
        let player2 = config.roster[selected_p2].clone();
        let stage = home_stage_index(&config, &player1, &player2).unwrap_or(0);
        if let Some(st) = config.stages.get(stage) { config.ground = st.ground; config.stage_name = st.name.clone(); }
        let p1 = Fighter::new(player1, config.width as f32 * 0.31, config.ground, Facing::Right);
        let p2 = Fighter::new(player2, config.width as f32 * 0.69, config.ground, Facing::Left);
        let timer_ticks = config.round_seconds.saturating_mul(60);
        let cpu_difficulty = config.cpu_difficulty;
        let learning_path = config.root.join("data/ai_learning.json");
        let mut ai_controller = AiController::load_learning(&learning_path);
        ai_controller.set_difficulty(cpu_difficulty);
        // Os dois agentes podem consultar o mesmo perfil para o modo de observação.
        // Cada agente mantém estado aleatório independente para não espelhar movimentos.
        let mut ai_controller_p1 = AiController::load_learning(&learning_path);
        ai_controller_p1.set_difficulty(cpu_difficulty);
        ai_controller_p1.set_learning_enabled(false);
        ai_controller_p1.set_recording_enabled(false);
        ai_controller_p1.set_random_seed(0xC801_3EA4);
        Self { config, p1, p2, scene: Scene::Menu, mode: GameMode::Versus, menu_selection: 0, settings_selection: 0, tick: 0,
            timer_ticks, match_over: false, winner: 0, show_hitboxes: false, training_guard: false,
            particles: Vec::with_capacity(48), active_effects: Vec::with_capacity(12), exit_requested: false, selected_p1, selected_p2,
            current_stage: stage, stage_manually_selected: false, desired_music: "assets/audio/music/menu.mp3".into(),
            sound_events: Vec::with_capacity(8), round_wins: [0, 0], round_number: 1, round_pause_ticks: 0,
            last_round_winner: 0, campaign_order: Vec::new(), campaign_position: 0,
            story_title: String::new(), story_text: String::new(), story_pages: Vec::new(), story_page: 0, human_controller: HumanController,
            ai_controller, ai_controller_p1, combat: CombatSystem }
    }

    pub fn update(&mut self, input: FrameInput) {
        self.sound_events.clear();
        self.tick = self.tick.wrapping_add(1);
        if self.tick % 1800 == 0 && self.config.cpu_difficulty == 2 && matches!(self.mode, GameMode::Cpu | GameMode::Arcade) {
            self.ai_controller.save_learning(&self.config.root.join("data/ai_learning.json"));
        }
        self.update_active_effects();
        match self.scene {
            Scene::Menu => { self.update_menu(input); return; }
            Scene::Settings => { self.update_settings(input); return; }
            Scene::Story => {
                if input.escape {
                    self.scene = Scene::Menu;
                    self.desired_music = "assets/audio/music/menu.mp3".into();
                    self.sound_events.push("confirm".into());
                } else if input.confirm {
                    if self.story_page + 1 < self.story_pages.len() {
                        self.story_page += 1;
                        self.sound_events.push("select".into());
                    } else {
                        self.scene = Scene::Menu;
                        self.desired_music = "assets/audio/music/menu.mp3".into();
                        self.sound_events.push("confirm".into());
                    }
                }
                return;
            }
            Scene::Fight => {}
        }
        if input.escape {
            self.scene = Scene::Menu;
            self.particles.clear();
            self.active_effects.clear();
            self.desired_music = "assets/audio/music/menu.mp3".into();
            return;
        }
        if input.toggle_debug { self.show_hitboxes = !self.show_hitboxes; }
        if self.mode == GameMode::Training {
            if input.toggle_training_guard { self.training_guard = !self.training_guard; self.sound_events.push("select".into()); }
            if input.restart_training { self.reset_fight(); self.sound_events.push("confirm".into()); return; }
        }
        if self.match_over {
            if input.confirm { self.continue_after_match(); }
            return;
        }
        if self.round_pause_ticks > 0 {
            self.round_pause_ticks -= 1;
            self.update_particles();
            if self.round_pause_ticks == 0 { self.begin_next_round(); }
            return;
        }

        // O Fighter.js só atualiza a direção no estado livre e no chão. Ataques,
        // stun e K.O. mantêm a orientação iniciada antes do movimento.
        if self.p1.is_grounded() && !self.p1.is_attacking() && self.p1.state != FighterState::Hit && !self.p1.is_ko() {
            self.p1.face_toward(self.p2.x);
        }
        if self.p2.is_grounded() && !self.p2.is_attacking() && self.p2.state != FighterState::Hit && !self.p2.is_ko() {
            self.p2.face_toward(self.p1.x);
        }
        let moves1 = self.config.moves.for_character(&self.p1.config);
        let moves2 = self.config.moves.for_character(&self.p2.config);
        let ctx1 = ControllerContext { actor: &self.p1, opponent: &self.p2, moves: &moves1, tick: self.tick };
        let p1_input = if self.mode == GameMode::AiVsAi {
            self.ai_controller_p1.sample(&ctx1, input.p1)
        } else {
            self.human_controller.sample(&ctx1, input.p1)
        };
        let p2_input = match self.mode {
            GameMode::Versus => {
                let ctx2 = ControllerContext { actor: &self.p2, opponent: &self.p1, moves: &moves2, tick: self.tick };
                self.human_controller.sample(&ctx2, input.p2)
            }
            GameMode::Cpu | GameMode::Arcade | GameMode::AiVsAi => {
                let ctx2 = ControllerContext { actor: &self.p2, opponent: &self.p1, moves: &moves2, tick: self.tick };
                self.ai_controller.sample(&ctx2, input.p2)
            }
            GameMode::Training => FighterInput { block: self.training_guard, ..Default::default() },
        };

        let old_state_1 = self.p1.state;
        let old_state_2 = self.p2.state;
        let old_hp_1 = self.p1.hp;
        let old_hp_2 = self.p2.hp;
        self.p1.update(p1_input, &moves1, self.config.width as f32, self.config.ground, self.config.gravity, self.config.jump_velocity);
        self.p2.update(p2_input, &moves2, self.config.width as f32, self.config.ground, self.config.gravity, self.config.jump_velocity);
        record_start_sounds(old_state_1, self.p1.state, &mut self.sound_events);
        record_start_sounds(old_state_2, self.p2.state, &mut self.sound_events);
        let events = self.combat.resolve_pair(&mut self.p1, &mut self.p2, &moves1, &moves2);
        // Depois da resolução sequencial: se P1 interrompeu P2, o golpe de P2 não cria FX.
        self.spawn_attack_effect(1, &moves1);
        self.spawn_attack_effect(2, &moves2);
        self.spawn_events(&events);
        for event in &events {
            self.sound_events.push(if event.blocked { "block".into() } else { move_sound(event.kind).into() });
        }
        if (old_hp_1 > 0 && self.p1.hp <= 0) || (old_hp_2 > 0 && self.p2.hp <= 0) { self.sound_events.push("ko".into()); }
        self.update_particles();

        if self.mode != GameMode::Training {
            self.timer_ticks = self.timer_ticks.saturating_sub(1);
            if self.p1.is_ko() || self.p2.is_ko() || self.timer_ticks == 0 { self.finish_round(); }
        }
    }

    fn update_menu(&mut self, input: FrameInput) {
        if input.escape { self.exit_requested = true; return; }
        if input.menu_up { self.menu_selection = (self.menu_selection + 6) % 7; self.sound_events.push("select".into()); }
        if input.menu_down { self.menu_selection = (self.menu_selection + 1) % 7; self.sound_events.push("select".into()); }
        if input.p1_char_prev { self.selected_p1 = wrap_index(self.selected_p1, self.config.roster.len(), false); self.sound_events.push("select".into()); }
        if input.p1_char_next { self.selected_p1 = wrap_index(self.selected_p1, self.config.roster.len(), true); self.sound_events.push("select".into()); }
        if input.p2_char_prev { self.selected_p2 = wrap_index(self.selected_p2, self.config.roster.len(), false); self.sound_events.push("select".into()); }
        if input.p2_char_next { self.selected_p2 = wrap_index(self.selected_p2, self.config.roster.len(), true); self.sound_events.push("select".into()); }
        if input.stage_prev && !self.config.stages.is_empty() {
            self.current_stage = wrap_index(self.current_stage, self.config.stages.len(), false); self.stage_manually_selected = true;
            self.apply_current_stage(); self.sound_events.push("select".into());
        }
        if input.stage_next && !self.config.stages.is_empty() {
            self.current_stage = wrap_index(self.current_stage, self.config.stages.len(), true); self.stage_manually_selected = true;
            self.apply_current_stage(); self.sound_events.push("select".into());
        }
        if input.confirm {
            self.sound_events.push("confirm".into());
            match self.menu_selection {
                0 => self.start_fight(GameMode::Versus),
                1 => self.start_fight(GameMode::Cpu),
                2 => self.start_fight(GameMode::Training),
                3 => self.start_fight(GameMode::Arcade),
                4 => self.start_fight(GameMode::AiVsAi),
                5 => { self.settings_selection = 0; self.scene = Scene::Settings; },
                _ => self.exit_requested = true,
            }
        }
    }

    fn update_settings(&mut self, input: FrameInput) {
        if input.escape {
            self.scene = Scene::Menu;
            self.sound_events.push("confirm".into());
            return;
        }
        if input.menu_up {
            self.settings_selection = (self.settings_selection + 5) % 6;
            self.sound_events.push("select".into());
        }
        if input.menu_down {
            self.settings_selection = (self.settings_selection + 1) % 6;
            self.sound_events.push("select".into());
        }
        if input.p1_char_prev { self.adjust_setting(-1); self.sound_events.push("select".into()); }
        if input.p1_char_next { self.adjust_setting(1); self.sound_events.push("select".into()); }
        if input.confirm {
            if self.settings_selection == 5 {
                self.scene = Scene::Menu;
            } else if self.settings_selection == 4 {
                self.show_hitboxes = !self.show_hitboxes;
            } else {
                self.adjust_setting(1);
            }
            self.sound_events.push("confirm".into());
        }
        if input.p1_char_prev || input.p1_char_next || input.confirm {
            self.persist_settings();
        }
    }

    fn persist_settings(&self) {
        let path = self.config.root.join("data/engine.ini");
        let mut lines = fs::read_to_string(&path).unwrap_or_default().lines().map(str::to_owned).collect::<Vec<_>>();
        let settings = [
            ("cpu_difficulty", self.config.cpu_difficulty.to_string()),
            ("round_seconds", self.config.round_seconds.to_string()),
            ("rounds_to_win", self.config.rounds_to_win.to_string()),
            ("arcade_opponents", self.config.arcade_opponents.to_string()),
        ];
        for (key, value) in settings {
            let prefix = format!("{}=", key);
            if let Some(line) = lines.iter_mut().find(|line| line.trim_start().starts_with(&prefix)) {
                *line = format!("{}={}", key, value);
            } else {
                lines.push(format!("{}={}", key, value));
            }
        }
        let mut content = lines.join("\n");
        content.push('\n');
        let _ = fs::write(path, content);
    }

    /// Ajusta os parâmetros de configuração sem alterar a lógica de combate.
    fn adjust_setting(&mut self, delta: i8) {
        match self.settings_selection {
            0 => {
                self.config.cpu_difficulty = (self.config.cpu_difficulty as i16 + delta as i16).rem_euclid(6) as u8;
                self.ai_controller.set_difficulty(self.config.cpu_difficulty);
                self.ai_controller_p1.set_difficulty(self.config.cpu_difficulty);
            }
            1 => {
                const DURATIONS: [u32; 3] = [30, 60, 99];
                let current = DURATIONS.iter().position(|seconds| *seconds == self.config.round_seconds).unwrap_or(1);
                let next = (current as i16 + delta as i16).rem_euclid(DURATIONS.len() as i16) as usize;
                self.config.round_seconds = DURATIONS[next];
                self.timer_ticks = self.config.round_seconds.saturating_mul(60);
            }
            2 => {
                self.config.rounds_to_win = (self.config.rounds_to_win as i16 + delta as i16 - 1)
                    .rem_euclid(5) as u8 + 1;
            }
            3 => {
                const OPPONENT_COUNTS: [usize; 6] = [3, 5, 8, 12, 16, 24];
                let current = OPPONENT_COUNTS.iter().position(|n| *n == self.config.arcade_opponents).unwrap_or(2);
                let next = (current as i16 + delta as i16).rem_euclid(OPPONENT_COUNTS.len() as i16) as usize;
                self.config.arcade_opponents = OPPONENT_COUNTS[next];
            }
            4 => self.show_hitboxes = !self.show_hitboxes,
            _ => self.scene = Scene::Menu,
        }
    }

    fn start_fight(&mut self, mode: GameMode) {
        self.mode = mode;
        // CPU e Arcade aprendem com a pessoa apenas em NORMAL. IA vs IA consulta o
        // perfil já aprendido, mas não grava as partidas automáticas nele.
        let adaptive_mode = matches!(mode, GameMode::Cpu | GameMode::Arcade | GameMode::AiVsAi);
        let recording_mode = matches!(mode, GameMode::Cpu | GameMode::Arcade);
        self.ai_controller.set_learning_enabled(adaptive_mode);
        self.ai_controller.set_recording_enabled(recording_mode);
        self.ai_controller_p1.set_learning_enabled(mode == GameMode::AiVsAi);
        self.ai_controller_p1.set_recording_enabled(false);
        if mode == GameMode::Arcade { self.stage_manually_selected = false; self.build_campaign_order(); }
        self.match_over = false;
        self.winner = 0;
        self.round_wins = [0, 0]; self.round_number = 1;
        self.prepare_stage();
        if mode == GameMode::Arcade && !self.campaign_order.is_empty() {
            self.campaign_position = 0;
            self.selected_p2 = self.campaign_order[0];
            self.prepare_stage();
        }
        self.reset_fight();
        self.scene = Scene::Fight;
    }

    pub fn reset_fight(&mut self) {
        let p1_cfg = self.selected_character(1);
        let p2_cfg = self.selected_character(2);
        self.p1 = Fighter::new(p1_cfg, self.config.width as f32 * 0.31, self.config.ground, Facing::Right);
        self.p2 = Fighter::new(p2_cfg, self.config.width as f32 * 0.69, self.config.ground, Facing::Left);
        self.timer_ticks = self.config.round_seconds.saturating_mul(60);
        self.match_over = false; self.winner = 0; self.training_guard = false;
        self.round_wins = [0, 0]; self.round_number = 1; self.round_pause_ticks = 0; self.last_round_winner = 0;
        self.particles.clear(); self.active_effects.clear(); self.ai_controller.reset();
        self.ai_controller.set_difficulty(self.config.cpu_difficulty);
        self.ai_controller_p1.reset();
        self.ai_controller_p1.set_difficulty(self.config.cpu_difficulty);
    }

    fn begin_next_round(&mut self) {
        let p1_cfg = self.selected_character(1);
        let p2_cfg = self.selected_character(2);
        self.p1 = Fighter::new(p1_cfg, self.config.width as f32 * 0.31, self.config.ground, Facing::Right);
        self.p2 = Fighter::new(p2_cfg, self.config.width as f32 * 0.69, self.config.ground, Facing::Left);
        self.timer_ticks = self.config.round_seconds.saturating_mul(60);
        self.ai_controller.reset();
        self.ai_controller_p1.reset();
    }

    fn finish_round(&mut self) {
        let round_winner = if self.p1.hp == self.p2.hp { 0 } else if self.p1.hp > self.p2.hp { 1 } else { 2 };
        self.last_round_winner = round_winner;
        if round_winner == 0 { self.round_wins[0] = self.round_wins[0].saturating_add(1); self.round_wins[1] = self.round_wins[1].saturating_add(1); }
        else { self.round_wins[(round_winner - 1) as usize] = self.round_wins[(round_winner - 1) as usize].saturating_add(1); }
        let needed = self.config.rounds_to_win;
        if self.round_wins[0] >= needed || self.round_wins[1] >= needed {
            self.match_over = true;
            self.winner = if self.round_wins[0] == self.round_wins[1] { 0 } else if self.round_wins[0] > self.round_wins[1] { 1 } else { 2 };
            // Procura a fala do vencedor em qualquer modo. Se não houver gravação,
            // o adaptador Windows tenta sintetizar voices.win.text do character.json.
            let winner_id = match self.winner {
                1 => Some(self.p1.config.id.as_str()),
                2 => Some(self.p2.config.id.as_str()),
                _ => None,
            };
            if let Some(id) = winner_id {
                self.sound_events.push(format!("voice:{}:win", id));
            }
            if self.config.cpu_difficulty == 2 && matches!(self.mode, GameMode::Cpu | GameMode::Arcade) {
                self.ai_controller.save_learning(&self.config.root.join("data/ai_learning.json"));
            }
        } else {
            self.round_number = self.round_number.saturating_add(1);
            self.round_pause_ticks = 120;
        }
    }

    fn continue_after_match(&mut self) {
        self.sound_events.push("confirm".into());
        if self.mode != GameMode::Arcade { self.reset_fight(); return; }
        if self.winner != 1 { self.return_to_menu(); return; }
        self.campaign_position += 1;
        if self.campaign_position >= self.campaign_order.len() {
            self.open_story(); return;
        }
        self.selected_p2 = self.campaign_order[self.campaign_position];
        self.round_wins = [0, 0]; self.round_number = 1; self.stage_manually_selected = false;
        self.prepare_stage(); self.reset_fight(); self.scene = Scene::Fight;
    }

    fn return_to_menu(&mut self) {
        self.scene = Scene::Menu;
        self.match_over = false;
        self.active_effects.clear();
        self.desired_music = "assets/audio/music/menu.mp3".into();
    }

    fn open_story(&mut self) {
        let character = self.selected_character(1);
        let path = self.config.root.join("story").join(&character.id).join("historia.txt");
        self.story_text = fs::read_to_string(path).unwrap_or_else(|_| format!("{} venceu a campanha Brazil Fight Combat.\n\nA jornada continua.", character.name));
        self.story_title = format!("FINAL DE {}", character.name.to_uppercase());
        self.story_pages = paginate_story(&self.story_text, 98, 11);
        if self.story_pages.is_empty() { self.story_pages.push("A jornada continua.".into()); }
        self.story_page = 0;
        self.scene = Scene::Story; self.desired_music = "assets/audio/music/menu.mp3".into();
    }

    fn build_campaign_order(&mut self) {
        let player_id = self.selected_character(1).id.clone();
        let mut normal: Vec<usize> = self.config.roster.iter().enumerate()
            .filter(|(_, character)| character.id != player_id && character.campaign_role == CampaignRole::Normal)
            .map(|(index, _)| index).collect();

        // Fisher-Yates com PRNG local; o número de adversários normais é configurável.
        let mut state = (self.tick as u32).wrapping_add(0x91E1_0DA5).max(1);
        for i in (1..normal.len()).rev() {
            state ^= state << 13; state ^= state >> 17; state ^= state << 5;
            normal.swap(i, (state as usize) % (i + 1));
        }
        normal.truncate(self.config.arcade_opponents.min(normal.len()));
        self.campaign_order = normal;

        // Há no máximo um sub-chefe e um chefe; os papéis vêm do JSON.
        for role in [CampaignRole::SubBoss, CampaignRole::Boss] {
            if let Some(index) = self.config.roster.iter().position(|character| character.campaign_role == role && character.id != player_id) {
                self.campaign_order.push(index);
            }
        }
        if self.campaign_order.is_empty() && self.config.roster.len() > 1 {
            self.campaign_order.push((self.selected_p1 + 1) % self.config.roster.len());
        }
    }

    fn selected_character(&self, player: u8) -> CharacterConfig {
        let index = if player == 1 { self.selected_p1 } else { self.selected_p2 };
        self.config.roster.get(index).cloned().unwrap_or_else(|| if player == 1 { self.config.player1.clone() } else { self.config.player2.clone() })
    }

    fn prepare_stage(&mut self) {
        if self.config.stages.is_empty() { self.current_stage = 0; return; }
        if !self.stage_manually_selected {
            let p1 = self.selected_character(1); let p2 = self.selected_character(2);
            self.current_stage = home_stage_index(&self.config, &p1, &p2)
                .unwrap_or_else(|| (self.tick as usize).wrapping_add(self.selected_p1 * 7 + self.selected_p2) % self.config.stages.len());
        }
        self.apply_current_stage();
        let stage = self.config.stages[self.current_stage.min(self.config.stages.len() - 1)].clone();
        // A música exclusiva de personagem só deve tocar no mapa que pertence a ele.
        // Se a escolha automática sorteou um cenário neutro, usa a música do cenário
        // (ou battle.mp3), em vez de levar o tema de um personagem para qualquer mapa.
        let character_music = if self.stage_manually_selected {
            None
        } else {
            let p2 = self.selected_character(2);
            let p1 = self.selected_character(1);
            [p2, p1].into_iter()
                .find(|character| character.stage.as_deref() == Some(stage.id.as_str()))
                .and_then(|owner| owner.music.as_deref().map(|id| music_path_for_id(id, &owner.id)))
        };
        let music = character_music
            .or_else(|| stage.music.as_deref().map(music_path_for_id_simple))
            .unwrap_or_else(|| "assets/audio/music/battle.mp3".to_owned());
        self.desired_music = music;
    }

    fn apply_current_stage(&mut self) {
        if let Some(stage) = self.config.stages.get(self.current_stage) {
            self.config.ground = stage.ground;
            self.config.stage_name = stage.name.clone();
        }
    }

    fn spawn_events(&mut self, events: &[CombatEvent]) {
        for event in events {
            let target = if event.attacker == 1 { 2 } else { 1 };
            let name = if event.blocked { "block" } else { "hit" };
            self.active_effects.push(ActiveEffect {
                name: name.to_owned(), attacker: event.attacker, target, age: 0,
                lifetime: if event.blocked { 12 } else { 16 },
            });
            let count = if event.blocked { 5 } else { 9 };
            for i in 0..count {
                if self.particles.len() >= 48 { break; }
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                let speed = if event.blocked { 1.5 } else { 2.6 };
                self.particles.push(Particle { x: event.x, y: event.y, vx: angle.cos() * speed, vy: angle.sin() * speed - 0.4, life: 18, blocked: event.blocked });
            }
        }
    }

    fn spawn_attack_effect(&mut self, attacker: u8, moves: &MoveSet) {
        let fighter = if attacker == 1 { &self.p1 } else { &self.p2 };
        let FighterState::Attack(kind) = fighter.state else { return; };
        let movement = moves.get(kind);
        // O efeito surge no primeiro tick ativo, e não durante o startup do golpe.
        if fighter.attack_ticks != movement.startup { return; }
        let name = match kind {
            MoveKind::Punch => "punch",
            MoveKind::Kick => "kick",
            MoveKind::Special => "special",
            MoveKind::Sweep => "sweep",
        };
        self.active_effects.push(ActiveEffect {
            name: name.to_owned(), attacker, target: if attacker == 1 { 2 } else { 1 },
            age: 0, lifetime: 12,
        });
    }

    fn update_active_effects(&mut self) {
        for effect in &mut self.active_effects { effect.age = effect.age.saturating_add(1); }
        self.active_effects.retain(|effect| effect.age < effect.lifetime);
    }

    fn update_particles(&mut self) {
        for p in &mut self.particles { p.x += p.vx; p.y += p.vy; p.vy += 0.12; p.life = p.life.saturating_sub(1); }
        self.particles.retain(|p| p.life > 0);
    }
    pub fn timer_seconds(&self) -> u32 { self.timer_ticks.saturating_add(59) / 60 }
}

fn paginate_story(text: &str, max_chars: usize, lines_per_page: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    let lines_per_page = lines_per_page.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + word.len() + 1 > max_chars {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() { line.push(' '); }
        // Um token excepcionalmente longo não é dividido; textos normais permanecem intactos.
        line.push_str(word);
    }
    if !line.is_empty() { lines.push(line); }
    if lines.is_empty() { return Vec::new(); }
    lines.chunks(lines_per_page).map(|page| page.join("\n")).collect()
}

fn home_stage_index(config: &EngineConfig, p1: &CharacterConfig, p2: &CharacterConfig) -> Option<usize> {
    p2.stage.as_deref().and_then(|id| config.stage_index(id)).or_else(|| p1.stage.as_deref().and_then(|id| config.stage_index(id)))
}
fn wrap_index(current: usize, count: usize, forward: bool) -> usize {
    if count == 0 { return 0; }
    if forward { (current + 1) % count } else if current == 0 { count - 1 } else { current - 1 }
}
fn move_sound(kind: MoveKind) -> &'static str { match kind { MoveKind::Punch => "punch", MoveKind::Kick | MoveKind::Sweep => "kick", MoveKind::Special => "special" } }
fn record_start_sounds(previous: FighterState, current: FighterState, output: &mut Vec<String>) {
    if previous == current { return; }
    match current {
        FighterState::Jump => output.push("jump".into()),
        _ => {}
    }
}
fn music_path_for_id(id: &str, character_id: &str) -> String {
    if id.contains('/') || id.ends_with(".mp3") || id.ends_with(".wav") || id.ends_with(".ogg") {
        if id.starts_with("assets/") || id.starts_with("chars/") { id.to_owned() } else { format!("chars/{}/{}", character_id, id) }
    } else { music_path_for_id_simple(id) }
}
fn music_path_for_id_simple(id: &str) -> String {
    if id == "menu" { return "assets/audio/music/menu.mp3".into(); }
    if id == "battle" { return "assets/audio/music/battle.mp3".into(); }
    if id.contains('/') || id.ends_with(".mp3") || id.ends_with(".wav") || id.ends_with(".ogg") {
        return id.to_owned();
    }
    format!("assets/audio/music/{}.wav", id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EngineConfig;
    use crate::input::FrameInput;
    #[test] fn story_text_is_split_into_pages_without_losing_words() {
        let text = (0..30).map(|n| format!("palavra{}", n)).collect::<Vec<_>>().join(" ");
        let pages = paginate_story(&text, 18, 2);
        assert!(pages.len() > 1);
        let restored = pages.join(" ").replace('\n', " ").split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(restored, text);
    }

    #[test] fn menu_can_enter_training_then_escape_to_menu() {
        let mut game = Game::new(EngineConfig::default()); game.menu_selection = 2;
        game.update(FrameInput { confirm: true, ..Default::default() });
        assert_eq!(game.scene, Scene::Fight); assert_eq!(game.mode, GameMode::Training);
        game.update(FrameInput { escape: true, ..Default::default() }); assert_eq!(game.scene, Scene::Menu);
    }
    #[test] fn roster_navigation_wraps_without_panicking() {
        let mut game = Game::new(EngineConfig::default()); game.selected_p1 = 0;
        game.update(FrameInput { p1_char_prev: true, ..Default::default() });
        assert_eq!(game.selected_p1, 1);
    }

    #[test] fn settings_screen_can_change_cpu_difficulty_and_return() {
        // O teste usa uma pasta temporária para não alterar o engine.ini do projeto real.
        let test_root = std::env::temp_dir().join(format!("bfc_settings_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(test_root.join("data"));
        let _ = std::fs::write(test_root.join("data/engine.ini"), "cpu_difficulty=2\nround_seconds=60\nrounds_to_win=2\narcade_opponents=8\n");
        let mut config = EngineConfig::default();
        config.root = test_root.clone();
        let mut game = Game::new(config);
        game.menu_selection = 5;
        game.update(FrameInput { confirm: true, ..Default::default() });
        assert_eq!(game.scene, Scene::Settings);
        game.update(FrameInput { p1_char_next: true, ..Default::default() });
        assert_eq!(game.config.cpu_difficulty, 3);
        game.update(FrameInput { escape: true, ..Default::default() });
        assert_eq!(game.scene, Scene::Menu);
        let _ = std::fs::remove_dir_all(test_root);
    }

    #[test]
    fn character_theme_does_not_follow_the_character_to_an_unrelated_random_stage() {
        let mut config = EngineConfig::default();
        let mut p1 = crate::config::CharacterConfig::default_red();
        p1.id = "hero".into();
        p1.music = Some("assets/audio/music/hero-theme.mp3".into());
        p1.stage = None;
        let mut p2 = crate::config::CharacterConfig::default_blue();
        p2.id = "rival".into();
        p2.stage = None;
        p2.music = None;
        config.player1 = p1.clone();
        config.player2 = p2.clone();
        config.roster = vec![p1, p2];
        config.stages = vec![
            crate::config::StageConfig { id: "home".into(), name: "Casa".into(), image: "home.png".into(), music: Some("stage-theme".into()), ground: 470.0 },
            crate::config::StageConfig { id: "neutral".into(), name: "Neutro".into(), image: "neutral.png".into(), music: None, ground: 470.0 },
        ];
        config.root = std::env::temp_dir().join(format!("bfc_music_test_{}", std::process::id()));
        let mut game = Game::new(config);
        game.tick = 0; // (tick + selected_p1 * 7 + selected_p2) % 2 == 1
        game.stage_manually_selected = false;
        game.prepare_stage();
        assert_eq!(game.current_stage, 1);
        assert_eq!(game.desired_music, "assets/audio/music/battle.mp3");
    }

    #[test]
    fn arcade_order_places_unique_subboss_and_boss_after_configured_normal_opponents() {
        let mut config = EngineConfig::default();
        let mut player = crate::config::CharacterConfig::default_red();
        player.id = "hero".into(); player.name = "Heroi".into();
        let mut normal_a = crate::config::CharacterConfig::default_blue();
        normal_a.id = "normal-a".into();
        let mut normal_b = crate::config::CharacterConfig::default_blue();
        normal_b.id = "normal-b".into();
        let mut normal_c = crate::config::CharacterConfig::default_blue();
        normal_c.id = "normal-c".into();
        let mut subboss = crate::config::CharacterConfig::default_blue();
        subboss.id = "amanda".into(); subboss.campaign_role = CampaignRole::SubBoss;
        let mut boss = crate::config::CharacterConfig::default_red();
        boss.id = "viktor-krayller".into(); boss.campaign_role = CampaignRole::Boss;
        config.roster = vec![player.clone(), normal_a, normal_b, normal_c, subboss, boss];
        config.player1 = player;
        config.player2 = config.roster[1].clone();
        config.arcade_opponents = 2;
        let mut game = Game::new(config);
        game.selected_p1 = 0;
        game.build_campaign_order();
        assert_eq!(game.campaign_order.len(), 4);
        assert_eq!(game.config.roster[game.campaign_order[2]].campaign_role, CampaignRole::SubBoss);
        assert_eq!(game.config.roster[game.campaign_order[3]].campaign_role, CampaignRole::Boss);
    }
}
