use std::panic;

use avian2d::prelude::{Collider, CollidingEntities, RigidBody};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_tiled::prelude::TiledObject;

use crate::{games::snake::SnakeGamePlugin, player::Player};

mod snake;

pub struct GamesPlugin;

impl Plugin for GamesPlugin {
  fn build(&self, app: &mut App) {
    app.register_type::<GameType>();
    app.register_type::<GameMachine>();

    app.add_observer(on_add_game_machine);

    app.add_systems(
      Update,
      (
        check_game_machine_trigger_zone_collision_with_player_system,
        interact_with_game_machine_system,
      )
        .chain(),
    );

    app.add_plugins(SnakeGamePlugin);
  }
}

#[derive(Default, Debug, Reflect)]
#[reflect(Default)]
enum GameType {
  #[default]
  Snake,
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

fn interact_with_game_machine_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  game_machine_query: Query<&GameMachine>,
  game_machine_interaction_zone_query: Query<(&GameMachineInteractionZone, &GlobalTransform)>,
  player_transform: Single<&GlobalTransform, With<Player>>,
) {
  if !keyboard_input.pressed(KeyCode::KeyE) {
    return;
  }

  for interaction_zone in game_machine_interaction_zone_query.iter() {
    let game_machine = game_machine_query
      .get(interaction_zone.0.game_machine_entity)
      .unwrap();

    println!("GameMachine interacted with game {:?}", game_machine.game);
  }
}

fn check_game_machine_trigger_zone_collision_with_player_system(
  colliding_entities_query: Query<(Entity, &CollidingEntities)>,
  player_single: Single<Entity, With<Player>>,
  mut counter: Local<usize>,
) {
  for (entity, colliding_entities) in &colliding_entities_query {
    if colliding_entities.contains(&player_single.entity()) {
      println!("Trigger zone is colliding with player {}", *counter);
      *counter += 1;
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
        Collider::rectangle(*tile_width, INTERACTION_ZONE_HEIGHT),
        CollidingEntities::default(),
        Sprite {
          color: Color::srgb(0.7, 0.3, 0.5),
          custom_size: Some(Vec2::new(*tile_width, INTERACTION_ZONE_HEIGHT)),
          ..default()
        },
      ));
    });
}
