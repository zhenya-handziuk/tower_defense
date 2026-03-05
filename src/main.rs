use crate::tiled::Loader;
use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(480, 320),
                        title: "Tower Defense".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TiledPlugin::default())
        .add_systems(Startup, (setup_game, load_enemy_paths))
        .add_systems(Update, (spawn_enemy, animate_sprite, move_enemy))
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    let map_handle: Handle<TiledMapAsset> = asset_server.load("map_1.tmx");

    commands.spawn((
        TiledMap(map_handle),
        TilemapAnchor::Center,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
    ));
}

#[derive(Component)]
struct Enemy {
    current_point: usize,
    speed: f32,
}

#[derive(Resource)]
struct EnemyPaths {
    paths: Vec<Vec2>,
}

#[derive(Resource)]
struct EnemySpawner {
    can_spawn: bool,
    // point_count: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn load_enemy_paths(mut commands: Commands) {
    let paths = load_paths();
    commands.insert_resource(EnemyPaths { paths });
    commands.insert_resource(EnemySpawner {
        can_spawn: true,
        // point_count: 1,
    });
}

fn load_paths() -> Vec<Vec2> {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/map_1.tmx")
        .expect("Failed to load map");

    let mut paths = Vec::new();

    let map_width = map.width as f32 * map.tile_width as f32;
    let map_height = map.height as f32 * map.tile_height as f32;

    for layer in map.layers() {
        if let tiled::LayerType::Objects(obj_layer) = layer.layer_type() {
            if layer.name == "EnemyPath" {
                for obj in obj_layer.objects() {
                    let base_x = obj.x;
                    let base_y = obj.y;
                    if let tiled::ObjectShape::Polyline { points } = &obj.shape {
                        paths = points
                            .iter()
                            .map(|point| {
                                let x = (base_x + point.0) as f32;
                                let y = (base_y + point.1) as f32;
                                Vec2::new(x - map_width / 2.0, -(y - map_height / 2.0))
                            })
                            .collect();
                        return paths;
                    }
                }
            }
        }
    }

    paths
}

fn spawn_enemy(
    mut commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    enemy_paths: Res<EnemyPaths>,
    asset_server: Res<AssetServer>,
) {
    if !enemy_spawner.can_spawn {
        return;
    }

    let start_point = enemy_paths.paths[0];
    let texture: Handle<Image> = asset_server.load("Pink_Monster_Walk_6.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 6, 1, None, None);
    let atlas = texture_atlas_layouts.add(layout);

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: atlas,
                index: 0,
            },
        ),
        AnimationTimer(Timer::from_seconds(0.12, TimerMode::Repeating)),
        Transform::from_translation(Vec3::new(start_point.x, start_point.y, 0.0)),
        Enemy {
            current_point: 0,
            speed: 50.0,
        },
    ));

    enemy_spawner.can_spawn = false;
}

fn move_enemy(
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy)>,
    mut enemy_spawn: ResMut<EnemySpawner>,
    enemy_paths: Res<EnemyPaths>,
    time: Res<Time>,
) {
    for (_, mut transform, mut enemy) in enemies.iter_mut() {
        let points = &enemy_paths.paths;
        if points.is_empty() {
            continue;
        }
        let next_index = (enemy.current_point + 1).min(points.len() - 1);
        let next_point = points[next_index];

        let direction = (next_point - Vec2::new(transform.translation.x, transform.translation.y))
            .normalize_or_zero();
        let distance = enemy.speed * time.delta_secs();
        let delta = next_point - Vec2::new(transform.translation.x, transform.translation.y);

        if delta.length() < distance {
            transform.translation.x = next_point.x;
            transform.translation.y = next_point.y;
            enemy.current_point += 1;

            if enemy.current_point == 1 {
                enemy_spawn.can_spawn = true;
            }

            if enemy.current_point >= points.len() - 1 {
                continue;
            }
        } else {
            transform.translation.x += direction.x * distance;
            transform.translation.y += direction.y * distance;
        }
    }
}

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut Sprite, &mut AnimationTimer)>) {
    for (mut sprite, mut timer) in &mut query {
        timer.tick(time.delta());

        if !timer.just_finished() {
            continue;
        }

        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        atlas.index = (atlas.index + 1) % 6;
    }
}
