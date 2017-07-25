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
	let send_or_receive = &args[2];

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
	testconfig.set_air_rate("1k");
	testconfig.set_address("FFFF"); // Channel 0
	e32_driver.set_mode(RadioMode::General);
	e32_driver.write_config(testconfig);
	
	

	let mut pty_master_file = create_pty_pair().unwrap();
	//let mut bufreader = BufReader::new(pty_master_file);

	/* I don't want to do obect oriented stuff for this loop... might do it later, for now, states
	0: no operation, shouldn't ever be this.
	1: receive packet
	2: flush received to output
	3: read from serial and send packet
	4: send previous packet again
	5: send heartbeat packet
	6: send retry packet
	*/
	let mut state = 0;
	match &send_or_receive[..] {
		"s" => state = 3,
		"r" => state = 1,
		_ => panic!("Invalid option specified, must be 'r' or 's'")
	}
	let CMD_BYTE_HEARTBEAT = 2;
	let CMD_BYTE_RETRY = 1;
	let mut previous_packet: Vec<u8> = vec![0];
	let mut received_packet: Vec<u8> = vec![0];
	// Start our operation loop
	loop {

		// for debug slow our whole loop down to 1hz
		sleep(Duration::from_secs(1));

		match state {
			// receive packet
			1 => {
				println!("Receive packet (1)");
				let mut packet_read = vec![0];
				
				match e32_driver.receive_packet(10_000) {
					Ok(bytes_read) => { packet_read = bytes_read; },

					Err(e) => {
						// Timed out, we need to send a retry packet
						state = 6;
						// hop out of this match and continue the loop
						continue;
					}
				}


				if packet_read.len() == 0 { panic!("Read 0 bytes, something is wrong."); }

				// Check our command byte and take appropriate action
				match packet_read[0] {
					1 => {
						// this is a retry message, send previous packet again
						state = 4;
					}
					2 | 0 => {
						// they had no data to send, or this is regular data
						state = 2;
					}
					_ => {
						// garbage in command byte, invalid state, panic
						panic!("Garbage in command byte, corrupt packet? {:?}", packet_read[0]);
					}
				}
			}
			// flush received data to output
			2 => {
				println!("flush received data to output (2)");
				if received_packet.len() == 0 { panic!("Attempted to output blank packet, this should never happen"); }
				// send our output without the command byte
				pty_master_file.write_all(&received_packet[1..]).expect("Failed to output data to pty");
				state = 3;
			}
			// read from serial and send packet
			3 => {
				println!("read from serial and send packet (3)");
				let mut packet_to_send: Vec<u8> = vec![0;57];
				previous_packet = packet_to_send.clone();
				// reads up to 57 bytes from serial
				let num_bytes_read = pty_master_file.read(&mut packet_to_send).unwrap();
				if num_bytes_read == 0 {
					// got no data from serial. send heartbeat instead.
					// TODO: add some delay here. There is no point in writing heartbeats as fast as we can
					state = 5;
					continue;
				}
				// make sure our command 
				let mut send_packet_w_cmd = vec![0];
				// push our data after the command byte
				send_packet_w_cmd.append(&mut packet_to_send);
				e32_driver.send_packet(&send_packet_w_cmd, 5000).expect("Error sending packet");
				// our turn to read a packet
				state = 1;
			}
			// send previous packet again
			4 => {
				println!("send previous packet again (4)");
				let mut send_packet_w_cmd = vec![0];
				// push our data after the command byte
				send_packet_w_cmd.append(&mut previous_packet.clone());
				e32_driver.send_packet(&send_packet_w_cmd, 5000).expect("Error sending previous packet");
				// our turn to read a packet
				state = 1;
			}
			// send heartbeat packet
			5 => {
				println!("send heartbeat packet (5)");
				let mut send_packet_w_cmd = vec![CMD_BYTE_HEARTBEAT];
				e32_driver.send_packet(&send_packet_w_cmd, 5000).expect("Error sending heartbeat packet");
				// our turn to read a packet
				state = 1;
			}
			// send retry packet
			6 => {
				println!("send retry packet (6)");
				let mut send_packet_w_cmd = vec![CMD_BYTE_RETRY];
				e32_driver.send_packet(&send_packet_w_cmd, 5000).expect("Error sending heartbeat packet");
				// our turn to read a packet
				state = 1;
			}
			_ => panic!("Invalid state... some code is bad")
		}; // we don't want to return from the match to keep the loop going

	}

}
