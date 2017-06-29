// TODO: Break the code that controls the radio into separate libs
// BEGIN includes for radio controller functionality
extern crate sysfs_gpio;
use sysfs_gpio::{Direction, Edge, Pin};
use std::fs::OpenOptions;
use std::io::Write;
use std::string::String;
use std::fmt;

extern crate bit_vec;
use bit_vec::BitVec;
// END includes for radio controller functionality

// Let's begin our definitions of the objects we'll use to control the LORA radios
#[allow(dead_code)]
struct Driver {
	// Control pin definitions
	m0: Pin,
	m1: Pin,
	aux: Pin,
	mode: RadioMode,
	tty_device: String
}

struct RadioConfig {
	head: u8,
	addh: u8,
	addl: u8,
	sped: u8,
	chan: u8,
	option: u8
}

impl RadioConfig {

	// create a new config with the default values from the doc
	pub fn new() -> RadioConfig {
		RadioConfig{ head: 192,	//C0x
					 addh: 18,	//12x
					 addl: 52,	//34x
					 sped: 24,	//18x
					 chan: 80,	//50x
					 option: 64	//40x
		}
	}
	pub fn raw(&self) -> [u8;6] {
		[self.head,self.addh,self.addl,self.sped,self.chan,self.option]
	}
}

// Constants for control of our radio modules
#[allow(dead_code)]
enum RadioMode {
	General,
	Wakeup,
	PowerSaving,
	Sleep
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
			   mode: RadioMode::Sleep,
			   tty_device: String::new()
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
	pub fn set_mode(&mut self, mode: RadioMode) {

		let poll_wait_time_ms = 10;

		// Only set the new mode if aux is high
		if self.aux.get_value().unwrap() == 0 {
			// Wait for the rising edge of aux
			self.wait_for_interrupt(self.aux,poll_wait_time_ms);
		}
		
		match mode {
			RadioMode::General => { self.m0.set_value(0).unwrap(); self.m1.set_value(0).unwrap() },
			RadioMode::Wakeup => { self.m0.set_value(0).unwrap(); self.m1.set_value(1).unwrap() },
			RadioMode::PowerSaving => { self.m0.set_value(1).unwrap(); self.m1.set_value(0).unwrap() },
			RadioMode::Sleep => { self.m0.set_value(1).unwrap(); self.m1.set_value(1).unwrap() },
		}

		// According to the radio's doc, we need to wait at least 2ms after switching modes.
		self.wait_for_interrupt(self.aux,poll_wait_time_ms);

		// Then set the mode variable in the driver struct
		self.mode = mode;
	}

	// Get a reference to the driver's mode
	pub fn get_mode(&self) -> &RadioMode {
		&self.mode
	}

	pub fn set_tty_device(&mut self, filepath: String) {
		self.tty_device = filepath;
	}

	// This function will write data out of the serial port to the console for now, but soon to the radio
	pub fn serial_write(&self, config: RadioConfig) {
		// generate our raw config to write out of the serial port
		let config_raw = config.raw();
		// We have to open a clone of the filename so as not to pass ownership to it.
		let mut file = OpenOptions::new().read(true).write(true).open(self.tty_device.clone()).unwrap();
		file.write_all(&config_raw).unwrap();
	}
}


fn main() {

	// we want it to be a mutable driver because we will be changing fields as we go.
	let mut e23_driver = Driver::new(1013,1015,1017);
	let (m0_pin, m1_pin, aux_pin) = e23_driver.get_control_gpio_pins();
	println!("M0: {} M1: {} AUX: {}", m0_pin, m1_pin, aux_pin);

	e23_driver.set_mode(RadioMode::Sleep);
	println!("Mode set to sleep");

	let testconfig = RadioConfig::new();

	e23_driver.set_tty_device(String::from("/dev/ttyS0"));
	e23_driver.serial_write(testconfig);

}
