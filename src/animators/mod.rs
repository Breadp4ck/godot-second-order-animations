use godot::prelude::*;

use crate::second_order_systems::*;

#[derive(GodotConvert, Var, Export, PartialEq, Eq, Debug)]
#[godot(via = GString)]
pub enum InterpolationMode {
    Process,
    Physics,
}

macro_rules! generate_animator {
    ($node_name:ident, $node_type:ty, $system_type:ty, $system_inner_type_default:expr, $get_node_value:expr, $set_node_value:expr) => {
        #[derive(GodotClass)]
        #[class(base=Node)]
        struct $node_name {
            #[export]
            depend: Option<Gd<$node_type>>,
            #[export]
            target: Option<Gd<$node_type>>,

            #[export]
            #[var(get, set = set_active)]
            active: bool,
            #[export]
            #[var(get, set = set_interpolation_mode)]
            interpolation_mode: InterpolationMode,

            #[export]
            #[var(get, set = set_period)]
            period: f32,
            #[export]
            #[var(get, set = set_damping)]
            damping: f32,
            #[export]
            #[var(get, set = set_response)]
            response: f32,

            system: $system_type,

            base: Base<Node>,
        }

        #[godot_api]
        impl $node_name {
            #[func]
            fn set_active(&mut self, value: bool) {
                if self.active != value {
                    self.active = value;
                    self._update_interpolation_process();
                }
            }
            #[func]
            fn set_interpolation_mode(&mut self, value: InterpolationMode) {
                if self.interpolation_mode != value {
                    self.interpolation_mode = value;
                    self._update_interpolation_process();
                }
            }
            #[func]
            fn set_period(&mut self, value: f32) {
                self.period = value;
                self.system.update_period(self.period);
            }
            #[func]
            fn set_damping(&mut self, value: f32) {
                self.damping = value;
                self.system.update_damping(self.damping);
            }
            #[func]
            fn set_response(&mut self, value: f32) {
                self.response = value;
                self.system.update_response(self.response);
            }

            fn _update_interpolation_process(&mut self) {
                let active = self.active;
                match self.interpolation_mode {
                    InterpolationMode::Process => {
                        self.base_mut().set_process(active);
                        self.base_mut().set_physics_process(false);
                    }
                    InterpolationMode::Physics => {
                        self.base_mut().set_process(false);
                        self.base_mut().set_physics_process(active);
                    }
                }
            }

            #[inline]
            fn _update(&mut self, delta: f64) {
                let input = $get_node_value(self.target.as_ref().unwrap());
                let output = self.system.update(input, delta);
                $set_node_value(self.depend.as_mut().unwrap(), output);
            }
        }

        #[godot_api]
        impl INode for $node_name {
            fn init(base: Base<Node>) -> Self {
                let (period, damping, response) = (1.0, 0.5, 2.0);
                let system = <$system_type>::new(period, damping, response);

                Self {
                    depend: None,
                    target: None,
                    active: false,
                    interpolation_mode: InterpolationMode::Physics,
                    period,
                    damping,
                    response,
                    system,
                    base,
                }
            }

            fn ready(&mut self) {
                self.system.update_initial_values(
                    $get_node_value(self.target.as_ref().unwrap()),
                    $get_node_value(self.depend.as_ref().unwrap()),
                    $system_inner_type_default,
                );

                self._update_interpolation_process();
            }

            fn process(&mut self, delta: f64) {
                self._update(delta);
            }

            fn physics_process(&mut self, delta: f64) {
                self._update(delta);
            }
        }
    };
}

generate_animator!(
    AnimatorPosition3D,
    Node3D,
    SecondOrderSystemVector3,
    Vector3::ZERO,
    |node: &Gd<Node3D>| { node.get_position() },
    |node: &mut Gd<Node3D>, value: Vector3| { node.set_position(value) }
);

generate_animator!(
    AnimatorRotation3D,
    Node3D,
    SecondOrderSystemQuaternion,
    Quaternion::default(),
    |node: &Gd<Node3D>| { node.get_quaternion() },
    |node: &mut Gd<Node3D>, value: Quaternion| { node.set_quaternion(value) }
);

generate_animator!(
    AnimatorScale3D,
    Node3D,
    SecondOrderSystemVector3,
    Vector3::ZERO,
    |node: &Gd<Node3D>| { node.get_scale() },
    |node: &mut Gd<Node3D>, value: Vector3| { node.set_scale(value) }
);

generate_animator!(
    AnimatorPosition2D,
    Node2D,
    SecondOrderSystemVector2,
    Vector2::ZERO,
    |node: &Gd<Node2D>| { node.get_position() },
    |node: &mut Gd<Node2D>, value: Vector2| { node.set_position(value) }
);

generate_animator!(
    AnimatorRotation2D,
    Node2D,
    SecondOrderSystemFloat,
    0.0,
    |node: &Gd<Node2D>| { node.get_rotation() },
    |node: &mut Gd<Node2D>, value: f32| { node.set_rotation(value) }
);

generate_animator!(
    AnimatorScale2D,
    Node2D,
    SecondOrderSystemVector2,
    Vector2::ZERO,
    |node: &Gd<Node2D>| { node.get_scale() },
    |node: &mut Gd<Node2D>, value: Vector2| { node.set_scale(value) }
);

generate_animator!(
    AnimatorSkew2D,
    Node2D,
    SecondOrderSystemFloat,
    0.0,
    |node: &Gd<Node2D>| { node.get_skew() },
    |node: &mut Gd<Node2D>, value: f32| { node.set_skew(value) }
);
