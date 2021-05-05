use crate::binance::types::*;
use crate::errors::*;
use crate::models::*;
use crate::traits::*;

use log::{info, warn};
use ws::{Handler, Handshake, Message, Result, Sender};
use std::thread;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::collections::BTreeMap;
use lazy_static::lazy_static;
use std::sync::{RwLock, Arc};
use crate::binance::spot_rest::Binance;
use std::iter::FromIterator;
// use std::ops::Index;


//static WEBSOCKET_URL: &str = "wss://stream.binance.com:9443/ws/btcusdt@depth20";

#[derive(Debug)]
pub enum WsEvent {
    // public stream
    OrderbookEvent(String, Orderbook),
    DepthEvent(String, DepthOrderbookEvent),
    KlineEvent(String, Kline),
    TickerEvent(String, Ticker),
    TradeEvent(Trade),
    ResponseEvent(ResponseEvent),

    // private stream
    AccountUpdateEvent(AccountUpdateEvent),
    OrderTradeEvent(OrderTradeEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseEvent {
    id: u64,
    result: Option<String>,
}

pub struct BinanceWs<'a> {
    host: String,
    subs: Vec<String>,
    out: Option<Sender>,

    handler: Box<dyn FnMut(WsEvent) -> Result<()> + 'a>,
}

impl<'a> BinanceWs<'a> {
    pub fn new(host: &str) -> Self {
        BinanceWs {
            host: host.into(),
            subs: vec![],
            out: None,
            handler: Box::new(|event| {
                info!("event: {:?}", event);
                Ok(())
            }),
        }
    }

    pub fn connect<Callback: Clone>(&mut self, handler: Callback)
    where
        Callback: FnMut(WsEvent) -> Result<()> + 'a,
    {
        info!("connect begin");
        let res = ws::connect(self.host.clone(), |out| BinanceWs {
            host: self.host.clone(),
            subs: self.subs.clone(),
            out: Some(out),
            handler: Box::new(handler.clone()),
        });

        info!("connect result: {:?}", res);
        res.unwrap();
    }

    fn send(&self, msg: &str) {
        match &self.out {
            Some(out) => {
                let _ = out.send(msg);
            }
            None => {
                warn!("self.out is None");
            }
        }
    }

    fn deseralize(&self, s: &str) -> APIResult<WsEvent> {
        if s.find("result") != None {
            let resp: ResponseEvent = serde_json::from_str(s)?;
            return Ok(WsEvent::ResponseEvent(resp));
        }
        //let val: Value = serde_json::from_str(s)?;
        if s.find("\"stream\"") != None {
            if s.find("kline") != None {
                let resp: StreamMessage<KlineEvent> = serde_json::from_str(&s)?;
                Ok(WsEvent::KlineEvent(resp.get_symbol(), resp.data.kline.into()))
            } else if s.find("lastUpdateId") != None {
                let resp: StreamMessage<RawOrderbook> = serde_json::from_str(&s)?;
                Ok(WsEvent::OrderbookEvent(resp.get_symbol(), resp.data.into()))
            } else if s.find("aggTrade") != None {
                let resp: StreamMessage<TradeEvent> = serde_json::from_str(&s)?;
                Ok(WsEvent::TradeEvent(resp.data.into()))
            } else if s.find("A") != None && s.find("B") != None {
                let resp: StreamMessage<BookTickerEvent> = serde_json::from_str(&s)?;
                Ok(WsEvent::TickerEvent(resp.get_symbol(),resp.data.into()))
            }else if s.find("\"depthUpdate\"") != None{
                let resp: StreamMessage<DepthOrderbookEvent> = serde_json::from_str(&s)?;
                Ok(WsEvent::DepthEvent(resp.get_symbol(), resp.data.into()))
            } else {
                Err(Box::new(ExError::ApiError("msg channel not found".into())))
            }
        }else {
            if s.find("kline") != None {
                let resp: KlineEvent = serde_json::from_str(&s)?;
                Ok(WsEvent::KlineEvent(resp.symbol, resp.kline.into()))
            } else if s.find("lastUpdateId") != None {
                // let resp: RawOrderbook = serde_json::from_str(&s)?;
                // Ok(WsEvent::OrderbookEvent(resp.into()))
                Err(Box::new(ExError::ApiError("Single orderbook not support".into())))
            } else if s.find("aggTrade") != None {
                let resp: TradeEvent = serde_json::from_str(&s)?;
                Ok(WsEvent::TradeEvent(resp.into()))
            } else if s.find("A") != None && s.find("B") != None {
                let resp: BookTickerEvent = serde_json::from_str(&s)?;
                Ok(WsEvent::TickerEvent(resp.symbol.clone(), resp.into()))
            } else {
                Err(Box::new(ExError::ApiError("msg channel not found".into())))
            }


        }
    }



    pub fn build_local_orderbook(steams_ws_url:&str, rest_url:&str, symbols:Vec<String>){

        let mut symbols_str:Vec<&str> = Vec::new();

        let mut endpoints: Vec<String> = Vec::new();

        for symbol in symbols.iter() {
            symbols_str.push(&symbol[..]);
            endpoints.push(format!("{}@depth@100ms", symbol.to_lowercase()));
        }
        init_static_local_orderbook(&symbols_str);

        let handler = |event: WsEvent| {
            match event {
                WsEvent::DepthEvent(s,doe) => {
                    // println!("{:?}", &doe);
                    let arc_local_orderbook = Arc::clone(&LOCAL_ORDERBOOK);
                    let mutex_hashmap = arc_local_orderbook.read().unwrap();
                    let mutex_local_orderbook = mutex_hashmap.get(&s).unwrap();
                    let mut local_orderbook = mutex_local_orderbook.write().unwrap();
                    if local_orderbook.on_depth_event(doe){
                        let url = String::from(rest_url);
                        let symbol = s.clone();
                        thread::spawn(move || {
                            get_depth_snapshot(url, symbol);
                        });
                    }
                }
                WsEvent::TickerEvent(s,e) => {
                    println!("{:?}: {:?}",s, e);
                }
                _ => {
                    println!("{:?}", event);
                }
            }
            Ok(())
        };
        let full_ws_url = format!("{}/stream?streams={}",steams_ws_url,endpoints.join("/"));
        let mut ws = BinanceWs::new(&full_ws_url[..]);
        ws.connect(handler);
    }
}

fn get_depth_snapshot(rest_url:String, symbol:String){
    info!("get depth snapshot by rest, symbol: {}", &symbol);
    let rest = Binance::new(None, None, rest_url);
    let ret = rest.get_orderbook(&symbol[..], 1000);
    match ret{
        Ok(orderbook) =>{
            let arc_local_orderbook = Arc::clone(&LOCAL_ORDERBOOK);
            let mutex_hashmap = arc_local_orderbook.read().unwrap();
            let mutex_local_orderbook = mutex_hashmap.get(&symbol).unwrap();
            let mut local_orderbook = mutex_local_orderbook.write().unwrap();
            if local_orderbook.rest_update_id==0{
                local_orderbook.save_depth_snapshot(orderbook);
            }
        },
        Err(error) =>{
            println!("get orderbook error: {}", error);
        },
    }
}

// fn timestamp() -> i64 {
//     let start = SystemTime::now();
//     let since_the_epoch = start
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards");
//     let ms = since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64;
//     ms
// }

#[derive(Debug,  Clone)]
pub struct LocalOrderBook{
    symbol:String,
    asks:BTreeMap<OrderedFloat<f64>,f64>,
    bids:BTreeMap<OrderedFloat<f64>,f64>,
    depth_cache:BTreeMap<u64,DepthOrderbookEvent>,
    rest_update_id:u64,
    ws_final_update_id:u64,
    ready: bool,
}

impl LocalOrderBook{
    fn new(symbol:String) -> LocalOrderBook{
        LocalOrderBook{
            symbol,
            asks:BTreeMap::new(),
            bids:BTreeMap::new(),
            depth_cache:BTreeMap::new(),
            rest_update_id:0,
            ws_final_update_id:0,
            ready:false,
        }
    }

    pub fn print_order_book(symbol:&str){
        let arc_local_orderbook = Arc::clone(&LOCAL_ORDERBOOK);
        let mutex_hashmap = arc_local_orderbook.read().unwrap();
        let mutex_local_orderbook = mutex_hashmap.get(symbol).unwrap();
        let lob = mutex_local_orderbook.read().unwrap();
        println!("Local Orderbook:\nasks:\n{:?}\nbids:\n{:?}", &lob.asks, &lob.bids);
    }

    pub fn query_buy_price(symbol: &str, amount: f64) -> (f64,bool) {
        let arc_local_orderbook = Arc::clone(&LOCAL_ORDERBOOK);
        let mutex_hashmap = arc_local_orderbook.read().unwrap();
        let mutex_local_orderbook = mutex_hashmap.get(&symbol.to_lowercase()).unwrap();
        let lob = mutex_local_orderbook.read().unwrap();
        if !lob.ready{
            return (0.0, false);
        }
        let mut map = BTreeMap::new();
        let mut accumulative_amount = 0.0;
        for en in &(lob.asks){
            if amount - accumulative_amount <= *en.1 {
                map.insert(en.0, amount-accumulative_amount);
                accumulative_amount = amount;
                break;
            }else{
                map.insert(en.0, *en.1);
                accumulative_amount += *en.1;
            }
        }
        if accumulative_amount == amount{
            let mut total_money = 0.0;
            for en in map{
                total_money += en.0.into_inner()*en.1;
            }
            return (total_money/amount, true);
        }
        warn!("{} total asks amount {} is not enough, return price 999999999999.0", symbol, amount);
        (999999999999.0, true)
    }

    pub fn query_sell_price(symbol: &str, amount: f64) -> (f64,bool) {
        let arc_local_orderbook = Arc::clone(&LOCAL_ORDERBOOK);
        let mutex_hashmap = arc_local_orderbook.read().unwrap();
        let mutex_local_orderbook = mutex_hashmap.get(&symbol.to_lowercase()).unwrap();
        let lob = mutex_local_orderbook.read().unwrap();
        if !lob.ready{
            return (0.0, false);
        }
        let mut map = BTreeMap::new();
        let mut accumulative_amount = 0.0;
        let mut vec = Vec::from_iter(&(lob.bids));
        vec.reverse();
        for en in vec{
            if amount - accumulative_amount <= *en.1 {
                map.insert(en.0, amount-accumulative_amount);
                accumulative_amount = amount;
                break;
            }else{
                map.insert(en.0, *en.1);
                accumulative_amount += *en.1;
            }
        }
        if accumulative_amount == amount{
            let mut total_money = 0.0;
            for en in map{
                total_money += en.0.into_inner()*en.1;
            }
            return (total_money/amount, true);
        }
        warn!("{} total bids amount {} is not enough, return price 0.0", symbol, amount);
        (0.0, true)
    }

    fn save_depth_snapshot(&mut self, orderbook:Orderbook){
        for entry in &(orderbook.asks){
            self.asks.insert(OrderedFloat(entry.price), entry.amount);
        }
        for entry in &(orderbook.bids){
            self.bids.insert(OrderedFloat(entry.price), entry.amount);
        }
        self.rest_update_id = orderbook.timestamp;
    }

    fn update(&mut self, final_update_id:u64, depth_event: DepthOrderbookEvent) -> bool {
        if final_update_id<=self.rest_update_id {
            return true;
        // }else if depth_event.first_update_id<=self.rest_update_id+1&&depth_event.final_update_id>=self.rest_update_id+1{
        }else{
            if self.ws_final_update_id!=0&&self.ws_final_update_id+1!=depth_event.first_update_id{
                info!("package lost, rebuild symbol {} local orderbook", &(self.symbol));
                self.ready = false;
                self.rest_update_id =0;
                self.ws_final_update_id =0;
                self.depth_cache.clear();
                self.depth_cache.insert(final_update_id, depth_event);
                return false;
            }
            for entry in &(depth_event.asks) {
                if entry.qty==0.0{
                    self.asks.remove(&OrderedFloat(entry.price));
                }else {
                    self.asks.insert(OrderedFloat(entry.price), entry.qty);
                }
            }
            for entry in &(depth_event.bids){
                if entry.qty==0.0{
                    self.bids.remove(&OrderedFloat(entry.price));
                }else {
                    self.bids.insert(OrderedFloat(entry.price), entry.qty);
                }
            }
            // info!("first_update_id: {}, final_update_id: {}", depth_event.first_update_id,depth_event.final_update_id);
            self.ws_final_update_id = final_update_id;
        // }else{
        //     warn!("symbol {} unknown branch, ready: {}, first_update_id: {}, final_update_id: {}", &(self.symbol), self.ready, depth_event.first_update_id,depth_event.final_update_id);
        }
        true
    }

    fn on_depth_event(&mut self, depth_event:DepthOrderbookEvent) -> bool{
        if self.rest_update_id==0{
            self.depth_cache.insert(depth_event.final_update_id, depth_event);
            return self.depth_cache.len()>30&&self.depth_cache.len()%30==0;
        }else if self.depth_cache.len()>0{
            info!("rest depth snapshot is done, update symbol {} local orderbook", &(self.symbol));
            let depth_cache = self.depth_cache.clone();
            for en in depth_cache{
                if !self.update(en.0,en.1.clone() ){
                    return false;
                }
            }
            self.depth_cache.clear();
            self.ready=true;
            info!("build local orderbook finished, symbol: {}", &(self.symbol));

        }
        false
    }
}

lazy_static! {
    static ref LOCAL_ORDERBOOK:Arc<RwLock<HashMap<String, RwLock<LocalOrderBook>>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn init_static_local_orderbook(symbols:&Vec<&str>){
    let lo = Arc::clone(&LOCAL_ORDERBOOK);
    let mut map = lo.write().unwrap();
    for symbol in symbols{
        map.insert(symbol.to_lowercase(), RwLock::new(LocalOrderBook::new(String::from(symbol.to_lowercase()))));
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamMessage<E> {
    stream: String,
    data: E,
}

impl<E> StreamMessage<E> {
    fn get_symbol(&self) -> String{
        self.stream.split('@').next().unwrap().to_owned()
    }
}


impl<'a> SpotWs for BinanceWs<'a> {
    fn sub_kline(&mut self, symbol: &str, period: &str) {
        let msg = format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@kline_{}\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            period,
            self.subs.len() + 1,
        );
        self.send(msg.as_str());
        self.subs.push(msg);
    }

    fn sub_orderbook(&mut self, symbol: &str) {
        let msg = format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@depth20\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        );
        self.send(msg.as_str());
        self.subs.push(msg);
    }

    fn sub_trade(&mut self, symbol: &str) {
        let msg = format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@aggTrade\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        );
        self.send(msg.as_str());
        self.subs.push(msg);
    }

    fn sub_ticker(&mut self, symbol: &str) {
        let msg = format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@bookTicker\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        );
        info!("{:?}", msg);
        self.send(msg.as_str());
        self.subs.push(msg);
    }

    fn sub_order_update(&mut self, _symbol: &str) {
        unimplemented!()
    }
}

impl<'a> Handler for BinanceWs<'a> {
    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        match &self.out {
            // /*
            Some(out) => self.subs.iter().for_each(|s| {
                let _ = out.send(s.as_str());
            }),
            // // */
            // Some(_) => {
            //     info!("ws connected");
            // }
            None => {
                warn!("self.out is None");
            }
        }
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        // info!("{:?}", msg);
        match self.deseralize(&msg.to_string()) {
            Ok(event) => {
                let _ = (self.handler)(event);
            }
            Err(err) => {
                warn!("deseralize msg error: {:?}", err);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static WEBSOCKET_URL: &str = "wss://stream.binance.com:9443/ws/btcusdt@depth20";
    //#[test]
    fn test_binancews() {
        env_logger::init();

        let handler = |event: WsEvent| {
            match event {
                WsEvent::OrderbookEvent(s, e) => {
                    info!("orderbook: {:?}", e);
                }
                _ => {
                    info!("event: {:?}", event);
                }
            }
            Ok(())
        };
        let mut binance = BinanceWs::new(WEBSOCKET_URL);
        binance.connect(handler);
        //binance.sub_orderbook("btcusdt");
        binance.sub_ticker("btcusdt");
        binance.sub_kline("btcusdt", "5m");
        binance.sub_trade("btcusdt");
    }
}
