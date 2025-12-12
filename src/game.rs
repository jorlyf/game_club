use bevy::prelude::*;
use bevy_ecs_tiled::tiled::TiledPlugin;

use crate::{
  machines::snake::snake::spawn_snake_machine,
  player::{PlayerPlugin, spawn_player},
  tilemap::spawn_map,
};

const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

pub fn run_game() {
  App::new()
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugins(PlayerPlugin)
    .add_plugins(TiledPlugin::default())
    .add_systems(Startup, setup)
    .run();
}

fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  let mut projection = OrthographicProjection::default_2d();
  projection.scale = 0.15;
  projection.scaling_mode = bevy::camera::ScalingMode::AutoMin {
    min_width: 1080f32,
    min_height: 1080f32,
  };

  commands.spawn((
    Camera2d,
    Camera {
      ..Default::default()
    },
    Projection::Orthographic(projection),
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
  ));

  spawn_map(&mut commands, &asset_server);

  spawn_player(
    &mut commands,
    &asset_server,
    &mut texture_atlas_layouts,
    Vec2::new(10f32, 10f32),
  );

  spawn_snake_machine(&mut commands, &asset_server, Vec2::new(12f32, 10f32));
}
