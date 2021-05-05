use crate::errors::*;
use crate::models::*;

use log::{ error};


pub trait SpotRest {
    fn get_symbol(&self, base_currency: &str, trade_currency: &str) -> String;
    fn get_symbols(&self) -> APIResult<Vec<SymbolInfo>>;
    fn get_balance(&self, asset: &str) -> APIResult<Balance>;
    fn create_order(
        &self,
        symbol: &str,
        price: f64,
        amount: f64,
        action: &str,
        order_type: &str,
        client_order_id: &str,
    ) -> APIResult<String>;
    fn create_market_order(
        &self,
        symbol: &str,
        amount: f64,
        action: &str,
        client_order_id: &str,
    ) -> APIResult<String>;
    fn create_market_order_with_retry(
        &self,
        symbol: &str,
        amount: f64,
        action: &str,
        client_order_id: &str,
        retry_times: u8,
    ) -> APIResult<String>{
        let mut i:u8 = 0;
        while i < retry_times{
            i=i+1;
            let ret = self.create_market_order(symbol,amount,action, client_order_id);
            if let Err(error)=ret{
                println!("create_market_order failed, client_order_id: {}, error: {:?}", client_order_id, error);
                if i == retry_times{
                    return Err(error);
                }
            }else{
                return Ok(ret?.into());
            }
        }
        panic!("create_market_order panic")
    }

    fn create_limit_order(
        &self,
        symbol: &str,
        price: f64,
        amount: f64,
        action: &str,
        client_order_id: &str,
    ) -> APIResult<String>;
    fn cancel(&self, symbol: &str, id: &str) -> APIResult<bool>;
    fn cancel_all(&self, symbol: &str) -> APIResult<bool>;
    fn cancel_all_with_retry(&self, symbol: &str, retry_times: u8) -> APIResult<bool>{
        let mut i:u8 = 0;
        while i < retry_times{
            i=i+1;
            let ret = self.cancel_all(symbol);
            if let Err(error)=ret{
                println!("cancel_all_with_retry failed, error: {:?}", error);
                if i == retry_times{
                    return Err(error);
                }
            }else{
                return Ok(ret?.into());
            }
        }
        panic!("cancel_all_with_retry panic")
    }
    fn get_order(&self, symbol: &str, id: &str) -> APIResult<Order>;
    fn get_order_with_retry(&self, symbol: &str, id: &str, retry_times: u8) -> APIResult<Order>{
        let mut i:u8 = 0;
        while i < retry_times{
            i=i+1;
            let ret = self.get_order(symbol, id);
            if let Err(error)=ret{
                error!("get_order_with_retry failed, order id: {}, error: {:?}", id, error);
                if i == retry_times{
                    return Err(error);
                }
            }else{
                return Ok(ret?.into());
            }
        }
        panic!("get_order_with_retry panic")
    }

    fn get_order_by_client_id(&self, symbol: &str, client_order_id: &str) -> APIResult<Order>;
    fn get_order_by_client_id_with_retry(&self, symbol: &str, client_order_id: &str, retry_times: u8) -> APIResult<Order>{
        let mut i:u8 = 0;
        while i < retry_times{
            i=i+1;
            let ret = self.get_order_by_client_id(symbol, client_order_id);
            if let Err(error)=ret{
                error!("get_order_by_client_id_with_retry failed, client order id: {}, error: {:?}", client_order_id, error);
                if i == retry_times{
                    return Err(error);
                }
            }else{
                return Ok(ret?.into());
            }
        }
        panic!("get_order_by_client_id_with_retry panic")
    }
    fn get_open_orders(&self, symbol: &str) -> APIResult<Vec<Order>>;
    fn get_history_orders(&self, symbol: &str) -> APIResult<Vec<Order>>;

    fn get_orderbook(&self, symbol: &str, depth: u32) -> APIResult<Orderbook>;
    fn get_ticker(&self, symbol: &str) -> APIResult<Ticker>;
    fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> APIResult<Vec<Kline>>;

    fn query_buy_price(&self, symbol:&str, amount:f64) -> (f64, bool);
    fn query_sell_price(&self, symbol:&str, amount:f64) -> (f64, bool);
}

pub trait FutureRest {

    fn get_balance(&self, asset: &str) -> APIResult<Balance>;
    fn create_order(
        &self,
        symbol: &str,
        price: f64,
        amount: f64,
        action: &str,
        order_type: &str,
    ) -> APIResult<String>;
    fn cancel(&self, id: &str) -> APIResult<bool>;
    fn cancel_all(&self, symbol: &str) -> APIResult<bool>;
    fn get_order(&self, id: &str) -> APIResult<Order>;
    fn get_open_orders(&self, symbol: &str) -> APIResult<Vec<Order>>;
    fn get_history_orders(&self, symbol: &str) -> APIResult<Vec<Order>>;

    fn get_orderbook(&self, symbol: &str, depth: u32) -> APIResult<Orderbook>;
    fn get_ticker(&self, symbol: &str) -> APIResult<Ticker>;
    fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> APIResult<Vec<Kline>>;
}

pub trait SpotWs {
    fn sub_orderbook(&mut self, symbol: &str);
    fn sub_kline(&mut self, symbol: &str, period: &str);
    fn sub_ticker(&mut self, symbol: &str);
    fn sub_trade(&mut self, symbol: &str);

    fn sub_order_update(&mut self, symbol: &str);
}

pub trait FutureWs {
    fn sub_orderbook(&self, symbol: &str);
    fn sub_kline(&self, symbol: &str, period: &str);
    fn sub_ticker(&self, symbol: &str);
    fn sub_trade(&self, symbol: &str);

    fn sub_order_update(&self, symbol: &str);
}
