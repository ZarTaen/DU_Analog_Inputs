mod input_datastructures;
mod helpers;
mod config_file_handling;

use std::fs::{File};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use crate::input_datastructures::{AxisMap, GameInputs, SelfMadeAxis, wrap_events};
use enigo::Enigo;
use keyboard_query::{DeviceQuery, DeviceState};
use spin_sleep::LoopHelper;
use crate::helpers::{log_write, open_file, read_file};
use crate::config_file_handling::{handle_config, axis_mapping_config};
use pasts::Loop;
use stick::{Event, Listener};
use toml::Value;

fn main() {
    let mut logfile = open_file("Logfile.txt");
    let settings = match handle_config(){
        Ok(t) => t,
        Err(e) => {
            log_write(&mut logfile, "Error", &e);
            return
        }
    };
    let axis_mapping = match axis_mapping_config(&mut logfile){
        Ok(t) => t,
        Err(e) => {
            log_write(&mut logfile, "Error", &e);
            return
        }
    };

    let device_pollrate = settings.get("device_pollrate").unwrap().parse::<u16>().unwrap();
    let input_sendrate_division = settings.get("input_sendrate_division").unwrap().parse::<u8>().unwrap();
    let device_number = settings.get("device_number").unwrap().parse::<u64>().unwrap();
    let debug = settings.get("debug").unwrap().parse::<bool>().unwrap();
    let sixaxis = settings.get("sixaxis").unwrap().parse::<bool>().unwrap();
    let (tx,rx ): (Sender<(u64,Event)>, Receiver<(u64,Event)>) = mpsc::channel();
    thread::spawn(move|| {
        pasts::block_on(event_loop(debug, device_number, tx))
    });
    game_send_loop(rx, device_pollrate, input_sendrate_division, logfile, axis_mapping, debug, sixaxis);
}

async fn event_loop(debug: bool, active_device: u64, transmitter: Sender<(u64, Event)>){
    let mut game_devices = GameInputs {
        listener: Listener::default(),
        controllers: Vec::new(),
        transmitter,
        debug,
        active_device
    };

    let player_id = Loop::new(&mut game_devices)
        .when(|context| &mut context.listener, GameInputs::connect)
        .poll(|context| &mut context.controllers, GameInputs::event_match)
        .await;
}

fn game_send_loop(receiver: Receiver<(u64, Event)>, pollrate: u16,
                  input_sendrate_division: u8, mut logfile: File, axis_mapping: AxisMap,
                  debug: bool, sixaxis: bool){
    let mut mouse = Enigo::new();
    let mut loop_helper = LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(pollrate);
    let mut counter: u8=1;
    let mut input_state = SelfMadeAxis::new();
    if debug{
        input_state.toggle_debug();
    }

    let keyboard = DeviceState::new();
    let mut key_toggle = false;
    let mut key_on = false;
    loop{
        loop_helper.loop_start();
        let old_key_on = key_on;
        let key_list = keyboard.get_keys();
        if key_list.len()>0{
            for i in key_list{
                if i == 163{
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
        while let event_message = receiver.try_recv() {
            match event_message{
                Ok((_, event)) => {
                    match wrap_events(event) {
                        Ok(event_value) => {
                            if event_value.1.is_some() {
                                let val = event_value.1.unwrap();
                                match val {
                                    Value::Float(flo) => {
                                        match axis_mapping.mapping.get(&event_value.0) {
                                            Some(mapped_event) => {
                                                input_state.update_axis_state(mapped_event, flo as f32)
                                            },
                                            _ => {}
                                        };
                                    }
                                    Value::Boolean(_) => {
                                        //TODO: Keymapping
                                    }
                                    _ => {}
                                }
                            }
                        },
                        _ => {}
                    }
                }
                Err(e) => {
                    match e{
                        TryRecvError::Empty => {
                            //All entries have been read.
                            break;
                        }
                        TryRecvError::Disconnected =>{
                            log_write(&mut logfile, "Error", "Input loop disappeared.");
                            return
                        }
                    }
                }
            }
        }
        counter+=1;
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
        loop_helper.loop_sleep();
    }
}