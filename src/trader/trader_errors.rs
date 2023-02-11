use market_common::market::{BuyError, LockBuyError, LockSellError, SellError};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TraderSupplyError {
    MarketNotFound,
    GoodsNotFound,
    MarketInsufficientSupply,
    TraderInsufficientFunds,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TraderDemandError {
    MarketNotFound,
    MarketInsufficientFunds,
    TraderInsufficientGoods,
}

impl From<LockBuyError> for TraderSupplyError {
    fn from(e: LockBuyError) -> Self {
        match e {
            LockBuyError::InsufficientGoodQuantityAvailable {..} => TraderSupplyError::MarketInsufficientSupply,
            _ => panic!("Unhandled error: {:?}. Byebye!", e)
        }
    }
}

impl From<BuyError> for TraderSupplyError {
    fn from(value: BuyError) -> Self {
        match value {
            BuyError::InsufficientGoodQuantity { .. } => TraderSupplyError::TraderInsufficientFunds,
            _ => panic!("Unhandled error: {:?}. Byebye!", value)
        }
    }
}

impl From<LockSellError> for TraderDemandError {
    fn from(e: LockSellError) -> Self {
        match e {
            LockSellError::InsufficientDefaultGoodQuantityAvailable { .. } => TraderDemandError::MarketInsufficientFunds,
            _=> panic!("Unhandled error: {:?}. Byebye!", e)
        }
    }
}

impl From<SellError> for TraderDemandError {
    fn from(value: SellError) -> Self {
        match value {
            SellError::InsufficientGoodQuantity { .. } => TraderDemandError::TraderInsufficientGoods,
            _ => panic!("Unhandled error: {:?}. Byebye!", value)
        }
    }
}

impl From<TraderSupplyError> for TraderDemandError {
    fn from(value: TraderSupplyError) -> Self {
        match value {
            TraderSupplyError::MarketNotFound => TraderDemandError::MarketNotFound,
            _ => panic!("Something went terribly wrong: the code tried to convert a supply error into a demand error.")
        }
    }
}