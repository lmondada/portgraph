use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use portgraph::graph::{Graph, GraphMut, PortGraph};
use portgraph::substitute::{BoundedSubgraph, OpenGraph, Rewrite, Substitute};

/// Create line graph, connected with two parallel edges at each step.
///
/// o -2-> o -2-> o -2-> o -2-> o   ...
///
fn make_line_graph(size: usize) -> PortGraph<usize, isize> {
    let mut graph = PortGraph::with_capacity(size, size * 2);
    let mut prev_node = graph.add_node_with_ports(0, vec![], vec![-1, -2]);

    for i in 1..size {
        let w = i as isize;
        let new_node = graph.add_node_with_ports(i, vec![w, w + 1], vec![-w, -w - 1]);
        graph.link_nodes(prev_node, 0, new_node, 0).unwrap();
        graph.link_nodes(prev_node, 1, new_node, 1).unwrap();
        prev_node = new_node;
    }

    graph
}

/// Create an acyclic graph with `layer` layers and 2 nodes per layer, connected sequentially as follows.
///
/// o ---> o ---> o ---> o ---> o   ...
/// |                    |
/// v                    v
/// o ---> o ---> o ---> o ---> o   ...
///
fn make_two_track_dag(layers: usize) -> PortGraph<usize, isize> {
    let mut graph = PortGraph::with_capacity(2 * layers, 6 * layers);
    if layers == 0 {
        return graph;
    } else if layers == 1 {
        graph.add_node(0, 1, 1);
        graph.add_node(1, 1, 1);
        return graph;
    }

    let mut prev_nodes = [
        graph.add_node_with_ports(0, vec![-1], vec![-1]),
        graph.add_node_with_ports(0, vec![-2], vec![-2]),
    ];

    for layer in 1..layers {
        let i = layer * 2;
        let w = i as isize;
        let new_nodes = [
            graph.add_node_with_ports(i, vec![-w], vec![-w, w]),
            graph.add_node_with_ports(i + 1, vec![-w - 1, w], vec![-w - 1]),
        ];
        graph.link_nodes(prev_nodes[0], 0, new_nodes[0], 0).unwrap();
        graph.link_nodes(prev_nodes[1], 0, new_nodes[1], 0).unwrap();
        // Add an edge connecting both nodes every third layer
        if layer % 3 == 0 {
            graph.link_nodes(new_nodes[0], 1, new_nodes[1], 1).unwrap();
        }
        prev_nodes = new_nodes;
    }

    graph
}

/// Remove one every five nodes from the graph.
fn remove_every_five<'a>(graph: &mut impl GraphMut<'a, usize, isize>) {
    let mut to_remove = Vec::new();
    for (i, v) in graph.nodes_iter().enumerate() {
        if i % 5 == 0 {
            to_remove.push(v);
        }
    }
    for node in to_remove {
        graph.remove_node(node);
    }
}

/// Remove nodes from the graph in an unordered way until it is empty.
fn remove_all_unordered<'a>(graph: &mut impl GraphMut<'a, usize, isize>) {
    while graph.node_count() > 5 {
        remove_every_five(graph);
    }
    // Remove all remaining nodes
    while graph.node_count() > 0 {
        graph.remove_node(graph.nodes_iter().next().unwrap());
    }
}

fn bench_make_portgraph(c: &mut Criterion) {
    let mut g = c.benchmark_group("graph creation");
    g.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for size in [100, 10_000, 1_000_000] {
        g.bench_with_input(
            BenchmarkId::new("make_line_graph", size),
            &size,
            |b, size| b.iter(|| black_box(make_line_graph(*size))),
        );
    }
    g.finish();
}

fn bench_clone_portgraph(c: &mut Criterion) {
    let mut g = c.benchmark_group("graph cloning");
    g.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for size in [100, 10_000, 1_000_000] {
        g.bench_with_input(
            BenchmarkId::new("clone_line_graph", size),
            &size,
            |b, size| {
                let graph = make_line_graph(*size);
                b.iter(|| black_box(graph.clone()))
            },
        );
    }
    g.finish();
}

fn bench_remove_unordered(c: &mut Criterion) {
    let mut g = c.benchmark_group("remove vertices unordered");
    g.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for size in [0, 100, 10_000] {
        g.bench_with_input(
            BenchmarkId::new("remove_vertices_unordered", size),
            &size,
            |b, size| {
                let graph = make_two_track_dag(*size);
                b.iter(|| black_box(remove_all_unordered(&mut graph.clone())))
            },
        );
    }
    g.finish();
}

fn generate_rewrite() -> (PortGraph<i8, i8>, Rewrite<PortGraph<i8, i8>, i8, i8>) {
    let mut g = PortGraph::<i8, i8>::with_capacity(3, 2);

    let n0 = g.add_node(0, 0, 2);
    let n1 = g.add_node(1, 1, 1);
    let n2 = g.add_node(2, 2, 0);

    g.link_nodes(n0, 0, n1, 0).unwrap();
    g.link_nodes(n1, 0, n2, 0).unwrap();

    let p0 = g.output(n0, 0).unwrap();
    let p1 = g.input(n2, 0).unwrap();

    let mut g2 = PortGraph::<i8, i8>::with_capacity(2, 1);
    // node to be inserted
    let n3 = g2.add_node(3, 1, 1);
    let p2 = g2.input(n3, 0).unwrap();
    let p3 = g2.output(n3, 0).unwrap();

    let rewrite = Rewrite::new(
        BoundedSubgraph::new([n1].into_iter().collect(), vec![p0], vec![p1]),
        OpenGraph::new(g2, vec![p2], vec![p3]),
    );

    (g, rewrite)
}

fn bench_apply_rewrite(c: &mut Criterion) {
    let mut g = c.benchmark_group("run a rewrite operation");
    g.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    g.bench_function("apply_rewrite", |b| {
        let (graph, rewrite) = generate_rewrite();
        b.iter(|| black_box(graph.clone().apply_rewrite(rewrite.clone())))
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_make_portgraph,
    bench_clone_portgraph,
    bench_remove_unordered,
    bench_apply_rewrite,
);
criterion_main!(benches);
