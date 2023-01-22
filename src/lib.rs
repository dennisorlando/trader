pub mod trader;

#[cfg(test)]
mod tests {
    
    use bfb::bfb_market::Bfb;
    use bose::market::BoseMarket;
    use dogemarket::dogemarket::DogeMarket;
    use market_common::market::Market;
    use crate::trader::Trader;
    
    use crate::trader::MarketKind::{BFB, BOSE, DOGE};

    #[test]
    fn trader_example() {

        let bose = BoseMarket::new_random();
        let doge = DogeMarket::new_random();
        let bfb = Bfb::new_random();

        let trader = Trader::new(100_000)
            .add_market(BFB, doge)
            .add_market(DOGE, bfb)
            .add_market(BOSE, bose);

        trader.run(5);
        trader.run(5);
        println!("{:#?}", trader);
        trader.run(5);
    }
}
