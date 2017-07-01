// TODO: Break the code that controls the radio into separate libs
// BEGIN includes for radio controller functionality
extern crate sysfs_gpio;
use sysfs_gpio::{Direction, Edge, Pin};
//use std::fs::OpenOptions;
use std::io::Write;
use std::io::Read;
use std::string::String;
use std::thread::sleep;
use std::time::Duration;
//use std::process::Command;

extern crate bit_vec;
use bit_vec::BitVec;

extern crate serial;
use serial::prelude;
use serial::posix;
use serial::SerialDevice;
use serial::SerialPortSettings;
//use serial::SerialPort;
//use serial::SerialDevice;

// END includes for radio controller functionality
//extern crate serialport;
//use serialport::prelude::*;


// Let's begin our definitions of the objects we'll use to control the LORA radios

#[derive(Copy, Clone)]
struct RadioConfig {
	head: u8,
	addh: u8,
	addl: u8,
	sped: u8,
	chan: u8,
	option: u8
}

// Initially only the options to change important configuration data
// will be exposed, things like serial parity bit and module address high byte
// will be left out for now/until they're needed
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

	// This function outputs the actual binary data that needs to be sent out the UART connection.
	pub fn raw(&self) -> [u8;6] {
		[self.head,self.addh,self.addl,self.sped,self.chan,self.option]
	}

	pub fn set_serial_rate(&mut self, ser_speed: &str) {
		// These bits are where the serial rate config option lives.
		let config_bits_offsets = [5,4,3];
		let config_bits_values;
		// Needs to be matched as reference because rust
		match ser_speed {
			"1200" => {config_bits_values = [0,0,0]}, // bits alre already all 0, do nothing
			"2400" => {config_bits_values = [0,0,1]},
			"4800" => {config_bits_values = [0,1,0]},
			"9600" => {config_bits_values = [0,1,1]},
			"19200" => {config_bits_values = [1,0,0]},
			"38400" => {config_bits_values = [1,0,1]},
			"57600" => {config_bits_values = [1,1,0]},
			"115200" => {config_bits_values = [1,1,1]},
			_ => panic!("Incorrect serial speed specified, halting")
		}

		// Save our changes in settings back to the RadioConfig struct
		self.sped = self.change_bits(self.sped,&config_bits_offsets,&config_bits_values);
	}

	pub fn set_air_rate(&mut self, air_speed: &str) {
		// These bits are where the air rate config option lives.
		let config_bits_offsets = [2,1,0];
		let config_bits_values;
		// Needs to be matched as reference because rust
		match air_speed {
			"1k" => {config_bits_values = [0,0,0]}, // bits alre already all 0, do nothing
			"2k" => {config_bits_values = [0,0,1]},
			"5k" => {config_bits_values = [0,1,0]},
			"10k" => {config_bits_values = [0,1,1]},
			"12k" => {config_bits_values = [1,0,0]},
			"15k" => {config_bits_values = [1,0,1]},
			"20k" => {config_bits_values = [1,1,0]},
			"25k" => {config_bits_values = [1,1,1]},
			_ => panic!("Incorrect air speed specified, halting")
		}

		// Save our changes in settings back to the RadioConfig struct
		self.sped = self.change_bits(self.sped,&config_bits_offsets,&config_bits_values);
	}

	pub fn set_transmit_power(&mut self, transmit_power: &str) {
		// These bits are where the air rate config option lives.
		let config_bits_offsets = [2,1,0];
		let config_bits_values;
		// Needs to be matched as reference because rust
		match transmit_power {
			"20dBm" => {config_bits_values = [0,0,0]}, // bits alre already all 0, do nothing
			"17dBm" => {config_bits_values = [0,0,1]},
			"14dBm" => {config_bits_values = [0,1,0]},
			"11dBm" => {config_bits_values = [0,1,1]},
			"8dBm" => {config_bits_values = [1,0,0]},
			"5dBm" => {config_bits_values = [1,0,1]},
			"2.5dBm" => {config_bits_values = [1,1,0]},
			"0dBm" => {config_bits_values = [1,1,1]},
			_ => panic!("Incorrect transmit power specified, halting")
		}

		// Save our changes in settings back to the RadioConfig struct
		self.option = self.change_bits(self.option,&config_bits_offsets,&config_bits_values);
	}

	// original_byte - the command byte we want to modify
	// target_bits - the bits we want to modify, e.g. [7,6,5]
	// bits to write - the result we want, e.g. [false, true, true]
	fn change_bits(&self, original_byte: u8, target_bits: &[i8], bits_to_write: &[u8]) -> u8 {
		// DOC: 	[76543210]
		// RUST:	[01234567]
		let mut bv = BitVec::from_bytes(&[original_byte]);
		let mut bits_to_write_vec = BitVec::from_elem(bits_to_write.len(),false);
		// convert our passed arg to a bitvec
		for i in 0..bits_to_write.len() {
			match bits_to_write[i] {
				0 => bits_to_write_vec.set(i,false),
				1 => bits_to_write_vec.set(i,true),
				_ => panic!("Invalid bits passed to change_bits, halting")
			}
		}

		// make a local copy of the target bits
		let mut target_bits = target_bits.to_vec();
		// flip our target bits "endian-ness"
		for i in 0..target_bits.len() {
			target_bits[i] = (target_bits[i]-7).abs();
		}

		for i in 0..target_bits.len() {
			bv.set((target_bits[i] as usize),bits_to_write_vec[i]);
		}

		bv.to_bytes()[0]
	}
}

// Constants for control of our radio modules
#[allow(dead_code)]
#[derive(Clone)]
enum RadioMode {
	General,
	Wakeup,
	PowerSaving,
	Sleep
}

#[allow(dead_code)]
struct Driver {
	// Control pin definitions
	m0: Pin,
	m1: Pin,
	aux: Pin,
	mode: RadioMode,
	tty_device_string: String,
	tty_device: serial::SystemPort
}

// Driver functions
#[allow(dead_code)]
impl Driver {

	// Driver should be instantiated with the 3 main pins to control the radio. M0, M1, and AUX.
	pub fn new(m0_pin_num: u64, m1_pin_num: u64, aux_pin_num: u64, tty_str: &str) -> Driver {
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

		let mut port = serial::open(tty_str).unwrap();
		//port.configure(tty_settings).unwrap();
		port.set_timeout(Duration::from_secs(1)).unwrap();

		Driver{m0: Pin::new(m0_pin_num), 
			   m1: Pin::new(m1_pin_num), 
			   aux: Pin::new(aux_pin_num),
			   mode: RadioMode::Sleep,
			   tty_device_string: String::from(tty_str),
			   tty_device: port
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
				Some(value) => println!("Aux high: {}",value),
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

		self.wait_for_interrupt(self.aux,poll_wait_time_ms);
		// Wait at least 2 ms as per the datasheet
		sleep(Duration::from_millis(2));

		// Then set the mode variable in the driver struct
		self.mode = mode;
	}

	// Get a reference to the driver's mode
	pub fn get_mode(&self) -> &RadioMode {
		&self.mode
	}

	/*
	pub fn set_tty_device(&mut self, filepath: String) {
		self.tty_device_string = filepath;
	}
	*/

	
	fn set_tty_baud(&mut self, new_baud: serial::BaudRate) {
		let mut tty_settings = self.tty_device.read_settings().unwrap();
		tty_settings.set_baud_rate(new_baud).unwrap();
		// Set the new baud rate.
		self.tty_device.write_settings(&tty_settings).unwrap();
	}

	fn get_tty_baud(&self) -> serial::BaudRate {
		let tty_settings = self.tty_device.read_settings().unwrap();
		tty_settings.baud_rate().unwrap()
	}
	

	pub fn write_config(&mut self, config: RadioConfig) {

		// Declare our read buffer
		let mut read_buf: Vec<u8> = (0..6).collect();

		// Save the previous mode
		let prev_mode = self.mode.clone();
		self.set_mode(RadioMode::Sleep);

		// Wait at least 2 ms as per the datasheet
		sleep(Duration::from_millis(2));

		// We need to set the linux tty mode to 9200 baud, first saving the old
		//let mut tty_settings = self.tty_device.read_settings().unwrap();
		let orig_baud = self.get_tty_baud();
		//tty_settings.set_baud_rate(serial::Baud9600).unwrap();
		// Set the new baud rate.
		self.set_tty_baud(serial::Baud9600);


		// that is the speed that the device operates at in sleep mode
		self.serial_write(config.raw().as_ref());
		// verify what it returns before continuing
		let bytes_read = self.serial_read(&mut read_buf);
		println!("Config: read {} bytes in response", bytes_read);
		// Return to the mode we were in previously
		self.set_mode(prev_mode);

		// If the configs aren't the same, something went wrong and we need to quit
		if read_buf != config.raw() {
			panic!("Config wasn't written successfully! {:?} vs {:?}",bytes_read,config.raw());
		}
		println!("Config written successfully");

		//Return the device baud rate to the original
		self.set_tty_baud(orig_baud);
	}

	pub fn serial_write(&mut self, data: &[u8]) {

		let bytes_wrote = self.tty_device.write(data).unwrap();
		println!("Wrote {} bytes", bytes_wrote);

	}

	// TODO: look into buffered reader
	pub fn serial_read(&mut self, buf: &mut Vec<u8>) -> usize {

		let bytes_read = self.tty_device.read(buf).unwrap();
		bytes_read

	}

	pub fn set_tty_params(&mut self, br: serial::BaudRate, 
									 cs: serial::CharSize,
									 p: serial::Parity,
									 sb: serial::StopBits,
									 fc: serial::FlowControl ) {
									 
		let mut settings = self.tty_device.read_settings().unwrap();
		settings.set_baud_rate(br).unwrap();
		settings.set_char_size(cs);
		settings.set_parity(p);
		settings.set_stop_bits(sb);
		settings.set_flow_control(fc);

		self.tty_device.write_settings(&settings).unwrap();
	}
	
}


fn main() {

	// we want it to be a mutable driver because we will be changing fields as we go.
	let mut e23_driver = Driver::new(1013,1015,1017, "/dev/ttyS0");
	e23_driver.set_tty_params(
		serial::Baud9600,
		serial::Bits8,
		serial::ParityNone,
		serial::Stop1,
		serial::FlowNone
		);
	//e23_driver.set_tty_device(String::from("/dev/ttyS0"));
	//e23_driver.scrape_tty_baud();
	//let (m0_pin, m1_pin, aux_pin) = e23_driver.get_control_gpio_pins();
	//println!("M0: {} M1: {} AUX: {}", m0_pin, m1_pin, aux_pin);

	//e23_driver.set_mode(RadioMode::General);
	//println!("Mode set to general");

	let mut testconfig = RadioConfig::new();
	//testconfig.set_transmit_power("20dBm");
	testconfig.set_serial_rate("115200");
	//set our serial rate to 115200 as well
	e23_driver.set_tty_baud(serial::Baud115200);
	//e23_driver.set_tty_baud("9600");
	e23_driver.write_config(testconfig); // Write after change
	
	//Check to see if the changes were made correctly
	//e23_driver.set_mode(RadioMode::Sleep);
	//e23_driver.serial_write(&[193,193,193]);
	//sleep(Duration::from_millis(2));
	//e23_driver.serial_read();

	//e23_driver.set_tty_baud("115200");


}
