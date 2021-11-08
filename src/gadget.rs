pub mod add;
use crate::gadget::add::*;
use halo2::{pasta::Fp};

impl super::Config {
    pub(super) fn add_chip(&self) -> AddChip<Fp> {
        AddChip::construct(self.add_config.clone())
    }
}