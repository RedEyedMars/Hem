use packed_simd::{f32x4, m32x4};

#[derive(PartialEq, Clone, Debug)]
pub struct Progress4 {
    progress: f32x4,
    progress_special: f32x4,
    progress_per_tick_multiplier: f32x4,
    rand_data: f32x4,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ProgressAttribute {
    progress_per_tick: f32x4,
    progress_per_special: f32x4,
    progress_completion: f32x4,
    progress_perfect_completion: f32x4,
    progress_chance_to_special: f32x4,
}

impl ProgressAttribute {
    pub fn new() -> ProgressAttribute {
        ProgressAttribute {
            progress_per_tick: f32x4::splat(1f32),
            progress_per_special: f32x4::splat(5f32),
            progress_completion: f32x4::splat(100f32),
            progress_perfect_completion: f32x4::splat(150f32),
            progress_chance_to_special: f32x4::splat(0.05f32),
        }
    }
    pub fn reset(&mut self, index: usize) -> Result<(), failure::Error> {
        use crate::replace_ps_f32x4;
        self.progress_per_tick = replace_ps_f32x4(self.progress_per_tick, 1f32, index)?;
        self.progress_per_special = replace_ps_f32x4(self.progress_per_special, 5f32, index)?;
        self.progress_completion = replace_ps_f32x4(self.progress_completion, 100f32, index)?;
        self.progress_perfect_completion =
            replace_ps_f32x4(self.progress_perfect_completion, 150f32, index)?;
        self.progress_chance_to_special =
            replace_ps_f32x4(self.progress_chance_to_special, 0.05f32, index)?;
        Ok(())
    }
}

impl Progress4 {
    pub fn new() -> Progress4 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Progress4 {
            progress: f32x4::splat(0f32),
            progress_special: f32x4::splat(0f32),
            progress_per_tick_multiplier: f32x4::splat(1f32),
            rand_data: f32x4::new(rng.gen(), rng.gen(), rng.gen(), rng.gen()),
        }
    }
    pub unsafe fn reset(&mut self, index: usize) -> Result<(), failure::Error> {
        use crate::replace_ps_f32x4;
        self.progress = replace_ps_f32x4(self.progress, 0f32, index)?;
        self.progress_special = replace_ps_f32x4(self.progress_special, 0f32, index)?;
        self.progress_per_tick_multiplier =
            replace_ps_f32x4(self.progress_per_tick_multiplier, 1f32, index)?;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.rand_data = replace_ps_f32x4(self.rand_data, rng.gen(), index)?;
        Ok(())
    }

    pub fn progress(
        &mut self,
        can_progress: m32x4,
        attributes: &ProgressAttribute,
    ) -> Result<(), failure::Error> {
        use crate::replace_ps_f32x4;

        if can_progress.any() {
            let can_special = self.rand_data.le(attributes.progress_chance_to_special);
            let special_can_progress = self
                .progress_special
                .lt(attributes.progress_perfect_completion - attributes.progress_completion);
            let special = can_progress & special_can_progress & can_special;
            let progress = can_progress;

            self.progress = progress.select(
                attributes.progress_per_tick * self.progress_per_tick_multiplier + self.progress,
                self.progress,
            );
            self.progress_special = special.select(
                attributes.progress_per_special * self.progress_per_tick_multiplier
                    + self.progress_special,
                self.progress_special,
            );

            if can_progress.any() {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let mut new_rand = [0f32; 4];
                self.rand_data.write_to_slice_aligned(&mut new_rand);
                for i in 0..4 {
                    if can_progress.extract(i) {
                        new_rand[i] = rng.gen();
                    }
                }
                self.rand_data = f32x4::from_slice_aligned(&new_rand);
            }
        }
        Ok(())
    }

    pub fn can_progress(&self, attributes: &ProgressAttribute) -> m32x4 {
        self.progress.lt(attributes.progress_completion)
    }
    pub fn completion_percentage(&self, attributes: &ProgressAttribute) -> f32x4 {
        (self.progress + self.progress_special) / (attributes.progress_completion)
    }
    pub fn percentage(&self, attributes: &ProgressAttribute) -> f32x4 {
        self.progress / attributes.progress_completion
    }
}
