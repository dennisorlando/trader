use std::cell::{RefCell};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use market_common::good::good_kind::GoodKind;
use market_common::good::good_kind::GoodKind::*;
use market_common::market::Market;

#[derive(Eq, Hash, PartialEq)]
pub enum MarketKind {
    BOSE,
    DOGE,
    BFB,
}

pub struct Trader
{

    //todo: rename this field from "closure" to something else
    closure: Box<dyn Fn()>,

    //market related fields
    owned_goods: HashMap<GoodKind, u32>,

    markets: HashMap<MarketKind, Rc<RefCell<dyn Market>>>,

}

//todo
impl Debug for Trader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate(){
            return write!(f, "➤ Trader Status:\n • Markets: {}\n • Money: {}",
                   self.markets.iter().map(|(_,t)| format!("\"{}\", ", (**t).borrow().get_name())  ).collect::<String>(),
                   self.owned_goods.get(&EUR).unwrap()
            );
        }
        write!(f, "➤ Trader status: (Money: {})", self.owned_goods.get(&EUR).unwrap())

    }
}

impl Trader {

    pub fn new(initial_money: u32) -> Trader {

        let mut owned_goods = HashMap::new();
        owned_goods.insert(EUR, initial_money);
        owned_goods.insert(USD, 0);
        owned_goods.insert(YEN, 0);
        owned_goods.insert(YUAN, 0);

        Trader {
            closure: Box::new(|| {}),
            owned_goods,
            markets: HashMap::new()
        }
    }
    pub fn add_market(mut self, kind : MarketKind, market : Rc<RefCell<dyn Market>>) -> Trader {
        self.markets.insert(kind, market);
        self
    }

    pub fn run(&self, iterations : u32){
        for _ in 0..iterations {
            (self.closure)();
        }
    }

    //This supposes that the users create a static function instead of an actual closure
    pub fn set_closure(&mut self, function : impl Fn() + 'static) {
        self.closure = Box::new(function);
    }


}