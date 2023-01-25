pub mod trader;

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::cell::RefCell;
    use std::rc::Rc;

    use bfb::bfb_market::Bfb;
    use bose::market::BoseMarket;
    use dogemarket::dogemarket::DogeMarket;
    use market_common::market::Market;

    use crate::trader::MarketKind::{BFB, BOSE, DOGE};
    use crate::trader::Trader;

    #[test]
    fn trader_example() {
        let bose = BoseMarket::new_random();
        let doge = DogeMarket::new_random();
        let bfb = Bfb::new_random();

        let trader = Trader::new()
            .with_market(BOSE, Rc::clone(&bose))
            .with_market(DOGE, Rc::clone(&doge))
            .with_market(BFB, Rc::clone(&bfb));

        trader.run(5);
        trader.run(5);
        println!("{:#?}", trader);
        trader.run(5);
    }
}
