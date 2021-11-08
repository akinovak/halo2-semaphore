use std::marker::PhantomData;

use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};

use super::AddInstruction;
use super::super::super::CellValue;

// #[derive(Clone)]
// pub struct Cellber<F: FieldExt> {
//     pub cell: Cell,
//     pub value: Option<F>,
// }

#[derive(Clone, Debug)]
pub struct AddConfig {
    pub advice: [Column<Advice>; 2],
    pub s_add: Selector,
}

#[derive(Debug)]
pub struct AddChip<F: FieldExt> {
    pub config: AddConfig,
    pub _marker: PhantomData<F>,
}

impl<F: FieldExt> AddChip<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 2],
    ) -> <Self as Chip<F>>::Config {
        for column in &advice {
            meta.enable_equality((*column).into());
        }

        let s_add = meta.selector();

        meta.create_gate("add", |meta| {
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[0], Rotation::next());
            let s_add = meta.query_selector(s_add);

            vec![s_add * (lhs + rhs - out)]
        });

        AddConfig {
            advice,
            s_add
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

impl<F: FieldExt> Chip<F> for AddChip<F> {
    type Config = AddConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> AddInstruction<F> for AddChip<F> {
    type Cell = CellValue<F>;

    fn add(
        &self,
        mut layouter: impl Layouter<F>,
        a: Self::Cell,
        b: Self::Cell,
    ) -> Result<Self::Cell, Error> {
        let config = self.config();

        let mut out = None;
        layouter.assign_region(
            || "add",
            |mut region: Region<'_, F>| {
                config.s_add.enable(&mut region, 0)?;

                let lhs = region.assign_advice(
                    || "lhs",
                    config.advice[0],
                    0,
                    || a.value.ok_or(Error::SynthesisError),
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    config.advice[1],
                    0,
                    || b.value.ok_or(Error::SynthesisError),
                )?;
                region.constrain_equal(a.cell, lhs)?;
                region.constrain_equal(b.cell, rhs)?;

                // Now we can assign the addition result into the output position.
                let value = a.value.and_then(|a| b.value.map(|b| a + b));
                let cell = region.assign_advice(
                    || "lhs + rhs",
                    config.advice[0],
                    1,
                    || value.ok_or(Error::SynthesisError),
                )?;

                out = Some(CellValue { cell, value });
                Ok(())
            },
        )?;

        Ok(out.unwrap())
    }
}
// ANCHOR END: add-instructions-impl