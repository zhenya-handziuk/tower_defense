use crate::tiled::Loader;
use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(800, 600),
                        title: "Tower Defense".to_string(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TiledPlugin::default())
        .add_systems(
            Startup,
            (setup_game, load_enemy_paths, spawn_tower_spots).chain(),
        )
        .add_systems(
            Update,
            (
                spawn_enemy,
                animate_sprite,
                move_enemy,
                place_tower,
                update_camera_scale,
            ),
        )
        .run();
}

#[derive(Component)]
struct CameraConfig {
    map_width: f32,
    map_height: f32,
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d::default(),
        CameraConfig {
            map_width: 480.0,
            map_height: 320.0,
        },
    ));

    let map_handle: Handle<TiledMapAsset> = asset_server.load("map_1_with_point.tmx");

    commands.spawn((
        TiledMap(map_handle),
        TilemapAnchor::Center,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
    ));
}

fn update_camera_scale(
    mut camera_query: Query<(&mut Projection, &CameraConfig)>,
    window: Query<&Window>,
) {
    let window = window.single().expect("REASON");

    for (mut projection, config) in &mut camera_query {
        let window_width = window.width();
        let window_height = window.height();
        let window_aspect = window_width / window_height;
        let map_aspect = config.map_width / config.map_height;

        if let Projection::Orthographic(ortho) = projection.as_mut() {
            if window_aspect > map_aspect {
                ortho.scale = config.map_height / window_height;
            } else {
                ortho.scale = config.map_width / window_width;
            }
        }
    }
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
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn load_enemy_paths(mut commands: Commands) {
    let (paths, tower_spots) = load_paths();
    commands.insert_resource(EnemyPaths { paths });
    commands.insert_resource(TowerSpots {
        points: tower_spots,
    });
    commands.insert_resource(EnemySpawner { can_spawn: true });
}

fn load_paths() -> (Vec<Vec2>, Vec<Vec2>) {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/map_1_with_point.tmx")
        .expect("Failed to load map");

    let mut paths = Vec::new();
    let mut tower_spots = Vec::new();

    let map_width = map.width as f32 * map.tile_width as f32;
    let map_height = map.height as f32 * map.tile_height as f32;

    for layer in map.layers() {
        if let tiled::LayerType::Objects(obj_layer) = layer.layer_type() {
            if layer.name == "EnemyPath" {
                for obj in obj_layer.objects() {
                    let base_x = obj.x;
                    let base_y = obj.y;
                    if let tiled::ObjectShape::Polyline { points } = &obj.shape {
                        points.iter().for_each(|point| {
                            let x = (base_x + point.0) as f32;
                            let y = (base_y + point.1) as f32;
                            paths.push(Vec2::new(x - map_width / 2.0, -(y - map_height / 2.0)));
                        });
                    }
                }
            }

            if layer.name == "Tower" {
                for obj in obj_layer.objects() {
                    if let tiled::ObjectShape::Point(pos_x, pos_y) = &obj.shape {
                        tower_spots.push(Vec2::new(
                            pos_x - map_width / 2.0,
                            -(pos_y - map_height / 2.0),
                        ));
                    }
                }
            }
        }
    }

    (paths, tower_spots)
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

// TOWERS
//
#[derive(Resource)]
struct TowerSpots {
    points: Vec<Vec2>,
}

#[derive(Component)]
struct TowerSpot {
    occupied: bool,
}

#[derive(Component)]
struct Tower;

fn spawn_tower_spots(
    mut commands: Commands,
    tower_spots: Res<TowerSpots>,
    asset_server: Res<AssetServer>,
) {
    let scale = 0.6;
    for point in &tower_spots.points {
        let tower_texture = asset_server.load("tower/1.png");
        commands.spawn((
            Sprite::from_image(tower_texture),
            Transform::from_xyz(point.x, point.y + (34.0 * scale), 0.)
                .with_scale(Vec3::splat(scale)),
            GlobalTransform::default(),
            TowerSpot { occupied: false },
        ));
    }
}

fn cursor_world_position(window: &Window, camera: (&Camera, &GlobalTransform)) -> Option<Vec2> {
    let (camera, camera_transform) = camera;
    let cursor = window.cursor_position();
    camera.viewport_to_world_2d(camera_transform, cursor?).ok()
}

// TODO доробити встановлення башти,
fn place_tower(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    mut spots: Query<(Entity, &Transform, &mut TowerSpot)>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    println!("Pressed another BUTTON");

    let window = windows.single();
    let camera = camera_q.single();

    let Some(cursor_world) =
        cursor_world_position(window.expect("REASON"), camera.expect("REASON"))
    else {
        println!("RETURN HERE");
        return;
    };

    for (_, tranform, mut spot) in spots.iter_mut() {
        println!("spot {:?}", spot.occupied);
        if spot.occupied {
            continue;
        }

        let position = tranform.translation.truncate();
        println!("position.distance {:?}", position.distance(cursor_world));

        if position.distance(cursor_world) < 30. {
            spot.occupied = true;
            let tower_texture = asset_server.load("tower/1.png");
            println!("position.x : {}, position.y : {}", position.x, position.y);

            commands.spawn((
                Sprite::from_image(tower_texture),
                Transform::from_xyz(position.x, position.y, 20.),
                Tower,
            ));
        }
    }
}
