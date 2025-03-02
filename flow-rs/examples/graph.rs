/**
 * \file flow-rs/examples/graph.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use anyhow::Result;
use clap::clap_app;
use flow_rs::prelude::*;
use std::time::Instant;

#[inputs(inp)]
#[outputs(out)]
#[derive(Node, Actor)]
struct Transport {
    _name: String,
}

impl Transport {
    fn new(name: String, _: &toml::value::Table) -> Transport {
        Transport {
            _name: name,
            inp: Default::default(),
            out: Default::default(),
        }
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}

    async fn exec(&mut self, _: &Context) -> Result<()> {
        if let Ok(envelope) = self.inp.recv::<u32>().await {
            self.out.send(envelope).await.ok();
        }
        Ok(())
    }
}

node_register!("Transport", Transport);

#[flow_rs::rt::main]
async fn main() {
    let matches = clap_app!(graph => 
        (version: "1.0")
        (author: "megvii")
        (@arg NODE_NUM: -n --node [N] "node num")
        (@arg CONCURRENCY: -c --concurrency [N] "concurrency")
        (@arg DATA_NUM: -d --data [N] "data num"))
    .get_matches();
    let node_num = matches
        .value_of("NODE_NUM")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap();
    let concurrency = matches
        .value_of("CONCURRENCY")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap();
    let data_num = matches
        .value_of("DATA_NUM")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap();

    let mut connects = vec![];
    let mut nodes = vec![];
    for i in 1..=node_num {
        nodes.push(format!(
            r#"
[[graphs.nodes]]
name = "trans{}"
ty = "Transport"
cloned = {}
            "#,
            i, concurrency
        ))
    }
    for i in 1..node_num {
        connects.push(format!(
            r#"
[[graphs.connections]]
cap = 64
ports = ["trans{}:out", "trans{}:inp"]
                "#,
            i,
            i + 1
        ));
    }
    let config = vec![
        format!(
            r#"
    main = "Example"
    [[graphs]]
    name = "Example"
    inputs=[{{name="in", cap=64, ports=["trans1:inp"]}}]
    outputs=[{{name="out", cap=64, ports=["trans{}:out"]}}]"#,
            node_num
        ),
        connects.join("\n"),
        nodes.join("\n"),
    ]
    .join("\n");

    let mut graph = load(None, config.as_str()).unwrap();
    let input = graph.input("in").unwrap();
    let output = graph.output("out").unwrap();
    let handle = graph.start();
    let sf = async_std::task::spawn(async move {
        for _ in 0..data_num {
            input
                .send(flow_rs::envelope::Envelope::new(16u32))
                .await
                .ok();
        }
        input
    });
    let rf = async_std::task::spawn(async move {
        for _ in 0..data_num {
            output.recv::<u32>().await.ok();
        }
    });
    let start = Instant::now();
    let (input, _) = futures_util::future::join(sf, rf).await;
    let duration = start.elapsed();
    println!(
        "Time elapsed in `example/graph.rs` [length:{}, width:{}, datas:{}] is: {:?}",
        node_num, concurrency, data_num, duration
    );
    input.close();
    handle.await;
}
