// TODO: Break the code that controls the radio into separate libs
// BEGIN includes for radio controller functionality
extern crate sysfs_gpio;
use sysfs_gpio::{Direction, Edge, Pin};
// END includes for radio controller functionality

use std::io;
use std::io::Write;

// Let's begin our definitions of the objects we'll use to control the LORA radios



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

	println!("Monitoring pin 0");
	match interrupt(0) {
		Ok(()) => println!("Interrupting Complete!"),
		Err(err) => println!("Error: {}", err),
}
	println!("Monitoring finished!");

}
