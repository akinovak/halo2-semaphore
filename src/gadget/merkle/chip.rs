use std::marker::PhantomData;

use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector, Expression},
    poly::Rotation,
};

use crate::utils::Var;
use super::MerkleInstructions;
use super::super::super::CellValue;

pub use super::super::add::{AddChip, AddConfig, AddInstruction};

// use super::add::*;

#[derive(Clone, Debug)]
pub struct MerkleConfig {
    pub advice: [Column<Advice>; 3],
    pub s_bool: Selector,
    pub s_swap: Selector,
    pub hash_config: AddConfig
}

#[derive(Clone, Debug)]
pub struct MerkleChip<F: FieldExt> {
    pub config: MerkleConfig,
    pub _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for MerkleChip<F> {
    type Config = MerkleConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> MerkleChip<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
    ) -> <Self as Chip<F>>::Config {
        for column in &advice {
            meta.enable_equality((*column).into());
        }

        let s_bool = meta.selector();

        meta.create_gate("bool", |meta| {
            let position_bit = meta.query_advice(advice[2], Rotation::cur());
            let s_bool = meta.query_selector(s_bool);
            vec![s_bool * position_bit.clone() * (Expression::Constant(F::one()) - position_bit)]
        });

        let s_swap = meta.selector();

        meta.create_gate("swap", |meta| {
            let a = meta.query_advice(advice[0], Rotation::cur());
            let b = meta.query_advice(advice[1], Rotation::cur());
            let bit = meta.query_advice(advice[2], Rotation::cur());
            let s_swap = meta.query_selector(s_swap);
            let l = meta.query_advice(advice[0], Rotation::next());
            let r = meta.query_advice(advice[1], Rotation::next());
            vec![s_swap * ((bit * F::from(2) * (b.clone() - a.clone()) - (l - a)) - (b - r))]
        });

        let hash_config = AddChip::configure(meta, advice[0..2].try_into().unwrap());

        MerkleConfig {
            advice,
            s_bool,
            s_swap,
            hash_config
        }
    }

    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }
}
// ANCHOR_END: chip-config

impl<F: FieldExt> MerkleInstructions<F> for MerkleChip<F> {
    type Cell = CellValue<F>;

    fn hash_layer(
        &self,
        mut layouter: impl Layouter<F>,
        leaf_or_digest: Self::Cell,
        sibling: Option<F>,
        position_bit: Option<F>,
        layer: usize,
    ) -> Result<Self::Cell, Error> {

        let config = self.config.clone();

        let add_chip = AddChip::<F>::construct(config.hash_config.clone());

        let mut left_digest = None;
        let mut right_digest = None;
        // let mut digest = None;

        layouter.assign_region(
            || format!("hash on (layer {})", layer),
            |mut region| {
                let mut row_offset = 0;

                let left_or_digest_value = leaf_or_digest.value();

                let left_or_digest_cell = region.assign_advice(
                    || format!("witness leaf or digest (layer {})", layer),
                    config.advice[0],
                    row_offset,
                    || left_or_digest_value.ok_or(Error::SynthesisError),
                )?;

                let sibling_cell = region.assign_advice(
                    || format!("witness sibling (layer {})", layer),
                    config.advice[1],
                    row_offset,
                    || sibling.ok_or(Error::SynthesisError),
                )?;

                let position_bit_cell = region.assign_advice(
                    || format!("witness positional_bit (layer {})", layer),
                    config.advice[2],
                    row_offset,
                    || position_bit.ok_or(Error::SynthesisError),
                )?;

                if layer > 0 {
                    region.constrain_equal(leaf_or_digest.cell(), left_or_digest_cell)?;
                }


                config.s_bool.enable(&mut region, row_offset)?;
                config.s_swap.enable(&mut region, row_offset)?;


                let (l_value, r_value): (F, F) = if position_bit == Some(F::zero()) {
                    (left_or_digest_value.ok_or(Error::SynthesisError)?, sibling.ok_or(Error::SynthesisError)?)
                } else {
                    (sibling.ok_or(Error::SynthesisError)?, left_or_digest_value.ok_or(Error::SynthesisError)?)
                };

                row_offset += 1;

                let l_cell = region.assign_advice(
                    || format!("witness left (layer {})", layer),
                    config.advice[0],
                    row_offset,
                    || Ok(l_value),
                )?;


                let r_cell = region.assign_advice(
                    || format!("witness right (layer {})", layer),
                    config.advice[1],
                    row_offset,
                    || Ok(r_value),
                )?;

                left_digest = Some(CellValue { cell: l_cell, value: Some(l_value) });
                right_digest = Some(CellValue { cell: r_cell, value: Some(r_value) });

                Ok(())
            },
        )?;

        let digest = add_chip.add(layouter.namespace(|| format!("digest on (layer {})", layer)), left_digest.unwrap(), right_digest.unwrap())?;

        Ok(digest)
    }
}