use bfc_engine::config::{CharacterConfig, EngineConfig, MoveSet};
use bfc_engine::fighter::{Facing, Fighter, FighterState, MoveKind};
use bfc_engine::game::{Game, GameMode, Scene};
use bfc_engine::input::FrameInput;

#[test]
fn fighters_start_with_configured_health_and_ground_position() {
    let fighter = Fighter::new(CharacterConfig::default_red(), 250.0, 470.0, Facing::Right);
    assert_eq!(fighter.hp, 200);
    assert_eq!(fighter.feet_y, 470.0);
    assert!(fighter.is_grounded());
}

#[test]
fn hitboxes_only_exist_in_active_attack_window() {
    let mut fighter = Fighter::new(CharacterConfig::default_red(), 300.0, 470.0, Facing::Right);
    let moves = MoveSet::default();
    fighter.state = FighterState::Attack(MoveKind::Punch);
    fighter.attack_ticks = 2;
    assert!(fighter.attack_rect(&moves).is_none());
    fighter.attack_ticks = 6;
    assert!(fighter.attack_rect(&moves).is_some());
}

#[test]
fn training_mode_can_be_entered_from_menu() {
    let mut game = Game::new(EngineConfig::default());
    game.menu_selection = 2;
    game.update(FrameInput { confirm: true, ..Default::default() });
    assert_eq!(game.scene, Scene::Fight);
    assert_eq!(game.mode, GameMode::Training);
}
