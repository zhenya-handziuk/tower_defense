use crate::tiled::{LayerType, ObjectShape};
use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tower Defense".to_string(),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: WindowResolution::new(512, 512),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TiledPlugin::default())
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    let map_handle: Handle<TiledMapAsset> = asset_server.load("map_1.tmx");

    commands
        .spawn((
            TiledMap(map_handle),
            TilemapAnchor::Center,
            Visibility::default(),
        ))
        .observe(on_map_created);
}

fn on_map_created(trigger: On<TiledEvent<MapCreated>>, map_assets: Res<Assets<TiledMapAsset>>) {
    // Get the map asset from the event
    if let Some(map) = trigger.event().get_map(&map_assets) {
        // Iterate over all layers and filter for object layers
        for layer in map.layers() {
            if let LayerType::Objects(object_layer) = layer.layer_type() {
                println!("Object Layer: {}", layer.name);

                // Iterate over objects in the layer
                for object in object_layer.objects() {
                    println!("  Object: {} (id: {})", object.name, object.id());

                    // Access object shape (for polylines, etc.)
                    match &object.shape {
                        ObjectShape::Polyline { points } => {
                            println!("    Polyline with {} points", points.len());
                            for (i, point) in points.iter().enumerate() {
                                println!("      Point {}: ({}, {})", i, point.0, point.1);
                            }
                        }
                        ObjectShape::Polygon { points } => {
                            println!("    Polygon with {} points", points.len());
                        }
                        ObjectShape::Rect { width, height } => {
                            println!("    Rectangle: {}x{}", width, height);
                        }
                        ObjectShape::Point(x, y) => {
                            println!("    Point: ({}, {})", x, y);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
