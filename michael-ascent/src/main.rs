use std::time::Instant;
use boxcar::Vec;
use ascent::*;


// https://arxiv.org/pdf/2103.15217

pub mod tc {
    use ascent::ascent_par;
    ascent_par! {
        pub struct TCProgram;
        relation edge(i32, i32);
        relation path(i32, i32);

        path(*x, *y) <-- edge(x, y);
        path(*x, *z) <-- edge(x, y), path(y, z);
    }
}

fn loop_graph(nodes: usize) -> Vec<(i32, i32)> {
    let res = Vec::new();
    let nodes = nodes as i32;
    for x in 0..nodes {
        res.push((x, (x + 1) % nodes));
    }
    res
}

// fn complete_graph(nodes: usize) -> Vec<(i32, i32, u32)> {
//     let mut res = vec![];
//     let nodes = nodes as i32;
//     for x in 0..nodes {
//         for y in 0..nodes {
//             if x != y {
//                 res.push((x, y, 1));
//             }
//         }
//     }
//     res
// }

/**
 * This function benchmarks given a graph
 */
fn bench_tc_for_graph(graph: Vec<(i32, i32)>, name: &str) {
    // let mut tc = AscentProgram::default();
    // for i in 0..nodes_count {
    //     tc.edge.push((i, i + 1));
    // }

    let before = Instant::now();

    let mut tc = tc::TCProgram::default();
    // tc.edge = graph;
    tc.edge = graph;

    tc.run();

    let elapsed = before.elapsed();
    // println!("tc for {} took {:?}", name, elapsed);
    // println!("path size: {}", tc.path.len());
}

/**
 * This will generate a random graph and feed it to bench_tc_for_graph()
 */
// fn random_graph(nodes: usize) -> Vec<(i32, i32)> {}

fn bench_tc_path_join_path(nodes_count: i32) {
    let linear_graph = loop_graph(nodes_count as usize);
    bench_tc_for_graph(linear_graph, "linear graph");
}

fn main() {
    bench_tc_path_join_path(100_000);
}
