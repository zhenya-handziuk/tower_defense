use crate::tiled::Loader;
use bevy::{color::Color, prelude::*, window::WindowResolution};
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(512, 512),
                        title: "Tower Defense".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TiledPlugin::default())
        .add_systems(Startup, setup_game)
        .add_systems(Startup, spawn_enemy)
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    let map_handle: Handle<TiledMapAsset> = asset_server.load("map_1.tmx");

    commands.spawn((
        TiledMap(map_handle),
        TilemapAnchor::Center,
        Visibility::default(),
    ));
}

#[derive(Component)]
struct Enemy {
    path: Vec<Vec2>,
    current: usize,
    speed: f32,
}

fn load_paths() -> Vec<Vec<Vec2>> {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/map_1.tmx")
        .expect("Failed to load map");

    let mut paths = Vec::new();

    for layer in map.layers() {
        if let tiled::LayerType::Objects(obj_layer) = layer.layer_type() {
            if layer.name == "EnemyPath" {
                for obj in obj_layer.objects() {
                    if let tiled::ObjectShape::Polyline { points } = &obj.shape {
                        let path = points
                            .iter()
                            .map(|point| Vec2::new(point.0 as f32, point.1 as f32))
                            .collect();
                        paths.push(path);
                    }
                }
            }
        }
    }

    paths
}

fn spawn_enemy(mut commands: Commands) {
    let paths = load_paths();
    let path = paths.first().unwrap().clone();

    println!("Path: {:?}", path);
    println!("First point: {:?}", path[0]);

    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(path[0].x, path[0].y, 0.0)),
        Enemy {
            path,
            current: 0,
            speed: 100.0,
        },
    ));
}
