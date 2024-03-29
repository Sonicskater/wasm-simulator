use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use itertools::{Group, GroupBy, Itertools};

use clap::{App, Arg};
use wasmer::{Module, Store};
use fifo::FiFo;
use gdsize::GdSize;
use lfu::LFU;
use lru::LRU;
use simulator_shared_types::FileRecord;
use crate::cached_policy::{WasmCachedBincodePolicyModule, WasmCachedBytemuckPolicyModule, WasmCachedPairPolicyModule};
use crate::native_modules::NativePolicyModule;
use crate::policy::{PolicyModule, WasmBincodePolicyModule, WasmBytemuckPolicyModule, WasmPairPolicyModule};

mod policy;
mod native_modules;
mod cached_policy;

use plotters::prelude::*;


fn main() {

    let matches = App::new("Caching Policy Simulator")
        .version("0.1")
        .author("Devon Hockley")
        .about("A caching policy simulator implemented using WASM")
        .arg(Arg::with_name("sample")
            .help("Sets the input data sample")
            .index(1)
            .required(true)
        )
        .get_matches();

    let file_path = matches.value_of("sample").unwrap();

    println!("Using input file: {}", file_path);



    let file = File::open(file_path).unwrap();
    let mut decompressed = GzDecoder::new(file);



    let mut string = String::new();
    decompressed.read_to_string(&mut string).unwrap();

    let data : Vec<FileRecord<i32>> = string.lines().map(
        |line| {
            if let [first, second, ..] = line.trim().split_ascii_whitespace().collect::<Vec<&str>>().as_slice() {
                (i32::from_str(first).unwrap(),i64::from_str(second).unwrap())
            } else {
                panic!()
            }
        }
    ).map(
        |(first, second)| {
            FileRecord::<i32>{
                label : first,
                size : second
            }
        }
    ).collect();

    let mut map = HashMap::<i32,i64>::new();
    for i in data.clone(){
        if map.contains_key(&i.label){

        } else {
            map.insert(i.label, i.size);
        }
    }

    let data : Vec<FileRecord<i32>> = data.iter().map(|i | {
        let size = map.get(&i.label).unwrap();
        FileRecord::<i32>{
            label: i.label,
            size: *size
        }
    }).collect();

    let mut results : Vec<SimResult> = vec![];

    //let module_names = vec!["wasm_pair_fifo"];

    let mut size : i64 = 512 * 1024 * 4;
    while size < 1024*1024*1024*8 {
        size *= 2;

        let store = Store::default();

        let mut policies: Vec<(&str,Box<dyn PolicyModule<i32>>)> = vec![];
        policies.push((
            "Native FiFo"
             ,Box::new(NativePolicyModule::<FiFo<i32>,i32>::new()
            )
        ));



        let wasm_pair = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_fifo.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmPairPolicyModule::from_module(module))
        };

        policies.push(("WASM Pair FiFo",wasm_pair));

        let wasm_pair = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_fifo.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedPairPolicyModule::from_module(module))
        };

        policies.push(("WASM Cached Pair FiFo",wasm_pair));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_fifo.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBincodePolicyModule::from_module(module))
        };

        policies.push(("WASM Bincode FiFo",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_fifo.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBincodePolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bincode FiFo",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_fifo.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBytemuckPolicyModule::from_module(module))
        };

        policies.push(("WASM Bytemuck FiFo",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_fifo.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBytemuckPolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bytemuck FiFo",wasm_bincode));

        policies.push(("Native LRU",Box::new(NativePolicyModule::<LRU<i32>,i32>::new())));
        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_lru.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmPairPolicyModule::from_module(module))
        };

        policies.push(("WASM Pair LRU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_lru.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedPairPolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Pair LRU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_lru.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBincodePolicyModule::from_module(module))
        };

        policies.push(("WASM Bincode LRU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_lru.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBincodePolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bincode LRU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_lru.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBytemuckPolicyModule::from_module(module))
        };

        policies.push(("WASM Bytemuck LRU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_lru.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBytemuckPolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bytemuck LRU",wasm_bincode));


        policies.push(("Native LFU",Box::new(NativePolicyModule::<LFU<i32>,i32>::new())));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_lfu.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmPairPolicyModule::from_module(module))
        };

        policies.push(("WASM Pair LFU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_lfu.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedPairPolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Pair LFU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_lfu.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBincodePolicyModule::from_module(module))
        };

        policies.push(("WASM Bincode LFU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_lfu.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBincodePolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bincode LFU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_lfu.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBytemuckPolicyModule::from_module(module))
        };

        policies.push(("WASM Bytemuck LFU",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_lfu.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBytemuckPolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bytemuck LFU",wasm_bincode));

        policies.push(("Native GdSize",Box::new(NativePolicyModule::<GdSize<i32>,i32>::new())));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_gdsize.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmPairPolicyModule::from_module(module))
        };

        policies.push(("WASM Pair GdSize",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_pair_gdsize.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedPairPolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Pair GdSize",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_gdsize.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBincodePolicyModule::from_module(module))
        };

        policies.push(("WASM Bincode GdSize",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_bincode_gdsize.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBincodePolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bincode GdSize",wasm_bincode));


        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_gdsize.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmBytemuckPolicyModule::from_module(module))
        };

        policies.push(("WASM Bytemuck GdSize",wasm_bincode));

        let wasm_bincode = {
            let path = Path::new("./modules/wasm32-unknown-unknown/release/wasm_c_gdsize.wasm");
            let module = Module::from_file(&store,path).expect("Module Not Found");
            Box::new(WasmCachedBytemuckPolicyModule::from_module(module))
        };

        policies.push(("Cached WASM Bytemuck GdSize",wasm_bincode));

        for (name, mut policy) in policies {

            let start = std::time::Instant::now();
            policy.initialize(size);
            for file in &data {
                policy.send_request((*file).clone())
            }
            let (total, hits) = policy.stats();
            let end = std::time::Instant::now();


            let alg_string = match name {
                x if x.contains("FiFo") => Alg::Fifo,
                x if x.contains("GdSize") => Alg::GdSize,
                x if x.contains("LRU") => Alg::LRU,
                x if x.contains("LFU") => Alg::LFU,
                _ => panic!()
            };

            let a = SimResult{
                size: size,
                alg: alg_string,
                name: name,
                hits: hits,
                time: (end-start).as_secs_f64(),
                hitrate: (hits as f32/total as f32 * 100.0)
            };

            results.push(a)

        }

    }

    for (key,group) in &results.clone().into_iter().group_by(|a| a.size){
        println!("Size: {0:<10} ",key/(1024*1024));
        for a in group{
            println!("Name: {0:<30} | Hits: {1:<10} | Time: {2:<10} | Hitrate: {3:<10}", a.name, a.hits,a.time, a.hitrate);
            //println!("{0:<30} {1:<10} {2:<10} {3:<10}", a.name, a.hits, a.time, a.hitrate);
        }
    }


    let mode = if cfg!(debug_assertions){
        "Debug"
    } else {
        "Release"
    };

    let backend = if cfg!(feature = "llvm"){
        "LLVM"
    } else if cfg!(feature = "cranelift"){
        "Cranelift"
    } else if cfg!(feature = "singlepass"){
        "Singlepass"
    } else {
        panic!()
    };

    const D_YELLOW : RGBColor = RGBColor{
        0: 185,
        1: 185,
        2: 0
    };

    const D_GREEN : RGBColor = RGBColor{
        0: 0,
        1: 185,
        2: 0
    };

    const DR_YELLOW : RGBColor = RGBColor{
        0: 100,
        1: 100,
        2: 0
    };



    let colors: Vec<ShapeStyle> = vec![
                                        BLACK.mix(0.0).filled(),
                                        BLACK.filled(),
                                        DR_YELLOW.mix(0.7).filled(),
                                        DR_YELLOW.mix(0.4).filled(),
                                        BLUE.mix(0.7).filled(),
                                        BLUE.mix(0.4).filled(),
                                        D_GREEN.mix(0.7).filled(),
                                        D_GREEN.mix(0.4).filled()
    ];

    for (size,group) in &results.clone().into_iter().group_by(|a| a.size){

        let group : Vec<SimResult> = group.into_iter().collect();
        let file = format!("result_graphs/test_{}_{}.png", size/(1024*1024),mode);
        let file = Path::new(file.as_str());
        let root = BitMapBackend::new(file, (600, 400)).into_drawing_area();

        root.fill(&WHITE);

        let x_spec =(0u32..((group.len() + 4) as u32)).with_key_points(
            vec![4,12,20,28]
        );
        //let x_text_spec = plotters::prelude::ToGroupByRange::group_by(0u32..(group.len() as u32), 6).into_segmented();

        let y_range = if cfg!(debug_assertions){
            0f64..10f64
        } else {
            0f64..1f64
        };

        let caption = format!("Simulation Runtimes ({}) - {} MB", mode, size/(1024*1024));
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .caption(caption.as_str(),("sans-serif",30.0))
            .build_cartesian_2d(x_spec, y_range)
            .unwrap();

        &chart.configure_mesh()
            .x_labels(4)
            .x_label_formatter(&|x| {

                match x  {
                    4 => "FIFO",
                    12 => "LRU",
                    20 => "LFU",
                    28 => "GD-SIZE",
                    _ => {"E"}
                }.to_string()
            })
            .x_desc("Caching Policy")
            .y_desc("Runtime (s)")
            .draw().unwrap();


        let by_alg = group.into_iter().group_by(|a| a.alg);



        for (group,(_,i)) in by_alg.into_iter().enumerate(){

            let names = vec!["FILLER","Native","Pair","Cached Pair","Bincode","Cached Bincode","Bytemuck","Cached Bytemuck"];

            let i = i.map(|b| b.time);

            // create blank column to insert at start to separate groups
            let blank = std::iter::once(0.0); // blank column for spacer

            let data = blank
                .chain(i).enumerate().zip(colors.clone())
                .map(|((a,b),color)| (((group as u32)*8+a as u32,b),color)); // re-wrap data columns from structs to raw pair (might not be needed)

            // Combine and map to rectangles
            let rects = data.map(|((a,b),color)|{
                Rectangle::new([(a,0f64),(a+1,b)],color)
            });



            if group == 0 {
                for (i,rect) in rects.into_iter().enumerate(){
                    if i == 0 {
                        chart.draw_series(vec![rect]).unwrap();
                    } else {
                        let color = colors[i].clone();
                        chart.draw_series(vec![rect]).unwrap()
                            .label(names[i])
                            .legend(move |(x, y)| Rectangle::new([(x, y-5), (x + 20, y+5)], color.clone()),);
                    }

                }
            } else {
                chart.draw_series(rects);
            }


        }

        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .position(SeriesLabelPosition::UpperLeft)
            .draw().unwrap();


        root.present().unwrap();
    }

    let by_algs = results.into_iter().group_by(|a| a.alg);

    let natives = by_algs.into_iter().map(|(a,b)|
        b.group_by(|b| b.size)
            .into_iter()
            .map(|(a,mut b)|{
                b.next().unwrap()
            }).next().unwrap()
    ).into_iter().collect::<Vec<SimResult>>();

    println!("{:?}", natives);

    let natives_by_alg : Vec<(Alg,Vec<&SimResult>)> = natives.iter().group_by(|a| a.alg).into_iter().map(|(a,b)| (a,b.collect())).collect();

    println!("{:?}", natives_by_alg);

    let natives_by_alg : HashMap<Alg,Vec<SimResult>> = natives_by_alg.into_iter()
        .map(|(a,b)|
            (a,b.first().unwrap().clone().clone())
        ).into_group_map()
        .iter()
        .map(|(a,b)|
                 (a.clone(),b.clone())
        )
        .collect();

    // let natives: Vec<Vec<SimResult>> = by_algs.into_iter().map(|(a,b)|
    //     b.group_by(|a| a.size )
    //         .into_iter()
    //         .map(|(a,mut b)|
    //             b.next().unwrap()
    //         ).next().unwrap()
    // ).group_by(|a| a.alg).into_iter().collect();

    //println!("{:?}", natives_by_alg);


    let colors: Vec<RGBColor> = vec![RED,GREEN,BLUE,MAGENTA];

    {
        let file = format!("result_graphs/hitrate_{}.png",mode);
        let file = Path::new(file.as_str());
        let root = BitMapBackend::new(file, (600, 400)).into_drawing_area();

        let y_range = 70f64..100f64;
        let x_spec = (2i64*(1024*1024)..(150*1024*1024)).with_key_points(vec![(25*1024*1024),(50*1024*1024),(75*1024*1024),(100*1024*1024),(125*1024*1024)]);

        let caption = format!("Simulator Hitrates ({})",mode);

        root.fill(&WHITE);

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .caption(caption.as_str(),("sans-serif",30.0))
            .build_cartesian_2d(x_spec, y_range)
            .unwrap();

        &chart.configure_mesh()
            .x_labels(8)
            .x_label_formatter(&|x| {
                let mgs = (x / (1024*1024));
                format!("{} MB", mgs)
            })
            .x_desc("Cache Size")
            .y_desc("Hitrate (%)")
            .draw().unwrap();

        let display_order = vec![Alg::Fifo,Alg::LRU,Alg::LFU,Alg::GdSize];

        let results: Vec<Vec<SimResult>> = display_order.iter().map(|a|{
            natives_by_alg.get(a).unwrap().clone()
        }
        ).collect();

        for (i,c) in results.iter().zip(colors) {

            let name = match i.first().unwrap().alg {
                Alg::Fifo => "FiFo",
                Alg::GdSize => "GdSize",
                Alg::LFU => "LFU",
                Alg::LRU => "LRU",
            };
            let count = i.len();
            chart.draw_series(
                i.iter().map(|a|{
                   Circle::new((a.size,a.hitrate as f64),2, c.mix(0.5).filled())
                }).take(count-6)
            ).unwrap().label(name).legend(move |(x, y)| Rectangle::new([(x, y-5), (x + 20, y+5)], c.mix(0.5).filled()),);

            chart.draw_series(
                LineSeries::new(
                    i.iter().take(count-6).map(|a| ((a.size,a.hitrate as f64))),
                    c
                )
            );
        }
        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .position(SeriesLabelPosition::LowerRight)
            .draw().unwrap();


        root.present().unwrap();


    }


}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
enum Alg{
    Fifo,
    LFU,
    LRU,
    GdSize,
}

#[derive(Clone, Copy,Debug)]
struct SimResult{
    size: i64,
    alg: Alg,
    name: &'static str,
    hits: i32,
    time: f64,
    hitrate: f32
}