// TODO: Break the code that controls the radio into separate libs
// BEGIN includes for radio controller functionality
extern crate sysfs_gpio;
use sysfs_gpio::{Direction, Edge, Pin};
// END includes for radio controller functionality

//use std::io;
//use std::io::Write;

// Constants for control of our radio modules
#[allow(dead_code)]
enum DriverMode {
	General,
	Wakeup,
	PowerSaving,
	Sleep
}

// Let's begin our definitions of the objects we'll use to control the LORA radios
#[allow(dead_code)]
struct Driver {
	// Control pin definitions
	m0: Pin,
	m1: Pin,
	aux: Pin,
	mode: DriverMode
}

// Driver functions
#[allow(dead_code)]
impl Driver {

	// Driver should be instantiated with the 3 main pins to control the radio. M0, M1, and AUX.
	pub fn new(m0_pin_num: u64, m1_pin_num: u64, aux_pin_num: u64) -> Driver {
		let m0_pin = Pin::new(m0_pin_num);
		let m1_pin = Pin::new(m1_pin_num);
		let aux_pin = Pin::new(aux_pin_num);

		//TODO: Find a better way to do error handling
		match m0_pin.set_direction(Direction::Out) {
			Ok(()) => println!("M0 set correctly"),
			Err(e) => panic!("{}, correct gpio pin?",e)
		}
		match m1_pin.set_direction(Direction::Out) {
			Ok(()) => println!("M1 set correctly"),
			Err(e) => println!("{}, correct gpio pin?",e)
		}
		match aux_pin.set_direction(Direction::In) {
			Ok(()) => println!("AUX set correctly"),
			Err(e) => println!("{}, correct gpio pin?",e)
		}
		match aux_pin.set_edge(Edge::RisingEdge) {
			Ok(()) => println!("AUX rising edge set correctly"),
			Err(e) => println!("{}, correct gpio pin?",e)
		}


		Driver{m0: Pin::new(m0_pin_num), 
			   m1: Pin::new(m1_pin_num), 
			   aux: Pin::new(aux_pin_num),
			   mode: DriverMode::Sleep
			}
	}

	// We might need to define our own error for this. Right ne we just panic if we never see the interrupt we're expecting
	fn wait_for_interrupt(&self, pin: sysfs_gpio::Pin, timeout: u32) {
		//let input = Pin::new(pin);
		//input.set_direction(Direction::In)?;
		//pin.set_edge(Edge::RisingEdge)?;
		let mut poller = pin.get_poller().unwrap();
		//If the pin is already high by the time we get here there will be an error
		while pin.get_value().unwrap() != 1 {
			match poller.poll(timeout as isize).unwrap() {
				Some(value) => println!("Aux interrupt: {}",value),
				None => print!(".")
			}
		}
	}

	pub fn get_control_gpio_pins(&self) -> (u64, u64, u64) {
		(self.m0.get_pin_num(), self.m1.get_pin_num(), self.aux.get_pin_num())
	}

// This function will simply panic if there is any error, because it isn't something we can continue operating with.
	pub fn set_mode(&self, mode: DriverMode) {

		let poll_wait_time_ms = 10;

		// Only set the new mode if aux is high
		if self.aux.get_value().unwrap() == 0 {
			// Wait for the rising edge of aux
			self.wait_for_interrupt(self.aux,poll_wait_time_ms);
		}
		
		match mode {
			DriverMode::General => { self.m0.set_value(0).unwrap(); self.m1.set_value(0).unwrap() },
			DriverMode::Wakeup => { self.m0.set_value(0).unwrap(); self.m1.set_value(1).unwrap() },
			DriverMode::PowerSaving => { self.m0.set_value(1).unwrap(); self.m1.set_value(0).unwrap() },
			DriverMode::Sleep => { self.m0.set_value(1).unwrap(); self.m1.set_value(1).unwrap() },
		}

		// According to the radio's doc, we need to wait at least 2ms after switching modes.
		self.wait_for_interrupt(self.aux,poll_wait_time_ms);
	}

}


fn main() {

	let e23_driver = Driver::new(1013,1015,1017);
	let (m0_pin, m1_pin, aux_pin) = e23_driver.get_control_gpio_pins();
	println!("M0: {} M1: {} AUX: {}", m0_pin, m1_pin, aux_pin);

	

	e23_driver.set_mode(DriverMode::General);
	println!("Mode set to general");
	e23_driver.set_mode(DriverMode::Wakeup);
	println!("Mode set to wakeup");
	e23_driver.set_mode(DriverMode::PowerSaving);
	println!("Mode set to powersaving");
	e23_driver.set_mode(DriverMode::Sleep);
	println!("Mode set to sleep");


}
