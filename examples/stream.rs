extern crate rsex;

use rsex::binance::spot_ws::{BinanceWs, LocalOrderBook};
use std::{thread};
// use log::{info, warn};
use std::time::Duration;

fn main() {
    env_logger::init();
    // let args: Vec<String> = env::args().collect();
    // let mut symbol = "BTCUSDT".to_string();
    // if args.len() > 1 {
    //     symbol = args[1].to_uppercase();
    // }
    // if !symbol.ends_with("USDT") {
    //     symbol += "USDT";
    // }
    //
    // let handler = |event: WsEvent| {
    //     match event {
    //         WsEvent::TickerEvent(s,e) => {
    //             println!("{:?}: {:?}",s, e);
    //         }
    //         _ => {
    //             println!("{:?}", event);
    //         }
    //     }
    //     Ok(())
    // };
    //
    // // let url = "wss://stream.binance.com:9443/ws/btcusdt@bookTicker";
    // let url = "wss://stream.binance.com:9443/stream?streams=btcusdt@bookTicker/ethusdt@bookTicker/eosusdt@bookTicker/btcusdt@depth/ethusdt@depth/eosusdt@depth";
    // let url = "wss://stream.binance.com:9443/stream?streams=btcusdt@depth/ethusdt@depth/eosusdt@depth";
    // // let url = "wss://stream.binance.com:9443/stream";
    // let mut ws = BinanceWs::new(url);
    // ws.connect(handler);
    // println!("connected");
    // ws.sub_ticker(&symbol);
    thread::spawn(move || {
        // BinanceWs::build_local_orderbook("wss://stream.binance.com:9443","https://api.binance.com",vec!["btcusdt"]);//, "ethusdt"
        BinanceWs::build_local_orderbook("wss://testnet.binance.vision","https://testnet.binance.vision",vec![ String::from("btcusdt"),String::from("ethusdt"),String::from("eosusdt")]);
    });
    thread::sleep(Duration::from_millis(150000));
    LocalOrderBook::print_order_book("btcusdt");
}
