use std::env;
use std::ptr;

// For threading and channels
//use std::thread;
use std::thread::sleep;

use std::time::Duration;
use std::fs::File;
use std::os::unix::io::IntoRawFd;
use std::os::unix::io::FromRawFd;
use std::io::Read;
use std::io::BufReader;
use std::io::Write;
use std::io::Error;
use std::io::ErrorKind;
//use std::io::prelude::*;

extern crate floating_duration;
//use floating_duration::TimeAsFloat;

extern crate serial;

extern crate lora_driver;
use lora_driver::RadioConfig;
use lora_driver::Driver;
use lora_driver::RadioMode;

extern crate libc;
use libc::termios;
//use libc::winsize;


// Testing for Bryant
//extern crate serialport;

pub fn create_pty_pair() -> Result<File, Error> {
	// I do NOT like the config targets... but for now it may be necessary
	#[cfg(any(target_arch = "arm", target_arch = "x86_64"))]
	let mut test_termios = termios { 
				c_iflag: 	libc::IGNBRK+libc::IGNPAR+libc::IXON+libc::IXOFF,
				c_oflag: 	libc::IXON+libc::IXOFF,
				c_cflag: 	0,
				c_lflag: 	0,
				c_line:		0,
				c_cc:		[0;32],
				c_ispeed:	115200,
				c_ospeed:	115200
			};
	#[cfg(target_arch = "mips")]
	let mut test_termios = termios { 
			c_iflag: 	libc::IGNBRK+libc::IGNPAR+libc::IXON+libc::IXOFF,
			c_oflag: 	libc::IXON+libc::IXOFF,
			c_cflag: 	0,
			c_lflag: 	0,
			c_line:		0,
			c_cc:		[0;32],
			__c_ispeed:	115200,
			__c_ospeed:	115200
		};

/*
	let mut test_winsize = winsize {
		ws_row:		100,
		ws_col:		256,
		ws_xpixel:	0,
		ws_ypixel:	0
	};
*/

	let mut amaster: libc::c_int = 0;
	let mut aslave: libc::c_int = 0;
	let mut name: libc::c_char = 0;

	let mut test_openpty = -1;
	unsafe {
		test_openpty = libc::openpty(&mut amaster, &mut aslave, &mut name, &mut test_termios, ptr::null()); //, &mut test_winsize
	}
	match test_openpty {
		0 => {
			let pty_master_file: File;
			unsafe { pty_master_file = File::from_raw_fd(amaster); }
			Ok(pty_master_file)
		},
		-1 => Err(Error::new(ErrorKind::Other, "Openpty failed")),
		_ => Err(Error::new(ErrorKind::Other, "Invalid openpty return code"))
	}
}

fn main() {

	// Get args that were passed
	let args: Vec<String> = env::args().collect();
	let platform = &args[1];

	let mut e32_driver;
	let mut testconfig = RadioConfig::new();

	match &platform[..] {
		"vocore" => { 
						e32_driver = Driver::new(22,23,24, "/dev/ttyS0");
						//set our serial rate to 57600 as well
						e32_driver.set_tty_params(
							serial::Baud115200,
							serial::Bits8,
							serial::ParityNone,
							serial::Stop1,
							serial::FlowNone
						);
						e32_driver.set_tty_baud(serial::Baud57600);
						testconfig.set_serial_rate("57600");
					}
		"chip" => { 
						e32_driver = Driver::new(1013,1015,1017, "/dev/ttyS0");
						e32_driver.set_tty_params(
							serial::Baud115200,
							serial::Bits8,
							serial::ParityNone,
							serial::Stop1,
							serial::FlowNone
						);
						testconfig.set_serial_rate("115200");
				  }
		_ => panic!("Please enter either 'vocore' or 'chip'")
	}
	

	testconfig.set_transmit_power("20dBm");
	testconfig.set_air_rate("10k");
	testconfig.set_address("FFFF"); // Channel 0
	e32_driver.set_mode(RadioMode::General);
	e32_driver.write_config(testconfig);
	
	
	

	//println!("rc: {:?}, amaster: {:?}, aslave: {:?}, name: {:?}",test_openpty,amaster,aslave,name);

	let mut pty_master_file = create_pty_pair().unwrap();
	//let mut bufreader = BufReader::new(pty_master_file);

	// Just read from the port and send packet along until we send a kill signal
	loop {
		let mut read_vec: Vec<u8> = vec![0;256];
		let num_bytes_read = pty_master_file.read(&mut read_vec).unwrap();
		read_vec.truncate(num_bytes_read);
		print!("Sending: {}", String::from_utf8(read_vec.clone()).unwrap());
		e32_driver.send_packet(&read_vec,5000).unwrap();
		let recv_vec = e32_driver.receive_packet(120_000).unwrap();
		println!("Received: {}", String::from_utf8(recv_vec.clone()).unwrap());
		pty_master_file.write_all(&recv_vec).unwrap();


		//sleep(Duration::from_secs(1));
	}

}
