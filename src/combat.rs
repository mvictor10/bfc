//! Combate: aplica colisões na mesma ordem de resolução do FightScene.js original (P1, depois P2).
use crate::config::MoveSet;
use crate::fighter::{Fighter, FighterState, MoveKind};

#[derive(Clone, Copy, Debug)]
pub struct CombatEvent {
    pub x: f32,
    pub y: f32,
    pub blocked: bool,
    pub damage: i32,
    pub kind: MoveKind,
    /// 1 = jogador 1, 2 = jogador 2.
    pub attacker: u8,
}

#[derive(Default)]
pub struct CombatSystem;

impl CombatSystem {
    /// Interface compatível com testes/protótipos que usam a mesma tabela para ambos.
    pub fn resolve(&mut self, a: &mut Fighter, b: &mut Fighter, moves: &MoveSet) -> Vec<CombatEvent> {
        self.resolve_pair(a, b, moves, moves)
    }

    /// Usa overrides próprios de cada personagem e processa P1 antes de P2, como no JS de origem.
    pub fn resolve_pair(&mut self, a: &mut Fighter, b: &mut Fighter, moves_a: &MoveSet, moves_b: &MoveSet) -> Vec<CombatEvent> {
        let mut events = Vec::with_capacity(2);
        if let Some(event) = resolve_one(a, b, moves_a, 1) { events.push(event); }
        // O primeiro acerto pode ter mudado o segundo lutador para hit/block/ko;
        // por isso a hitbox do segundo é recalculada, não capturada antes do primeiro dano.
        if let Some(event) = resolve_one(b, a, moves_b, 2) { events.push(event); }
        events
    }
}

fn resolve_one(attacker: &mut Fighter, target: &mut Fighter, moves: &MoveSet, attacker_id: u8) -> Option<CombatEvent> {
    if attacker.is_ko() || target.is_ko() { return None; }
    let attack_rect = attacker.attack_rect(moves)?;
    let body_rect = target.body_rect();
    if !attack_rect.intersects(body_rect) { return None; }

    let kind = match attacker.state { FighterState::Attack(kind) => kind, _ => return None };
    let movement = moves.get(kind);
    let blocked = target.can_block_attack_from(attacker.x);
    let full_damage = ((movement.damage as f32) * attacker.config.power).round() as i32;
    let damage = if blocked { ((full_damage as f32 * 0.10).round() as i32).max(1) } else { full_damage.max(1) };
    let combo_continues = target.state == FighterState::Hit;
    let impact_x = (attack_rect.x + attack_rect.w * 0.5).clamp(0.0, 960.0);
    let impact_y = (attack_rect.y + attack_rect.h * 0.5).max(0.0);

    attacker.mark_move_hit();
    target.receive_hit(damage, movement.stun, movement.knockback, blocked, attacker.x);
    if !blocked {
        attacker.combo_hits = if combo_continues { attacker.combo_hits.saturating_add(1) } else { 1 };
        attacker.combo_timer = 60;
    }

    Some(CombatEvent { x: impact_x, y: impact_y, blocked, damage, kind, attacker: attacker_id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CharacterConfig, MoveSet};
    use crate::fighter::{Facing, Fighter, FighterState, MoveKind};

    #[test]
    fn punch_damages_once_during_a_single_attack() {
        let mut a = Fighter::new(CharacterConfig::default_red(), 300.0, 470.0, Facing::Right);
        let mut b = Fighter::new(CharacterConfig::default_blue(), 350.0, 470.0, Facing::Left);
        a.state = FighterState::Attack(MoveKind::Punch); a.attack_ticks = 6;
        let mut system = CombatSystem; let before = b.hp;
        let events = system.resolve(&mut a, &mut b, &MoveSet::default());
        assert_eq!(events.len(), 1); assert!(b.hp < before); assert!(a.has_hit_this_move());
        let after = b.hp; assert!(system.resolve(&mut a, &mut b, &MoveSet::default()).is_empty()); assert_eq!(b.hp, after);
    }

    #[test]
    fn frontal_block_reduces_damage() {
        let mut a = Fighter::new(CharacterConfig::default_red(), 300.0, 470.0, Facing::Right);
        let mut b = Fighter::new(CharacterConfig::default_blue(), 350.0, 470.0, Facing::Left);
        b.state = FighterState::Block; a.state = FighterState::Attack(MoveKind::Punch); a.attack_ticks = 6;
        let mut system = CombatSystem; let events = system.resolve(&mut a, &mut b, &MoveSet::default());
        assert_eq!(events.len(), 1); assert!(events[0].blocked); assert!(events[0].damage <= 2);
    }

    #[test]
    fn blocked_hit_preserves_block_state_and_uses_half_knockback() {
        let mut attacker = Fighter::new(CharacterConfig::default_red(), 300.0, 470.0, Facing::Right);
        let mut defender = Fighter::new(CharacterConfig::default_blue(), 350.0, 470.0, Facing::Left);
        defender.state = FighterState::Block;
        attacker.state = FighterState::Attack(MoveKind::Punch);
        attacker.attack_ticks = 6;
        let initial_vx = defender.velocity_x;
        let mut system = CombatSystem;
        let events = system.resolve(&mut attacker, &mut defender, &MoveSet::default());
        assert_eq!(events.len(), 1);
        assert!(events[0].blocked);
        assert_eq!(defender.state, FighterState::Block);
        assert_eq!(defender.hit_stun, 0);
        assert!(defender.velocity_x > initial_vx);
    }

    #[test]
    fn first_hit_interrupts_the_second_attack_like_the_original_scene() {
        let mut a = Fighter::new(CharacterConfig::default_red(), 300.0, 470.0, Facing::Right);
        let mut b = Fighter::new(CharacterConfig::default_blue(), 350.0, 470.0, Facing::Left);
        a.state = FighterState::Attack(MoveKind::Punch); a.attack_ticks = 6;
        b.state = FighterState::Attack(MoveKind::Punch); b.attack_ticks = 6;
        let mut system = CombatSystem;
        let events = system.resolve_pair(&mut a, &mut b, &MoveSet::default(), &MoveSet::default());
        assert_eq!(events.len(), 1);
        assert_eq!(b.state, FighterState::Hit);
        assert!(!a.is_ko());
    }
}
