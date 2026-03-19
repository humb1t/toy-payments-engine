#![allow(clippy::unwrap_used)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use primitive_fixed_point_decimal::fpdec;
use toy_payments_engine::{
    State,
    Transaction,
    process_transactions,
    process_transactions_parallel,
    transaction::{Deposit, Withdrawal},
};

fn generate_transactions(num_clients: u16, transactions_per_client: u32) -> Vec<Transaction> {
    let mut transactions = Vec::new();
    let mut tx_id: u32 = 0;

    for client_id in 1..=num_clients {
        for _ in 0..transactions_per_client {
            tx_id += 1;
            let amount = fpdec!(100.0);
            transactions.push(Transaction::Deposit(Deposit::new(client_id, tx_id, amount)));
        }
        for _ in 0..transactions_per_client / 2 {
            tx_id += 1;
            let amount = fpdec!(50.0);
            transactions.push(Transaction::Withdrawal(Withdrawal::new(client_id, tx_id, amount)));
        }
    }
    transactions
}

fn bench_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential");

    for num_clients in [10, 100, 1000, 10000u16] {
        let transactions = generate_transactions(num_clients, 10);
        let total_txs = transactions.len();

        group.throughput(Throughput::Elements(total_txs as u64));
        group.bench_with_input(
            BenchmarkId::new("clients", num_clients),
            &transactions,
            |b, transactions: &Vec<Transaction>| {
                b.iter(|| {
                    let mut state = State::default();
                    process_transactions(black_box(transactions.clone().into_iter().map(Ok)), &mut state)
                        .expect("processing failed");
                    black_box(state);
                });
            },
        );
    }
    group.finish();
}

fn bench_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel");
    let shard_count = 10;

    for num_clients in [10, 100, 1000, 10000u16] {
        let transactions = generate_transactions(num_clients, 10);
        let total_txs = transactions.len();

        group.throughput(Throughput::Elements(total_txs as u64));
        group.bench_with_input(
            BenchmarkId::new("clients", num_clients),
            &transactions,
            |b, transactions: &Vec<Transaction>| {
                b.iter(|| {
                    let result = process_transactions_parallel(black_box(transactions.clone()), shard_count);
                    black_box(result.expect("processing failed"));
                });
            },
        );
    }
    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");
    let shard_count = 10;

    for num_clients in [100, 1000, 10000u16] {
        let transactions = generate_transactions(num_clients, 10);
        let total_txs = transactions.len();

        group.throughput(Throughput::Elements(total_txs as u64));

        group.bench_with_input(
            BenchmarkId::new("sequential", num_clients),
            &transactions,
            |b, transactions: &Vec<Transaction>| {
                b.iter(|| {
                    let mut state = State::default();
                    process_transactions(black_box(transactions.clone().into_iter().map(Ok)), &mut state)
                        .expect("processing failed");
                    black_box(state);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel", num_clients),
            &transactions,
            |b, transactions: &Vec<Transaction>| {
                b.iter(|| {
                    let result = process_transactions_parallel(black_box(transactions.clone()), shard_count);
                    black_box(result.expect("processing failed"));
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_sequential, bench_parallel, bench_comparison);
criterion_main!(benches);
