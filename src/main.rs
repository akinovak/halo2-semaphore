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
    merkle::{MerkleChip, MerkleConfig, MerklePath},
    poseidon::{Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig, Hash as PoseidonHash}
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

impl SemaphoreCircuit {
    fn hash(
        &self,
        config: Config,
        mut layouter: impl Layouter<Fp>,
        message: [CellValue<Fp>; 2],
        to_hash: &str,
    ) -> Result<CellValue<Fp>, Error> {
        let config = config.clone();

        let poseidon_chip = config.construct_poseidon_chip();

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

        let loaded_message = poseidon_hasher.witness_message_pieces(
            config.poseidon_config,
            layouter.namespace(|| format!("witnessing: {}", to_hash)),
            message
        )?;

        let word = poseidon_hasher.hash(layouter.namespace(|| format!("hashing: {}", to_hash)), loaded_message)?;
        let digest: CellValue<Fp> = word.inner().into();

        Ok(digest)
    }
}

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
        let merkle_config = MerkleChip::configure(meta, advices[0..3].try_into().unwrap(), poseidon_config.clone());

        Config {
            advices, 
            instance,
            merkle_config,
            poseidon_config,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {

        let merkle_chip = config.construct_merkle_chip();

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
        let identity_commitment = self.hash(
            config.clone(), 
            layouter.namespace(|| "hash to identity commitment"),
            identity_commitment_message,
            "identity commitment"
        )?;

        // println!("Identity Commitment: {:?}", identity_commitment.value());

        let nullifier_hash_message = [identity_nullifier, external_nulifier];
        let nullifier_hash = self.hash(
            config.clone(), 
            layouter.namespace(|| "hash to nullifier hash"),
            nullifier_hash_message,
            "nullifier hash"
        )?;

        // println!("Nullifier hash: {:?}", nullifier_hash.value());

        let merkle_inputs = MerklePath {
            chip: merkle_chip,
            leaf_pos: self.position_bits,
            path: self.path
        };

        let calculated_root = merkle_inputs.calculate_root(
            layouter.namespace(|| "merkle root calculation"),
            identity_commitment
        )?;
        
        self.expose_public(layouter.namespace(|| "constrain external_nullifier"), config.instance, external_nulifier, EXTERNAL_NULLIFIER)?;
        self.expose_public(layouter.namespace(|| "constrain nullifier_hash"), config.instance, nullifier_hash, NULLIFIER_HASH)?;
        self.expose_public(layouter.namespace(|| "constrain root"), config.instance, calculated_root, ROOT)?;
        Ok({})
    }
}


fn main() {
    use halo2::{dev::MockProver};

    use crate:: {
        primitives::poseidon::{Hash}
    };

    let k = 10;

    let identity_trapdoor = Fp::from(2);
    let identity_nullifier = Fp::from(3);
    let external_nullifier = Fp::from(5);
    let path = [Fp::from(1), Fp::from(1), Fp::from(1), Fp::from(1)];
    let position_bits = [Fp::from(0), Fp::from(0), Fp::from(0), Fp::from(0)];

    let message = [identity_nullifier, external_nullifier];
    let nullifier_hash = Hash::init(P128Pow5T3, ConstantLength::<2>).hash(message);

    let commitment_message = [identity_trapdoor, identity_nullifier];
    let identity_commitment = Hash::init(P128Pow5T3, ConstantLength::<2>).hash(commitment_message);

    let mut root = identity_commitment;

    for el in path {
        root = Hash::init(P128Pow5T3, ConstantLength::<2>).hash([root, el]);
    }

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
    public_inputs[0] += Fp::one();
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert!(prover.verify().is_err());
}
