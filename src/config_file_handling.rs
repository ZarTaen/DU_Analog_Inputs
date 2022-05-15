use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use config::{Config, FileFormat};
use config::File as File2;
use crate::helpers::open_file;
use crate::input_datastructures::{AxisVariations, AxisMap, EventWrap};
use crate::{log_write, read_file};

const DEFAULTCONFIGSTRING:&str = r#"device_pollrate = "250" #This is the pollrate per second to check on device state.
input_sendrate_division = "6" #This value tells how many polls to wait before sending an input to mouse.
device_number ="0" #This value is the device number mentioned for the device.
debug = true #This value prints debug information to the console to enumerate all input codes. Disable this once you finished your mapping.
sixaxis = true #This value toggles between sixaxis and 5axis encoding. Sixaxis is used when true"#;

const CONFIGNAME: &str = "config.toml";
const AXISMAPPINGNAME: &str = "axis_mapping.toml";

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
    let sets= vec!["device_pollrate","input_sendrate_division", "device_number", "debug", "sixaxis"];
    if settings.len()==0{
        file.write(DEFAULTCONFIGSTRING.as_bytes());
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
    match settings.get(sets[2]).unwrap().parse::<u64>(){
        Ok(_) => {}
        Err(e) => return Err(e.to_string())
    };
    match settings.get(sets[3]).unwrap().parse::<bool>(){
        Ok(_) => {}
        Err(e) => return Err(e.to_string())
    };
    match settings.get(sets[4]).unwrap().parse::<bool>(){
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
                File::create(AXISMAPPINGNAME);
                file = open_file(AXISMAPPINGNAME);
                let mut new_map = AxisMap {
                    mapping: HashMap::new()
                };
                //Push Xbox Gamepad Mappings.
                new_map.mapping.insert(EventWrap::JoyX, AxisVariations::XAxis1);
                new_map.mapping.insert(EventWrap::JoyY, AxisVariations::YAxis1);
                new_map.mapping.insert(EventWrap::CamX, AxisVariations::XAxis2);
                new_map.mapping.insert(EventWrap::CamY, AxisVariations::YAxis2);
                new_map.mapping.insert(EventWrap::TriggerL, AxisVariations::ZAxis2);
                new_map.mapping.insert(EventWrap::TriggerR, AxisVariations::ZAxis1);
                let serialized = match toml::to_string_pretty(&new_map) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(e.to_string())
                    }
                };
                file.write(&*serialized.into_bytes());
                axis_mapping_config(logfile)
            } else {
                //Lmao, your own fault if you fail something in the mapping. But if its valid, its valid.
                Ok(map)
            }
        },
        Err(_) => {
            log_write(logfile,"Error", &format!("Invalid Mapping. {} has been reset.",AXISMAPPINGNAME));
            File::create(AXISMAPPINGNAME);
            file = open_file(AXISMAPPINGNAME);
            let mut new_map = AxisMap {
                mapping: HashMap::new()
            };
            //Push Xbox Gamepad Mappings.
            new_map.mapping.insert(EventWrap::JoyX, AxisVariations::XAxis1);
            new_map.mapping.insert(EventWrap::JoyY, AxisVariations::YAxis1);
            new_map.mapping.insert(EventWrap::CamX, AxisVariations::XAxis2);
            new_map.mapping.insert(EventWrap::CamY, AxisVariations::YAxis2);
            new_map.mapping.insert(EventWrap::TriggerL, AxisVariations::LesserAxis1);
            new_map.mapping.insert(EventWrap::TriggerR, AxisVariations::LesserAxis2);
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
            file.write(&*serialized.as_bytes());
            axis_mapping_config(logfile)
        }
    };
}
