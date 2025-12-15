use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<SpawnTilemapMessage>()
      .add_message::<DespawnTilemapMessage>()
      .add_systems(Update, spawn_map)
      .add_systems(Update, despawn_map);
  }
}

#[derive(Message)]
pub struct SpawnTilemapMessage;

#[derive(Message)]
pub struct DespawnTilemapMessage;

fn spawn_map(
  mut commands: Commands,
  spawn_tilemap_messages: MessageReader<SpawnTilemapMessage>,
  asset_server: Res<AssetServer>,
) {
  if spawn_tilemap_messages.is_empty() {
    return;
  }

  commands
    .spawn((
      TiledMap(asset_server.load("maps/map.tmx")),
      TilemapAnchor::Center,
    ))
    .observe(
      |map_created: On<TiledEvent<MapCreated>>,
       assets: Res<Assets<TiledMapAsset>>,
       query: Query<(&Name, &TiledMapStorage), With<TiledMap>>| {
        let Ok((name, storage)) = query.get(map_created.event().origin) else {
          return;
        };
        info!("=> Observer TiledMapCreated was triggered for map '{name}'");

        let Some(map) = map_created.event().get_map(&assets) else {
          return;
        };
        info!("Loaded map: {:?}", map);

        for (id, entity) in storage.objects() {
          info!(
            "(map) Object ID {:?} was spawned as entity {:?}",
            id, entity
          );
        }
      },
    );
}

fn despawn_map(
  mut commands: Commands,
  despawn_tilemap_messages: MessageReader<DespawnTilemapMessage>,
  query: Query<Entity, With<TiledMap>>,
) {
  if despawn_tilemap_messages.is_empty() {
    return;
  }

  for entity in query.iter() {
    commands.entity(entity).despawn();
  }
}
