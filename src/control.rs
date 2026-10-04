pub mod state;
pub mod torque;

use state::State;

pub struct Inputs(/* pedal, brake, etc */);
pub struct Outputs(/* torque, etc */);

pub struct Controller() {
    state: State
}

impl 

