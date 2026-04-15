use crate::optimizer::genome::Genome;

pub struct Population {
    pub individuals: Vec<Genome>,
}

impl Population {
    pub fn new(size: usize) -> Self {
        let mut individuals = Vec::with_capacity(size);

        for _ in 0..size {
            individuals.push(Genome::random());
        }

        Self { individuals }
    }
}
