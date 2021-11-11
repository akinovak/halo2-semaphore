pub mod add;
pub mod merkle;
use crate::gadget::add::*;
use crate::gadget::merkle::*;
use halo2::{
    arithmetic::FieldExt
};

impl<F: FieldExt> super::Config<F> {
    pub(super) fn construct_add_chip(&self) -> AddChip<F> {
        AddChip::construct(self.add_config.clone())
    }

    pub(super) fn construct_merkle_chip(&self) -> MerkleChip<F> {
        MerkleChip::construct(self.merkle_config.clone())
    }
}