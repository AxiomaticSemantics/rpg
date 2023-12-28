use clap::{error::*, Args, CommandFactory, Parser, Subcommand, ValueEnum};

use crate::app::AppState;

/*
use std::str::FromStr;

use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseVecError;

impl FromStr for MyVec3 {
    type Err = ParseVecError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec: Vec<f32> = s
            .split(',')
            .map(|v| v.parse::<f32>().unwrap_or_default())
            .collect::<Vec<f32>>();
        //.ok_or(ParseVecError)?;

        Ok(MyVec3 { vec })
    }
}*/

#[derive(Parser, Debug)]
#[command(about = None, long_about = None, version = None, no_binary_name(true))]
pub struct ConsoleCommands {
    #[command(subcommand)]
    commands: ConsoleSubCommands,
}

#[derive(Subcommand, Debug)]
pub enum ConsoleSubCommands {
    /// Control the in-game directional light
    Light(Light),
    /// Control the in-game time
    Time(Time),
    /// Control the game state
    State(State),
    /// Toggle the in-game pause state
    Pause(GamePause),
    /// Adjusts volume.
    Volume(Volume),
    /// Available commands are: light, time, state
    Help,
}

#[derive(Args, Clone, Debug)]
pub struct Time {
    #[arg(value_enum, num_args(1), short, long)]
    set: Option<f32>,
}

#[derive(Args, Clone, Debug)]
pub struct Light {
    #[arg(num_args(0..=4), allow_negative_numbers(true), value_delimiter = ',', short, long)]
    rot: Option<Vec<f32>>,
    #[arg(num_args(0..=3), allow_negative_numbers(true), value_delimiter = ',', short, long)]
    pos: Option<Vec<f32>>,
    #[arg(num_args(0..=4), value_delimiter = ',', short, long)]
    color: Option<Vec<f32>>,
}

#[derive(Args, Clone, Debug)]
pub struct State {
    #[arg(value_enum, num_args(1), short, long)]
    set: Option<AppState>,
}

#[derive(Args, Clone, Debug)]
pub struct GamePause {
    //#[arg(num_args(0))]
    //set: Option<bool>,
}

#[derive(Args, Clone, Debug)]
pub struct Volume {
    #[arg(num_args(0..=1), short, long)]
    global: Option<f32>,
}
