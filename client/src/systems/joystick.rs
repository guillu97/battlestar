use bevy::prelude::*;
use crate::components::{Joystick, JoystickKnob, JoystickBundle, JoystickKnobBundle};

// Détecte si on est sur mobile
fn is_mobile() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        window()
            .and_then(|w| w.navigator().user_agent().ok())
            .map(|ua| {
                ua.contains("Mobile")
                    || ua.contains("Android")
                    || ua.contains("iPhone")
                    || ua.contains("iPad")
            })
            .unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn setup_joystick(mut commands: Commands) {
    if !is_mobile() {
        return;
    }

    // Cercle externe du joystick (fond)
    commands
        .spawn(JoystickBundle {
            joystick: Joystick {
                input: Vec2::ZERO,
            },
            node: Node {
                position_type: PositionType::Absolute,
                left: Val::Px(30.0),
                bottom: Val::Px(30.0),
                width: Val::Px(120.0),
                height: Val::Px(120.0),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.6)),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        })
        .with_children(|parent| {
            parent.spawn(JoystickKnobBundle {
                knob: JoystickKnob,
                node: Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(30.0),
                    top: Val::Px(30.0),
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
            });
        });
}

pub fn update_joystick(
    mut joysticks: Query<(&mut Joystick, &GlobalTransform), Without<JoystickKnob>>,
    touches: Res<Touches>,
    mut knob_query: Query<&mut Node, With<JoystickKnob>>,
    windows: Query<&Window>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };

    for (mut joystick, global_transform) in &mut joysticks {
        // Position du joystick à l'écran (coordonnées UI)
        let joystick_pos = global_transform.translation().truncate();
        let joystick_center = joystick_pos + Vec2::new(60.0, 60.0); // Centre du joystick (120x120 / 2)

        let mut new_input = Vec2::ZERO;

        // Vérifier les touches
        for touch in touches.iter() {
            let touch_pos = touch.position();
            // Inverser Y car les coordonnées UI ont Y qui augmente vers le bas
            let window_height: f32 = window.height();
            let touch_screen_pos = Vec2::new(touch_pos.x, window_height - touch_pos.y);
            let offset = touch_screen_pos - joystick_center;

            // Vérifier si le toucher est dans le joystick
            if offset.length() < 60.0 {
                let normalized = offset.normalize_or_zero();
                let distance = (offset.length() / 50.0).min(1.0);
                new_input = normalized * distance;
            }
        }

        joystick.input = new_input;

        // Mettre à jour la position du knob
        for mut knob_node in knob_query.iter_mut() {
            let knob_offset = new_input * 25.0; // Rayon max de déplacement
            knob_node.left = Val::Px(30.0 + knob_offset.x);
            knob_node.top = Val::Px(30.0 - knob_offset.y); // Inverser Y pour UI
        }
    }
}
