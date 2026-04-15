use crate::indicator::engine::IndicatorEngine;
use crate::indicator::ast::CompiledIndicator;
use crate::models::candle::Candle;
use crate::optimizer::population::Population;
use crate::optimizer::fitness::fitness;
use crate::optimizer::evolution::{crossover, mutate};
use crate::optimizer::genome::{Genome, genome_to_strategy};

#[derive(Debug)]
pub struct OptimizationResult {
    pub best_genome: Genome,
    pub buy_condition: String,
    pub sell_condition: String,
    pub score: f64,
    pub backtest: crate::backtest::types::BacktestResult,
}

pub fn run_optimizer(
    candles: &[Candle],
    compiled: &CompiledIndicator,
    population_size: usize,
    generations: usize,
) -> OptimizationResult {
    let mut population = Population::new(population_size);
    let mut best: Option<(f64, Genome, crate::backtest::types::BacktestResult)> = None;

    for generation in 0..generations {
        let mut scored: Vec<_> = population
            .individuals
            .iter()
            .map(|genome| {
                let indicator = IndicatorEngine::new(compiled.clone());
                let (score, result) = fitness(genome, candles, indicator);
                (score, genome.clone(), result)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((score, genome, result)) = scored.first() {
            best = Some((*score, genome.clone(), result.clone()));
        }

        let survivors_count = (population_size as f64 * 0.2).max(1.0) as usize;
        let survivors: Vec<_> = scored
            .iter()
            .take(survivors_count)
            .map(|(_, genome, _)| genome.clone())
            .collect();

        let mut next = survivors.clone();
        while next.len() < population_size {
            let a = &survivors[fastrand::usize(..survivors.len())];
            let b = &survivors[fastrand::usize(..survivors.len())];
            let mut child = crossover(a, b);
            mutate(&mut child);
            next.push(child);
        }

        population.individuals = next;

        println!("Gen {} Best Score: {}", generation + 1, scored.first().map_or(0.0, |entry| entry.0));
    }

    let (score, best_genome, backtest) = best.expect("Optimizer should always produce a best genome");
    let (buy_condition, sell_condition) = genome_to_strategy(&best_genome);

    OptimizationResult {
        best_genome,
        buy_condition,
        sell_condition,
        score,
        backtest,
    }
}
