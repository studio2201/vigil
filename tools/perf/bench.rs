// tools/perf/bench.rs — §18 bench harness for vigil.
//
// Lives as a `#[test]` in tests/integration.rs via `include!` so it is
// exercised by `cargo test --release perf_vigil_scan_within_budget`.
// Honors §18: std::time only, median-of-5, line-oriented output.
//
// Budget: vigil scan on 10k deps (synthetic) ≤ 800 ms median ±25%.

use std::time::Instant;

#[test]
fn perf_vigil_scan_within_budget() {
    let fixture = synth_manifest_with_n_deps(10_000);
    let budget_ms: f64 = 800.0;
    let tolerance: f64 = 0.25;
    let ceiling_ms = budget_ms * (1.0 + tolerance);

    // Warm-up: prime allocator and caches, but do not measure.
    let _ = vigil::scan(&fixture, &policy());

    // Five timed runs.
    let mut samples: Vec<f64> = Vec::with_capacity(5);
    for _ in 0..5 {
        let t = Instant::now();
        let _ = vigil::scan(&fixture, &policy());
        let elapsed_ms = t.elapsed().as_secs_f64() * 1000.0;
        samples.push(elapsed_ms);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ms = samples[2]; // median of 5

    let pass = median_ms <= ceiling_ms;
    println!(
        "verb=vigil_scan median_ms={:.3} budget_ms={:.0} pass={}",
        median_ms, budget_ms, pass
    );
    assert!(
        pass,
        "vigil_scan regression: median {:.1}ms > ceiling {:.1}ms",
        median_ms, ceiling_ms
    );
}

// Fixture builders — synthetic, committed (per §18-C5).
fn synth_manifest_with_n_deps(n: usize) -> vigil::Manifest {
    let mut dependencies = Vec::with_capacity(n);
    for i in 0..n {
        dependencies.push(vigil::Dependency {
            name: format!("synth-dep-{}", i),
            version: "1.0.0".to_string(),
            is_dev: i % 4 == 0,
            days_inactive: Some((i % 500) as u32),
        });
    }
    vigil::Manifest {
        kind: vigil::ManifestKind::PackageJson,
        path: "synthetic-package.json".to_string(),
        dependencies,
    }
}

fn policy() -> vigil::Policy {
    vigil::Policy::default()
}
