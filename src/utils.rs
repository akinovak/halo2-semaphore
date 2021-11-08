use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Layouter},
    plonk::{Column, Advice, Instance, Error},
};

// #[derive(Clone)]
// pub struct Number<F: FieldExt> {
//     pub cell: Cell,
//     pub value: Option<F>,
// }

#[derive(Copy, Clone, Debug)]
pub struct CellValue<F: FieldExt> {
    pub cell: Cell,
    pub value: Option<F>,
}

pub trait Var<F: FieldExt>: Copy + Clone + std::fmt::Debug {
    fn new(cell: Cell, value: Option<F>) -> Self;
    fn cell(&self) -> Cell;
    fn value(&self) -> Option<F>;
}

impl<F: FieldExt> Var<F> for CellValue<F> {
    fn new(cell: Cell, value: Option<F>) -> Self {
        Self { cell, value }
    }

    fn cell(&self) -> Cell {
        self.cell
    }

    fn value(&self) -> Option<F> {
        self.value
    }
}

pub trait UtilitiesInstructions<F: FieldExt> {
    type Var: Var<F>;

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        column: Column<Advice>,
        value: Option<F>,
    ) -> Result<Self::Var, Error> {
        layouter.assign_region(
            || "load private",
            |mut region| {
                let cell = region.assign_advice(
                    || "load private",
                    column,
                    0,
                    || value.ok_or(Error::SynthesisError),
                )?;
                Ok(Var::new(cell, value))
            },
        )
    }

    fn constrain_public(
        &self,
        mut layouter: impl Layouter<F>,
        column: Column<Instance>,
        var: impl Var<F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(var.cell(), column, row)
    }
}