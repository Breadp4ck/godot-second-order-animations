use godot::{
    engine::{notify::NodeNotification, Engine},
    prelude::*,
};

use crate::second_order_systems::*;

#[derive(GodotConvert, Var, Export, PartialEq, Eq, Debug, Copy, Clone)]
#[godot(via = GString)]
pub enum InterpolationMode {
    Process,
    Physics,
}

#[derive(Debug)]
enum AnimatorError {
    NodeNotSpecified(&'static str),
}

impl std::fmt::Display for AnimatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AnimatorError::NodeNotSpecified(node) => {
                write!(f, "The {} node is not specified.", node)
            }
        }
    }
}

impl std::error::Error for AnimatorError {}

macro_rules! generate_animator {
    // This macro generates animator classes for different node properties and types.
    // Parameters:
    // $node_name: The name of the generated animator class.
    // $node_type: The type of the target node (e.g., Node3D, Node2D).
    // $system_type: The type of the second-order system used for interpolation.
    // $system_inner_type_default: The default value for the system's inner type (e.g., Vector3::ZERO).
    // $get_node_value: A closure to get the current value from the target node.
    // $set_node_value: A closure to set the new value to the target node.
    ($node_name:ident, $node_type:ty, $system_type:ty, $system_inner_type_default:expr, $get_node_value:expr, $set_node_value:expr) => {
        #[derive(GodotClass)]
        #[class(tool, base=Node)]
        struct $node_name {
            #[export]
            follower: Option<Gd<$node_type>>,
            #[export]
            target: Option<Gd<$node_type>>,

            #[export]
            #[var(get, set = set_active)]
            active: bool,
            #[export]
            #[var(get, set = set_run_in_editor)]
            run_in_editor: bool,
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
                }

                if self.active && self._validate().is_ok() {
                    self._update_initial_values();
                }
            }
            #[func]
            fn set_run_in_editor(&mut self, value: bool) {
                if self.run_in_editor != value {
                    self.run_in_editor = value;
                }

                if self.active && self._validate().is_ok() {
                    self._update_initial_values();
                }
            }
            #[func]
            fn set_interpolation_mode(&mut self, value: InterpolationMode) {
                if self.interpolation_mode != value {
                    self.interpolation_mode = value;
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

            fn _update_initial_values(&mut self) {
                self.system.update_initial_values(
                    $get_node_value(self.target.as_ref().unwrap()),
                    $get_node_value(self.follower.as_ref().unwrap()),
                    $system_inner_type_default,
                );
            }

            fn _update(&mut self, delta: f64) {
                let input = $get_node_value(self.target.as_ref().unwrap());
                let output = self.system.update(input, delta);
                $set_node_value(self.follower.as_mut().unwrap(), output);
            }

            fn _validate(&self) -> Result<(), AnimatorError> {
                if self.target.is_none() {
                    return Err(AnimatorError::NodeNotSpecified("target"));
                }
                if self.follower.is_none() {
                    return Err(AnimatorError::NodeNotSpecified("follower"));
                }

                Ok(())
            }

            fn _proceed_notification(
                &mut self,
                notification: NodeNotification,
            ) -> Result<(), AnimatorError> {
                if !self.active || (Engine::singleton().is_editor_hint() && !self.run_in_editor) {
                    return Ok(());
                }

                match (notification, self.interpolation_mode) {
                    (NodeNotification::Process, InterpolationMode::Process) => {
                        self._validate()?;

                        let delta = self.base().get_process_delta_time();
                        self._update(delta);
                    }
                    (NodeNotification::PhysicsProcess, InterpolationMode::Physics) => {
                        self._validate()?;

                        let delta = self.base().get_physics_process_delta_time();
                        self._update(delta);
                    }
                    (NodeNotification::Ready, _) => {
                        self._validate()?;
                        self.base_mut().set_process(true);
                        self._update_initial_values();
                    }
                    _ => {}
                }

                Ok(())
            }
        }

        #[godot_api]
        impl INode for $node_name {
            fn init(base: Base<Node>) -> Self {
                let (period, damping, response) = (1.0, 0.5, 2.0);
                let system = <$system_type>::new(period, damping, response);

                Self {
                    follower: None,
                    target: None,
                    active: true,
                    run_in_editor: false,
                    interpolation_mode: InterpolationMode::Physics,
                    period,
                    damping,
                    response,
                    system,
                    base,
                }
            }

            // The process and physics_process methods are used when the node has no script attached.
            // The on_notification method is used otherwise. Related to https://github.com/godot-rust/gdext/issues/111

            fn process(&mut self, delta: f64) {
                if !self.active || (Engine::singleton().is_editor_hint() && !self.run_in_editor) {
                    return;
                }

                if let Err(err) = self._validate() {
                    godot_warn!("Animator error: {}", err);
                    return;
                }

                self._update(delta);
            }

            fn physics_process(&mut self, delta: f64) {
                if !self.active || (Engine::singleton().is_editor_hint() && !self.run_in_editor) {
                    return;
                }

                if let Err(err) = self._validate() {
                    godot_warn!("Animator error: {}", err);
                    return;
                }

                self._update(delta);
            }

            fn on_notification(&mut self, notification: NodeNotification) {
                if let Err(err) = self._proceed_notification(notification) {
                    godot_warn!("Animator error: {}", err);
                }
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
