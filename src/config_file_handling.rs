use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use config::{Config, FileFormat};
use config::File as File2;
use sdl2::controller::{Axis, Button};
use crate::helpers::open_file;
use crate::input_datastructures::{AxisVariations, AxisMap, KeyList};
use crate::{gamepad_axis_number, gamepad_button_number, KeyMap, log_write, read_file};

const DEFAULTCONFIGSTRING:&str = r#"device_pollrate = "250" #This is the pollrate per second to check on device state.
input_sendrate_division = "6" #This value tells how many polls to wait before sending an input to mouse.
debug = true #This value prints debug information to the console to enumerate all input codes. Disable this once you finished your mapping.
sixaxis = true #This value toggles between sixaxis and 5axis encoding. Sixaxis is used when true"#;

//device_number ="0" #This value is the device number mentioned for the device.
const CONFIGNAME: &str = "config.toml";
const AXISMAPPINGNAME: &str = "axis_mapping.toml";
const KEYMAPPINGNAME: &str = "key_mapping.toml";

///Creates or Reads basic configuration.
pub(crate) fn handle_config() -> Result<HashMap<String,String>, String>{
    let mut file = open_file(CONFIGNAME);
    let builder = Config::builder().add_source(File2::new(CONFIGNAME, FileFormat::Toml));
    let config;
    match builder.build_cloned(){
        Ok(t) => {
            config = t;
        },
        Err(e) =>{
            return Err(e.to_string())
        }
    }
    //Stuff needs to be written into the file first.
    let settings = match config.try_deserialize::<HashMap<String, String>>() {
        Ok(t) => t,
        Err(e) => return Err(e.to_string())
    };
    let sets= vec!["device_pollrate","input_sendrate_division", "debug", "sixaxis"];
    if settings.len()==0{
        file.write(DEFAULTCONFIGSTRING.as_bytes()).ok();
        return handle_config();
    }
    for i in &sets {
        match settings.get(*i) {
            None => {
                //Stuff needs to be written into the file first.
                return Err(format!("Configuration Entry {} not found. Delete file for new generation.", i))
            },
            Some(_) => {}
        };
    }
    match settings.get(sets[0]).unwrap().parse::<u16>(){
        Ok(_) => {}
        Err(e) => return Err(e.to_string())
    };
    match settings.get(sets[1]).unwrap().parse::<u8>(){
        Ok(_) => {}
        Err(e) => return Err(e.to_string())
    };
    /*match settings.get(sets[2]).unwrap().parse::<u64>(){
        Ok(_) => {}
        Err(e) => return Err(e.to_string())
    };*/
    match settings.get(sets[2]).unwrap().parse::<bool>(){
        Ok(_) => {}
        Err(e) => return Err(e.to_string())
    };
    match settings.get(sets[3]).unwrap().parse::<bool>(){
        Ok(_) => {}
        Err(e) => return Err(e.to_string())
    };
    return Ok(settings)
}

///Reads Mapping or creates default Xbox Gamepad Mapping
pub(crate) fn axis_mapping_config(logfile: &mut File) -> Result<AxisMap, String>{
    let mut file = read_file(AXISMAPPINGNAME);
    let mut mapping = vec![];
    let mut bufread = BufReader::new(file);
    match bufread.read_to_end(&mut mapping){
        Ok(_) => {}
        Err(e) => println!("{}", e)
    };
    let a = match String::from_utf8(mapping){
        Ok(t) => t,
        Err(e) => return Err(e.to_string()),
    };
    return match toml::from_str::<AxisMap>(&*a) {
        Ok(map) => {
            if map.mapping.len() == 0 {
                File::create(AXISMAPPINGNAME).ok();
                file = open_file(AXISMAPPINGNAME);
                let mut new_map = AxisMap {
                    mapping: HashMap::new()
                };
                let mut device_map = HashMap::new();
                //Push Xbox Gamepad Mappings.
                device_map.insert(gamepad_axis_number(Axis::LeftX), AxisVariations::XAxis1);
                device_map.insert(gamepad_axis_number(Axis::LeftY), AxisVariations::YAxis1);
                device_map.insert(gamepad_axis_number(Axis::RightX), AxisVariations::XAxis2);
                device_map.insert(gamepad_axis_number(Axis::RightY), AxisVariations::YAxis2);
                device_map.insert(gamepad_axis_number(Axis::TriggerLeft), AxisVariations::ZAxis2);
                device_map.insert(gamepad_axis_number(Axis::TriggerRight), AxisVariations::ZAxis1);
                //Uses the GUID
                new_map.mapping.insert("030000005e040000ff02000000007200".to_string(), device_map);
                let serialized = match toml::to_string_pretty(&new_map) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(e.to_string())
                    }
                };
                file.write(&*serialized.into_bytes()).ok();
                axis_mapping_config(logfile)
            } else {
                //Lmao, your own fault if you fail something in the mapping. But if its valid, its valid.
                Ok(map)
            }
        },
        Err(_) => {
            log_write(logfile,"Error", &format!("Invalid Mapping. {} has been reset.",AXISMAPPINGNAME));
            File::create(AXISMAPPINGNAME).ok();
            file = open_file(AXISMAPPINGNAME);
            let mut new_map = AxisMap {
                mapping: HashMap::new()
            };
            let mut device_map = HashMap::new();
            //Push Xbox Gamepad Mappings.
            device_map.insert(gamepad_axis_number(Axis::LeftX), AxisVariations::XAxis1);
            device_map.insert(gamepad_axis_number(Axis::LeftY), AxisVariations::YAxis1);
            device_map.insert(gamepad_axis_number(Axis::RightX), AxisVariations::XAxis2);
            device_map.insert(gamepad_axis_number(Axis::RightY), AxisVariations::YAxis2);
            device_map.insert(gamepad_axis_number(Axis::TriggerLeft), AxisVariations::ZAxis2);
            device_map.insert(gamepad_axis_number(Axis::TriggerRight), AxisVariations::ZAxis1);
            new_map.mapping.insert("030000005e040000ff02000000007200".to_string(), device_map);
            let serialized = match toml::to_string_pretty(&new_map) {
                Ok(t) => t,
                Err(e) => {
                    return Err(e.to_string())
                }
            };
            match toml::from_str::<AxisMap>(&*serialized){
                Ok(t) => t,
                Err(e) => return Err(e.to_string())
            };
            file.write(&*serialized.as_bytes()).ok();
            axis_mapping_config(logfile)
        }
    };
}

///Key Mapping or creates default Xbox Gamepad Mapping
pub(crate) fn key_mapping_config(logfile: &mut File) -> Result<KeyMap, String>{
    let mut file = read_file(KEYMAPPINGNAME);
    let mut mapping = vec![];
    let mut bufread = BufReader::new(file);
    match bufread.read_to_end(&mut mapping){
        Ok(_) => {}
        Err(e) => println!("{}", e)
    };
    let a = match String::from_utf8(mapping){
        Ok(t) => t,
        Err(e) => return Err(e.to_string()),
    };
    return match toml::from_str::<KeyMap>(&*a) {
        Ok(map) => {
            if map.mapping.len() == 0 {
                File::create(KEYMAPPINGNAME).ok();
                file = open_file(KEYMAPPINGNAME);
                let mut new_map = KeyMap {
                    mapping: HashMap::new()
                };
                let mut hashy_map = HashMap::new();
                //Push Xbox Gamepad Mappings.
                hashy_map.insert(gamepad_button_number(Button::RightShoulder), KeyList{ key_list: vec![18] });
                hashy_map.insert(gamepad_button_number(Button::LeftShoulder), KeyList{ key_list: vec![17] });
                hashy_map.insert(gamepad_button_number(Button::DPadUp), KeyList{ key_list: vec![82] });    //D-Pad Up
                hashy_map.insert(gamepad_button_number(Button::DPadDown), KeyList{ key_list: vec![84] });  //D-Pad Down
                hashy_map.insert(gamepad_button_number(Button::Start), KeyList{ key_list:vec![220, 66] }); //Start
                hashy_map.insert(gamepad_button_number(Button::Back), KeyList{ key_list:vec![220, 9] }); //Second Center Button left
                hashy_map.insert(gamepad_button_number(Button::X), KeyList{ key_list: vec![88] }); //X
                new_map.mapping.insert("030000005e040000ff02000000007200".to_string(), hashy_map);
                let serialized = match toml::to_string_pretty(&new_map) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(e.to_string())
                    }
                };
                file.write(&*serialized.into_bytes()).ok();
                key_mapping_config(logfile)
            } else {
                //Lmao, your own fault if you fail something in the mapping. But if its valid, its valid.
                Ok(map)
            }
        },
        Err(_) => {
            log_write(logfile,"Error", &format!("Invalid Mapping. {} has been reset.",KEYMAPPINGNAME));
            File::create(KEYMAPPINGNAME).ok();
            file = open_file(KEYMAPPINGNAME);
            let mut new_map = KeyMap {
                mapping: HashMap::new()
            };
            let mut hashy_map = HashMap::new();
            hashy_map.insert(gamepad_button_number(Button::RightShoulder), KeyList{ key_list: vec![18] });
            hashy_map.insert(gamepad_button_number(Button::LeftShoulder), KeyList{ key_list: vec![17] });
            hashy_map.insert(gamepad_button_number(Button::DPadUp), KeyList{ key_list: vec![82] });    //D-Pad Up
            hashy_map.insert(gamepad_button_number(Button::DPadDown), KeyList{ key_list: vec![84] });  //D-Pad Down
            hashy_map.insert(gamepad_button_number(Button::Start), KeyList{ key_list:vec![220, 66] }); //Start
            hashy_map.insert(gamepad_button_number(Button::Back), KeyList{ key_list:vec![220, 9] }); //Second Center Button left
            hashy_map.insert(gamepad_button_number(Button::X), KeyList{ key_list: vec![88] }); //X
            /*
            List to Map:
            Speed up: R,
            Speed down: T,
            Cycle Camera?: Insert,
            Ascend: Space,
            Incrase altitude: Alt+ Space,
            Descend: C,
            Decrease Altitude: Alt+ C,
            Alt,
            Landing Gear: G,
            Antigravity: Alt+G,
            Booster: B,
            Cycle vector display: X,
            Initiate Warp: Alt+J,
            Board/Dock to closest: Alt+T,
            Deboard/Undock: Alt+Z,m
            Option 1-9: Alt+1-9
            Toggle Marks: V,
            */
            new_map.mapping.insert("030000005e040000ff02000000007200".to_string(), hashy_map);
            let serialized = match toml::to_string_pretty(&new_map) {
                Ok(t) => t,
                Err(e) => {
                    return Err(e.to_string())
                }
            };
            match toml::from_str::<KeyMap>(&*serialized){
                Ok(t) => t,
                Err(e) => return Err(e.to_string())
            };
            file.write(&*serialized.as_bytes()).ok();
            key_mapping_config(logfile)
        }
    };
}