use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
  fn build(&self, app: &mut App) {}
}

pub fn spawn_map(commands: &mut Commands, asset_server: &Res<AssetServer>) {
  commands
    // Load a map and set its anchor point to the
    // center instead of the default bottom-left
    .spawn((
      TiledMap(asset_server.load("maps/map.tmx")),
      TilemapAnchor::Center,
    ))
    // Add an "in-line" observer to detect when
    // the map has finished loading
    .observe(
      |map_created: On<TiledEvent<MapCreated>>,
       assets: Res<Assets<TiledMapAsset>>,
       query: Query<(&Name, &TiledMapStorage), With<TiledMap>>| {
        // We can access the map components via a regular query
        let Ok((name, storage)) = query.get(map_created.event().origin) else {
          return;
        };
        info!("=> Observer TiledMapCreated was triggered for map '{name}'");

        // Or directly the underneath raw tiled::Map data
        let Some(map) = map_created.event().get_map(&assets) else {
          return;
        };
        info!("Loaded map: {:?}", map);

        // Additionally, we can access Tiled items using the TiledMapStorage
        // component: we can retrieve Tiled items entity and access
        // their own components with another query (not shown here).
        // This can be useful if you want for instance to create a resource
        // based upon tiles or objects data but make it available only when
        // the map is actually spawned.
        for (id, entity) in storage.objects() {
          info!(
            "(map) Object ID {:?} was spawned as entity {:?}",
            id, entity
          );
        }
      },
    );
}
