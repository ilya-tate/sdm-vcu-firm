#[derive(Debug)]
//Vehicle States
/*
state::Ready - Brakes pressed --> state::Standby
state::Standby - Brakes released --> state::Ready, Button pushed --> state::Forward
state::Forward - Button pushed off --> state::Standby, Overheated --> state::Fault
state::Fault - Functional --> state::Ready
 */
enum State{
    Ready,
    StandBy,
    Forward,
    Fault,
    Reverse,
    Self_Driving,
}
struct CAR{
    state: State,
    button_pressed: bool,
    brakes: bool,
    broken: bool
}

impl CAR{
    fn new() -> Self{
        CAR{
            state: State::Ready,
            button_pressed: false,
            brakes: false,
            broken: false
        }
    }
    //changes between state Ready and Standby based on brake sensor
    fn brakes_sensor(&mut self, brakes: bool) -> &Self{
        self.brakes=brakes;
        match self.state{
            State::Ready => {
                if brakes{
                    self.state = State::StandBy;
                    self
                }
                else{
                    self.state = State::Ready;
                    self
                }
            }
            State::StandBy => {
                if !brakes{
                    self.state = State::Ready;
                }
                self

            }
            //saying for any other change state stays the same
          _  =>self,
        }
    }
    //start or off button: moves between Standby and Forward
     fn button_sensor(&mut self, button_pressed: bool) -> &Self{
       self.button_pressed=button_pressed;
       match self.state{
        State::StandBy => {
            if button_pressed
            {
                self.state = State::Forward;
                
            }
            self
        }
        State::Forward => {
            if !button_pressed{
                self.state = State::StandBy;
                
            }
            if self.broken{
                self.state = State::Fault;
                
            }
            self
        }
        _ => self, 
       }

     }
     //if an error code presence 
     fn error_sensor(&mut self, broken:bool) -> &Self{
        self.broken = broken;
        match self.state{
            State::Fault => {
                if !broken{
                    self.state = State::Ready;
                    
                }
            }
               _ => {
                if broken{
                    self.state = State::Fault;
                }
            }
        }
        self
    } 
}

fn main(){
    let mut car = CAR::new();
    println!("The state of the Car is: {:?}", car.state );
    car.brakes_sensor(true);
    println!("The state of the Car is: {:?}", car.state );
    car.button_sensor(true);
    println!("The state of the Car is: {:?}", car.state );
    car.error_sensor(true);
    println!("The state of the Car is: {:?}", car.state );
    
}

