extern crate rsex;

use rsex::binance::spot_ws::{BinanceWs, WsEvent};
use rsex::traits::SpotWs;
use std::{env, thread};
use log::{info, warn};

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let mut symbol = "BTCUSDT".to_string();
    if args.len() > 1 {
        symbol = args[1].to_uppercase();
    }
    if !symbol.ends_with("USDT") {
        symbol += "USDT";
    }

    let handler = |event: WsEvent| {
        match event {
            WsEvent::TickerEvent(e) => {
                println!("{:?}", e);
            }
            _ => {
                println!("{:?}", event);
            }
        }
        Ok(())
    };

    let url = "wss://stream.binance.com:9443/ws/btcusdt@bookTicker";
    // let url = "wss://stream.binance.com:9443/stream?streams=btcusdt@bookTicker/ethusdt@bookTicker/eosusdt@bookTicker";
    // let url = "wss://stream.binance.com:9443/stream";
    let mut ws = BinanceWs::new(url.into());
    ws.connect(handler);
    println!("connected");
    ws.sub_ticker(&symbol);
}
