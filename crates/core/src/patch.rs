use crate::{cpal::CpalEvent, PolySample, SampleTiming};

pub trait Patch: Send {
	fn next_sample(&mut self, sample_timing: &SampleTiming) -> PolySample;
}

pub trait OutPatch: Patch {
	fn write_data(
		&mut self,
		output: &mut [f32],
		channels: usize,
		sample_timing: &mut SampleTiming,
	) -> Option<CpalEvent>;
}

#[derive(Default)]
pub struct MasterPatch {
	patches: Vec<Box<dyn Patch>>,
}

impl MasterPatch {
	pub fn new() -> Self {
		MasterPatch {
			patches: Vec::new(),
		}
	}

	pub fn add_patch<P: 'static + Patch>(&mut self, patch: P) {
		self.patches.push(Box::new(patch));
	}
}

impl Patch for MasterPatch {
	fn next_sample(&mut self, sample_timing: &SampleTiming) -> PolySample {
		let mut master = PolySample(vec![]);
		for patch in &mut self.patches {
			let patch_sample = patch.next_sample(sample_timing).0;
			if patch_sample.is_empty() {
				return PolySample(vec![]);
			}
			for (i, sample) in patch_sample.into_iter().enumerate() {
				match master.0.get_mut(i) {
					None => {
						master.0.push(sample);
					}
					Some(current_sample) => {
						*current_sample += sample;
					}
				}
			}
		}
		master
	}
}

impl OutPatch for MasterPatch {
	fn write_data(
		&mut self,
		output: &mut [f32],
		channels: usize,
		sample_timing: &mut SampleTiming,
	) -> Option<CpalEvent> {
		for frame in output.chunks_mut(channels) {
			let next_sample = self.next_sample(sample_timing).0;

			if next_sample.is_empty() {
				return Some(CpalEvent::Exit);
			}

			let mut next_samples =
				next_sample.into_iter().chain(std::iter::repeat(0.0));
			for sample in frame.iter_mut() {
				*sample = next_samples.next().unwrap();
			}
			sample_timing.tick();
		}
		None
	}
}
