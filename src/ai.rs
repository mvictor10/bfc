//! CPU adaptativa do Brazil Fight Combat.
//! A aprendizagem é estatística, persistente e intencionalmente ativa apenas no nível NORMAL.
use crate::controller::{Controller, ControllerContext};
use crate::fighter::{Fighter, FighterState, MoveKind};
use crate::input::FighterInput;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct AiLearning {
    pub observations: u64,
    pub punches: u64,
    pub kicks: u64,
    pub specials: u64,
    pub sweeps: u64,
    pub blocks: u64,
    pub jumps: u64,
    pub crouches: u64,
}

impl AiLearning {
    fn from_path(path: &Path) -> Self {
        let Ok(bytes) = fs::read(path) else { return Self::default(); };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else { return Self::default(); };
        let get = |name: &str| value.get(name).and_then(Value::as_u64).unwrap_or(0);
        Self {
            observations: get("observations"), punches: get("punches"), kicks: get("kicks"),
            specials: get("specials"), sweeps: get("sweeps"), blocks: get("blocks"),
            jumps: get("jumps"), crouches: get("crouches"),
        }
    }

    fn persist(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        let value = json!({
            "schema_version": 1,
            "observations": self.observations,
            "punches": self.punches,
            "kicks": self.kicks,
            "specials": self.specials,
            "sweeps": self.sweeps,
            "blocks": self.blocks,
            "jumps": self.jumps,
            "crouches": self.crouches,
        });
        fs::write(path, serde_json::to_vec_pretty(&value).unwrap_or_default())
    }

    fn ratio(&self, count: u64) -> f32 {
        count as f32 / self.observations.max(1) as f32
    }

    fn attack_ratio(&self) -> f32 {
        self.ratio(self.punches + self.kicks + self.specials + self.sweeps)
    }
}

#[derive(Clone, Debug)]
pub struct AiController {
    difficulty: u8,
    cooldown: u8,
    intention: Intention,
    random_seed: u32,
    random_state: u32,
    learning: AiLearning,
    /// Permite usar estatísticas aprendidas quando o nível é NORMAL.
    learning_enabled: bool,
    /// Só registra e persiste observações em partidas contra uma pessoa.
    recording_enabled: bool,
    last_observed_state: Option<FighterState>,
}

#[derive(Clone, Copy, Debug)]
enum Intention { Approach, Retreat, Guard, Jump, Attack }

impl Default for AiController {
    fn default() -> Self {
        Self {
            // Dificuldade 2 corresponde a NORMAL na lista de seis níveis.
            difficulty: 2, cooldown: 0, intention: Intention::Approach,
            random_seed: 0xA341_316C, random_state: 0xA341_316C, learning: AiLearning::default(),
            learning_enabled: true, recording_enabled: true, last_observed_state: None,
        }
    }
}

impl AiController {
    pub fn load_learning(path: &Path) -> Self {
        let mut controller = Self::default();
        controller.learning = AiLearning::from_path(path);
        controller
    }

    pub fn set_difficulty(&mut self, difficulty: u8) {
        self.difficulty = difficulty.min(5);
        self.cooldown = 0;
    }

    /// Liga/desliga o uso do perfil adaptativo (válido somente no nível NORMAL).
    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.learning_enabled = enabled;
        self.last_observed_state = None;
    }

    /// Separa aprendizagem durante partidas reais de observação/replay entre CPUs.
    pub fn set_recording_enabled(&mut self, enabled: bool) {
        self.recording_enabled = enabled;
        self.last_observed_state = None;
    }

    pub fn set_random_seed(&mut self, seed: u32) {
        self.random_seed = seed.max(1);
        self.random_state = self.random_seed;
    }

    pub fn save_learning(&self, path: &Path) {
        if self.recording_enabled && self.learning_enabled && self.difficulty == 2 {
            let _ = self.learning.persist(path);
        }
    }

    pub fn learning_observations(&self) -> u64 { self.learning.observations }

    pub fn reset(&mut self) {
        self.cooldown = 0;
        self.intention = Intention::Approach;
        self.random_state = self.random_seed;
        self.last_observed_state = None;
    }

    fn random_u32(&mut self) -> u32 {
        // Xorshift32: sem alocação, mutex ou dependência externa na simulação.
        let mut value = self.random_state;
        value ^= value << 13; value ^= value >> 17; value ^= value << 5;
        self.random_state = value.max(1);
        self.random_state
    }

    fn random(&mut self) -> f32 {
        (self.random_u32() & 0x00FF_FFFF) as f32 / 16_777_216.0
    }

    fn observe_opponent(&mut self, opponent: &Fighter) {
        if self.last_observed_state == Some(opponent.state) { return; }
        self.last_observed_state = Some(opponent.state);
        self.learning.observations = self.learning.observations.saturating_add(1);
        match opponent.state {
            FighterState::Attack(MoveKind::Punch) => self.learning.punches = self.learning.punches.saturating_add(1),
            FighterState::Attack(MoveKind::Kick) => self.learning.kicks = self.learning.kicks.saturating_add(1),
            FighterState::Attack(MoveKind::Special) => self.learning.specials = self.learning.specials.saturating_add(1),
            FighterState::Attack(MoveKind::Sweep) => self.learning.sweeps = self.learning.sweeps.saturating_add(1),
            FighterState::Block => self.learning.blocks = self.learning.blocks.saturating_add(1),
            FighterState::Jump => self.learning.jumps = self.learning.jumps.saturating_add(1),
            FighterState::Crouch => self.learning.crouches = self.learning.crouches.saturating_add(1),
            _ => {}
        }
    }
}

impl Controller for AiController {
    fn sample(&mut self, ctx: &ControllerContext<'_>, _human_input: FighterInput) -> FighterInput {
        // Conhecimento adaptativo só atua em NORMAL. No modo IA vs IA o perfil é
        // consultado, mas não é atualizado por partidas automáticas.
        let adaptive = self.difficulty == 2 && self.learning_enabled;
        if adaptive && self.recording_enabled { self.observe_opponent(ctx.opponent); }

        let distance = (ctx.opponent.x - ctx.actor.x).abs();
        let toward_right = ctx.opponent.x > ctx.actor.x;
        let opponent_attacking = ctx.opponent.is_attacking();

        if self.cooldown == 0 {
            // Muito fácil, fácil, normal, difícil, muito difícil e hardcore.
            self.cooldown = [42, 30, 18, 12, 8, 5][self.difficulty as usize];
            let roll = self.random();
            self.intention = if roll > 0.94 {
                Intention::Jump
            } else if distance > 190.0 {
                Intention::Approach
            } else if distance < 110.0 {
                if roll < 0.50 { Intention::Attack }
                else if roll < 0.72 { Intention::Retreat }
                else { Intention::Guard }
            } else if roll < 0.62 { Intention::Approach } else { Intention::Attack };
        } else {
            self.cooldown -= 1;
        }

        let mut output = FighterInput::default();
        match self.intention {
            Intention::Approach if distance > 100.0 => {
                output.right = toward_right; output.left = !toward_right;
            }
            Intention::Retreat => { output.left = toward_right; output.right = !toward_right; }
            Intention::Jump => { output.up = true; output.right = toward_right; output.left = !toward_right; }
            Intention::Guard if opponent_attacking => output.block = true,
            Intention::Attack if distance < 160.0 && !ctx.actor.is_attacking() => {
                let roll = self.random();
                if adaptive {
                    let block_ratio = self.learning.ratio(self.learning.blocks);
                    let jump_ratio = self.learning.ratio(self.learning.jumps);
                    let punch_ratio = self.learning.ratio(self.learning.punches);
                    let kick_ratio = self.learning.ratio(self.learning.kicks);
                    // A CPU escolhe respostas simples à tendência observada, sem treino de rede neural.
                    if block_ratio > 0.24 && roll < 0.68 {
                        output.special = true;
                    } else if jump_ratio > 0.20 && roll < 0.56 {
                        output.kick = true;
                    } else if punch_ratio > kick_ratio && roll < 0.66 {
                        output.kick = true;
                    } else if roll < 0.44 {
                        output.punch = true;
                    } else if roll < 0.82 {
                        output.kick = true;
                    } else if distance < 125.0 {
                        output.special = true;
                    } else {
                        output.punch = true;
                    }
                } else if roll < 0.50 {
                    output.punch = true;
                } else if roll < 0.85 {
                    output.kick = true;
                } else {
                    output.special = true;
                }
                self.intention = Intention::Approach;
            }
            _ => {}
        }

        if opponent_attacking && distance < 145.0 {
            let mut probability = [0.00, 0.02, 0.05, 0.13, 0.22, 0.38][self.difficulty as usize];
            if adaptive {
                let attack_ratio = self.learning.attack_ratio();
                let special_ratio = self.learning.ratio(self.learning.specials);
                probability += (attack_ratio * 0.16 + special_ratio * 0.12).clamp(0.0, 0.28);
            }
            if self.random() < probability.min(0.65) {
                output.block = true; output.punch = false; output.kick = false; output.special = false;
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_supports_six_levels() {
        let mut cpu = AiController::default();
        cpu.set_difficulty(99);
        assert_eq!(cpu.difficulty, 5);
    }

    #[test]
    fn reset_rewinds_random_state_without_erasing_learning() {
        let mut cpu = AiController::default();
        cpu.learning.observations = 12;
        let initial = cpu.random_u32();
        cpu.reset();
        assert_eq!(cpu.random_u32(), initial);
        assert_eq!(cpu.learning_observations(), 12);
    }

    #[test]
    fn custom_random_seed_survives_round_reset() {
        let mut cpu = AiController::default();
        cpu.set_random_seed(0xC801_3EA4);
        let first = cpu.random_u32();
        cpu.reset();
        assert_eq!(cpu.random_u32(), first);
    }

    #[test]
    fn ai_vs_ai_can_consult_learning_without_recording_the_automated_opponent() {
        use crate::config::MoveSet;
        use crate::controller::{Controller, ControllerContext};
        use crate::fighter::{Facing, Fighter};

        let mut cpu = AiController::default();
        cpu.set_learning_enabled(true);
        cpu.set_recording_enabled(false);
        let actor = Fighter::new(crate::config::CharacterConfig::default_red(), 200.0, 470.0, Facing::Right);
        let mut opponent = Fighter::new(crate::config::CharacterConfig::default_blue(), 300.0, 470.0, Facing::Left);
        opponent.state = FighterState::Attack(MoveKind::Punch);
        let moves = MoveSet::default();
        let context = ControllerContext { actor: &actor, opponent: &opponent, moves: &moves, tick: 1 };
        let _ = cpu.sample(&context, FighterInput::default());
        assert_eq!(cpu.learning_observations(), 0);
    }

    #[test]
    fn observation_and_persistence_can_be_disabled_while_adaptation_stays_enabled() {
        let mut cpu = AiController::default();
        cpu.set_learning_enabled(true);
        cpu.set_recording_enabled(false);
        cpu.learning.observations = 10;
        // Sem uma luta real contra a pessoa, a persistência não deve substituir o perfil.
        let path = std::env::temp_dir().join(format!("bfc_ai_read_only_{}.json", std::process::id()));
        let _ = fs::remove_file(&path);
        cpu.save_learning(&path);
        assert!(!path.exists());
        assert_eq!(cpu.learning_observations(), 10);
    }
}
