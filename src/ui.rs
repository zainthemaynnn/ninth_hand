use bevy::prelude::*;
use noisy_bevy::simplex_noise_2d;

pub struct NineUiPlugin;

impl Plugin for NineUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, do_rumble);
    }
}

#[derive(Component)]
#[require(UiTransform)]
pub struct UiRumble {
    pub center: Val2,
    pub strength: Val2,
    pub speed: f32,
    pub t: f32,
}

impl Default for UiRumble {
	fn default() -> Self {
	    Self {
	    	center: Val2::ZERO,
	    	strength: Val2::ZERO,
	    	speed: 1.0,
	    	t: 0.0,
	    }
	}
}

pub fn do_rumble(mut rumble_query: Query<(&mut UiTransform, &mut UiRumble)>, time: Res<Time>) {
    for (mut tf, mut rumble) in rumble_query {
    	rumble.t += time.delta_secs() * rumble.speed;

        let noise_x = rumble.strength.x * simplex_noise_2d(Vec2::new(rumble.t, 0.0));
        let noise_y = rumble.strength.y * simplex_noise_2d(Vec2::new(rumble.t, 100.0));

        tf.translation = rumble.center.try_add(Val2::new(noise_x, noise_y)).unwrap();
    }
}
