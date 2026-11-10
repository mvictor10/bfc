//! Controladores seguem a mesma interface (Strategy), sem acoplar a entidade ao teclado.
use crate::config::MoveSet;
use crate::fighter::Fighter;
use crate::input::FighterInput;

pub struct ControllerContext<'a> {
    pub actor: &'a Fighter,
    pub opponent: &'a Fighter,
    pub moves: &'a MoveSet,
    pub tick: u64,
}

pub trait Controller {
    fn sample(&mut self, context: &ControllerContext<'_>, human_input: FighterInput) -> FighterInput;
}

#[derive(Default)]
pub struct HumanController;
impl Controller for HumanController {
    fn sample(&mut self, _context: &ControllerContext<'_>, human_input: FighterInput) -> FighterInput { human_input }
}
