pub mod trader;

#[cfg(test)]
mod tests {
    
   

    use std::rc::Rc;

    use bfb::bfb_market::Bfb;
    use bose::market::BoseMarket;
    
    
    use market_common::good::good_kind::GoodKind::{YUAN};


    use market_common::market::Market;
    use market_common::subscribe_each_other;

    use crate::trader::MarketKind::{BFB, BOSE};
    use crate::trader::{MarketKind, Trader};

    #[test]
    fn trader_example() {

        let closure = |trader : &mut Trader| {
            //trader.print_liquidity();
            //trader.print_goods();

            for i in 0..1000 {
                let price = trader.get_demand_price(BFB, YUAN);
                println!("Price {}: {}", i, price);

                trader.sell(BFB, YUAN, 0.01).expect("Example trader does not successed");

                let price = trader.get_demand_price(BFB, YUAN);

                println!("Price {}: {}", i, price);

            }

        };

        let bose = BoseMarket::new_random();
        //screw DOGE let doge = DogeMarket::new_random();
        let bfb = Bfb::new_random();
        let tase = tase::TASE::new_random();

        let mut trader = Trader::new_super_duper_amazing_trader(10.0)
            .with_market(BOSE, Rc::clone(&bose))
            //.with_market(DOGE, Rc::clone(&doge))
            .with_market(BFB, Rc::clone(&bfb))
            .with_market(MarketKind::TASE, Rc::clone(&tase))
            .with_initial_money(10001.0)
            .with_good(YUAN, 10000000.0);

        //todo: move this macro inside the trader definition.
        subscribe_each_other!(bose, bfb, tase);

        trader.set_strategy(closure);

        //trader.run(1);
        println!("{:#?}", trader);
    }
}
