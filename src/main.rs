mod input_datastructures;
mod helpers;
mod config_file_handling;

use std::fs::{File};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use crate::input_datastructures::{AxisMap,GameDevices, gamepad_axis_number, gamepad_button_number, get_hat_numbers, KeyMap, KeyPressHandler, SelfMadeAxis};
use enigo::Enigo;
use keyboard_query::{DeviceQuery, DeviceState};
use spin_sleep::LoopHelper;
use crate::helpers::{log_write, open_file, read_file};
use crate::config_file_handling::{handle_config, axis_mapping_config, key_mapping_config};
//use stick::{Event, Listener};
use flume::{unbounded, Sender, Receiver,TryRecvError};
use sdl2::event::Event;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn tool() -> Result<(), String>{
    let mut logfile = open_file("Logfile.txt");
    let settings = handle_config()?;
    let axis_mapping = axis_mapping_config(&mut logfile)?;
    let key_mapping =  key_mapping_config(&mut logfile)?;
    let device_pollrate = settings.get("device_pollrate").unwrap().parse::<u16>().unwrap();
    let input_sendrate_division = settings.get("input_sendrate_division").unwrap().parse::<u8>().unwrap();
    let debug = settings.get("debug").unwrap().parse::<bool>().unwrap();
    let sixaxis = settings.get("sixaxis").unwrap().parse::<bool>().unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _dummy_window = match video_subsystem.window("Analog Input Dummy", 2, 1)
        .position_centered()
        // Window is "hidden" (but may appear in the task-bar)
        .set_window_flags(sdl2_sys::SDL_WindowFlags::SDL_WINDOW_HIDDEN as u32)
        .build(){
        Ok(t) => t,
        Err(e) => return Err(e.to_string())
    };
    sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");
    let joystick_subsystem = sdl_context.joystick()?;
    let controller_subsystem = sdl_context.game_controller()?;
    let mut event_pump = sdl_context.event_pump()?;

    match controller_subsystem.load_mappings("gamecontrollerdb.txt"){
        Err(e) => log_write(&mut logfile, "Error", e.to_string().as_str()),
        _ => {}
    }
    let (tx, rx): (Sender<(String, Event)>, Receiver<(String, Event)>) = unbounded();
    let electric_atomic_seppuku = Arc::new(AtomicBool::from(false));
    let electric_atomic_seppuku2 = electric_atomic_seppuku.clone();
    let thread = thread::spawn(move|| {
        game_send_loop(rx, device_pollrate, input_sendrate_division, &mut logfile,
                       axis_mapping, key_mapping, debug, sixaxis, electric_atomic_seppuku2);
        return logfile
    });
    let mut devices = GameDevices{
        transmitter:tx,
        debug,
        joystick_subsystem,
        controller_subsystem,
        controllers: Default::default(),
        joysticks: Default::default(),
        device_guids: Default::default(),
    };
    let mut loop_helper = LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(10);
    'outer: loop{
        loop_helper.loop_start();
        for event in event_pump.wait_timeout_iter(1) {
            if devices.event_filter(event).is_err() {
                electric_atomic_seppuku.store(true, Ordering::Release);
                break 'outer;
            }
        }
        loop_helper.loop_sleep();
    }
    thread.join().unwrap();
    Ok(())
}

fn main(){
    println!("Version: v{}", VERSION);
    let a = tool();
    let mut logfile = &mut open_file("Logfile.txt");
    match a{
        Err(e) => log_write(&mut logfile, "Error", &e),
        _ => {}
    }
}

fn game_send_loop(receiver: Receiver<(String, Event)>, pollrate: u16,
                  input_sendrate_division: u8, logfile: &mut File, axis_mapping: AxisMap,
                  key_mapping: KeyMap, debug: bool, sixaxis: bool, electric_atomic_seppuku: Arc<AtomicBool>){
    let mut mouse = Enigo::new();
    let mut loop_helper = LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(pollrate);
    let mut counter: u8=1;
    let mut input_state = SelfMadeAxis::new();
    let mut input_state_key = KeyPressHandler::new();
    if debug{
        input_state.toggle_debug();
    }

    let keyboard = DeviceState::new();
    let mut key_toggle = false;
    let mut key_on = false;
    let mut key_one = false;
    let mut key_two = false;
    let mut key_three = false;
    loop{
        loop_helper.loop_start();
        
        let old_key_on = key_on;
        let key_list = keyboard.get_keys();
        if key_list.len()>0{
            for i in key_list{
                if i == 163 || i == 220{
                    key_one = true
                }
                //if key_one && key_two &&key_three{
                if key_one{
                    //key_two = false;
                    key_one = false;
                    //key_three = false;
                    key_on = true;
                    break;
                }
                key_on = false;
            };
        }else{
            key_on = false;
        }
        if key_on == false && old_key_on == true{
            key_toggle = !key_toggle;
        }
        if !electric_atomic_seppuku.load(Ordering::Acquire) {
            while let event_message = receiver.try_recv() {
                match event_message{
                    Ok((id, event)) => {
                        match event{
                            Event::JoyAxisMotion {axis_idx, value ,..} => {
                                match axis_mapping.mapping.get(&id){
                                    Some(device_mapping) => {
                                        match device_mapping.get(&axis_idx){
                                            Some(mapped_event) => {
                                                let value = value as f32 / i16::MAX as f32;
                                                if debug && (value.abs()>0.15 ||value ==0.0){
                                                    println!("Device: {}| Axis: {}| Value: {}", id,axis_idx, value)
                                                }
                                                input_state.update_axis_state(mapped_event, value);
                                            }
                                            None => {}
                                        }
                                    }
                                    None => {}
                                }
                            }
                            Event::JoyBallMotion {ball_idx, xrel, yrel, .. } => {
                                if debug{
                                    println!("JoyBallMotion: {} | Not Supported.", ball_idx);
                                log_write(logfile, "Error", "JoyBallMotion not supported.")
                                }
                            }
                            //The buttons beyond value 128 are reserved for HATs.
                            //For example an 8 way HAT with centered has 9 possible ids for each state.
                            //They are handled as if they were active buttons. I recommend not to map the centered state!
                            Event::JoyHatMotion {hat_idx, state , ..} => {
                                match key_mapping.mapping.get(&id) {
                                    None => {}
                                    Some(device_mapping) => {
                                        let hat_numbers = get_hat_numbers(hat_idx, state);
                                        if debug{
                                            println!("Device: {}| HAT: {}| Values: {:?}", id,hat_idx, hat_numbers)
                                        }
                                        //This sets all hat_codes to false that are unpressed and sets the one true that is true.
                                        for hat_code in hat_numbers{
                                            match device_mapping.get(&hat_code.0){
                                                None => {}
                                                Some(keys) => {
                                                    if debug {
                                                        println!("Keys: {:?}, active: {:?}", keys.key_list, hat_code.1);
                                                    }
                                                    input_state_key.update_key_presses((keys.key_list.clone(), hat_code.1))
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Event::JoyButtonDown {button_idx , ..} => {
                                match key_mapping.mapping.get(&id) {
                                    None => {}
                                    Some(device_mapping) => {
                                        if debug{
                                            println!("Device: {}| Button: {}| Value: {:?}", id,button_idx, true)
                                        }
                                        match device_mapping.get(&button_idx){
                                            None => {}
                                            Some(keys) => {
                                                if debug {
                                                    println!("Keys: {:?}", keys.key_list);
                                                }
                                                input_state_key.update_key_presses((keys.key_list.clone(), true))
                                            }
                                        }
                                    }
                                }
                            }
                            Event::JoyButtonUp {button_idx , ..} => {
                                match key_mapping.mapping.get(&id) {
                                    None => {}
                                    Some(device_mapping) => {
                                        if debug{
                                            println!("Device: {}| Button: {}| Value: {:?}", id,button_idx, false)
                                        }
                                        match device_mapping.get(&button_idx){
                                            None => {}
                                            Some(keys) => {
                                                if debug {
                                                    println!("Keys: {:?}", keys.key_list);
                                                }
                                                input_state_key.update_key_presses((keys.key_list.clone(), false))
                                            }
                                        }
                                    }
                                }
                            }
                            Event::ControllerAxisMotion {axis, value , ..} => {
                                match axis_mapping.mapping.get(&id){
                                    None => {}
                                    Some(device_mapping) => {
                                        let value = value as f32 / i16::MAX as f32;
                                        if debug && (value.abs()>0.15 ||value ==0.0) {
                                            println!("Device: {}| Axis: {}| Value: {}", id,gamepad_axis_number(axis), value)
                                        }
                                        match device_mapping.get(&gamepad_axis_number(axis)){
                                            None => {}
                                            Some(ax) => {
                                                input_state.update_axis_state(ax, value)
                                            }
                                        }
                                    }
                                }
                            }
                            Event::ControllerButtonDown {button , ..} => {
                                match key_mapping.mapping.get(&id) {
                                    None => {}
                                    Some(device_mapping) => {
                                        if debug{
                                            println!("Device: {}| Button: {}| Value: {:?}", id,gamepad_button_number(button), true)
                                        }
                                        match device_mapping.get(&gamepad_button_number(button)){
                                            None => {}
                                            Some(keys) => {
                                                if debug {
                                                    println!("Keys: {:?}", keys.key_list);
                                                }
                                                input_state_key.update_key_presses((keys.key_list.clone(), true))
                                            }
                                        }
                                    }
                                }
                            }
                            Event::ControllerButtonUp {button, ..} => {
                                match key_mapping.mapping.get(&id) {
                                    None => {}
                                    Some(device_mapping) => {
                                        if debug{
                                            println!("Device: {}| Button: {}| Value: {:?}", id,gamepad_button_number(button), false)
                                        }
                                        match device_mapping.get(&gamepad_button_number(button)){
                                            None => {}
                                            Some(keys) => {
                                                if debug {
                                                    println!("Keys: {:?}", keys.key_list);
                                                }
                                                input_state_key.update_key_presses((keys.key_list.clone(), false))
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        match e{
                            TryRecvError::Empty => {
                                break;
                            }
                            TryRecvError::Disconnected =>{
                                input_state_key.return_keys_to_0(&mut mouse);
                                log_write(logfile, "Error", "Input loop disappeared.");
                                return
                            }
                        }
                    }
                }
            }
            counter+=1;
            input_state_key.send_key_presses(&mut mouse);
            if counter != input_sendrate_division || !key_toggle{
                loop_helper.loop_sleep();
                continue
            }else{
                counter =0;
            }
            if sixaxis{
                input_state.encode_6axis_to_mouse(&mut mouse)
            }else{
                input_state.encode_5axis_to_mouse(&mut mouse)
            }
        }else{
            input_state_key.return_keys_to_0(&mut mouse);
            break
        }
        loop_helper.loop_sleep();
    }
}