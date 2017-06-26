// TODO: Break the code that controls the radio into separate libs
// BEGIN includes for radio controller functionality
extern crate sysfs_gpio;
use sysfs_gpio::{Direction, Edge, Pin};
// END includes for radio controller functionality

use std::io;
use std::io::Write;

// Let's begin our definitions of the objects we'll use to control the LORA radios
struct Controller {
	// Control pin definitions
	m0: Pin,
	m1: Pin,
	aux: Pin
}

// Controller functions
impl Controller {

	// Controller should be instantiated with the 3 main pins to control the radio. M0, M1, and AUX.
	pub fn new(m0_pin_num: u64, m1_pin_num: u64, aux_pin_num: u64) -> Controller {
		let m0_pin = Pin::new(m0_pin_num);
		let m1_pin = Pin::new(m1_pin_num);
		let aux_pin = Pin::new(aux_pin_num);

		//TODO: Find a better way to do error handling
		match m0_pin.set_direction(Direction::Out) {
			Ok(()) => println!("good"),
			Err(e) => println!("{}",e)
		}
		match m1_pin.set_direction(Direction::Out) {
			Ok(()) => println!("good"),
			Err(e) => println!("{}",e)
		}
		match aux_pin.set_direction(Direction::Out) {
			Ok(()) => println!("good"),
			Err(e) => println!("{}",e)
		}


		Controller{m0: Pin::new(m0_pin_num), m1: Pin::new(m1_pin_num), aux: Pin::new(aux_pin_num)}
	}

	pub fn get_control_gpio_pins(&self) -> (u64, u64, u64) {
		(self.m0.get_pin_num(), self.m1.get_pin_num(), self.aux.get_pin_num())
	}
}



//Polling code. Use this later
fn interrupt(pin: u64) -> sysfs_gpio::Result<()>{
	let input = Pin::new(pin);
	input.set_direction(Direction::In)?;
	input.set_edge(Edge::FallingEdge)?;
	let mut poller = input.get_poller()?;

	loop {
		match poller.poll(1000)? {
			Some(value) => println!("{}", value),
			None => {
				let mut stdout = io::stdout();
				stdout.write_all(b".")?;
				stdout.flush()?;
			}
		}
	}
	

}

fn main() {

	/*
	println!("Monitoring pin 0");
	match interrupt(0) {
		Ok(()) => println!("Interrupting Complete!"),
		Err(err) => println!("Error: {}", err),
	}
	println!("Monitoring finished!");
	*/

	let e23_controller = Controller::new(1,2,3);
	let (m0_pin, m1_pin, aux_pin) = e23_controller.get_control_gpio_pins();
	println!("M0: {} M1: {} AUX: {}", m0_pin, m1_pin, aux_pin);

}
