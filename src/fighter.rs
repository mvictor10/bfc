//! Entidade de combate: atributos, física e máquina de estados explícita.
use crate::config::{CharacterConfig, MoveSet};
use crate::geometry::Rect;
use crate::input::FighterInput;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MoveKind { Punch, Kick, Special, Sweep }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Facing { Left, Right }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FighterState {
    Idle,
    Walk,
    Run,
    Jump,
    Crouch,
    Block,
    Attack(MoveKind),
    Hit,
    Ko,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub config: CharacterConfig,
    pub x: f32,
    pub feet_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub hp: i32,
    pub facing: Facing,
    pub state: FighterState,
    /// Relógio contínuo da entidade, usado para animacões em loop (equivalente a f.t no JS).
    pub animation_ticks: u64,
    pub state_ticks: u16,
    pub attack_ticks: u16,
    pub hit_stun: u16,
    pub combo_hits: u16,
    pub combo_timer: u16,
    did_hit: bool,
    walk_ticks: u16,
    ground_y: f32,
}

impl Fighter {
    pub fn new(config: CharacterConfig, x: f32, ground_y: f32, facing: Facing) -> Self {
        let hp = config.hp;
        Self {
            config, x, feet_y: ground_y, velocity_x: 0.0, velocity_y: 0.0, hp,
            facing, state: FighterState::Idle, animation_ticks: 0, state_ticks: 0, attack_ticks: 0,
            hit_stun: 0, combo_hits: 0, combo_timer: 0, did_hit: false,
            walk_ticks: 0, ground_y,
        }
    }

    pub fn is_grounded(&self) -> bool { self.feet_y >= self.ground_y - 0.5 }
    pub fn is_ko(&self) -> bool { self.hp <= 0 || self.state == FighterState::Ko }
    pub fn is_attacking(&self) -> bool { matches!(self.state, FighterState::Attack(_)) }
    pub fn has_hit_this_move(&self) -> bool { self.did_hit }
    pub fn mark_move_hit(&mut self) { self.did_hit = true; }

    pub fn face_toward(&mut self, target_x: f32) {
        self.facing = if target_x < self.x { Facing::Left } else { Facing::Right };
    }

    pub fn body_rect(&self) -> Rect {
        // O jogo original reduz a hurtbox ao agachar e quando o lutador está em K.O.
        let crouching = matches!(self.state, FighterState::Crouch | FighterState::Ko);
        let h = if crouching { self.config.body_height * 0.65 } else { self.config.body_height };
        Rect::new(self.x - self.config.body_width * 0.5, self.feet_y - h, self.config.body_width, h)
    }

    /// Retorna uma hitbox apenas durante os ticks ativos do ataque.
    pub fn attack_rect(&self, moves: &MoveSet) -> Option<Rect> {
        let FighterState::Attack(kind) = self.state else { return None; };
        if self.did_hit { return None; }
        let mv = moves.get(kind);
        if self.attack_ticks < mv.startup || self.attack_ticks >= mv.startup.saturating_add(mv.active) { return None; }
        let x = match self.facing {
            Facing::Right => self.x + 10.0,
            Facing::Left => self.x - 10.0 - mv.reach,
        };
        // Alinha a hitbox às coordenadas do Fighter.js original (y medido a partir dos pés).
        let y = self.feet_y - mv.y_offset;
        Some(Rect::new(x, y, mv.reach, 50.0))
    }

    pub fn can_block_attack_from(&self, attacker_x: f32) -> bool {
        self.state == FighterState::Block && self.is_grounded() && match self.facing {
            Facing::Right => attacker_x >= self.x,
            Facing::Left => attacker_x <= self.x,
        }
    }

    pub fn receive_hit(&mut self, damage: i32, stun: u16, knockback: f32, blocked: bool, attacker_x: f32) {
        if self.is_ko() { return; }
        self.hp = (self.hp - damage.max(0)).max(0);
        let direction = if self.x >= attacker_x { 1.0 } else { -1.0 };
        if self.hp <= 0 {
            self.state = FighterState::Ko;
            self.hit_stun = 0;
            self.state_ticks = 0;
            self.animation_ticks = 0;
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
        } else if blocked {
            // O CombatSystem.js aplica apenas 10% de dano e meia força de empurrão;
            // não cancela a defesa nem adiciona stun ao defensor.
            self.velocity_x = direction * knockback * 0.5;
        } else {
            self.velocity_x = direction * knockback;
            self.hit_stun = stun;
            self.state = FighterState::Hit;
            self.state_ticks = 0;
            self.animation_ticks = 0;
        }
    }

    pub fn update(&mut self, input: FighterInput, moves: &MoveSet, world_width: f32, ground_y: f32, gravity: f32, jump_velocity: f32) {
        self.ground_y = ground_y;
        self.animation_ticks = self.animation_ticks.wrapping_add(1);
        if self.hp <= 0 { self.state = FighterState::Ko; return; }
        let previous_state = self.state;
        self.state_ticks = self.state_ticks.saturating_add(1);
        if self.combo_timer > 0 { self.combo_timer -= 1; } else { self.combo_hits = 0; }

        if self.hit_stun > 0 {
            self.hit_stun -= 1;
            if self.hit_stun == 0 { self.state = FighterState::Idle; self.state_ticks = 0; }
            self.apply_physics(gravity, world_width);
            return;
        }

        if let FighterState::Attack(kind) = self.state {
            self.attack_ticks = self.attack_ticks.saturating_add(1);
            let duration = moves.get(kind).duration();
            if self.attack_ticks >= duration {
                self.state = FighterState::Idle;
                self.state_ticks = 0;
                self.attack_ticks = 0;
                self.did_hit = false;
            }
            self.apply_physics(gravity, world_width);
            return;
        }

        if input.down && self.is_grounded() && input.kick {
            self.start_attack(MoveKind::Sweep);
            self.apply_physics(gravity, world_width);
            return;
        }
        if input.special { self.start_attack(MoveKind::Special); self.apply_physics(gravity, world_width); return; }
        if input.kick { self.start_attack(MoveKind::Kick); self.apply_physics(gravity, world_width); return; }
        if input.punch { self.start_attack(MoveKind::Punch); self.apply_physics(gravity, world_width); return; }

        if input.down && self.is_grounded() {
            self.state = if input.block { FighterState::Block } else { FighterState::Crouch };
            self.walk_ticks = 0;
        } else if input.block && self.is_grounded() {
            self.state = FighterState::Block;
            self.walk_ticks = 0;
        } else if input.up && self.is_grounded() {
            self.velocity_y = -jump_velocity;
            self.state = FighterState::Jump;
            self.state_ticks = 0;
            self.walk_ticks = 0;
        } else if input.left != input.right {
            self.velocity_x = if input.left { -self.config.speed } else { self.config.speed };
            if self.walk_ticks > 30 { self.velocity_x *= 1.35; }
            self.walk_ticks = self.walk_ticks.saturating_add(1);
            self.state = if self.walk_ticks > 30 { FighterState::Run } else { FighterState::Walk };
        } else {
            self.walk_ticks = 0;
            self.state = if self.is_grounded() { FighterState::Idle } else { FighterState::Jump };
            self.velocity_x *= 0.55;
            if self.velocity_x.abs() < 0.15 { self.velocity_x = 0.0; }
        }

        if input.left && !input.right { self.velocity_x = -self.config.speed; }
        if input.right && !input.left { self.velocity_x = self.config.speed; }
        if input.block || input.down { self.velocity_x *= 0.45; }
        // state_ticks é equivalente a stateT do Fighter.js e reinicia apenas na
        // transição de estado, para animações once começarem no primeiro quadro.
        if self.state != previous_state { self.state_ticks = 0; }
        self.apply_physics(gravity, world_width);
    }

    fn start_attack(&mut self, kind: MoveKind) {
        self.state = FighterState::Attack(kind);
        self.animation_ticks = 0;
        self.state_ticks = 0;
        self.attack_ticks = 0;
        self.did_hit = false;
        self.walk_ticks = 0;
        self.velocity_x *= 0.2;
    }

    fn apply_physics(&mut self, gravity: f32, world_width: f32) {
        self.x += self.velocity_x;
        self.velocity_x *= 0.78;
        if !self.is_grounded() || self.velocity_y < 0.0 {
            self.velocity_y += gravity;
            self.feet_y += self.velocity_y;
            if self.feet_y >= self.ground_y {
                self.feet_y = self.ground_y;
                self.velocity_y = 0.0;
                if self.state == FighterState::Jump { self.state = FighterState::Idle; }
            }
        } else {
            self.feet_y = self.ground_y;
            self.velocity_y = 0.0;
        }
        let margin = self.config.body_width * 0.5;
        self.x = self.x.clamp(margin, (world_width - margin).max(margin));
    }
}
