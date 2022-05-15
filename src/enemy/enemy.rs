use std::f32::consts::PI;
use bevy::core::FixedTimestep;
use bevy::ecs::schedule::ShouldRun;
use rand::{Rng, thread_rng};
use crate::{App, Commands, default, Enemy, ENEMY_LASER_SIZE, ENEMY_MAX, ENEMY_SIZE, EnemyCount, FromEnemy, GameTextures, Laser, Movable, Plugin, Quat, Query, Res, ResMut, SPRITE_SCALE, SpriteBundle, SpriteSize, SystemSet, Time, TIME_STEP, Transform, Vec3, Velocity, WinSize, With};
use crate::enemy::formation::{Formation, FormationMaker};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FormationMaker::default())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.))
                    .with_system(enemy_spawn_system),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(enemy_fire_criteria)
                    .with_system(enemy_fire_system),
            )
            .add_system(enemy_move_system);
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    mut formation_maker: ResMut<FormationMaker>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {
    if enemy_count.0 < ENEMY_MAX {
        let formation = formation_maker.make(&win_size);
        let (x, y) = formation.data.start;

        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.enemy.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 10.),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..default()
                },
                ..default()
            })
            .insert(Enemy)
            .insert(formation)
            .insert(SpriteSize::from(ENEMY_SIZE));

        enemy_count.0 += 1;
    }
}

fn enemy_fire_criteria() -> ShouldRun {
    if thread_rng().gen_bool(1. / 60.) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn enemy_fire_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for transform in enemy_query.iter() {
        let (x, y) = (transform.translation.x, transform.translation.y);
        // spawn enemy laser sprite
        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.enemy_laser.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y - 15., 0.),
                    rotation: Quat::from_rotation_x(PI),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..default()
                },
                ..default()
            })
            .insert(Laser)
            .insert(SpriteSize::from(ENEMY_LASER_SIZE))
            .insert(FromEnemy)
            .insert(Movable { auto_despawn: true })
            .insert(Velocity { x: 0., y: -1. });
    }
}

fn enemy_move_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Formation), With<Enemy>>,
) {
    let now = time.seconds_since_startup() as f32;

    for (mut transform, mut formation) in query.iter_mut() {
        // current position
        let (x_original, y_original) = (transform.translation.x, transform.translation.y);

        // max distance
        let max_distance = TIME_STEP * formation.data.speed;

        // fixtures
        let dir: f32 = if formation.data.start.0 < 0. {-1.} else {-1.};
        let (x_pivot, y_pivot) = formation.data.pivot;
        let (x_radius, y_radius) = formation.data.radius;

        //compute next angle (based on time for now)
        let angle = formation.angle + dir * formation.data.speed * TIME_STEP / (x_radius.min(y_radius) * PI / 2.);

        //compute target x/y
        let x_dst = x_radius * angle.cos() + x_pivot;
        let y_dst = y_radius * angle.sin() + y_pivot;

        //compute distance
        let dx = x_original - x_dst;
        let dy = y_original - y_dst;
        let distance = (dx * dx + dy * dy).sqrt();
        let distance_ratio = if distance != 0. {
            max_distance / distance
        } else {
            0.
        };

        //compute final x/y
        let x = x_original - dx * distance_ratio;
        let x = if dx > 0. { x.max(x_dst) } else { x.min(x_dst) };
        let y = y_original - dy * distance_ratio;
        let y = if dy > 0. { y.max(y_dst) } else { y.min(y_dst) };

        // start rotating the formation angle only when sprite is on or close to ellipse
        if distance < max_distance * formation.data.speed / 20. {
            formation.angle = angle;
        }

        let translation = &mut transform.translation;
        (translation.x, translation.y) = (x, y);
    }
}
