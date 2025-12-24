use std::env;

use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use bevy_ecs_tiled::{
  prelude::*,
  tiled::{TiledPlugin, TiledPluginConfig},
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
  games::GamesPlugin,
  player::{Player, PlayerPlugin, SpawnPlayerMessage},
  tilemap::{SpawnTilemapMessage, TilemapPlugin},
};

const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

pub fn run_game() {
  App::new()
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .register_type::<ExitFromGameTriggerZone>()
    .add_observer(on_add_exit_from_game_trigger)
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugins((
      PhysicsPlugins::default().with_length_unit(20.0),
      avian2d::debug_render::PhysicsDebugPlugin,
    ))
    .add_plugins(EguiPlugin::default())
    .add_plugins(WorldInspectorPlugin::new())
    .add_plugins(TiledPlugin(TiledPluginConfig {
      tiled_types_export_file: Some(
        env::current_dir()
          .unwrap()
          .join("properties.json"),
      ),
      tiled_types_filter: TiledFilter::from(
        regex::RegexSet::new([
          r"game_club::*",
          r"^bevy_sprite::text2d::Text2d$",
          r"^bevy_text::text::TextColor$",
          r"^bevy_ecs::name::Name$",
        ])
        .unwrap(),
      ),
    }))
    .add_plugins(TilemapPlugin)
    .add_plugins(PlayerPlugin)
    .add_plugins(GamesPlugin)
    .add_systems(Startup, setup)
    .add_systems(PostUpdate, move_lobby_camera_to_player)
    .run();
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct ExitFromGameTriggerZone {}

#[derive(Component)]
struct LobbyCamera;

fn on_add_exit_from_game_trigger(
  add_game_machine: On<Add, ExitFromGameTriggerZone>,
  query: Query<(&ExitFromGameTriggerZone, &GlobalTransform)>,
) {
  let entity = add_game_machine.event().entity;

  let Ok((trigger, global_transform)) = query.get(entity) else {
    return;
  };

  info!(
    "New Trigger [{:?} @ {:?}]",
    trigger,
    global_transform.translation(),
  );
}

fn move_lobby_camera_to_player(
  mut camera: Single<&mut Transform, With<LobbyCamera>>,
  player: Single<&Transform, (With<Player>, Without<LobbyCamera>)>,
) {
  camera.translation = player.translation;
}

fn setup(
  mut commands: Commands,
  mut spawn_tilemap_messages: MessageWriter<SpawnTilemapMessage>,
  mut spawn_player_messages: MessageWriter<SpawnPlayerMessage>,
) {
  let mut projection = OrthographicProjection::default_2d();
  projection.scale = 0.3;
  projection.scaling_mode = bevy::camera::ScalingMode::AutoMin {
    min_width: 1200.0,
    min_height: 800.0,
  };

  commands.spawn((
    LobbyCamera,
    Camera2d,
    Camera {
      ..Default::default()
    },
    Projection::Orthographic(projection),
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
  ));

  spawn_tilemap_messages.write(SpawnTilemapMessage);
  spawn_player_messages.write(SpawnPlayerMessage {
    position: Vec2::new(0.0, 0.0),
  });
}
