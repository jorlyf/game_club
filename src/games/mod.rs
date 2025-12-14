use bevy::prelude::*;

pub mod snake;

pub struct GamesPlugin;

impl Plugin for GamesPlugin {
  fn build(&self, app: &mut App) {
    app.register_type::<Game>();
    app.register_type::<GameMachine>();
    app.add_observer(on_add_game_machine);
  }
}

#[derive(Default, Debug, Reflect)]
#[reflect(Default)]
pub enum Game {
  #[default]
  Snake,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
struct GameMachine {
  game: Game,
}

fn on_add_game_machine(
  add_game_machine: On<Add, GameMachine>,
  query: Query<(&GameMachine, &GlobalTransform)>,
  mut _commands: Commands,
) {
  let entity = add_game_machine.event().entity;

  let Ok((game_machine, global_transform)) = query.get(entity) else {
    return;
  };

  info!(
    "New Machine [{:?} @ {:?}]",
    game_machine,
    global_transform.translation()
  );
}
