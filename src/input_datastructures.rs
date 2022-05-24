use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Formatter};
use std::num::ParseIntError;
use std::ops::Neg;
use std::str::FromStr;
use std::sync::mpsc::Sender;


use stick::{Controller, Event, Listener};
use std::task::Poll;
use std::task::Poll::{Pending};
use enigo::{Enigo, Key, KeyboardControllable, MouseControllable};
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};
use toml::Value;

pub(crate) struct GameInputs{
    pub(crate) listener: Listener,
    pub(crate) controllers: Vec<Controller>,
    pub(crate) debug: bool,
    pub(crate) transmitter: Sender<(u64, Event)>
}

type Exit = usize;

impl GameInputs{
    pub(crate) fn connect(&mut self, controller: Controller) -> Poll<Exit> {
        println!(
            "Connected id: {}, name: {}",
            controller.id(),
            controller.name(),
        );
        self.controllers.push(controller);
        Pending
    }
    pub(crate) fn event_match(&mut self, id: usize, event: Event) -> Poll<Exit>{
        if self.debug{
            println!("Device #{}: {}", id, event);
        }
        self.transmitter.send((id as u64, event));
        Pending
    }
}


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
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum EventWrap{
    Disconnect,
    Exit,
    ActionA,
    ActionB,
    ActionC,
    ActionH,
    ActionV,
    ActionD,
    MenuL,
    MenuR,
    Joy,
    Cam,
    BumperL,
    BumperR,
    TriggerL,
    TriggerR,
    Up,
    Down,
    Left,
    Right,
    PovUp,
    PovDown,
    PovLeft,
    PovRight,
    HatUp,
    HatDown,
    HatLeft,
    HatRight,
    TrimUp,
    TrimDown,
    TrimLeft,
    TrimRight,
    MicUp,
    MicDown,
    MicLeft,
    MicRight,
    JoyX,
    JoyY,
    JoyZ,
    CamX,
    CamY,
    CamZ,
    Slew,
    Throttle,
    ThrottleL,
    ThrottleR,
    Volume,
    Wheel,
    Rudder,
    Gas,
    Brake,
    MicPush,
    Trigger,
    Bumper,
    ActionM,
    ActionL,
    ActionR,
    Pinky,
    PinkyForward,
    PinkyBackward,
    FlapsUp,
    FlapsDown,
    BoatForward,
    BoatBackward,
    AutopilotPath,
    AutopilotAlt,
    EngineMotorL,
    EngineMotorR,
    EngineFuelFlowL,
    EngineFuelFlowR,
    EngineIgnitionL,
    EngineIgnitionR,
    SpeedbrakeBackward,
    SpeedbrakeForward,
    ChinaBackward,
    ChinaForward,
    Apu,
    RadarAltimeter,
    LandingGearSilence,
    Eac,
    AutopilotToggle,
    ThrottleButton,
    MouseX,
    MouseY,
    Mouse,
    Number,
    PaddleLeft,
    PaddleRight,
    PinkyLeft,
    PinkyRight,
    Context,
    Dpi,
    ScrollX,
    ScrollY,
    Scroll,
    ActionWheelX,
    ActionWheelY,
}

pub(crate) fn wrap_events(event: Event) -> Result<(EventWrap, Option<Value>), ()>{
    Ok(match event{
        Event::Disconnect => (EventWrap::Disconnect, None),
        Event::Exit(value) => (EventWrap::Exit, Some(Value::from(value))),
        Event::ActionA(value) => (EventWrap::ActionA,Some(Value::from(value))),
        Event::ActionB(value) => (EventWrap::ActionB,Some(Value::from(value))),
        Event::ActionC(value) => (EventWrap::ActionC,Some(Value::from(value))),
        Event::ActionH(value) => (EventWrap::ActionH,Some(Value::from(value))),
        Event::ActionV(value) => (EventWrap::ActionV,Some(Value::from(value))),
        Event::ActionD(value) => (EventWrap::ActionD,Some(Value::from(value))),
        Event::MenuL(value) => (EventWrap::MenuL, Some(Value::from(value))),
        Event::MenuR(value) => (EventWrap::MenuR, Some(Value::from(value))),
        Event::Joy(value) => (EventWrap::Joy,Some(Value::from(value))),
        Event::Cam(value) => (EventWrap::Cam,Some(Value::from(value))),
        Event::BumperL(value) => (EventWrap::BumperL,Some(Value::from(value ))),
        Event::BumperR(value) => (EventWrap::BumperR,Some(Value::from(value ))),
        Event::TriggerL(value) => (EventWrap::TriggerL,Some(Value::from(value ))),
        Event::TriggerR(value) => (EventWrap::TriggerR,Some(Value::from(value ))),
        Event::Up(value) => (EventWrap::Up,Some(Value::from(value))),
        Event::Down(value) => (EventWrap::Down,Some(Value::from(value ))),
        Event::Left(value) => (EventWrap::Left,Some(Value::from(value))),
        Event::Right(value) => (EventWrap::Right,Some(Value::from(value))),
        Event::PovUp(value) => (EventWrap::PovUp,Some(Value::from(value))),
        Event::PovDown(value) => (EventWrap::PovDown,Some(Value::from(value))),
        Event::PovLeft(value) => (EventWrap::PovLeft,Some(Value::from(value))),
        Event::PovRight(value) => (EventWrap::PovRight,Some(Value::from(value))),
        Event::HatUp(value) => (EventWrap::HatUp,Some(Value::from(value))),
        Event::HatDown(value) => (EventWrap::HatDown,Some(Value::from(value))),
        Event::HatLeft(value) => (EventWrap::HatLeft,Some(Value::from(value))),
        Event::HatRight(value) => (EventWrap::HatRight,Some(Value::from(value))),
        Event::TrimUp(value) => (EventWrap::TrimUp,Some(Value::from(value))),
        Event::TrimDown(value) => (EventWrap::TrimDown,Some(Value::from(value))),
        Event::TrimLeft(value) => (EventWrap::TrimLeft,Some(Value::from(value))),
        Event::TrimRight(value) => (EventWrap::TrimRight,Some(Value::from(value))),
        Event::MicUp(value) => (EventWrap::MicUp,Some(Value::from(value))),
        Event::MicDown(value) => (EventWrap::MicDown,Some(Value::from(value))),
        Event::MicLeft(value) => (EventWrap::MicLeft,Some(Value::from(value))),
        Event::MicRight(value) => (EventWrap::MicRight,Some(Value::from(value))),
        Event::JoyX(value) => (EventWrap::JoyX,Some(Value::from(value))),
        Event::JoyY(value) => (EventWrap::JoyY,Some(Value::from(value))),
        Event::JoyZ(value) => (EventWrap::JoyZ,Some(Value::from(value))),
        Event::CamX(value) => (EventWrap::CamX,Some(Value::from(value))),
        Event::CamY(value) => (EventWrap::CamY,Some(Value::from(value))),
        Event::CamZ(value) => (EventWrap::CamZ,Some(Value::from(value))),
        Event::Slew(value) => (EventWrap::Slew,Some(Value::from(value))),
        Event::Throttle(value) => (EventWrap::Throttle,Some(Value::from(value))),
        Event::ThrottleL(value) => (EventWrap::ThrottleL,Some(Value::from(value))),
        Event::ThrottleR(value) => (EventWrap::ThrottleR,Some(Value::from(value))),
        Event::Volume(value) => (EventWrap::Volume,Some(Value::from(value))),
        Event::Wheel(value) => (EventWrap::Wheel,Some(Value::from(value))),
        Event::Rudder(value) => (EventWrap::Rudder,Some(Value::from(value))),
        Event::Gas(value) => (EventWrap::Gas,Some(Value::from(value))),
        Event::Brake(value) => (EventWrap::Brake,Some(Value::from(value))),
        Event::MicPush(value) => (EventWrap::MicPush,Some(Value::from(value))),
        Event::Trigger(value) => (EventWrap::Trigger,Some(Value::from(value))),
        Event::Bumper(value) => (EventWrap::Bumper,Some(Value::from(value))),
        Event::ActionM(value) => (EventWrap::ActionM,Some(Value::from(value))),
        Event::ActionL(value) => (EventWrap::ActionL,Some(Value::from(value))),
        Event::ActionR(value) => (EventWrap::ActionR,Some(Value::from(value))),
        Event::Pinky(value) => (EventWrap::Pinky,Some(Value::from(value))),
        Event::PinkyForward(value) => (EventWrap::PinkyForward,Some(Value::from(value))),
        Event::PinkyBackward(value) => (EventWrap::PinkyBackward,Some(Value::from(value))),
        Event::FlapsUp(value) => (EventWrap::FlapsUp,Some(Value::from(value))),
        Event::FlapsDown(value) => (EventWrap::FlapsDown,Some(Value::from(value))),
        Event::BoatForward(value) => (EventWrap::BoatForward,Some(Value::from(value))),
        Event::BoatBackward(value) => (EventWrap::BoatBackward,Some(Value::from(value))),
        Event::AutopilotPath(value) => (EventWrap::AutopilotPath,Some(Value::from(value))),
        Event::AutopilotAlt(value) => (EventWrap::AutopilotAlt,Some(Value::from(value))),
        Event::EngineMotorL(value) => (EventWrap::EngineMotorL,Some(Value::from(value))),
        Event::EngineMotorR(value) => (EventWrap::EngineMotorR, Some(Value::from(value))),
        Event::EngineFuelFlowL(value) => (EventWrap::EngineFuelFlowL,Some(Value::from(value))),
        Event::EngineFuelFlowR(value) => (EventWrap::EngineFuelFlowR,Some(Value::from(value))),
        Event::EngineIgnitionL(value) => (EventWrap::EngineIgnitionL,Some(Value::from(value))),
        Event::EngineIgnitionR(value) => (EventWrap::EngineIgnitionR,Some(Value::from(value))),
        Event::SpeedbrakeBackward(value) => (EventWrap::SpeedbrakeBackward,Some(Value::from(value))),
        Event::SpeedbrakeForward(value) => (EventWrap::SpeedbrakeForward,Some(Value::from(value))),
        Event::ChinaBackward(value) => (EventWrap::ChinaBackward,Some(Value::from(value))),
        Event::ChinaForward(value) => (EventWrap::ChinaForward,Some(Value::from(value))),
        Event::Apu(value) => (EventWrap::Apu,Some(Value::from(value))),
        Event::RadarAltimeter(value) => (EventWrap::RadarAltimeter,Some(Value::from(value))),
        Event::LandingGearSilence(value) => (EventWrap::LandingGearSilence,Some(Value::from(value))),
        Event::Eac(value) => (EventWrap::Eac,Some(Value::from(value))),
        Event::AutopilotToggle(value) => (EventWrap::AutopilotToggle,Some(Value::from(value))),
        Event::ThrottleButton(value) => (EventWrap::ThrottleButton,Some(Value::from(value))),
        Event::MouseX(value) => (EventWrap::MouseX,Some(Value::from(value))),
        Event::MouseY(value) => (EventWrap::MouseY,Some(Value::from(value))),
        Event::Mouse(value) => (EventWrap::Mouse,Some(Value::from(value))),
        //TODO: Unnamed programmable buttons are not supported yet
        Event::Number(identifier, value) => (EventWrap::Number,Some(Value::from(value))),
        Event::PaddleLeft(value) => (EventWrap::PaddleLeft, Some(Value::from(value))),
        Event::PaddleRight(value) => (EventWrap::PaddleRight, Some(Value::from(value))),
        Event::PinkyLeft(value) => (EventWrap::PinkyLeft, Some(Value::from(value))),
        Event::PinkyRight(value) => (EventWrap::PinkyRight, Some(Value::from(value))),
        Event::Context(value) => (EventWrap::Context, Some(Value::from(value))),
        Event::Dpi(value) => (EventWrap::Dpi, Some(Value::from(value))),
        Event::ScrollX(value) => (EventWrap::ScrollX, Some(Value::from(value))),
        Event::ScrollY(value) => (EventWrap::ScrollY, Some(Value::from(value))),
        Event::Scroll(value) => (EventWrap::Scroll, Some(Value::from(value))),
        Event::ActionWheelX(value) => (EventWrap::ActionWheelX, Some(Value::from(value))),
        Event::ActionWheelY(value) => (EventWrap::ActionWheelY, Some(Value::from(value))),
        _ => return Err(()),
    })
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AxisMap {
    #[serde_as(as = "HashMap<DisplayFromStr, HashMap<DisplayFromStr, DisplayFromStr>>")]
    pub(crate) mapping: HashMap<u64, HashMap<EventWrap, AxisVariations>>
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct KeyMap {
    #[serde_as(as = "HashMap<DisplayFromStr, HashMap<DisplayFromStr, DisplayFromStr>>")]
    pub(crate) mapping: HashMap<u64, HashMap<EventWrap, KeyList>>
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

impl fmt::Display for EventWrap{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}",match self{
            EventWrap::Disconnect =>     "Disconnect",
            EventWrap::Exit =>           "Exit",
            EventWrap::ActionA =>        "ActionA",
            EventWrap::ActionB =>        "ActionB",
            EventWrap::ActionC =>        "ActionC",
            EventWrap::ActionH =>        "ActionH",
            EventWrap::ActionV =>        "ActionV",
            EventWrap::ActionD =>        "ActionD",
            EventWrap::MenuL =>          "MenuL",
            EventWrap::MenuR =>          "MenuR",
            EventWrap::Joy =>            "Joy",
            EventWrap::Cam =>            "Cam",
            EventWrap::BumperL =>        "BumperL",
            EventWrap::BumperR =>        "BumperR",
            EventWrap::TriggerL =>       "TriggerL",
            EventWrap::TriggerR =>       "TriggerR",
            EventWrap::Up =>             "Up" ,
            EventWrap::Down =>           "Down" ,
            EventWrap::Left =>           "Left"  ,
            EventWrap::Right =>          "Right"  ,
            EventWrap::PovUp =>          "PovUp"  ,
            EventWrap::PovDown =>        "PovDown" ,
            EventWrap::PovLeft =>        "PovLeft"  ,
            EventWrap::PovRight =>       "PovRight" ,
            EventWrap::HatUp =>          "HatUp" ,
            EventWrap::HatDown =>        "HatDown",
            EventWrap::HatLeft =>        "HatLeft" ,
            EventWrap::HatRight =>       "HatRight" ,
            EventWrap::TrimUp =>         "TrimUp",
            EventWrap::TrimDown =>       "TrimDown",
            EventWrap::TrimLeft =>       "TrimLeft" ,
            EventWrap::TrimRight =>      "TrimRight",
            EventWrap::MicUp =>          "MicUp"  ,
            EventWrap::MicDown =>        "MicDown",
            EventWrap::MicLeft =>        "MicLeft" ,
            EventWrap::MicRight =>       "MicRight",
            EventWrap::JoyX =>           "JoyX"  ,
            EventWrap::JoyY =>           "JoyY" ,
            EventWrap::JoyZ =>           "JoyZ",
            EventWrap::CamX =>           "CamX" ,
            EventWrap::CamY =>           "CamY",
            EventWrap::CamZ =>           "CamZ",
            EventWrap::Slew =>           "Slew",
            EventWrap::Throttle =>       "Throttle" ,
            EventWrap::ThrottleL =>      "ThrottleL", 
            EventWrap::ThrottleR =>      "ThrottleR", 
            EventWrap::Volume =>         "Volume"        ,
            EventWrap::Wheel =>          "Wheel"         ,
            EventWrap::Rudder =>         "Rudder"        ,
            EventWrap::Gas =>            "Gas"           ,
            EventWrap::Brake =>          "Brake"         ,
            EventWrap::MicPush =>        "MicPush"       ,
            EventWrap::Trigger =>        "Trigger"       ,
            EventWrap::Bumper =>         "Bumper"        ,
            EventWrap::ActionM =>        "ActionM"       ,
            EventWrap::ActionL =>        "ActionL"       ,
            EventWrap::ActionR =>        "ActionR"       ,
            EventWrap::Pinky =>          "Pinky"         ,
            EventWrap::PinkyForward =>   "PinkyForward"  ,
            EventWrap::PinkyBackward =>  "PinkyBackward" ,
            EventWrap::FlapsUp =>        "FlapsUp"       ,
            EventWrap::FlapsDown =>      "FlapsDown"     ,
            EventWrap::BoatForward =>    "BoatForward"   ,
            EventWrap::BoatBackward =>   "BoatBackward"  ,
            EventWrap::AutopilotPath =>  "AutopilotPath" ,
            EventWrap::AutopilotAlt =>   "AutopilotAlt"  ,
            EventWrap::EngineMotorL =>   "EngineMotorL"  ,
            EventWrap::EngineMotorR =>   "EngineMotorR"  ,
            EventWrap::EngineFuelFlowL =>   "EngineFuelFlowL",
            EventWrap::EngineFuelFlowR =>   "EngineFuelFlowR",
            EventWrap::EngineIgnitionL =>   "EngineIgnitionL",
            EventWrap::EngineIgnitionR =>   "EngineIgnitionR",
            EventWrap::SpeedbrakeBackward =>"SpeedbrakeBackward",
            EventWrap::SpeedbrakeForward => "SpeedbrakeForward" ,
            EventWrap::ChinaBackward =>     "ChinaBackward"     ,
            EventWrap::ChinaForward =>      "ChinaForward"      ,
            EventWrap::Apu =>               "Apu"               ,
            EventWrap::RadarAltimeter =>    "RadarAltimeter"    ,
            EventWrap::LandingGearSilence =>"LandingGearSilence",
            EventWrap::Eac =>               "Eac"               ,
            EventWrap::AutopilotToggle =>   "AutopilotToggle"   ,
            EventWrap::ThrottleButton =>    "ThrottleButton"    ,
            EventWrap::MouseX =>            "MouseX",
            EventWrap::MouseY =>            "MouseY",
            EventWrap::Mouse =>             "Mouse" ,
            EventWrap::Number =>            "Number",
            EventWrap::PaddleLeft =>        "PaddleLeft" ,
            EventWrap::PaddleRight =>       "PaddleRight",
            EventWrap::PinkyLeft =>         "PinkyLeft"  ,
            EventWrap::PinkyRight =>        "PinkyRight" ,
            EventWrap::Context =>           "Context",
            EventWrap::Dpi =>               "Dpi"    ,
            EventWrap::ScrollX =>           "ScrollX",
            EventWrap::ScrollY =>           "ScrollY",
            EventWrap::Scroll =>            "Scroll" ,
            EventWrap::ActionWheelX =>      "ActionWheelX",
            EventWrap::ActionWheelY =>      "ActionWheelY"
        })
    }
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
#[derive(Debug)]
pub(crate) struct EventWrapFromStrError;
impl fmt::Display for EventWrapFromStrError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for EventWrap{
    type Err = EventWrapFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s{
            "Disconnect"   => EventWrap::Disconnect   ,
            "Exit"         => EventWrap::Exit         ,
            "ActionA"      => EventWrap::ActionA      ,
            "ActionB"      => EventWrap::ActionB      ,
            "ActionC"      => EventWrap::ActionC      ,
            "ActionH"      => EventWrap::ActionH      ,
            "ActionV"      => EventWrap::ActionV      ,
            "ActionD"      => EventWrap::ActionD      ,
            "MenuL"        => EventWrap::MenuL        ,
            "MenuR"        => EventWrap::MenuR        ,
            "Joy"          => EventWrap::Joy          ,
            "Cam"          => EventWrap::Cam          ,
            "BumperL"      => EventWrap::BumperL      ,
            "BumperR"      => EventWrap::BumperR      ,
            "TriggerL"     => EventWrap::TriggerL     ,
            "TriggerR"     => EventWrap::TriggerR     ,
            "Up"           => EventWrap::Up           ,
            "Down"         => EventWrap::Down         ,
            "Left"         => EventWrap::Left         ,
            "Right"        => EventWrap::Right        ,
            "PovUp"        => EventWrap::PovUp        ,
            "PovDown"      => EventWrap::PovDown      ,
            "PovLeft"      => EventWrap::PovLeft      ,
            "PovRight"     => EventWrap::PovRight     ,
            "HatUp"        => EventWrap::HatUp        ,
            "HatDown"      => EventWrap::HatDown      ,
            "HatLeft"      => EventWrap::HatLeft      ,
            "HatRight"     => EventWrap::HatRight     ,
            "TrimUp"       => EventWrap::TrimUp       ,
            "TrimDown"     => EventWrap::TrimDown     ,
            "TrimLeft"     => EventWrap::TrimLeft     ,
            "TrimRight"    => EventWrap::TrimRight    ,
            "MicUp"        => EventWrap::MicUp        ,
            "MicDown"      => EventWrap::MicDown      ,
            "MicLeft"      => EventWrap::MicLeft      ,
            "MicRight"     => EventWrap::MicRight     ,
            "JoyX"         => EventWrap::JoyX         ,
            "JoyY"         => EventWrap::JoyY         ,
            "JoyZ"         => EventWrap::JoyZ         ,
            "CamX"         => EventWrap::CamX         ,
            "CamY"         => EventWrap::CamY         ,
            "CamZ"         => EventWrap::CamZ         ,
            "Slew"         => EventWrap::Slew         ,
            "Throttle"     => EventWrap::Throttle     ,
            "ThrottleL"    => EventWrap::ThrottleL    ,
            "ThrottleR"    => EventWrap::ThrottleR    ,
            "Volume"       => EventWrap::Volume       ,
            "Wheel"        => EventWrap::Wheel        ,
            "Rudder"       => EventWrap::Rudder       ,
            "Gas"          => EventWrap::Gas          ,
            "Brake"        => EventWrap::Brake        ,
            "MicPush"      => EventWrap::MicPush      ,
            "Trigger"      => EventWrap::Trigger      ,
            "Bumper"       => EventWrap::Bumper       ,
            "ActionM"      => EventWrap::ActionM      ,
            "ActionL"      => EventWrap::ActionL      ,
            "ActionR"      => EventWrap::ActionR      ,
            "Pinky"        => EventWrap::Pinky        ,
            "PinkyForward" => EventWrap::PinkyForward ,
            "PinkyBackward"=> EventWrap::PinkyBackward,
            "FlapsUp"      => EventWrap::FlapsUp      ,
            "FlapsDown"    => EventWrap::FlapsDown    ,
            "BoatForward"  => EventWrap::BoatForward  ,
            "BoatBackward" => EventWrap::BoatBackward ,
            "AutopilotPath"=> EventWrap::AutopilotPath,
            "AutopilotAlt" => EventWrap::AutopilotAlt ,
            "EngineMotorL" => EventWrap::EngineMotorL ,
            "EngineMotorR" => EventWrap::EngineMotorR ,
            "EngineFuelFlowL"   => EventWrap::EngineFuelFlowL   ,
            "EngineFuelFlowR"   => EventWrap::EngineFuelFlowR   ,
            "EngineIgnitionL"   => EventWrap::EngineIgnitionL   ,
            "EngineIgnitionR"   => EventWrap::EngineIgnitionR   ,
            "SpeedbrakeBackward"=> EventWrap::SpeedbrakeBackward,
            "SpeedbrakeForward" => EventWrap::SpeedbrakeForward ,
            "ChinaBackward"     => EventWrap::ChinaBackward     ,
            "ChinaForward"      => EventWrap::ChinaForward      ,
            "Apu"               => EventWrap::Apu               ,
            "RadarAltimeter"    => EventWrap::RadarAltimeter    ,
            "LandingGearSilence"=> EventWrap::LandingGearSilence,
            "Eac"               => EventWrap::Eac               ,
            "AutopilotToggle"   => EventWrap::AutopilotToggle   ,
            "ThrottleButton"    => EventWrap::ThrottleButton    ,
            "MouseX"            => EventWrap::MouseX            ,
            "MouseY"            => EventWrap::MouseY            ,
            "Mouse"             => EventWrap::Mouse             ,
            "Number"            => EventWrap::Number            ,
            "PaddleLeft"        => EventWrap::PaddleLeft        ,
            "PaddleRight"       => EventWrap::PaddleRight       ,
            "PinkyLeft"         => EventWrap::PinkyLeft         ,
            "PinkyRight"        => EventWrap::PinkyRight        ,
            "Context"           => EventWrap::Context           ,
            "Dpi"               => EventWrap::Dpi               ,
            "ScrollX"           => EventWrap::ScrollX           ,
            "ScrollY"           => EventWrap::ScrollY           ,
            "Scroll"            => EventWrap::Scroll            ,
            "ActionWheelX"      => EventWrap::ActionWheelX      ,
            "ActionWheelY"      => EventWrap::ActionWheelY,
            _ => return Err(EventWrapFromStrError)
        })
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
            println!("Axis: {}, Value: {}", axis, value);
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
    total_key_state: HashSet<u16>
}

struct KeyboardPlaceholder {}

impl KeyPressHandler{
    pub fn new() -> KeyPressHandler{
       KeyPressHandler{
           current_key_state: HashSet::new(),
           current_off_key_state: HashSet::new(),
           total_key_state: HashSet::new()
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
