use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::{
  games::snake::SnakeGamePlugin,
  player::{DespawnPlayerMessage, Player},
  tilemap::DespawnTilemapMessage,
};

mod snake;

pub struct GamesPlugin;

impl Plugin for GamesPlugin {
  fn build(&self, app: &mut App) {
    app.init_resource::<CurrentGameState>();

    app.register_type::<GameType>();
    app.register_type::<GameMachine>();

    app.add_message::<GameMachineTriggerZoneEnterMessage>();
    app.add_message::<GameLaunchMessage>();

    app.add_observer(on_add_game_machine);

    app.add_systems(
      Update,
      (
        check_game_machine_trigger_zone_collision_with_player_system,
        launch_game_system,
      )
        .chain(),
    );

    app.add_plugins(SnakeGamePlugin);
  }
}

#[derive(Resource)]
pub struct CurrentGameState {
  pub current_game: Option<GameType>,
}

impl Default for CurrentGameState {
  fn default() -> Self {
    Self { current_game: None }
  }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Debug, Reflect)]
#[reflect(Default)]
pub enum GameType {
  #[default]
  Snake,
}

impl std::fmt::Display for GameType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GameType::Snake => write!(f, "Snake"),
    }
  }
}

#[derive(Message)]
pub struct GameLaunchMessage {
  pub game: GameType,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
struct GameMachine {
  game: GameType,
}

#[derive(Component, Debug)]
#[require(Transform, Collider)]
struct GameMachineInteractionZone {
  game_machine_entity: Entity,
}

#[derive(Message)]
struct GameMachineTriggerZoneEnterMessage {
  game_machine_entity: Entity,
}

fn launch_game_system(
  mut trigger_zone_messages: MessageReader<GameMachineTriggerZoneEnterMessage>,
  mut game_launch_messages: MessageWriter<GameLaunchMessage>,
  mut despawn_tilemap_messages: MessageWriter<DespawnTilemapMessage>,
  mut despawn_player_messages: MessageWriter<DespawnPlayerMessage>,
  mut game_state: ResMut<CurrentGameState>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  game_machine_query: Query<&GameMachine>,
) {
  if trigger_zone_messages.is_empty() {
    return;
  }

  if !keyboard_input.just_pressed(KeyCode::KeyE) {
    return;
  }

  for message in trigger_zone_messages.read() {
    let Ok(game_machine) = game_machine_query.get(message.game_machine_entity) else {
      continue;
    };

    game_state.current_game = Some(game_machine.game);

    game_launch_messages.write(GameLaunchMessage {
      game: game_machine.game,
    });

    despawn_tilemap_messages.write(DespawnTilemapMessage);
    despawn_player_messages.write(DespawnPlayerMessage);

    break;
  }
}

fn check_game_machine_trigger_zone_collision_with_player_system(
  mut trigger_zone_messages: MessageWriter<GameMachineTriggerZoneEnterMessage>,
  colliding_entities_query: Query<(Entity, &GameMachineInteractionZone, &CollidingEntities)>,
  player_single: Single<Entity, With<Player>>,
) {
  for (_, interaction_zone, colliding_entities) in &colliding_entities_query {
    if colliding_entities.contains(&player_single.entity()) {
      trigger_zone_messages.write(GameMachineTriggerZoneEnterMessage {
        game_machine_entity: interaction_zone.game_machine_entity,
      });
    }
  }
}

fn on_add_game_machine(
  add_game_machine: On<Add, GameMachine>,
  query: Query<&TiledObject, With<GameMachine>>,
  mut commands: Commands,
) {
  let entity = add_game_machine.event().entity;

  let Ok(game_machine_tile_object) = query.get(entity) else {
    return;
  };

  let (tile_width, _) = match game_machine_tile_object {
    TiledObject::Tile { width, height } => (width, height),
    _ => {
      panic!("Unexpected TiledObject type for GameMachine")
    }
  };

  const INTERACTION_ZONE_WIDTH: f32 = 16.0;
  const INTERACTION_ZONE_HEIGHT: f32 = 12.0;

  commands
    .entity(entity)
    .with_children(|parent| {
      parent.spawn((
        Name::new("GameMachineInteractionZone"),
        GameMachineInteractionZone {
          game_machine_entity: entity,
        },
        Transform::from_xyz(*tile_width / 2.0, -INTERACTION_ZONE_HEIGHT / 2.0, 0.0),
        Collider::rectangle(INTERACTION_ZONE_WIDTH, INTERACTION_ZONE_HEIGHT),
        CollidingEntities::default(),
        Sprite {
          color: Color::srgb(0.7, 0.3, 0.5),
          custom_size: Some(Vec2::new(INTERACTION_ZONE_WIDTH, INTERACTION_ZONE_HEIGHT)),
          ..default()
        },
      ));
    });
}
