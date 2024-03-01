use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_yoetz::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(YoetzPlugin::<EnemyBehavior>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                control_player,
                (enemies_idle, enemies_detect_player).in_set(YoetzSystemSet::Suggest),
                (enemies_do_nothing, enemies_follow_player).in_set(YoetzSystemSet::Act),
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(0.025 * Vec3::ONE),
        ..Default::default()
    });

    commands.spawn((
        Player,
        SpriteBundle {
            sprite: Sprite {
                color: Color::YELLOW,
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    commands.spawn((
        Enemy,
        YoetzAdvisor::<EnemyBehavior>::new(2.0),
        SpriteBundle {
            transform: Transform::from_xyz(-5.0, 5.0, 0.0),
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn control_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) {
        direction += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction -= Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += Vec3::X;
    }

    for mut player_transform in query.iter_mut() {
        player_transform.translation += 10.0 * time.delta_seconds() * direction;
    }
}

enum EnemyBehavior {
    Idle,
    Chase {
        target_entity: Entity,
        vec_to_target: Vec3,
    },
}

impl YoetzSuggestion for EnemyBehavior {
    type Key = EnemyBehaviorKey;
    type OmniQuery = EnemyBehaviorOmniQuery;

    fn key(&self) -> Self::Key {
        match self {
            EnemyBehavior::Idle => Self::Key::Idle,
            EnemyBehavior::Chase { target_entity, .. } => Self::Key::Chase {
                target_entity: *target_entity,
            },
        }
    }

    fn remove_components(key: &Self::Key, cmd: &mut EntityCommands) {
        cmd.remove::<(EnemyBehaviorIdle, EnemyBehaviorChase)>();
        match key {
            EnemyBehaviorKey::Idle => {
                cmd.remove::<EnemyBehaviorIdle>();
            }
            EnemyBehaviorKey::Chase { .. } => {
                cmd.remove::<EnemyBehaviorChase>();
            }
        }
    }

    fn add_components(&self, cmd: &mut EntityCommands) {
        match self {
            EnemyBehavior::Idle => {
                cmd.insert(EnemyBehaviorIdle);
            }
            EnemyBehavior::Chase {
                target_entity,
                vec_to_target,
            } => {
                cmd.insert(EnemyBehaviorChase {
                    target_entity: *target_entity,
                    vec_to_target: *vec_to_target,
                });
            }
        }
    }

    fn update_into_components(
        self,
        components: &mut <Self::OmniQuery as bevy::ecs::query::WorldQuery>::Item<'_>,
    ) -> Result<(), Self> {
        match self {
            EnemyBehavior::Idle => {
                if components.idle.is_some() {
                    Ok(())
                } else {
                    Err(Self::Idle)
                }
            }
            EnemyBehavior::Chase {
                target_entity,
                vec_to_target,
            } => {
                if let Some(chase) = components.chase.as_mut() {
                    chase.vec_to_target = vec_to_target;
                    Ok(())
                } else {
                    Err(Self::Chase {
                        target_entity,
                        vec_to_target,
                    })
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum EnemyBehaviorKey {
    Idle,
    Chase { target_entity: Entity },
}

#[derive(bevy::ecs::query::QueryData)]
#[query_data(mutable)]
struct EnemyBehaviorOmniQuery {
    idle: Option<&'static mut EnemyBehaviorIdle>,
    chase: Option<&'static mut EnemyBehaviorChase>,
}

#[derive(Component)]
struct EnemyBehaviorIdle;

#[derive(Component, Debug)]
struct EnemyBehaviorChase {
    #[allow(dead_code)]
    target_entity: Entity,
    vec_to_target: Vec3,
}

fn enemies_idle(mut query: Query<&mut YoetzAdvisor<EnemyBehavior>, With<Enemy>>) {
    for mut advisor in query.iter_mut() {
        advisor.suggest(5.0, EnemyBehavior::Idle);
    }
}

fn enemies_detect_player(
    mut enemies_query: Query<(&mut YoetzAdvisor<EnemyBehavior>, &GlobalTransform), With<Enemy>>,
    player_query: Query<(Entity, &GlobalTransform), With<Player>>,
) {
    for (mut advisor, enemy_transform) in enemies_query.iter_mut() {
        let enemy_position = enemy_transform.translation();
        for (player_entity, player_transform) in player_query.iter() {
            let player_position = player_transform.translation();
            let vec_to_player = player_position - enemy_position;
            advisor.suggest(
                10.0 - vec_to_player.length(),
                EnemyBehavior::Chase {
                    target_entity: player_entity,
                    vec_to_target: vec_to_player,
                },
            );
        }
    }
}

fn enemies_do_nothing(query: Query<&EnemyBehaviorIdle>) {
    for _ in query.iter() {
        // TODO: change to random walk?
    }
}
fn enemies_follow_player(mut query: Query<(&EnemyBehaviorChase, &mut Transform)>, time: Res<Time>) {
    for (chase, mut transform) in query.iter_mut() {
        let Some(direction) = chase.vec_to_target.try_normalize() else {
            continue;
        };
        transform.translation += 5.0 * time.delta_seconds() * direction;
    }
}
