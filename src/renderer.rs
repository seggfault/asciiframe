use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use indicatif::ProgressBar;
use opencv::prelude::*;
use opencv::{core, imgproc, videoio};
use terminal_size::{terminal_size, Height, Width};

use crate::converter;
use crate::error::{Error, Result};

pub fn render(
	filename: &Path,
	output: Option<&Path>,
	strategy: u8,
) -> Result<()> {
	let mut capture =
		videoio::VideoCapture::from_file(filename.to_str().unwrap(), 0)?;
	let frame_count: u64 = capture.get(videoio::CAP_PROP_FRAME_COUNT)? as u64;
	let time_d: f32 = (1.0 / capture.get(videoio::CAP_PROP_FPS)?) as f32;
	let pb = ProgressBar::new(frame_count);

	if let Some(p) = output {
		File::create(p)?.write_all(
			"#!/bin/bash\n# This file was auto-generated by asciiframe"
				.as_bytes(),
		)?;
	}

	for _i in 0..frame_count {
		let start = SystemTime::now();

		let mut frame = Mat::default();
		// CV_8UC3
		capture.read(&mut frame)?;

		let mut resized = Mat::default();

		if let Some((Width(w), Height(h))) = terminal_size() {
			imgproc::resize(
				&frame,
				&mut resized,
				core::Size {
					width: i32::from(w - 1),
					height: i32::from(h - 1),
				},
				0.0,
				0.0,
				imgproc::INTER_AREA,
			)?;

			if let Some(p) = output {
				// if output to file
				render_frame_to_file(&resized, strategy, p)?;
				pb.inc(1);
			} else {
				// if output to stdout
				render_frame_stdout(&resized, strategy)?;

				let elapsed = start.elapsed().unwrap().as_secs_f32();
				if elapsed < time_d {
					sleep(Duration::from_millis(
						((time_d - elapsed) * 1000.0) as u64,
					));
				}
			}
		} else {
			return Err(Error::from("Unable to get terminal size"));
		}
	}

	Ok(())
}

fn render_frame_stdout(frame: &Mat, strategy: u8) -> Result<()> {
	println!("{esc}c", esc = 27 as char);
	println!("{}", converter::convert_frame(frame, strategy)?);

	Ok(())
}

fn render_frame_to_file(frame: &Mat, strategy: u8, path: &Path) -> Result<()> {
	let mut fout = OpenOptions::new().append(true).open(path)?;
	// echo -en '\u001b[0;0H'
	let txt = format!(
		"clear\necho '{}'\nsleep 0.033\n",
		converter::convert_frame(frame, strategy)?
	);
	fout.write_all(txt.as_bytes())?;

	Ok(())
}
