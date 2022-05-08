mod gamepad_datastructures;
use gamepad_datastructures::my_gamepad_init;
use std::fs::{File, OpenOptions};
use std::collections::HashMap;
use std::io::Write;
use std::process::exit;
use std::thread;
use std::time::{Duration, Instant};
use gilrs::{Error, EventType, Gamepad, GamepadId, Gilrs, Mapping};
use gilrs::ev::Code;
use config::{Config, FileFormat};
use config::File as File2;
use crate::gamepad_datastructures::{MyGamepad, Stick, Trigger};
use enigo::{Enigo, Key, MouseControllable};
use keyboard_query::{DeviceQuery, DeviceState};
use spin_sleep::LoopHelper;
const ANALOGPOLLRATE: u16 = 125; //in times per second, 125 = 8ms per loop iteration.
const INPUTCONVERTFREQUENCY:u8 = 10; //in loop iterations, 9 = 10 loop iterations of e.g. 8ms =


fn main() {
    let mut gilrs = match Gilrs::new(){
        Ok(t) => t,
        Err(e) => {
            println!("{}", e);
            exit(0)
        }
    };
    gilrs.counter();
    update_gamepad_list(&gilrs);
    let placeholder_gamepadid: usize = 0;
    let active_gilrs_gamepad = gilrs.gamepads().collect::<Vec<(GamepadId, Gamepad)>>()[placeholder_gamepadid];

    let mut active_gamepad_cache = match my_gamepad_init(&active_gilrs_gamepad.1){
        None => panic!(),
        Some(t) =>t
    };
    gamepad_loop(&mut active_gamepad_cache, gilrs, placeholder_gamepadid)

}

fn gamepad_loop(active_gamepad: &mut MyGamepad, mut gilrs: Gilrs, active_gamepad_id: usize){
    let active_gamepad_id = gilrs.gamepads().collect::<Vec<(GamepadId, Gamepad)>>()[active_gamepad_id].0;
    let mut mouse = Enigo::new();
    let mut start_time;
    let mut jitter = 1;
    let mut loop_helper = LoopHelper::builder()
        .report_interval_s(0.5) // report every half a second
        .build_with_target_rate(ANALOGPOLLRATE);//125
    let mut counter: u8=1;
    let keyboard = DeviceState::new();
    let mut key_toggle = false;
    let mut key_on = false;

    'outer: loop{
        loop_helper.loop_start();
        start_time = Instant::now();
        let old_key_on = key_on;
        for i in keyboard.get_keys(){
            if i == 163{
                key_on = true;
                break;
            }
            key_on = false;
        };
        if keyboard.get_keys().len() == 0{
            key_on = false;
        }
        if key_on == false && old_key_on == true{
            if key_toggle == true {
                key_toggle = false;
            }else{
                key_toggle = true;
            }
        }

        while let Some(event) = gilrs.next_event() {
            if event.id == active_gamepad_id{
                match event.event {
                    EventType::ButtonPressed(name, event_code) => {
                        if let Some(button_entry) = active_gamepad.buttons.get_mut(&event_code) {
                            button_entry.state = true;
                            button_entry.pressed_time_stamp = Some(event.time);
                        }
                    }
                    EventType::ButtonReleased(name, event_code) => {
                        if let Some(button_entry) = active_gamepad.buttons.get_mut(&event_code) {
                            button_entry.state = false;
                            button_entry.position = 0.0;
                            button_entry.pressed_time_stamp = None;
                        }
                    }
                    EventType::ButtonChanged(name, state, event_code) => {
                        if let Some(button_entry) = active_gamepad.buttons.get_mut(&event_code) {
                            button_entry.position = state;
                            button_entry.state = true;
                            button_entry.pressed_time_stamp = Some(event.time);
                        } else if active_gamepad.left_analog_trigger.code == event_code {
                            active_gamepad.left_analog_trigger.position = state;
                            active_gamepad.left_analog_trigger.pressed_time_stamp = Some(event.time);
                        } else if active_gamepad.right_analog_trigger.code == event_code {
                            active_gamepad.right_analog_trigger.position = state;
                            active_gamepad.right_analog_trigger.pressed_time_stamp = Some(event.time);
                        }
                    }
                    EventType::AxisChanged(name, state, event_code) => {
                        if active_gamepad.left_stick.x_code == event_code {
                            active_gamepad.left_stick.position_x = state;
                            active_gamepad.left_stick.x_timestamp = Some(event.time);
                        } else if active_gamepad.left_stick.y_code == event_code {
                            active_gamepad.left_stick.position_y = state;
                            active_gamepad.left_stick.y_timestamp = Some(event.time);
                        } else if active_gamepad.right_stick.x_code == event_code {
                            active_gamepad.right_stick.position_x = state;
                            active_gamepad.right_stick.x_timestamp = Some(event.time);
                        } else if active_gamepad.right_stick.y_code == event_code {
                            active_gamepad.right_stick.position_y = state;
                            active_gamepad.right_stick.y_timestamp = Some(event.time);
                        }
                    }
                    EventType::Connected => {}
                    EventType::Disconnected => {
                        println!("disconnected");
                        //break 'outer
                    }
                    EventType::Dropped => {}
                    _ => {}
                }
            }
        }
        counter+=1;
        if counter== INPUTCONVERTFREQUENCY && key_toggle {
            emulate_mouse_gamepad(&active_gamepad.left_stick, &active_gamepad.right_stick, &active_gamepad.left_analog_trigger, &active_gamepad.right_analog_trigger, &mut mouse, jitter);
            jitter= -jitter;
        counter = 0;
        }
         loop_helper.loop_sleep();
        //println!("{:?}", start_time.elapsed());
    }
}

fn update_gamepad_list(gilrs: &Gilrs){
    let mut file = match OpenOptions::new().create(true).truncate(true).write(true).read(true).open("device_list.txt"){
        Ok(t) => t,
        Err(e) => panic!()
    };
    let mut gamepads = HashMap::new();
    for i in gilrs.gamepads() {
        gamepads.insert(i.0.to_string().parse::<usize>().unwrap(), i.1);
    }
    file.write(format!("{:#?}", gamepads).as_bytes());
}

fn handle_config() -> Result<Config, String>{
    let mut builder = Config::builder()
        .add_source(File2::new("config", FileFormat::Toml));
    match builder.build(){
        Ok(config) => {
            Ok(config)
        }
        Err(e) => {
            return Err(e.to_string())
        }
    }
}

fn emulate_mouse_gamepad(left_stick: &Stick, right_stick: &Stick, left_trigger: &Trigger, right_trigger: &Trigger, mouse: &mut Enigo, jitter: i8) {
    let mut mouse_x = 0;
    let mut mouse_y = 0;
    let mut mouse_x_add = 0;
    let mut mouse_y_add = 0;
    let mut left_stick_x_multiplier = 400.0;
    let mut left_stick_y_multiplier = 400.0;
    let mut right_stick_x_multiplier = 400.0;
    let mut right_stick_y_multiplier = 400.0;
    if left_stick.position_y >0.0{
        mouse_y_add+=401;
    }else{
        left_stick_y_multiplier = -400.0;
    }
    if right_stick.position_y >0.0{
        mouse_y_add+=401000;
    }else{
        right_stick_y_multiplier = -400.0;
    }
    if left_stick.position_x > 0.0{
        mouse_x_add+=401;
    }else{
        left_stick_x_multiplier = -400.0;
    }
    //1xxx-200xxx
    if right_stick.position_x > 0.0{
        mouse_x_add+=401000;
    }else{
        right_stick_x_multiplier = -400.0;
    }

    mouse_x = mouse_x+((right_trigger.position*40.0) as i32 * 1000000);
    mouse_x = mouse_x+((right_stick.position_x*right_stick_x_multiplier) as i32 *     1000);
    mouse_x = mouse_x+((left_stick.position_x*left_stick_x_multiplier) as i32);


    mouse_y = mouse_y+((left_trigger.position*40.0) as i32 *    1000000);
    mouse_y = mouse_y+((right_stick.position_y*right_stick_y_multiplier) as i32 *     1000);
    mouse_y = mouse_y+((left_stick.position_y*left_stick_y_multiplier) as i32);

    mouse_x+=mouse_x_add;
    mouse_y+=mouse_y_add;
    //println!("{}, {}, {}, {}", left_stick.position_x*100.0, right_stick.position_x*1000.0, left_trigger.position*10000.0,right_trigger.position*100000.0);
    //println!("{}, {}",left_stick.position_y*100.0, right_stick.position_y*1000.0);


    if mouse_x == 0 {
        mouse_x= jitter as i32;
    }
    if mouse_y == 0 {
        mouse_y= jitter as i32;
    }
    mouse.mouse_move_relative(mouse_x, mouse_y);
}
