use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Formatter};
use std::ops::Neg;
use std::str::FromStr;
use flume::Sender;
use enigo::{Enigo, Key, KeyboardControllable, MouseControllable};
use sdl2::controller::{Axis, Button, GameController};
use sdl2::{GameControllerSubsystem, JoystickSubsystem};
use sdl2::event::Event;
use sdl2::joystick::{HatState, Joystick};
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

//Lesser_axis are axis such as triggers or throttle and can work simultanously.
//This also means that negative values will be unexpected.
//other axis represent both directions, for example left_stick x axis on a gamepad.
#[derive(Debug)]
pub(crate) struct SelfMadeAxis{
    x_axis1:f32,
    x_axis2:f32,
    y_axis1:f32,
    y_axis2:f32,
    z_axis1:f32,
    z_axis2:f32,
    lesser_axis1:f32,
    lesser_axis2:f32,
    debug: bool
}

pub(crate) fn gamepad_axis_number(axis: Axis) -> u8{
    match axis{
        Axis::LeftX => 0,
        Axis::LeftY => 1,
        Axis::RightX => 2,
        Axis::RightY => 3,
        Axis::TriggerLeft => 4,
        Axis::TriggerRight => 5
    }
}

pub(crate) fn gamepad_button_number(button: Button) -> u8{
    match button{
        Button::A => 0,
        Button::B => 1,
        Button::X => 2,
        Button::Y => 3,
        Button::Back => 6,
        Button::Guide => 128,
        Button::Start => 7,
        Button::LeftStick => 8,
        Button::RightStick => 9,
        Button::LeftShoulder => 4,
        Button::RightShoulder => 5,
        Button::DPadUp => 10,
        Button::DPadDown => 11,
        Button::DPadLeft => 12,
        Button::DPadRight => 13,
        Button::Misc1 => 14,
        Button::Paddle1 => 15,
        Button::Paddle2 => 16,
        Button::Paddle3 => 17,
        Button::Paddle4 => 18,
        Button::Touchpad => 19,
    }
}
///Okay, so the space beyond 128 is used for HATs.
///The idea is that any single device wont have 100+ buttons and especially not enough HATs to fill everything to 255.
///This function assigns a number for each hatstate based on id and hatstate.
/// Centered means no button is pressed, so the state is None. All other HATs have to be negated!
pub(crate) fn get_hat_numbers(hat_idx: u8, state: HatState) -> Vec<(u8, bool)>{
    let base = 129+hat_idx*9;
    let mut negation_vector = vec![];
    //Fill up with negated HAT values.
    for i in base..base+9{
        negation_vector.push((i, false));
    }
    //set active HAT
    let active = match state {
        HatState::Centered => 0,
        HatState::Up => 1,
        HatState::Right => 2,
        HatState::Down => 3,
        HatState::Left => 4,
        HatState::RightUp => 5,
        HatState::RightDown => 6,
        HatState::LeftUp => 7,
        HatState::LeftDown => 8
    } as u8;
    negation_vector[active as usize].1 = true;
    negation_vector
}

//First HashMap: String = GUID of device. Second HashMap is axis/buttonid and the corresponding result to the right.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AxisMap {
    #[serde_as(as = "HashMap<DisplayFromStr, HashMap<DisplayFromStr, DisplayFromStr>>")]
    pub(crate) mapping: HashMap<String, HashMap<u8, AxisVariations>>
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct KeyMap {
    #[serde_as(as = "HashMap<DisplayFromStr, HashMap<DisplayFromStr, DisplayFromStr>>")]
    pub(crate) mapping: HashMap<String, HashMap<u8, KeyList>>
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct KeyList{
    pub(crate) key_list : Vec<u16>
}

impl fmt::Display for KeyList{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buffer =  String::new();
        for i in &self.key_list{
            buffer.push_str(&i.to_string());
            buffer.push_str(",");
        }
        buffer.pop();
        write!(f, "{}", buffer)
    }
}

pub(crate) struct KeyListFromStrError{}

impl fmt::Display for KeyListFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("KeyListFromStrError")
    }
}

impl FromStr for KeyList {
    type Err = KeyListFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut vector = vec![];
        for i in s.split(','){
            vector.push(match i.parse::<u16>(){
                Ok(t) => t,
                Err(_) => return Err(KeyListFromStrError {}),
            })
        }
        Ok(KeyList{
            key_list: vector
        })
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum AxisVariations{
    XAxis1,
    XAxis2,
    YAxis1,
    YAxis2,
    ZAxis1,
    ZAxis2,
    LesserAxis1,
    LesserAxis2,
    Nothing
}

pub(crate) struct AxisVariationsFromStrError;

impl fmt::Display for AxisVariationsFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("AxisVariationsFromStrError")
    }
}

impl FromStr for AxisVariations{
    type Err = AxisVariationsFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s{
            "XAxis1" => Ok(AxisVariations::XAxis1),
            "XAxis2" => Ok(AxisVariations::XAxis2),
            "YAxis1" => Ok(AxisVariations::YAxis1),
            "YAxis2" => Ok(AxisVariations::YAxis2),
            "ZAxis1" => Ok(AxisVariations::ZAxis1),
            "ZAxis2" => Ok(AxisVariations::ZAxis2),
            "LesserAxis1" => Ok(AxisVariations::LesserAxis1),
            "LesserAxis2" => Ok(AxisVariations::LesserAxis2),
            "Unmapped" => Ok(AxisVariations::Nothing),
            _ => Err(AxisVariationsFromStrError)
        }
    }
}

///Here is where all the action happens
impl SelfMadeAxis{
    pub(crate) fn new() -> SelfMadeAxis{
        SelfMadeAxis{
            x_axis1: 0.0,
            x_axis2: 0.0,
            y_axis1: 0.0,
            y_axis2: 0.0,
            z_axis1: 0.0,
            z_axis2: 0.0,
            lesser_axis1: 0.0,
            lesser_axis2: 0.0,
            debug: false
        }
    }
    pub(crate) fn toggle_debug(&mut self)->bool{
        self.debug = !self.debug;
        self.debug
    }

    ///Lesser Axis will not stay negative, but instead be used as absolute positive value!
    pub(crate) fn update_axis_state(&mut self, axis: &AxisVariations, value: f32){
        if self.debug {
            //println!("Axis: {}, Value: {}", axis, value);
            //println!("Input State before: {}", self);
        }
        match axis{
            AxisVariations::XAxis1 => self.x_axis1 = value,
            AxisVariations::XAxis2 => self.x_axis2 = value,
            AxisVariations::YAxis1 => self.y_axis1 = value,
            AxisVariations::YAxis2 => self.y_axis2 = value,
            AxisVariations::ZAxis1 => self.z_axis1 = value,
            AxisVariations::ZAxis2 => self.z_axis2 = value,
            AxisVariations::LesserAxis1 => self.lesser_axis1 = value,
            AxisVariations::LesserAxis2 => self.lesser_axis2 = value,
            AxisVariations::Nothing => {}
        }
        if self.debug {
            //println!("Input State after: {}", self);
        }
    }

    ///Intended for Gamepads. As long as less than 6 axis is useful, this can be used for everything else as well.
    pub(crate) fn encode_5axis_to_mouse(&mut self, mouse:&mut Enigo){
        let mut x_axis1_multiplier = 127.0;//2x
        let mut y_axis1_multiplier = 127.0;//2x
        let mut x_axis2_multiplier = 127.0;//2x
        let mut y_axis2_multiplier = 127.0;//2x
        let lesser_axis1_multiplier = 63.0;//1x
        let lesser_axis2_multiplier = 63.0;//1x

        //Each Axis has 0 to axis_multiplier for negative and axis_multiplier+1 to axis_multiplier*2+1 for positive values

        let x_axis1_add = SelfMadeAxis::calc_add_and_mult(&self.x_axis1, &mut x_axis1_multiplier);
        let y_axis1_add = SelfMadeAxis::calc_add_and_mult(&self.y_axis1, &mut y_axis1_multiplier);
        let x_axis2_add = SelfMadeAxis::calc_add_and_mult(&self.x_axis2, &mut x_axis2_multiplier);
        let y_axis2_add = SelfMadeAxis::calc_add_and_mult(&self.y_axis2, &mut y_axis2_multiplier);

        let canary = 4194304;
        let mut mouse_x = canary;
        mouse_x += ((self.lesser_axis1.abs()* lesser_axis1_multiplier).ceil() as i32)<<0;
        mouse_x += ((self.x_axis2* x_axis2_multiplier + x_axis2_add).ceil() as i32)<<14;
        mouse_x += ((self.x_axis1* x_axis1_multiplier + x_axis1_add).ceil() as i32) <<6;

        let mut mouse_y = canary;
        mouse_y += ((self.lesser_axis2.abs()* lesser_axis2_multiplier).ceil() as i32)<<0;
        mouse_y += ((self.y_axis2* y_axis2_multiplier + y_axis2_add).ceil() as i32)<<14;
        mouse_y += ((self.y_axis1* y_axis1_multiplier + y_axis1_add).ceil() as i32) <<6;

        if self.debug {
            //println!("Mouse X: {:b}", mouse_x);
            //println!("Mouse Y: {:b}", mouse_y);
        }
        mouse.mouse_move_relative(mouse_x, mouse_y);
    }

    ///Intended for Joysticks, or basically anything other than Gamepads.
    pub(crate) fn encode_6axis_to_mouse(&mut self, mouse:&mut Enigo){
        let mut x_axis1_multiplier = 63.0;//2x
        let mut y_axis1_multiplier = 63.0;//2x
        let mut x_axis2_multiplier = 63.0;//2x
        let mut y_axis2_multiplier = 63.0;//2x
        let mut z_axis1_multiplier = 63.0;//2x
        let mut z_axis2_multiplier = 63.0;//2x

        //Each Axis has 0 to axis_multiplier for negative and axis_multiplier+1 to axis_multiplier*2+1 for positive values

        let x_axis1_add = SelfMadeAxis::calc_add_and_mult(&self.x_axis1, &mut x_axis1_multiplier);
        let y_axis1_add = SelfMadeAxis::calc_add_and_mult(&self.y_axis1, &mut y_axis1_multiplier);
        let x_axis2_add = SelfMadeAxis::calc_add_and_mult(&self.x_axis2, &mut x_axis2_multiplier);
        let y_axis2_add = SelfMadeAxis::calc_add_and_mult(&self.y_axis2, &mut y_axis2_multiplier);
        let z_axis1_add = SelfMadeAxis::calc_add_and_mult(&self.z_axis1, &mut z_axis1_multiplier);
        let z_axis2_add = SelfMadeAxis::calc_add_and_mult(&self.z_axis2, &mut z_axis2_multiplier);

        let canary= 2097152;
        let mut mouse_x = canary;
        mouse_x += ((self.z_axis1* z_axis1_multiplier + z_axis1_add).ceil() as i32)<<0;
        mouse_x += ((self.x_axis2* x_axis2_multiplier + x_axis2_add).ceil() as i32)<<14;
        mouse_x += ((self.x_axis1* x_axis1_multiplier + x_axis1_add).ceil() as i32) <<7;

        let mut mouse_y = canary;
        mouse_y += ((self.z_axis2* z_axis2_multiplier + z_axis2_add).ceil() as i32)<<0;
        mouse_y += ((self.y_axis2* y_axis2_multiplier + y_axis2_add).ceil() as i32)<<14;
        mouse_y += ((self.y_axis1* y_axis1_multiplier + y_axis1_add).ceil() as i32) <<7;

        //println!("Mouse X: {:b}", mouse_x);
        //println!("Mouse Y: {:b}", mouse_y);

        if self.debug {
            //println!("Mouse X: {:b}", mouse_x);
            //println!("Mouse Y: {:b}", mouse_y);
        }

        mouse.mouse_move_relative(mouse_x, mouse_y);
    }
    ///Calculates the addition value and possibly reverses multiplier
    fn calc_add_and_mult(axis_value: &f32, multiplier: &mut f32) -> f32{
        return if *axis_value > 0.0 {
            let a = *multiplier + 1.0;
            a
        } else {
            *multiplier = multiplier.neg();
            0.0
        }
    }
}

#[derive(Debug)]
pub(crate) struct KeyPressHandler{
    pub(crate) current_key_state: HashSet<u16>,
    pub(crate) current_off_key_state: HashSet<u16>,
}

impl KeyPressHandler{
    pub fn new() -> KeyPressHandler{
       KeyPressHandler{
           current_key_state: HashSet::new(),
           current_off_key_state: HashSet::new(),
       }
    }

    pub(crate) fn update_key_presses(&mut self, event: (Vec<u16>, bool)){
        match event.1{
            true => {
                for i in event.0{
                    self.current_key_state.insert(i);
                }
            }
            false => {
                for i in event.0{
                    self.current_off_key_state.insert(i);
                }
            }
        }
        for i in self.current_off_key_state.clone(){
            if self.current_key_state.get(&i).is_some(){
                self.current_off_key_state.remove(&i);
            }
        }
    }
    pub(crate) fn send_key_presses(&mut self, keyboard: &mut Enigo){
        for i in &self.current_off_key_state{
            keyboard.key_up(Key::Raw(i.clone()));
        }
        for i in &self.current_key_state{
            keyboard.key_down(Key::Raw(i.clone()))
        }
        self.current_key_state.clear();
        self.current_off_key_state.clear();
    }
    ///Before the thread fokkin dies, this is a necessity to reset all keys to deactivated.
    /// Otherwise VOODOO
    pub(crate) fn return_keys_to_0(&mut self, keyboard: &mut Enigo){
        for i in &self.current_key_state{
            keyboard.key_up(Key::Raw(i.clone()));
        }
    }
}


impl fmt::Display for SelfMadeAxis{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "XAxis1: {} XAxis2: {} YAxis1: {} YAxis2: {} ZAxis1: {} ZAxis2: {} LesserAxis1: {} LesserAxis2: {} Debug: {}",
               self.x_axis1, self.x_axis2,
               self.y_axis1, self.y_axis2,
               self.z_axis1, self.z_axis2,
               self.lesser_axis1, self.lesser_axis2,
               self.debug
        )
    }
}

impl fmt::Display for AxisVariations{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self{
            AxisVariations::XAxis1 => "XAxis1",
            AxisVariations::XAxis2 => "XAxis2",
            AxisVariations::YAxis1 => "YAxis1",
            AxisVariations::YAxis2 => "YAxis2",
            AxisVariations::ZAxis1 => "ZAxis1",
            AxisVariations::ZAxis2 => "ZAxis2",
            AxisVariations::LesserAxis1 => "LesserAxis1",
            AxisVariations::LesserAxis2 => "LesserAxis2",
            AxisVariations::Nothing => "Unmapped",
        })
    }
}

pub(crate) struct GameDevices{
    pub(crate) transmitter: Sender<(String, Event)>,
    pub(crate) debug: bool,
    pub(crate) joystick_subsystem: JoystickSubsystem,
    pub(crate) controller_subsystem: GameControllerSubsystem,
    pub(crate) controllers: HashMap<u32,GameController>,
    pub(crate) joysticks: HashMap<u32,Joystick>,
    pub(crate) device_guids: HashMap<u32, String>
}

impl GameDevices{
    fn update_device_lists(&mut self, remove: bool, device_id: u32){
        if remove{
            self.joysticks.remove(&device_id);
            self.controllers.remove(&device_id);
            match self.device_guids.get(&device_id){
                None => {}
                Some(t) => {
                    if self.debug{
                        println!("Removed Device. {}",t);
                    }
                }
            }

            self.device_guids.remove(&device_id);
        }else{
            let guid = match self.get_guid(device_id){
                None => "Unknown GUID".to_string(),
                Some(guid) => guid,
            };
            self.device_guids.insert(device_id, guid.clone());
            if self.controller_subsystem.is_game_controller(device_id){
                match self.controller_subsystem.open(device_id){
                    Ok(t) => {
                        self.controllers.insert(device_id, t);
                        println!("Added Controller: {}, GUID: {}",self.get_name(device_id), guid)
                    }
                    Err(e) => println!("{}", e.to_string()),
                }
            }else{
                match self.joystick_subsystem.open(device_id){
                    Err(e) => println!("{}", e.to_string()),
                    Ok(joy) => {
                        self.joysticks.insert(device_id, joy);
                    }
                };
                println!("Added Joystick: {}, GUID: {}",self.get_name(device_id), guid)
            }
        }
    }
    pub(crate) fn get_guid(&self, id: u32) -> Option<String>{
        match self.joystick_subsystem.device_guid(id){
            Ok(t) => Some(t.to_string()),
            Err(_) => None
        }
    }
    pub(crate) fn get_name(&self, id: u32) -> String{
        match self.joystick_subsystem.name_for_index(id){
            Ok(t) => t,
            Err(_) => "No Name".to_string()
        }
    }

    pub(crate) fn event_filter(&mut self, event: Event) -> Result<(), ()>{
        let mut id = None;
        if match &event {
            Event::ControllerDeviceAdded { which, .. } => {
                self.update_device_lists(false, *which);
                false
            },
            Event::ControllerDeviceRemoved { which, .. } =>{
                self.update_device_lists(true, *which);
                false
            },
            Event::ControllerAxisMotion {which,..} =>{
                id = Some(*which);
                true
            }
            Event::ControllerButtonDown {which,..} =>{
                id = Some(*which);
                true
            }
            Event::ControllerButtonUp {which,..} =>{
                id = Some(*which);
                true
            }
            Event::Quit { .. } => return Err(()),
            Event::JoyAxisMotion { which, .. } => {
                id = Some(*which);
                !self.controller_subsystem.is_game_controller(*which)
            },
            Event::JoyBallMotion { which, .. } => {
                id = Some(*which);
                !self.controller_subsystem.is_game_controller(*which)
            },
            Event::JoyHatMotion { which, .. } => {
                id = Some(*which);
                !self.controller_subsystem.is_game_controller(*which)
            },
            Event::JoyButtonDown { which, .. } => {
                id = Some(*which);
                !self.controller_subsystem.is_game_controller(*which)
            },
            Event::JoyButtonUp { which, .. } => {
                id = Some(*which);
                !self.controller_subsystem.is_game_controller(*which)
            },
            Event::JoyDeviceAdded { which, .. } => {
                if !self.controller_subsystem.is_game_controller(*which){
                    self.update_device_lists(false, *which);
                }
                false
            }
            Event::JoyDeviceRemoved { which, .. } =>{
                if !self.controller_subsystem.is_game_controller(*which){
                    self.update_device_lists(true, *which);
                }
                false
            }
            _ => true
        } {
            if id.is_some(){
                match self.device_guids.get(&id.unwrap()){
                    None => {
                        panic!("A Device was reconnected, reconnected devices are currently broken.");
                    },
                    Some(t) => {
                        self.transmitter.send((t.clone(), event)).ok();
                    },
                }
            }
        }
        Ok(())
    }
}