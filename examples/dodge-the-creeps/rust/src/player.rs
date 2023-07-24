use godot::engine::{
    AnimatedSprite2D, Area2D, Area2DVirtual, CollisionShape2D, Engine, PhysicsBody2D,
};
use godot::prelude::*;

#[derive(Copy, Clone)]
#[repr(i32)]
pub enum Foo {
    Qux = 0,
    Bar = 1,
    Baz = 2,
}

impl Foo {
    pub const HINT_DESC: &str = "2/2:Qux:0,Bar:1,Baz:2";
}

impl godot::bind::property::Property for Foo {
    type Intermediate = i32;

    fn get_property(&self) -> Self::Intermediate {
        (*self) as Self::Intermediate
    }

    fn set_property(&mut self, value: Self::Intermediate) {
        *self = Self::try_from(value).expect("enum value should be in range");
    }
}

impl TryFrom<i32> for Foo {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Qux),
            1 => Ok(Self::Bar),
            2 => Ok(Self::Baz),
            _ => Err(())
        }
    }
}

impl godot::bind::property::Export for Foo {
    fn default_export_info() -> godot::bind::property::ExportInfo {
        godot::bind::property::ExportInfo {
            hint: godot::engine::global::PropertyHint::PROPERTY_HINT_ENUM,
            hint_string: "Qux:0,Bar:1,Baz:2".into(),
        }
    }
}

impl godot::bind::property::TypeStringHint for Foo {
    fn type_string() -> String {
        Self::HINT_DESC.to_owned()
    }
}

impl godot::builtin::meta::VariantMetadata for Foo {
    fn variant_type() -> godot::builtin::VariantType {
        godot::builtin::VariantType::Int
    }

    fn param_metadata() -> godot::sys::GDExtensionClassMethodArgumentMetadata {
        godot::sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_INT32
    }
}

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Player {
    #[export]
    foo: Foo,

    #[export]
    foos: Array<Foo>,

    speed: real,
    screen_size: Vector2,

    #[base]
    base: Base<Area2D>,
}

#[godot_api]
impl Player {
    #[signal]
    fn hit();

    #[func]
    fn on_player_body_entered(&mut self, _body: Gd<PhysicsBody2D>) {
        self.base.hide();
        self.base.emit_signal("hit".into(), &[]);

        let mut collision_shape = self
            .base
            .get_node_as::<CollisionShape2D>("CollisionShape2D");

        collision_shape.set_deferred("disabled".into(), true.to_variant());
    }

    #[func]
    pub fn start(&mut self, pos: Vector2) {
        self.base.set_global_position(pos);
        self.base.show();

        let mut collision_shape = self
            .base
            .get_node_as::<CollisionShape2D>("CollisionShape2D");

        collision_shape.set_disabled(false);
    }
}

#[godot_api]
impl Area2DVirtual for Player {
    fn init(base: Base<Area2D>) -> Self {
        Player {
            foo: Foo::Bar,
            foos: Array::new(),
            speed: 400.0,
            screen_size: Vector2::new(0.0, 0.0),
            base,
        }
    }

    fn ready(&mut self) {
        let viewport = self.base.get_viewport_rect();
        self.screen_size = viewport.size;
        self.base.hide();
    }

    fn process(&mut self, delta: f64) {
        // Don't process if running in editor. This part should be removed when
        // issue is resolved: https://github.com/godot-rust/gdext/issues/70
        if Engine::singleton().is_editor_hint() {
            return;
        }

        let mut animated_sprite = self
            .base
            .get_node_as::<AnimatedSprite2D>("AnimatedSprite2D");

        let mut velocity = Vector2::new(0.0, 0.0);

        // Note: exact=false by default, in Rust we have to provide it explicitly
        let input = Input::singleton();
        if input.is_action_pressed("move_right".into()) {
            velocity += Vector2::RIGHT;
        }
        if input.is_action_pressed("move_left".into()) {
            velocity += Vector2::LEFT;
        }
        if input.is_action_pressed("move_down".into()) {
            velocity += Vector2::DOWN;
        }
        if input.is_action_pressed("move_up".into()) {
            velocity += Vector2::UP;
        }

        if velocity.length() > 0.0 {
            velocity = velocity.normalized() * self.speed;

            let animation;

            if velocity.x != 0.0 {
                animation = "right";

                animated_sprite.set_flip_v(false);
                animated_sprite.set_flip_h(velocity.x < 0.0)
            } else {
                animation = "up";

                animated_sprite.set_flip_v(velocity.y > 0.0)
            }

            animated_sprite.play_ex().name(animation.into()).done();
        } else {
            animated_sprite.stop();
        }

        let change = velocity * real::from_f64(delta);
        let position = self.base.get_global_position() + change;
        let position = Vector2::new(
            position.x.clamp(0.0, self.screen_size.x),
            position.y.clamp(0.0, self.screen_size.y),
        );
        self.base.set_global_position(position);
    }
}
