//! Entrada por frame; ataques são eventos de pressionamento e movimentos são contínuos.
#[derive(Clone, Copy, Debug, Default)]
pub struct FighterInput {
    pub left: bool, pub right: bool, pub up: bool, pub down: bool,
    pub punch: bool, pub kick: bool, pub special: bool, pub block: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInput {
    pub p1: FighterInput, pub p2: FighterInput,
    pub menu_up: bool, pub menu_down: bool, pub confirm: bool, pub escape: bool,
    pub toggle_debug: bool, pub toggle_training_guard: bool, pub restart_training: bool,
    pub p1_char_prev: bool, pub p1_char_next: bool, pub p2_char_prev: bool, pub p2_char_next: bool,
    pub stage_prev: bool, pub stage_next: bool,
}
