mod trader_fancy_prints;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::{DerefMut};

use std::rc::Rc;
use colored::Colorize;

use market_common::good::good::Good;

use market_common::good::good_kind::GoodKind;
use market_common::good::good_kind::GoodKind::*;
use market_common::market::{Market, MarketGetterError};


static TRADER_NAME : &str = "TASE Trader";
static MARKET_NOT_FOUND_MSG: fn(MarketKind) -> String = |market : MarketKind| {
    format!("Market \"{:?}\" not found!", market).red().to_string()
};
static MARKET_GET_PRICE_QUANTITY_DEFAULT : f32 = 1000.0;


//this enum is utterly specific for our implementation and can't be generalized. Bad!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketKind {
    BOSE,
    DOGE,
    BFB,
}

pub struct Trader<'a> {
    //todo: rename this field from "closure" to something else
    closure: Box<dyn Fn(&mut Trader) + 'a>,

    //I hate this because Good already contains the GoodKind. If I used an u32 I would get issues in the buy function
    owned_goods: HashMap<GoodKind, Good>,

    markets: HashMap<MarketKind, Rc<RefCell<dyn Market>>>,

    amazingness: f32,

}

impl<'a> Debug for Trader<'a> {
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

impl<'a> Trader<'a> {
    pub fn new() -> Self {
        let mut owned_goods = HashMap::new();
        owned_goods.insert(EUR, Good::new(EUR, 1000.0));
        owned_goods.insert(USD, Good::new(USD, 0.0));
        owned_goods.insert(YEN,  Good::new(YEN, 0.0));
        owned_goods.insert(YUAN, Good::new(YUAN, 0.0));

        Trader {
            closure: Box::new(|_| {}),
            owned_goods,
            markets: HashMap::new(),
            amazingness: 1.0,
        }
    }

    pub fn new_super_duper_amazing_trader(amazingness: f32) -> Self {
        let mut owned_goods = HashMap::new();
        owned_goods.insert(EUR, Good::new(EUR, 1000.0));
        owned_goods.insert(USD, Good::new(USD, 0.0));
        owned_goods.insert(YEN, Good::new(YEN, 0.0));
        owned_goods.insert(YUAN, Good::new(YUAN, 0.0));

        Trader {
            closure: Box::new(|_| {}),
            owned_goods,
            markets: HashMap::new(),
            amazingness,
        }
    }

    pub fn with_market(mut self, kind: MarketKind, market: Rc<RefCell<dyn Market>>) -> Self {
        self.markets.insert(kind, market);
        self
    }

    //I'm pretty sure this is absolutely terrible, but I really liked the idea of having a modifiable closure inside the trader.
    //I'll trust LegionMammal978 on this one: https://users.rust-lang.org/t/pass-a-closure-that-takes-a-mutable-reference-to-self/73843/3
    pub fn run(&mut self, iterations: u32){
        for _ in 0..iterations {

            //steal the struct's operation and replace it with a temporary value:
            let closure = std::mem::replace(&mut self.closure, Box::new(|_| { println!("{}", "Woooosh! the closure you wrote paniced and destroyed everything.".purple()) }));
            closure(self);

            //put it back before he realizes:
            self.closure = closure;

        }
    }

    pub fn set_closure(&mut self, function: impl Fn(&mut Trader) + 'a) {
        self.closure = Box::new(function);
    }

    pub fn get_price(&self, market : MarketKind, kind : GoodKind) -> Result<f32, MarketGetterError> {
        self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market).as_str()).borrow().get_buy_price(kind, MARKET_GET_PRICE_QUANTITY_DEFAULT)
    }

    pub fn get_price_qt(&self, market : MarketKind, kind : GoodKind, quantity : f32) -> Result<f32, MarketGetterError> {
        self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market).as_str()).borrow().get_buy_price(kind, quantity)
    }

    pub fn buy(&mut self, market : MarketKind, kind : GoodKind, amount : f32) {

        let price = self.get_price_qt(market, kind, amount).expect(MARKET_NOT_FOUND_MSG(market).as_str());

        let token = match self.markets.get(&market)
            .expect(MARKET_NOT_FOUND_MSG(market)
            .as_str()).borrow_mut()
            .lock_buy(kind, amount, price, TRADER_NAME.to_string()) {
            Ok(s) => {
                s
            }
            Err(e) => {
                println!("{}", format!("Error! The trader issued a lock order from market \"{:?}\" for {} {} but the market returned an error: {:?}\
                    \n⤷ Call ignored.", market, amount, kind, e).red());
                return;
            }
        };

        let bought_goods = match self.markets.get(&market).expect(MARKET_NOT_FOUND_MSG(market)
            .as_str()).borrow_mut()
            .buy(token, self.owned_goods.get_mut(&EUR).expect("Euros disappeard from the trader's internal hashmap. Panic!")) {
            Ok(s) => {
                s
            }
            Err(e) => {
                println!("{}", format!("Error! The trader issued a buy order from market \"{:?}\" for {} {} but the market returned an error: {:?}\
                    \n⤷ Call ignored.", market, amount, kind, e).red());
                return;
            }
        };

        self.owned_goods.get_mut(&kind).expect(format!("{} disappeard from the trader's internal hashmap. Panic!", kind).as_str())
            .deref_mut().merge(bought_goods).expect("Couldn't add the bought goods to the trader's internal hashmap. Panic!");

    }

}
