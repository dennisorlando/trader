use std::{collections::HashMap, cmp::Ordering, cell::RefCell, rc::Rc};
use bfb::bfb_market::Bfb;
use bose::market::BoseMarket;
use dogemarket::dogemarket::DogeMarket;
use market_common::{good::good_kind::GoodKind::{self, EUR}, market::{good_label::GoodLabel, Market}};
use trader::trader::{Trader, MarketKind};


fn main() {
    let bose = BoseMarket::new_random();
    let doge = DogeMarket::new_random();//DogeMarket::new_with_quantities(248138.67, 35992804., 259583.25, 1844539.8);
    let bfb = Bfb::new_random(); // Bfb::new_with_quantities(1_000_000., 1_000_000., 1_000_000., 1_000_000.);

    let mut trader = Trader::new()
        .with_initial_money(1_000_000.)
        .with_market(MarketKind::DOGE, doge)
        .with_market(MarketKind::BFB, bfb)
        .with_market(MarketKind::BOSE, bose)
        ;
    trader.set_strategy(strategy);

    trader.run(1);

    println!("{:#?}", trader);

    

    // Call the visualizer
    let data = Rc::new(trader.data.clone());
    let liq = Rc::new(trader.liquidity.clone());
    gtk_plotter::gtk_plotter(data, liq);
}


// const BFB_LOCK_DURATION: u32 = 9;

fn strategy(trader: &mut Trader) {
    let bose = trader.get_market(MarketKind::BOSE).unwrap();
    let bfb = trader.get_market(MarketKind::BFB).unwrap();

    println!("BOSE begin with");
    print_goods_value(trader, &bose);

    let kind = GoodKind::USD; // because has most value in BFB
    // let init_qty = trader.get_good_qty(MarketKind::BFB, kind);

    // let mut last_bfb_lock: u32 = 0;
    // let mut curr_day = 0;
    let mut quantity: f32;
    for _ in 0..10 {
        quantity = trader.get_good_qty(MarketKind::BFB, kind) - 0.1;
        // print_goods_value(trader, &bfb);
        // Lock to increase price
        println!("Locking");
        trader.lock_without_buying(MarketKind::BFB, kind, quantity).unwrap();
        // print_goods_value(trader, &bfb);
        
        quantity = find_accepted_sell_qty(trader, &bfb, kind, quantity);
        println!("Sell quantity: {}", quantity);
        quantity = find_accepted_buy_qty(trader, &bose, kind, quantity);
        if quantity == 0. {
            break;
        }
        // Buy from BOSE and Sell to BFB at increased price
        trader.buy(MarketKind::BOSE, kind, quantity).unwrap();
        trader.sell(MarketKind::BFB, kind, quantity).unwrap();
        // print_goods_value(trader, &bfb);
        // 3 days passed
        // Waiting while lock expire
        trader.wait_for(8);
        // print_goods_value(trader, &bfb);
    }
    print_goods_value(trader, &bfb);

    println!("{:#?}", trader);
    trader.wait_for(10);
    println!("Selling strategy");
    let max_buy_price = bose.borrow().get_buy_price(kind, 1.).unwrap();
    for _ in 0..10 {
        // let kind = GoodKind::YEN; 
        // print_goods_value(trader, &bfb);
        let mut quantity = trader.get_good_qty(MarketKind::BFB, kind);
        let price = bfb.borrow().get_buy_price(kind, quantity).unwrap();
        // BUY
        println!("Want to buy from BFB {} for {} price {}", price, quantity, price/quantity);
        if price/quantity > max_buy_price {
            break;
        }
        trader.buy(MarketKind::BFB, kind, quantity).unwrap();
        // print_goods_value(trader, &bfb);

        // SELL back
        quantity = find_accepted_sell_qty(trader, &bfb, kind, quantity);
        println!("Sell quantity: {}", quantity);
        if quantity == 0. {
            break;
        }
        trader.sell(MarketKind::BFB, kind, quantity).unwrap();
        // print_goods_value(trader, &bfb);
    }
    print_goods_value(trader, &bfb);

    // LAST sell
    {
        let mut quantity = trader.get_owned_good_qty(kind);
        quantity = find_accepted_sell_qty(trader, &bfb, kind, quantity);
        println!("Last Sell quantity: {}", quantity);
        if quantity > 0. {
            trader.sell(MarketKind::BFB, kind, quantity).unwrap();    
        }
    }
    print_goods_value(trader, &bfb);

    println!("BOSE ends with");
    print_goods_value(trader, &bose);

    // Sell last Good to BOSE
    {
        let mut quantity = trader.get_owned_good_qty(kind);
        quantity = find_accepted_sell_qty(trader, &bose, kind, quantity);
        if quantity > 0. {
            println!("BOSE Sell quantity: {} at {}", quantity, bose.borrow().get_sell_price(kind, quantity).unwrap());
            trader.sell(MarketKind::BOSE, kind, quantity).unwrap();    
        }
    }

    println!("{:#?}\n", trader);
    // Sell last Good to DOGE
    {
        let doge = trader.get_market(MarketKind::DOGE).unwrap();
        let mut quantity = trader.get_owned_good_qty(kind);
        quantity = find_accepted_sell_qty(trader, &doge, kind, quantity);
        if quantity > 0. {
            println!("DOGE Sell quantity: {} at {}", quantity, doge.borrow().get_sell_price(kind, quantity).unwrap());
            trader.sell(MarketKind::DOGE, kind, quantity).unwrap();    
        }
    }

    println!("Want to sell to BOSE {}", bose.borrow().get_sell_price(GoodKind::YEN, 1000.).unwrap());
    println!("Want to buy from BOSE {}", bose.borrow().get_buy_price(GoodKind::YEN, 1000.).unwrap());


    println!("I have {} {}", trader.get_owned_good_qty(kind), kind);
    trader.wait_for(3);
}

fn find_accepted_sell_qty(_trader: &mut Trader, market: &Rc<RefCell<dyn Market>>, kind: GoodKind, initial_quantity: f32) -> f32 {
    if initial_quantity < 0.01 || market.borrow().get_budget() < 100. {
        return 0.;
    }
    for i in 1.. {
        let quantity = initial_quantity / i as f32;
        if quantity < 0.01 {
            return 0.;
        }
        match market.borrow().get_sell_price(kind, quantity) {
            Ok(p) => { if p < market.borrow().get_budget() { return quantity; } },
            Err(_) => {}
        }
    }
    panic!("Infinite loop finished")
}

fn find_accepted_buy_qty(_trader: &mut Trader, market: &Rc<RefCell<dyn Market>>, kind: GoodKind, initial_quantity: f32) -> f32 {
    if initial_quantity < 0.001 {
        return 0.;
    }
    for i in 1.. {
        let quantity = initial_quantity / i as f32;
        if quantity < 0.01 {
            return 0.;
        }
        match market.borrow().get_buy_price(kind, quantity) {
            Ok(_) => { return quantity; },
            Err(_) => {}
        }
    }
    panic!("Infinite loop finished")
}

fn print_goods_value(_trader: &mut Trader, market: &Rc<RefCell<dyn Market>>) {
    let mut goods = market.borrow().get_goods();
    goods.sort_by(|a, b| { compare(&a.good_kind, &b.good_kind) });
    for GoodLabel { good_kind, quantity, exchange_rate_buy, exchange_rate_sell } in goods {
        println!("{}: EUR value {} = {} * {}, sell = {}", good_kind, quantity*exchange_rate_buy, quantity, exchange_rate_buy, exchange_rate_sell);
        if quantity > 0. {
            println!("Buy half at {}", market.borrow().get_buy_price(good_kind, quantity/2.).unwrap());
        }
    }
    println!("");
}

fn compare(a: &GoodKind, b: &GoodKind) -> Ordering {
    let mut order: HashMap<GoodKind, u32> = HashMap::new();
    order.insert(EUR, 1);
    order.insert(GoodKind::USD, 2);
    order.insert(GoodKind::YEN, 3);
    order.insert(GoodKind::YUAN, 4);

    let na = order[a];
    let nb = order[b];
    na.cmp(&nb)
}
