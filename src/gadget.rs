use halo2::{
    pasta::Fp
};

pub mod merkle;
pub mod poseidon;


use crate::gadget::{
    merkle::*,
    poseidon::{Pow5T3Chip as PoseidonChip}
};

impl super::Config {
    pub(super) fn construct_merkle_chip(&self) -> MerkleChip {
        MerkleChip::construct(self.merkle_config.clone())
    }

    pub(super) fn construct_poseidon_chip(&self) -> PoseidonChip<Fp> {
        PoseidonChip::construct(self.poseidon_config.clone())
    }
}