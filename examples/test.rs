extern crate rsex;

use rsex::binance::spot_ws::{BinanceWs, WsEvent};
use rsex::traits::SpotWs;
use std::{env, thread};
use log::{info, warn};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::time::Duration;

fn main() {
    env_logger::init();

    println!("{}", "btcusdt".to_ascii_uppercase());
    thread::spawn(spawn_function1);
    // thread::sleep(Duration::from_millis(8000));
    // let mut map = BTreeMap::new();
    // map.insert(OrderedFloat(0.4),0.5);
    // map.insert(OrderedFloat(2.0/5.0),0.7);
    // if 0.4-0.40==0.0 {
    //     println!("{:?}", map);
    // }
}

fn spawn_function1() {
    thread::spawn(spawn_function2);
}

fn spawn_function2() {
    thread::sleep(Duration::from_millis(5000));
    println!("spawn_function2 running", );
}