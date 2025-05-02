use acir::acir_field::FieldElement;
type Fr = FieldElement;
use acir::AcirField;

// use ark_ff::Field;
use bn254_blackbox_solver::poseidon2_permutation;

const RATE: usize = 3;
const STATE_SIZE: usize = 4;

pub struct Poseidon2 {
    cache: [Fr; RATE],
    state: Vec<Fr>,
    cache_size: usize,
    squeeze_mode: bool,
}

impl Poseidon2 {
    pub fn new(iv: Fr) -> Self {
        let mut state = vec![Fr::zero(); STATE_SIZE];
        state[RATE] = iv;

        Poseidon2 {
            cache: [Fr::zero(); RATE],
            state,
            cache_size: 0,
            squeeze_mode: false,
        }
    }

    fn perform_duplex(&mut self) {
        for i in 0..RATE {
            if i < self.cache_size {
                self.state[i] += self.cache[i];
            }
        }

        // Replace with your actual Poseidon2 permutation function
        self.state = poseidon2_permutation(&self.state, 4).unwrap();
    }

    fn absorb(&mut self, input: Fr) {
        assert!(!self.squeeze_mode);

        if self.cache_size == RATE {
            self.perform_duplex();
            self.cache[0] = input;
            self.cache_size = 1;
        } else {
            self.cache[self.cache_size] = input;
            self.cache_size += 1;
        }
    }

    fn squeeze(&mut self) -> Fr {
        assert!(!self.squeeze_mode);
        self.perform_duplex();
        self.squeeze_mode = true;
        self.state[0]
    }

    pub fn hash(input: &[Fr], is_variable_length: bool) -> Fr {
        let iv = Fr::from(input.len() as u64) * Fr::from(1u128 << 64);
        let mut sponge = Poseidon2::new(iv);

        for i in 0..input.len() {
            sponge.absorb(input[i]);
        }

        if is_variable_length {
            sponge.absorb(Fr::from(1u64));
        }

        sponge.squeeze()
    }
}
