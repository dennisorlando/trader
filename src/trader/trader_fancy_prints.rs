use market_common::good::good_kind::GoodKind::EUR;
use crate::trader::Trader;

impl<'a> Trader<'a> {

    pub fn print_liquidity(&self) {
        println!("➤ Trader budget: {}€", self.owned_goods.get(&EUR).unwrap());
    }
    pub fn print_goods(&self) {
        println!(" ↳ Owned goods: {}", self.owned_goods.iter().map(|(_, t)| format!("{} {}, ", t.get_qty(), t.get_kind())).collect::<String>());
    }


}