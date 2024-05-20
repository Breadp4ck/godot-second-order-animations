use std::f32::consts::PI;

use godot::{
    builtin::{Quaternion, Vector2, Vector3},
    log::godot_print,
};

macro_rules! generate_systems_for_simple_types {
    ( $name:ident, $type:ty, $default:expr, $interpolation_step:ident ) => {
        pub struct $name {
            period: f32,
            damping: f32,
            response: f32,

            xp: $type,
            y: $type,
            yd: $type,

            k: (f32, f32, f32),
        }

        impl $name {
            pub fn new(period: f32, damping: f32, response: f32) -> Self {
                let k = Self::calculate_k(period, damping, response);

                Self {
                    period,
                    damping,
                    response,
                    xp: $default,
                    y: $default,
                    yd: $default,
                    k,
                }
            }

            pub fn update_period(&mut self, period: f32) {
                self.period = period;
                self.update_k();
            }

            pub fn update_damping(&mut self, damping: f32) {
                self.damping = damping;
                self.update_k();
            }

            pub fn update_response(&mut self, response: f32) {
                self.response = response;
                self.update_k();
            }

            pub fn update_initial_values(
                &mut self,
                previous: $type,
                current: $type,
                current_derevative: $type,
            ) {
                self.xp = previous;
                self.y = current;
                self.yd = current_derevative;
            }

            #[inline]
            fn update_k(&mut self) {
                self.k = Self::calculate_k(self.period, self.damping, self.response);
            }

            #[inline]
            fn calculate_k(period: f32, damping: f32, response: f32) -> (f32, f32, f32) {
                let (f, z, r) = (period, damping, response);

                let k0 = z / (PI * f);
                let k1 = 1. / ((2. * PI * f) * (2. * PI * f));
                let k2 = r * z / (2. * PI * f);

                (k0, k1, k2)
            }

            #[inline]
            fn interpolation_step(&mut self, x: $type, d: f32) {
                let (k1, k2, k3) = self.k;
                (self.xp, self.y, self.yd) =
                    $interpolation_step(k1, k2, k3, x, self.xp, self.y, self.yd, d);
            }

            #[inline]
            pub fn update(&mut self, input: $type, delta: f64) -> $type {
                self.interpolation_step(input, delta as f32);
                self.y
            }
        }
    };
}

macro_rules! generate_default_interpolation_step {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(
            k1: f32,
            k2: f32,
            k3: f32,
            x: $type,
            mut xp: $type,
            mut y: $type,
            mut yd: $type,
            d: f32,
        ) -> ($type, $type, $type) {
            let xd = (x - xp) / d;

            let k2_stable = f32::max(k2, 1.1 * (d * d + 0.5 * d * k1));

            xp = x;
            y += d * yd;
            yd = yd + d * (x + k3 * xd - y - k1 * yd) / k2_stable;

            (xp, y, yd)
        }
    };
}

#[inline]
fn interpolation_step_quaternion(
    k1: f32,
    k2: f32,
    k3: f32,
    mut x: Quaternion,
    mut xp: Quaternion,
    mut y: Quaternion,
    mut yd: Quaternion,
    d: f32,
) -> (Quaternion, Quaternion, Quaternion) {
    if x.dot(y) < 0.0 {
        x = -x;
    }

    // We normalized (x * xp.inverse()) and (x * y.inverse()) beacuse there is calculation error,
    // when quaternion rotations are very close. Normalization is not the fastest solution, but it works.

    let xd = (x * xp.inverse()).normalized().log() / d;
    let k2_stable = f32::max(k2, 1.1 * (d * d + 0.5 * d * k1));

    xp = x;
    y = (d * yd).to_exp() * y;
    yd += d * ((x * y.inverse()).normalized().log() + k3 * xd - k1 * yd) / k2_stable;

    (xp, y, yd)
}

generate_default_interpolation_step!(interpolation_step_vector3, Vector3);
generate_default_interpolation_step!(interpolation_step_vector2, Vector2);
generate_default_interpolation_step!(interpolation_step_float, f32);

generate_systems_for_simple_types!(
    SecondOrderSystemVector3,
    Vector3,
    Vector3::ZERO,
    interpolation_step_vector3
);
generate_systems_for_simple_types!(
    SecondOrderSystemVector2,
    Vector2,
    Vector2::ZERO,
    interpolation_step_vector2
);
generate_systems_for_simple_types!(SecondOrderSystemFloat, f32, 0.0, interpolation_step_float);
generate_systems_for_simple_types!(
    SecondOrderSystemQuaternion,
    Quaternion,
    Quaternion::default(),
    interpolation_step_quaternion
);
