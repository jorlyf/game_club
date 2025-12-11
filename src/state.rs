use bevy::state::state::States;

#[derive(States, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub enum AppState {
  #[default]
  Menu,
}

#[derive(States, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub enum GameState {
  #[default]
  Playing,
  Paused,
}
