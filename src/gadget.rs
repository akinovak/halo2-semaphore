pub mod add;
pub mod merkle;
pub mod poseidon;
use crate::gadget::add::*;
use crate::gadget::merkle::*;
use crate::gadget::poseidon::*;
use halo2::{
    arithmetic::FieldExt,
    pasta::Fp
};

use pasta_curves::{
    pallas, vesta,
};

impl<F: FieldExt> super::Config<F> {
    pub(super) fn construct_add_chip(&self) -> AddChip<F> {
        AddChip::construct(self.add_config.clone())
    }

    pub(super) fn construct_merkle_chip(&self) -> MerkleChip<F> {
        MerkleChip::construct(self.merkle_config.clone())
    }

    pub(super) fn construct_poseidon_chip(&self) -> Pow5T3Chip<Fp> {
        Pow5T3Chip::construct(self.pow5t3_config.clone())
    }
}