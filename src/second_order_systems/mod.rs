use std::f32::consts::PI;

use godot::builtin::{Vector2, Vector3};

macro_rules! generate_systems_for_simple_types {
    ( $name:ident, $type:ty, $default:expr ) => {
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
                let xd = (x - self.xp) / d;
                let (k1, k2, k3) = self.k;

                let k2_stable = f32::max(k2, 1.1 * (d * d + 0.5 * d * k1));

                self.xp = x;
                self.y += d * self.yd;
                self.yd = self.yd + d * (x + k3 * xd - self.y - k1 * self.yd) / k2_stable;
            }

            #[inline]
            pub fn update(&mut self, input: $type, delta: f64) -> $type {
                self.interpolation_step(input, delta as f32);
                self.y
            }
        }
    };
}

generate_systems_for_simple_types!(SecondOrderSystemVector3, Vector3, Vector3::ZERO);
generate_systems_for_simple_types!(SecondOrderSystemVector2, Vector2, Vector2::ZERO);
generate_systems_for_simple_types!(SecondOrderSystemFloat, f32, 0.0);
