// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Benchmarks for parallel vs sequential VRF calculations

extern crate criterion;
use self::criterion::*;
use akd_core::ecvrf::{VRFExpandedPrivateKey, VRFPublicKey};
use akd_core::VersionFreshness;
use akd_core::{ecvrf::VRFKeyStorage, AkdLabel, AkdValue};
use rand::distributions::Alphanumeric;
use rand::Rng;

fn bench_single_vrf(c: &mut Criterion) {
    let rng = rand::rngs::OsRng;

    // Generate a random label
    let label = AkdLabel::from(
        &rng.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>(),
    );

    let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    let key = runtime
        .block_on(akd_core::ecvrf::HardCodedAkdVRF.get_vrf_private_key())
        .unwrap();
    let expanded_key = VRFExpandedPrivateKey::from(&key);
    let pk = VRFPublicKey::from(&key);

    c.bench_function("Single VRF label generation", |b| {
        b.iter(|| {
            akd_core::ecvrf::HardCodedAkdVRF::get_node_label_with_expanded_key(
                &expanded_key,
                &pk,
                &label,
                VersionFreshness::Fresh,
                1,
            );
        })
    });
}

fn bench_parallel_vrfs(c: &mut Criterion) {
    // utilize all cores available
    let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    // A runtime which is capped at 4 worker threads (cores)
    let limited_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();

    // generate 1K labels to do VRFs for
    let labels = (0..1_000)
        .into_iter()
        .map(|i| {
            let name = format!("user {}", i);
            (
                AkdLabel::from(&name),
                VersionFreshness::Fresh,
                i as u64,
                AkdValue::from(&name),
            )
        })
        .collect::<Vec<_>>();
    let labels_clone = labels.clone();

    c.bench_function("Sequential VRFs", |b| {
        b.iter(|| {
            let key = runtime
                .block_on(akd_core::ecvrf::HardCodedAkdVRF.get_vrf_private_key())
                .unwrap();
            let expanded_key = VRFExpandedPrivateKey::from(&key);
            let pk = VRFPublicKey::from(&key);
            for (label, stale, version, _) in labels.iter() {
                akd_core::ecvrf::HardCodedAkdVRF::get_node_label_with_expanded_key(
                    &expanded_key,
                    &pk,
                    label,
                    *stale,
                    *version,
                );
            }
        })
    });

    c.bench_function("Parallel VRFs (all cores)", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let vrf = akd_core::ecvrf::HardCodedAkdVRF;
                vrf.get_node_labels(&labels_clone).await.unwrap();
            })
        })
    });

    c.bench_function("Parallel VRFs (4 cores)", |b| {
        b.iter(|| {
            limited_runtime.block_on(async {
                let vrf = akd_core::ecvrf::HardCodedAkdVRF;
                vrf.get_node_labels(&labels_clone).await.unwrap();
            })
        })
    });
}

criterion_group!(benches, bench_single_vrf, bench_parallel_vrfs);
criterion_main!(benches);
