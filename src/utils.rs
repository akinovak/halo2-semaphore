use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Layouter, Region},
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

pub fn copy<A, AR, F: FieldExt>(
    region: &mut Region<'_, F>,
    annotation: A,
    column: Column<Advice>,
    offset: usize,
    copy: &CellValue<F>,
) -> Result<CellValue<F>, Error>
where
    A: Fn() -> AR,
    AR: Into<String>,
{
    let cell = region.assign_advice(annotation, column, offset, || {
        copy.value.ok_or(Error::SynthesisError)
    })?;

    region.constrain_equal(cell, copy.cell)?;

    Ok(CellValue::new(cell, copy.value))
}

//HOW TO COPY
// let identity_nullifier_clone = layouter.assign_region(
//     || "copy identity nullifier",
//     |mut region| {
//         config.s_clone.enable(&mut region, 0)?;

//         let cloned = copy(
//             &mut region,
//             || "copy identity_nullifier",
//             config.advices[3],
//             0,
//             &identity_nullifier
//         )?;

//         Ok(cloned)
//     }
// );