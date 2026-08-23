// Handle different actor types e.g. bot, human
use crate::game::Action;

// TODO
pub struct GameView;

pub trait Agent {
    fn act(&mut self, view: &GameView, legal_actions: &[Action]) -> Action;
}
