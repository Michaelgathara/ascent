use std::{
    fs::File,
    io::{prelude::*, BufReader},
    time::Instant,
};

use ascent::*;

ascent_par! {
    struct CSPA;

    relation assign(u32, u32);
    relation dereference(u32, u32);

    relation value_flow(u32, u32);
    relation value_alias(u32, u32);
    relation memory_alias(u32, u32);

    // ValueFlow(y, x) :- Assign(y, x).
    // ValueFlow(x, y) :- Assign(x, z), MemoryAlias(z, y).
    // ValueFlow(x, y) :- ValueFlow(x, z), ValueFlow(z, y).
    // MemoryAlias(x, w) :- Dereference(y, x), ValueAlias(y, z), Dereference(z, w).
    // ValueAlias(x, y) :- ValueFlow(z, x), ValueFlow(z, y).
    // ValueAlias(x, y) :- ValueFlow(z, x), MemoryAlias(z, w),ValueFlow(w, y).
    // ValueFlow(x, x) :- Assign(x, y).
    // ValueFlow(x, x) :- Assign(y, x).
    // MemoryAlias(x, x) :- Assign(y, x).
    // MemoryAlias(x, x) :- Assign(x, y).

    value_flow(y, x) <-- assign(y, x);
    value_flow(x, y) <-- assign(x, z), memory_alias(z, y);
    value_flow(x, y) <-- value_flow(x, z), value_flow(z, y);
    memory_alias(x, w) <-- dereference(y, x), value_alias(y, z), dereference(z, w);
    value_alias(x, y) <-- value_flow(z, x), value_flow(z, y);
    value_alias(x, y) <-- value_flow(z, x), memory_alias(z, w), value_flow(w, y);
    value_flow(x, x) <-- assign(x, y);
    value_flow(x, x) <-- assign(y, x);
    memory_alias(x, x) <-- assign(y, x);
    memory_alias(x, x) <-- assign(x, y);

}

fn main() {
    let mut cspa = CSPA::default();

    let assign_facts_file =
        File::open("/home/benches/dataset/dataset/linux/assign.facts").expect("file not found");
    let assign_facts_reader = BufReader::new(assign_facts_file);
    cspa.assign = assign_facts_reader
        .lines()
        .map(|row| {
            let row = row.unwrap();
            let mut iter = row.split_whitespace();
            let x = iter.next().unwrap().parse::<u32>().unwrap();
            let y = iter.next().unwrap().parse::<u32>().unwrap();
            (x, y)
        })
        .collect();
    println!("assign file loaded");

    let dereference_facts_file =
        File::open("/home/benches/dataset/dataset/linux/dereference.facts").expect("file not found");
    let dereference_facts_reader = BufReader::new(dereference_facts_file);

    cspa.dereference = dereference_facts_reader
        .lines()
        .map(|row| {
            let row = row.unwrap();
            let mut iter = row.split_whitespace();
            let x = iter.next().unwrap().parse::<u32>().unwrap();
            let y = iter.next().unwrap().parse::<u32>().unwrap();
            (x, y)
        })
        .collect();
    println!("dereference file loaded");

    let start = Instant::now();
    cspa.run();
    let duration = start.elapsed();
    println!("Time elapsed in run() is: {:?}", duration);
    println!("value_flow: {}", cspa.value_flow.len());
    println!("value_alias: {}", cspa.value_alias.len());
    println!("memory_alias: {}", cspa.memory_alias.len());
}
