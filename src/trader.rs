mod trader_fancy_prints;
pub mod trader_errors;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use std::fs::File;
use std::rc::Rc;
use colored::Colorize;

use market_common::good::good::Good;

use market_common::good::good_kind::GoodKind;
use market_common::good::good_kind::GoodKind::*;
use market_common::market::good_label::GoodLabel;
use market_common::market::{Market, MarketGetterError};
use market_common::wait_one_day;
use std::io::Write;
use crate::trader::MarketKind::{BFB, DOGE, PANIC};
use crate::trader::trader_errors::{TraderDemandError, TraderSupplyError};


static TRADER_NAME : &str = "TASE Trader";
static MARKET_NOT_FOUND_MSG: fn(MarketKind) -> String = |market : MarketKind| {
    format!("Market \"{:?}\" not found!", market).red().to_string()
};
static DEFAULT_TRANSACTION_AMOUNT : f32 = 1000.0;
static INFINITY: f32 = 1_000_000.;

//this enum is utterly specific for our implementation and can't be generalized. Bad!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketKind {
    TASE,
    BOSE,
    DOGE,
    BFB,
    PANIC,
}

pub struct Trader {
    //todo: rename this field from "closure" to something else
    closure: Box<dyn Fn(&mut Trader)>,
    closure_just_modified: bool,

    //I hate this because Good already contains the GoodKind. If I used an u32 I would get issues in the buy function
    owned_goods: HashMap<GoodKind, Good>,

    markets: HashMap<MarketKind, Rc<RefCell<dyn Market>>>,

    pending_buy_orders: Vec<(MarketKind, String)>,
    pending_sell_orders: Vec<(MarketKind, String)>,

    //This is a very important and crucial field. It determines whether the trader gets free money after each transaction or not.
    amazingness: f32,

    // DATA for visualizer
    pub data: Vec<Vec<HashMap<GoodKind, Vec<f32>>>>,
    pub liquidity: HashMap<GoodKind, Vec<f32>>
}

impl Debug for Trader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            return write!(
                f,
                "➤ Trader Status:\n • Markets: {}\n • Money: {}",
                self.markets
                    .iter()
                    .map(|(_, t)| format!("\"{}\", ", (**t).borrow().get_name()))
                    .collect::<String>(),
                self.owned_goods.get(&EUR).unwrap()
            );
        }
        write!(
            f,
            "➤ Trader status: (Money: {})",
            self.owned_goods.get(&EUR).unwrap()
        )
    }
}

impl Drop for Trader {
    fn drop(&mut self) {
        //println!("{:?}", self.liquidity);
        // Save data to file
        let file = File::create("visualizer_data.txt");

        match file {
            Ok(mut file) => {

                let serialized_data = serde_json::to_string(&self.data);
                match serialized_data {
                    Ok(data) => {
                        write!(file, "{}", data).expect("The trader experienced an internal error while writing to \"visualizer_data.txt\".");
                    }
                    Err(e) => {
                        println!("{}", e.to_string());
                        println!("TRADER: couldn't serialize the data. The last state was NOT recorded to \"visualizer_data.txt\"");
                    }
                }

            }
            Err(e) => {
                println!("TRADER: couldn't open \"visualizer_data.txt\" because of one out of various reasons. Stacktrace:");
                println!("{}", e.to_string());
            }
        }
    }
}

impl Trader {

    fn initialize_data(&mut self, _market: &Rc<RefCell<dyn Market>>)  {
        let mut res = Vec::new();
        let h: HashMap<GoodKind, Vec<f32>> = HashMap::new();
        res.push(h); // sell
        let h: HashMap<GoodKind, Vec<f32>> = HashMap::new();
        res.push(h); // buy
        for op in res.iter_mut() {
            // op.insert(GoodKind::EUR, Vec::new());
            op.insert(GoodKind::USD, Vec::new());
            op.insert(GoodKind::YEN, Vec::new());
            op.insert(GoodKind::YUAN, Vec::new());
        }
        let h: HashMap<GoodKind, Vec<f32>> = HashMap::new();
        res.push(h); // liquidity
        
        {
            let op = res.get_mut(2).unwrap();
            op.insert(GoodKind::EUR, Vec::new());
            op.insert(GoodKind::USD, Vec::new());
            op.insert(GoodKind::YEN, Vec::new());
            op.insert(GoodKind::YUAN, Vec::new());
        }
        
        self.data.push(res);
    }

    pub fn new() -> Self {
        let mut owned_goods = HashMap::new();
        owned_goods.insert(EUR, Good::new(EUR, 1000.0));
        owned_goods.insert(USD, Good::new(USD, 0.0));
        owned_goods.insert(YEN,  Good::new(YEN, 0.0));
        owned_goods.insert(YUAN, Good::new(YUAN, 0.0));

        let mut liq: HashMap<GoodKind, Vec<f32>> = HashMap::new();
        for (gk, _) in owned_goods.iter() {
            liq.insert(*gk, Vec::new());
        }

        Trader {
            closure: Box::new(|_| {}),
            closure_just_modified: false,
            owned_goods,
            markets: HashMap::new(),
            pending_buy_orders: Vec::new(),
            pending_sell_orders: Vec::new(),
            amazingness: 1.0,
            data: Vec::new(),
            liquidity: liq 
        }
    }

    pub fn new_super_duper_amazing_trader(amazingness: f32) -> Self {
        let mut owned_goods = HashMap::new();
        owned_goods.insert(EUR, Good::new(EUR, 1000.0));
        owned_goods.insert(USD, Good::new(USD, 0.0));
        owned_goods.insert(YEN, Good::new(YEN, 0.0));
        owned_goods.insert(YUAN, Good::new(YUAN, 0.0));

        let mut liq: HashMap<GoodKind, Vec<f32>> = HashMap::new();
        for (gk, _) in owned_goods.iter() {
            liq.insert(*gk, Vec::new());
        }

        Trader {
            closure: Box::new(|_| {}),
            closure_just_modified: false,
            owned_goods,
            markets: HashMap::new(),
            pending_buy_orders: Vec::new(),
            pending_sell_orders: Vec::new(),
            amazingness,
            data: Vec::new(),
            liquidity: liq 
        }
    }

    pub fn with_market(mut self, kind: MarketKind, market: Rc<RefCell<dyn Market>>) -> Self {
        self.initialize_data(&market);
        self.markets.insert(kind, market);
        self
    }

    pub fn with_initial_money(mut self, money: f32) -> Self {
        *self.owned_goods.get_mut(&EUR).expect(format!("{}", "trader has no euros? Panic!".red()).as_str()) = Good::new(EUR, money);
        self
    }

    pub fn with_good(mut self, good : GoodKind, qty : f32) -> Self {
        self.owned_goods.insert(good, Good::new(good, qty));
        self
    }


    //I'm pretty sure this is absolutely terrible, but I really liked the idea of having a modifiable closure inside the trader.
    //I'll trust LegionMammal978 on this one: https://users.rust-lang.org/t/pass-a-closure-that-takes-a-mutable-reference-to-self/73843/3
    //This kind of backfired because given the fact that absolutely "noone" uses closures in this way, I had to write some acrobatic code to make it work.
    pub fn run(&mut self, iterations: i32){

        let mut i = 0;

        while i < iterations {

            //steal the struct's operation and replace it with a temporary value:
            let closure = std::mem::replace(&mut self.closure, Box::new(|_| { println!("the closure was replaced but not given back")}));
            closure(self);

            if self.closure_just_modified {
                self.closure_just_modified = false;
                //since the closure was modified, it means that the trader got in a new state. Therefore, we have to call the closure again.
                continue;
            }

            //
            self.closure = Box::new(closure);
            i += 1;

        }
    }

    fn save_data(&mut self) {
        let ordered_markets = vec![MarketKind::BFB, MarketKind::BOSE, MarketKind::DOGE];
        for (market_index, m) in ordered_markets.iter().enumerate() {
            let market = self.markets.get(m).unwrap();
            for (kind, _) in self.owned_goods.iter() {
                if *kind == EUR {
                    continue;
                }
                // SELL
                let quantity = 0.01;
                let price = match market.borrow().get_sell_price(*kind, quantity) {
                    Ok(p) => p/quantity,
                    Err(_) => INFINITY
                };
                self.data.get_mut(market_index).unwrap().get_mut(1).unwrap().get_mut(&kind).unwrap().push(price);

                // BUY
                let quantity = 0.01;
                let price = match market.borrow().get_buy_price(*kind, quantity) {
                    Ok(p) => p/quantity,
                    Err(_) => INFINITY
                };
                self.data.get_mut(market_index).unwrap().get_mut(0).unwrap().get_mut(&kind).unwrap().push(price);
            }

            for GoodLabel {good_kind, quantity, ..} in market.borrow().get_goods() {
                // LIQUIDITY
                self.data.get_mut(market_index).unwrap().get_mut(2).unwrap().get_mut(&good_kind).unwrap().push(quantity);
            }
        }

        // Trader good quantities
        for (gk, good) in self.owned_goods.iter() {
            self.liquidity.get_mut(gk).expect("Liquidity data lost something").push(good.get_qty());
        }
    }

    pub fn set_strategy(&mut self, function: impl Fn(&mut Trader) + 'static) {
        self.closure = Box::new(function);
        self.closure_just_modified = true;
    }

    pub fn get_owned_good_qty(&self, kind : GoodKind) -> f32 {
        self.owned_goods.get(&kind).expect(format!("trader has no {}? Panic!", kind).as_str()).get_qty()
    }

    //I (Dennis) renamed "buy" to "supply" because I was getting crazy in distinguishing between "buy" and "sell"
    pub fn get_supply_price(&self, market : MarketKind, kind : GoodKind) -> Result<f32, TraderSupplyError> {
        match self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market).as_str()).borrow().get_buy_price(kind, DEFAULT_TRANSACTION_AMOUNT) {
            Ok(price) => Ok(price),
            Err(e) => {
                match e {
                    MarketGetterError::InsufficientGoodQuantityAvailable { ..} => Err(TraderSupplyError::MarketInsufficientSupply),
                    _=> panic!("Unhandled error: {:?}", e)
                }
            }
        }
    }

    pub fn get_supply_price_qt(&self, market : MarketKind, kind : GoodKind, quantity : f32) -> Result<f32, TraderSupplyError> {
        match self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market).as_str()).borrow().get_buy_price(kind, quantity) {
            Ok(price) => Ok(price),
            Err(e) => {
                match e {
                    MarketGetterError::InsufficientGoodQuantityAvailable { ..} => Err(TraderSupplyError::MarketInsufficientSupply),
                    _=> panic!("Unhandled error: {:?}", e)
                }
            }
        }

    }

    pub fn print_market(&self, market : MarketKind) {
        let goods = self.get_market(market).unwrap().borrow().get_goods();
        println!("\n{:?}", market);
        goods.iter().for_each(|g| {
            println!("{:?}: {}", g.good_kind, g.quantity);
        });
    }



    //ai generated, should be fine
    pub fn get_demand_price(&self, market : MarketKind, kind : GoodKind) -> f32 {
        self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market).as_str()).borrow().get_sell_price(kind, DEFAULT_TRANSACTION_AMOUNT)
            .expect(format!("Couldn't get sell price for {} in market {:?}", kind, market).as_str())
    }
    //ai generated, should be fine
    pub fn get_demand_price_qt(&self, market : MarketKind, kind : GoodKind, quantity : f32) -> f32 {
        self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market).as_str()).borrow().get_sell_price(kind, quantity)
            .expect(format!("Couldn't get sell price for {} in market {:?}", kind, market).as_str())
    }

    //todo: abort the operation if you don't have enough money. Perhaps passing the "insufficientgoodquantityerror" to the output of this function?
    //returns an f32 representing the money you got from the transaction
    pub fn buy(&mut self, market : MarketKind, kind : GoodKind, amount : f32) -> Result<f32, TraderSupplyError> {

        let price = self.get_supply_price_qt(market, kind, amount)?;

        let token = self.get_market(market)?.borrow_mut()
            .lock_buy(kind, amount, price, TRADER_NAME.to_string())?;

        let bought_goods = self.get_market(market)?.borrow_mut().buy(token, self.owned_goods.get_mut(&EUR).unwrap())?;

        //save value because the goods will lose ownership
        let value = bought_goods.get_qty();

        self.owned_goods.get_mut(&kind).expect(format!("{} disappeard from the trader's internal hashmap. Panic!", kind).as_str())
            .merge(bought_goods).expect("Couldn't add the bought goods to the trader's internal hashmap. Panic!");

        self.save_data();

        Ok(value)
    }

    pub fn lock_without_buying(&mut self, market : MarketKind, kind : GoodKind, amount : f32) -> Result<(String, f32), TraderSupplyError> {

        let price = self.get_supply_price_qt(market, kind, amount)?;

        let token = self.get_market(market)?.borrow_mut()
            .lock_buy(kind, amount, price, TRADER_NAME.to_string())?;
        self.save_data();

        Ok((token, price))
    }

    pub fn lock_without_selling(&mut self, market : MarketKind, kind : GoodKind, amount : f32) -> Result<(String, f32), TraderDemandError> {

        let price = self.get_demand_price_qt(market, kind, amount);

        let token = self.get_market(market)?.borrow_mut()
            .lock_sell(kind, amount, price, TRADER_NAME.to_string())?;
        self.save_data();

        Ok((token, price))
    }

    pub fn get_market(&self, market : MarketKind) -> Result<Rc<RefCell<dyn Market>>, TraderSupplyError> {
        let market = self.markets.get(&market).ok_or(TraderSupplyError::MarketNotFound)?;
        Ok(Rc::clone(market))
    }

    //AI generated, should be fine
    //Nevermind it was not fine: Dennis fixed it.
    pub fn sell(&mut self, market : MarketKind, kind : GoodKind, amount : f32) -> Result<f32, TraderDemandError> {

        let price = self.get_demand_price_qt(market, kind, amount);

        let token = self.get_market(market)?
            .borrow_mut()
            .lock_sell(kind, amount, price, TRADER_NAME.to_string())?;

        let sold_goods = self.get_market(market)?
            .borrow_mut()
            .sell(token, self.owned_goods.get_mut(&kind).expect(format!("Trader has no {}", kind).as_str()))?;
        let value = sold_goods.get_qty();

        self.owned_goods.get_mut(&EUR).expect(format!("{} disappeard from the trader's internal hashmap. Panic!", kind).as_str())
            .merge(sold_goods).expect("Couldn't add the sold goods to the trader's internal hashmap. Panic!");
        
        self.save_data();
        Ok(value)
    }

    pub fn wait(&mut self){
        self.markets.values().for_each(|m| wait_one_day!(m));
        self.save_data();
    }

    pub fn wait_for(&mut self, days : u32){
        for _ in 0..days {
            self.wait();
            self.save_data();
        }
    }

    //a fancy lock_buy() wrapper that adds an order to the pending_orders vector
    /*pub fn pend_buy_order(&mut self, market : MarketKind, kind : GoodKind, amount : f32) {
        let price = self.get_supply_price_qt(market, kind, amount);

        let token = match self.markets.get(&market)
            .expect(MARKET_NOT_FOUND_MSG(market)
                .as_str()).borrow_mut()
            .lock_buy(kind, amount, price, TRADER_NAME.to_string()) {
            Ok(s) => {
                s
            }
            Err(e) => {
                println!("{}", format!("Error! The trader tried to pend a buy order from market \"{:?}\" for {} {} but the market returned an error: {:?}\
                    \n⤷ Call ignored.", market, amount, kind, e).red());
                return;
            }
        };

        self.pending_buy_orders.push((market, token));

    }

    //AI generated, should be fine
    pub fn pend_sell_order(&mut self, market : MarketKind, kind : GoodKind, amount : f32) {
        let price = self.get_demand_price_qt(market, kind, amount);

        let token = match self.markets.get(&market)
            .expect(MARKET_NOT_FOUND_MSG(market)
                .as_str()).borrow_mut()
            .lock_sell(kind, amount, price, TRADER_NAME.to_string()) {
            Ok(s) => {
                s
            }
            Err(e) => {
                println!("{}", format!("Error! The trader tried to pend a sell order from market \"{:?}\" for {} {} but the market returned an error: {:?}\
                    \n⤷ Call ignored.", market, amount, kind, e).red());
                return;
            }
        };

        self.pending_sell_orders.push((market, token));

    }

     */

    //Cashout all the pending orders. Todo: decide whether "cashout()" should be renamed to "cashout_pending_orders"
    /*pub fn cashout_buy_orders(&mut self, kind : GoodKind, amount : f32) {

        for (market, token) in self.pending_buy_orders.drain(..) {
            let bought_goods = match self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market)
                .as_str()).borrow_mut()
                .buy(token, self.owned_goods.get_mut(&EUR).expect("Euros disappeard from the trader's internal hashmap. Panic!")) {
                Ok(s) => {
                    s
                }
                Err(e) => {
                    println!("{}", format!("Error! The trader tried to cashout a pending order from market \"{:?}\" for {} {} but the market returned an error: {:?}\
                        \n⤷ Call ignored.", market, amount, kind, e).red());
                    return;
                }
            };

            self.owned_goods.get_mut(&kind).expect(format!("{} disappeard from the trader's internal hashmap. Panic!", kind).as_str())
                .deref_mut().merge(bought_goods).expect("Couldn't add the bought goods to the trader's internal hashmap. Panic!");
        }

    }

    //AI generated, should be fine
    pub fn cashout_sell_orders(&mut self, kind : GoodKind, amount : f32) {

            for (market, token) in self.pending_sell_orders.drain(..) {
                let sold_goods = match self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market)
                    .as_str()).borrow_mut()
                    .sell(token, self.owned_goods.get_mut(&kind).expect(format!("{} disappeard from the trader's internal hashmap. Panic!", kind).as_str())) {
                    Ok(s) => {
                        s
                    }
                    Err(e) => {
                        println!("{}", format!("Error! The trader tried to cashout a pending order from market \"{:?}\" for {} {} but the market returned an error: {:?}\
                            \n⤷ Call ignored.", market, amount, kind, e).red());
                        return;
                    }
                };

                self.owned_goods.get_mut(&EUR).expect("Euros disappeard from the trader's internal hashmap. Panic!")
                    .deref_mut().merge(sold_goods).expect("Couldn't add the sold goods to the trader's internal hashmap. Panic!");
            }

    }

     */

    //"cashout" all the owned goods, aka sell all the goods to the markets for euros. We'll automatically sell the goods to the highest bidder.
    // UNFINISHED
    pub fn bailout(&mut self) {
        self.get_goods().iter().for_each(|g| {

            if g.get_kind() == GoodKind::EUR {
                return;
            }

            let mut amount = self.get_owned_good_qty(g.get_kind());

            while amount > 0.0 {
                let best_buyer = self.best_buyer_for(g.get_kind(), f32::min(amount, 10000.0));
                self.sell(best_buyer, g.get_kind(), f32::min(amount, 10000.0)).unwrap();
                amount -= f32::min(amount, 10000.0);
            }

        });
    }

    pub fn get_goods(&mut self) -> Vec<Good> {
        self.owned_goods.iter().map(|(_, g)| g.clone()).collect()
    }

    pub fn get_capital(&self) -> f32{
        let mut capital = self.owned_goods.get(&EUR).unwrap().get_qty();
        capital += self.owned_goods.get(&USD).unwrap().get_qty()/USD.get_default_exchange_rate();
        capital += self.owned_goods.get(&YUAN).unwrap().get_qty()/YUAN.get_default_exchange_rate();
        capital += self.owned_goods.get(&YEN).unwrap().get_qty()/YEN.get_default_exchange_rate();

        capital
    }

    pub fn cheapest_supplier(&self, kind : GoodKind) -> MarketKind {

        if self.markets.len() == 0 {
            println!("{}", "The trader does not have any market. Defaulting \"cheapest supplier\" to DOGE".red());
            return DOGE;
        }

        let mut cheapest_supplier = DOGE;
        let lowest_price = f32::MAX;
        self.markets.iter().for_each(|(marketkind, market)| {
            //ignore bfb because of the bug
            if marketkind == &BFB {
                return;
            }
            if market.borrow().get_buy_price(kind, DEFAULT_TRANSACTION_AMOUNT).unwrap() < lowest_price {
                cheapest_supplier = *marketkind;
            }
        });
        cheapest_supplier
    }

    pub fn cheapest_supplier_for(&self, kind : GoodKind, quantity : f32) -> MarketKind {

        if self.markets.len() == 0 {
            println!("{}", "The trader does not have any market. Defaulting \"cheapest supplier\" to DOGE".red());
            return DOGE;
        }

        let mut cheapest_supplier = DOGE;
        let lowest_price = f32::MAX;
        self.markets.iter().for_each(|(marketkind, market)| {
            if market.borrow().get_buy_price(kind, quantity).unwrap() < lowest_price {
                cheapest_supplier = *marketkind;
            }
        });
        cheapest_supplier
    }

    //AI generated, should be OK
    pub fn best_buyer(&self, kind : GoodKind) -> MarketKind {

            if self.markets.len() == 0 {
                println!("{}", "The trader does not have any market. Defaulting \"best buyer\" to DOGE".red());
                return DOGE;
            }

            let mut best_buyer = DOGE;
            let highest_price = f32::MIN;
            self.markets.iter().for_each(|(marketkind, market)| {
                if market.borrow().get_sell_price(kind, DEFAULT_TRANSACTION_AMOUNT).unwrap() > highest_price {
                    best_buyer = *marketkind;
                }
            });
            best_buyer
    }

    //AI generated, should be OK
    pub fn best_buyer_for(&self, kind : GoodKind, quantity : f32) -> MarketKind {

            if self.markets.len() == 0 {
                panic!("{}", "The trader does not have any market. Defaulting \"best buyer\" to DOGE".red());
            }

            let mut best_buyer = PANIC;
            let mut highest_price = f32::MIN;
            self.markets.iter().for_each(|(marketkind, market)| {
                let price = market.borrow().get_sell_price(kind, quantity).unwrap();
                if price > highest_price && market.borrow().get_budget() > price {
                    best_buyer = *marketkind;
                    highest_price = price;
                }
            });
            if best_buyer == PANIC {
                panic!("no market has such amount of money.");
            }
            best_buyer
    }

    pub fn get_good_qty(&self, market : MarketKind, kind : GoodKind) -> f32 {

        let mut quantity = 0.0;

        let market = self.get_market(market);

        match market {
            Ok(m) => {
                m.borrow().get_goods()
                    .iter().for_each(|g| if g.good_kind == kind { quantity = g.quantity });
                quantity
            }
            Err(_) => { 0.0 }
        }
    }

}
