use halo2::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Advice, Instance, Circuit, Column, ConstraintSystem, Error},
    pasta::Fp
};

use pasta_curves::{
    pallas,
};

mod primitives;
mod gadget;
mod utils;

use gadget:: {
    add::{AddChip, AddConfig, AddInstruction},
    merkle::{MerkleChip, MerkleConfig, MerklePath},
    poseidon::{Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig, Hash as PoseidonHash, Word, StateWord}
};

use crate:: {
    utils::{UtilitiesInstructions, CellValue, Var},
    primitives::poseidon::{ConstantLength, P128Pow5T3}
};

pub const MERKLE_DEPTH: usize = 4;

// Absolute offsets for public inputs.
const EXTERNAL_NULLIFIER: usize = 0;
const NULLIFIER_HASH: usize = 1;
const ROOT: usize = 2;

// Semaphore config
#[derive(Clone, Debug)]
pub struct Config {
    advices: [Column<Advice>; 4],
    instance: Column<Instance>,
    add_config: AddConfig,
    merkle_config: MerkleConfig,
    poseidon_config: PoseidonConfig<Fp>,
}

// Semaphore circuit
#[derive(Debug, Default)]
pub struct SemaphoreCircuit {
    identity_trapdoor: Option<Fp>,
    identity_nullifier: Option<Fp>,
    external_nullifier: Option<Fp>,
    position_bits: Option<[Fp; MERKLE_DEPTH]>,
    path: Option<[Fp; MERKLE_DEPTH]>,
    root: Option<Fp>,
}

impl UtilitiesInstructions<pallas::Base> for SemaphoreCircuit {
    type Var = CellValue<pallas::Base>;
}

// impl SemaphoreCircuit {
//     fn calculate_identity_commitment()
// }

impl Circuit<pallas::Base> for SemaphoreCircuit 
{
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {

        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        let instance = meta.instance_column();
        meta.enable_equality(instance.into());

        for advice in advices.iter() {
            meta.enable_equality((*advice).into());
        }

        let add_config = AddChip::configure(meta, advices[0..2].try_into().unwrap());
        let merkle_config = MerkleChip::configure(meta, advices[0..3].try_into().unwrap());

        let rc_a = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        let rc_b = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];

        meta.enable_constant(rc_b[0]);

        let poseidon_config = PoseidonChip::configure(meta, P128Pow5T3, advices[0..3].try_into().unwrap(), advices[3], rc_a, rc_b);

        Config {
            advices, 
            instance,
            add_config,
            merkle_config,
            poseidon_config,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {

        let add_chip = config.construct_add_chip();
        let merkle_chip = config.construct_merkle_chip();

        let poseidon_chip = config.construct_poseidon_chip();
        let poseidon_chip_2 = config.construct_poseidon_chip();

        let mut poseidon_hasher: PoseidonHash
        <
            Fp, 
            PoseidonChip<Fp>, 
            P128Pow5T3, 
            ConstantLength<2_usize>, 
            3_usize, 
            2_usize
        > 
            = PoseidonHash::init(poseidon_chip, layouter.namespace(|| "init hasher"), ConstantLength::<2>)?;

        let mut poseidon_hasher_2: PoseidonHash
            <
                Fp, 
                PoseidonChip<Fp>, 
                P128Pow5T3, 
                ConstantLength<2_usize>, 
                3_usize, 
                2_usize
            > 
                = PoseidonHash::init(poseidon_chip_2, layouter.namespace(|| "init hasher"), ConstantLength::<2>)?;

        let identity_trapdoor = self.load_private(
            layouter.namespace(|| "witness identity_trapdoor"),
            config.advices[0],
            self.identity_trapdoor,
        )?;

        let identity_nullifier = self.load_private(
            layouter.namespace(|| "witness identity_nullifier"),
            config.advices[0],
            self.identity_nullifier,
        )?;

        let external_nulifier = self.load_private(
            layouter.namespace(|| "witness external nullifier"),
            config.advices[0],
            self.external_nullifier
        )?;

        self.load_private(
            layouter.namespace(|| "witness root"),
            config.advices[0],
            self.root,
        )?;

        let identity_commitment_message = [identity_trapdoor, identity_nullifier];
        let identity_commitment_loaded = poseidon_hasher.witness_message_pieces(
            config.poseidon_config.clone(),
            layouter.namespace(|| "identity commitment hash"),
            identity_commitment_message
        )?;

        let identity_commitment_word = poseidon_hasher.hash(layouter.namespace(|| "hash to identity commitment"), identity_commitment_loaded, "identity commitment")?;
        let identity_commitment: CellValue<Fp> = identity_commitment_word.inner().into();

        // println!("{:?}", identity_commitment.value());


        let nullifier_message = [identity_nullifier, external_nulifier];

        let nullifier_loaded = poseidon_hasher_2.witness_message_pieces(
            config.poseidon_config.clone(),
            layouter.namespace(|| "nullifier hash"),
            nullifier_message
        )?;

        let nullifier_hash_word = poseidon_hasher_2.hash(layouter.namespace(|| "hash to nullifier hash"), nullifier_loaded, "nullifier hash")?;
        let nullifier_hash: CellValue<Fp> = nullifier_hash_word.inner().into();

        println!("From circuit: {:?}", nullifier_hash.value());
    

        // let identity_commitment = add_chip.add(layouter.namespace(|| "commitment"), identity_nullifier, identity_trapdoor)?;
        // let nullifier_hash = add_chip.add(layouter.namespace(|| "nullifier"), identity_nullifier, external_nulifier)?;


        // let merkle_inputs = MerklePath {
        //     chip: merkle_chip,
        //     leaf_pos: self.position_bits,
        //     path: self.path
        // };

        // let calculated_root = merkle_inputs.calculate_root(
        //     layouter.namespace(|| "merkle root calculation"),
        //     identity_commitment
        // )?;

        
        // self.expose_public(layouter.namespace(|| "constrain external_nullifier"), config.instance, external_nulifier, EXTERNAL_NULLIFIER)?;
        // self.expose_public(layouter.namespace(|| "constrain nullifier_hash"), config.instance, nullifier_hash, NULLIFIER_HASH)?;
        // self.expose_public(layouter.namespace(|| "constrain root"), config.instance, calculated_root, ROOT)?;

        Ok({})
    }
}


fn main() {
    use halo2::{dev::MockProver};

    use crate:: {
        primitives::poseidon::{Hash}
    };

    let k = 7;

    let identity_trapdoor = Fp::from(2);
    let identity_nullifier = Fp::from(3);
    let external_nullifier = Fp::from(5);
    let path = [Fp::from(1), Fp::from(1), Fp::from(1), Fp::from(1)];
    let position_bits = [Fp::from(0), Fp::from(1), Fp::from(0), Fp::from(1)];
    let identity_commitment = identity_trapdoor + identity_nullifier;
    let message = [identity_trapdoor, identity_nullifier];
    // let identity_commitment = PoseidonHash::init(P128Pow5T3, ConstantLength::<2>).hash(message);
    // let nullifier_hash = identity_nullifier + external_nullifier;

    let message = [identity_nullifier, external_nullifier];
    let nullifier_hash = Hash::init(P128Pow5T3, ConstantLength::<2>).hash(message);
    println!("From main: {:?}", Some(nullifier_hash));

    let root = identity_commitment + Fp::from(4);

    let circuit = SemaphoreCircuit {
        identity_trapdoor: Some(identity_trapdoor),
        identity_nullifier: Some(identity_nullifier),
        external_nullifier: Some(external_nullifier),
        position_bits: Some(position_bits),
        path: Some(path),
        root: Some(root)
    };

    let mut public_inputs = vec![external_nullifier, nullifier_hash, root];

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    // public_inputs[0] += Fp::one();
    // let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    // assert!(prover.verify().is_err());
}
