use crate::optimizer::genome::Genome;

pub fn crossover(a: &Genome, b: &Genome) -> Genome {
    Genome {
        rsi_period: if fastrand::bool() { a.rsi_period } else { b.rsi_period },
        buy_threshold: (a.buy_threshold + b.buy_threshold) / 2.0,
        sell_threshold: (a.sell_threshold + b.sell_threshold) / 2.0,
    }
}

pub fn mutate(genome: &mut Genome) {
    if fastrand::bool(0.3) {
        genome.buy_threshold += fastrand::f64() * 10.0 - 5.0;
    }

    if fastrand::bool(0.3) {
        genome.sell_threshold += fastrand::f64() * 10.0 - 5.0;
    }

    if fastrand::bool(0.2) {
        genome.rsi_period = fastrand::usize(5..30);
    }

    genome.normalize();
}
