pub mod add;
pub mod merkle;
pub mod poseidon;
// use crate::gadget::add::*;
// use crate::gadget::merkle::*;
// use crate::gadget::poseidon::;
use halo2::{
    pasta::Fp
};

// use pasta_curves::{
//     pallas, vesta,
// };

use crate::gadget::{
    add::*,
    merkle::*,
    poseidon::{Pow5T3Chip as PoseidonChip}
};

impl super::Config {
    pub(super) fn construct_add_chip(&self) -> AddChip<Fp> {
        AddChip::construct(self.add_config.clone())
    }

    pub(super) fn construct_merkle_chip(&self) -> MerkleChip {
        MerkleChip::construct(self.merkle_config.clone())
    }

    pub(super) fn construct_poseidon_chip(&self) -> PoseidonChip<Fp> {
        PoseidonChip::construct(self.poseidon_config.clone())
    }
}