#[inline(always)]
pub fn select_discriminator(&self) -> DiscriminatorFn {
    self.discriminator_fn
}