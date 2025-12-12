use bevy::prelude::*;

pub fn spawn_snake_machine(
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  position: Vec2,
) {
  // commands
  //   .spawn(Sprite::from_image(
  //     asset_server.load("machines/snake/machine.gif"),
  //   ))
  //   .insert(Transform {
  //     translation: position.extend(1.0),
  //     ..Default::default()
  //   });
}
