use std::collections::HashMap;
use std::time::SystemTime;
use gilrs::Axis::{LeftStickX, LeftStickY, RightStickX, RightStickY};
use gilrs::Button::{C, DPadDown, DPadLeft, DPadRight, DPadUp, East, LeftThumb, LeftTrigger, LeftTrigger2, Mode, North, RightThumb, RightTrigger, RightTrigger2, Select, South, Start, West, Z};
use gilrs::ev::Code;
use gilrs::Gamepad;

#[derive (Debug, Clone)]
pub(crate) struct MyGamepad{
    pub(crate) left_stick:Stick,
    pub(crate) right_stick:Stick,
    force_feedback:bool,
    pub(crate) left_analog_trigger: Trigger,
    pub(crate) right_analog_trigger: Trigger,
    pub(crate) buttons: HashMap<Code,Button>,   //Xbox has around 14 or so buttons
}

//TODO: More input devices.
pub(crate) struct MyJoystick{
    main_stick: Stick,

}

#[derive (Debug, Clone)]
pub(crate) struct Stick{
    deadzone:f32,
    pub(crate) position_x:f32,
    pub(crate) position_y:f32,
    pub(crate) x_code:Code,
    pub(crate) y_code:Code,
    pub(crate) x_timestamp:Option<SystemTime>,
    pub(crate) y_timestamp:Option<SystemTime>
}

#[derive (Debug, Clone)]
pub(crate) struct Trigger{
    pub(crate) code:Code,
    deadzone:f32,
    pub(crate) position:f32,
    state:bool,
    pub(crate) pressed_time_stamp:Option<SystemTime>,
}

#[derive (Debug, Clone)]
pub(crate) struct Button{
    name:gilrs::ev::Button,
    pub(crate) position:f32,
    pub(crate) state:bool,
    pub(crate) pressed_time_stamp:Option<SystemTime>,
}

pub(crate) fn my_gamepad_init(gamepad_instance:&Gamepad) -> Option<MyGamepad> {
    let mut gamepad =  MyGamepad {
        left_stick: Stick {
            deadzone: 0.0,
            position_x: 0.0,
            position_y: 0.0,
            x_code: gamepad_instance.axis_code(LeftStickX)?,
            y_code: gamepad_instance.axis_code(LeftStickY)?,
            x_timestamp: None,
            y_timestamp: None
        },
        right_stick: Stick {
            deadzone: 0.0,
            position_x: 0.0,
            position_y: 0.0,
            x_code: gamepad_instance.axis_code(RightStickX)?,
            y_code: gamepad_instance.axis_code(RightStickY)?,
            x_timestamp: None,
            y_timestamp: None
        },
        force_feedback: gamepad_instance.is_ff_supported(),
        left_analog_trigger: Trigger {
            code: gamepad_instance.button_code(LeftTrigger2)?,
            deadzone: 0.0,
            position: 0.0,
            state: false,
            pressed_time_stamp: None
        },
        right_analog_trigger: Trigger {
            code: gamepad_instance.button_code(RightTrigger2)?,
            deadzone: 0.0,
            position: 0.0,
            state: false,
            pressed_time_stamp: None
        },
        buttons: HashMap::new()
    };
    button_enum(&mut gamepad, &gamepad_instance);
    save_deadzones(&mut gamepad, &gamepad_instance);
    Some(gamepad)
}

fn save_deadzones(gamepad: &mut MyGamepad, gamepad_instance:&Gamepad){
    gamepad.left_stick.deadzone = match gamepad_instance.deadzone(gamepad.left_stick.x_code){
        None => 10.0,
        Some(t) => t
    };
    gamepad.right_stick.deadzone = match gamepad_instance.deadzone(gamepad.right_stick.x_code){
        None => 10.0,
        Some(t) => t
    };
    gamepad.left_analog_trigger.deadzone = match gamepad_instance.deadzone(gamepad.left_analog_trigger.code){
        None => 0.0,
        Some(t) => t
    };
    gamepad.right_analog_trigger.deadzone = match gamepad_instance.deadzone(gamepad.right_analog_trigger.code){
        None => 0.0,
        Some(t) => t
    };
}

fn button_enum(gamepad: &mut MyGamepad, gamepad_instance:&Gamepad){
    let mut button_enum_vector = vec![];
    button_enum_vector.push(DPadLeft);
    button_enum_vector.push(DPadRight);
    button_enum_vector.push(DPadUp);
    button_enum_vector.push(DPadDown);
    button_enum_vector.push(West);
    button_enum_vector.push(East);
    button_enum_vector.push(North);
    button_enum_vector.push(South);
    button_enum_vector.push(Mode);
    button_enum_vector.push(Select);
    button_enum_vector.push(Start);
    button_enum_vector.push(LeftTrigger);
    button_enum_vector.push(RightTrigger);
    button_enum_vector.push(C);
    button_enum_vector.push(Z);
    button_enum_vector.push(LeftThumb);
    button_enum_vector.push(RightThumb);
    button_enum_vector.push(gilrs::ev::Button::Unknown);

    for i in button_enum_vector{
        let button = match gamepad_instance.button_code(i){
            None => continue,
            Some(t) => t
        };
        gamepad.buttons.insert(
            button, Button{
                name: i,
                state: false,
                pressed_time_stamp: None,
                position: 0.0
            });
    }
}