use super::Effect;
use crate::{PolySample, SampleTiming};
use anyhow::{anyhow, Result};
use plotters::prelude::*;
use std::{collections::VecDeque, path::Path};

#[derive(Clone)]
pub struct Oscilloscope {
	pub sample_interval: f32,
	pub buffer_duration: f32,
	last_sample_counter: usize,
	pub channel: u8,
	pub height: u32,
	pub aspect_ratio: f32,
	buffer: VecDeque<(f32, PolySample)>,
}

impl Oscilloscope {
	pub fn new(
		buffer_duration: f32,
		sample_interval: f32,
		channel: u8,
		height: u32,
		aspect_ratio: f32,
	) -> Self {
		Self {
			sample_interval,
			buffer_duration,
			last_sample_counter: 0,
			channel,
			height,
			aspect_ratio,
			buffer: VecDeque::new(),
		}
	}

	pub fn plot<P: AsRef<Path>>(&self, path: P) -> Result<()> {
		match self.buffer.len() {
			0 => return Err(anyhow!("No elements to plot in buffer")),
			1 => return Err(anyhow!("Only one element to plot in buffer")),
			_ => {}
		}
		let root = BitMapBackend::new(
			&path,
			(
				(self.aspect_ratio * self.buffer_duration * self.height as f32)
					as u32,
				self.height,
			),
		)
		.into_drawing_area();
		root.fill(&WHITE)?;
		let mut chart = ChartBuilder::on(&root)
			.margin(5)
			.x_label_area_size(30)
			.y_label_area_size(30)
			.build_cartesian_2d(
				self.buffer[0].0..self.buffer[self.buffer.len() - 1].0,
				-1f32..1f32,
			)?;

		chart.configure_mesh().draw()?;

		chart.draw_series(LineSeries::new(
			self.buffer.iter().map(|(time, poly_sample)| {
				(*time, poly_sample.0[self.channel as usize])
			}),
			&RED,
		))?;

		Ok(())
	}
}

impl Default for Oscilloscope {
	fn default() -> Self {
		Self::new(1.0, 0.001, 0, 512, 1.0)
	}
}

impl Effect for Oscilloscope {
	fn process(
		&mut self,
		sample_timing: &SampleTiming,
		poly_sample: PolySample,
	) -> PolySample {
		let buffer_size =
			sample_timing.duration_to_sample_count(self.buffer_duration);
		let accuracy_sample_count = sample_timing
			.duration_to_sample_count(self.sample_interval)
			.max(1);
		if self.last_sample_counter % accuracy_sample_count == 0 {
			if self.buffer.len() == buffer_size {
				self.buffer.pop_front();
			}
			self.buffer
				.push_back((sample_timing.sample_clock(), poly_sample.clone()));
		}
		self.last_sample_counter += 1;
		poly_sample
	}
}
