use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dao_core::{
    align::{Align, PolicyWeights},
    sense::Sense,
    upstream::UpstreamState,
    Intent,
};
use std::sync::Arc;
use std::time::Duration;

fn make_upstream(name: &str, latency_ms: u64, ok_ratio: u64) -> UpstreamState {
    let upstream = UpstreamState::new(
        name.to_string(),
        format!("http://{}.local", name),
        vec![Intent::new("realtime"), Intent::new("batch")],
        1,
    );

    for i in 0..64 {
        let ok = i % 100 < ok_ratio;
        upstream.record_request(Duration::from_millis(latency_ms + (i % 5)), ok);
    }

    upstream
}

fn build_align(upstreams: Arc<Vec<UpstreamState>>) -> Align {
    let sense = Sense::new(upstreams);
    let mut align = Align::new(sense);
    align.register_policy(
        "resonant".to_string(),
        PolicyWeights {
            w_load: 0.6,
            w_intent: 0.3,
            w_tempo: 0.1,
        },
    );
    align
}

fn bench_select_upstream(c: &mut Criterion) {
    let mut group = c.benchmark_group("dao_select_upstream");

    for upstream_count in [4usize, 8, 16, 32] {
        let upstreams: Vec<_> = (0..upstream_count)
            .map(|idx| make_upstream(&format!("upstream-{idx}"), 10 + idx as u64, 95))
            .collect();
        let arcs: Vec<_> = upstreams.iter().cloned().map(Arc::new).collect();
        let align = build_align(Arc::new(upstreams));
        let intent = Intent::new("realtime");

        group.bench_with_input(
            BenchmarkId::from_parameter(upstream_count),
            &upstream_count,
            |b, _| {
                b.iter(|| {
                    let selected = align.select_upstream(
                        "resonant",
                        black_box(&arcs),
                        Some(black_box(&intent)),
                    );
                    black_box(selected);
                });
            },
        );
    }

    group.finish();
}

fn bench_explain_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("dao_explain_selection");

    for upstream_count in [4usize, 8, 16] {
        let upstreams: Vec<_> = (0..upstream_count)
            .map(|idx| make_upstream(&format!("candidate-{idx}"), 15 + idx as u64, 92))
            .collect();
        let arcs: Vec<_> = upstreams.iter().cloned().map(Arc::new).collect();
        let align = build_align(Arc::new(upstreams));
        let intent = Intent::new("batch");

        group.bench_with_input(
            BenchmarkId::from_parameter(upstream_count),
            &upstream_count,
            |b, _| {
                b.iter(|| {
                    let explanation = align.explain_selection(
                        "bench-route",
                        "resonant",
                        black_box(&arcs),
                        Some(black_box(&intent)),
                    );
                    black_box(explanation);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_select_upstream, bench_explain_selection);
criterion_main!(benches);
