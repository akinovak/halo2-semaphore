use std::marker::PhantomData;

use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector, Expression},
    poly::Rotation,
};

use super::MerkleInstructions;
use super::super::super::CellValue;

pub use super::super::add::{AddChip, AddConfig, AddInstruction};

// use super::add::*;

#[derive(Clone, Debug)]
pub struct MerkleConfig {
    pub advice: [Column<Advice>; 3],
    pub s_bool: Selector,
    pub (super) hash_config: AddConfig, //mocking hash with addition
}

#[derive(Debug)]
pub struct MerkleChip<F: FieldExt> {
    pub config: MerkleConfig,
    pub _marker: PhantomData<F>,
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
            let position_bit = meta.query_advice(advice[0], Rotation::cur());
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

impl<F: FieldExt> MerkleInstructions<F> for MerkleChip<F> {
    type Cell = CellValue<F>;

    fn check_bool(
        &self,
        mut layouter: impl Layouter<F>,
        position_bit: Self::Cell,
        layer: usize,
    ) -> Result<(), Error> {

        println!("I'm in instruction");

        layouter.assign_region(
            || "hash layer",
            |mut region| {
                let mut row_offset = 0;

                let position_bit_cell = region.assign_advice(
                    || format!("positional_bit (layer {})", layer),
                    self.config.advice[0],
                    row_offset,
                    || position_bit.value.ok_or(Error::SynthesisError),
                )?;

                // region.constrain_equal(position_bit_cell, position_bit.cell);
                self.config.s_bool.enable(&mut region, row_offset)?;
                Ok(())
            },
        )?;

        Ok(())
    }
}