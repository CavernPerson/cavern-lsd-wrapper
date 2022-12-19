use cosmwasm_std::{Coin, QuerierWrapper, StdResult, Uint128};

pub fn compute_tax(_querier: &QuerierWrapper, _coin: &Coin) -> StdResult<Uint128> {
    /*
    // No taw rate anymore
    let terra_querier = TerraQuerier::new(querier);
    let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
    let tax_cap: Uint128 = (terra_querier.query_tax_cap(coin.denom.to_string())?).cap;
    Ok(std::cmp::min(
        (coin.amount.checked_sub(coin.amount.multiply_ratio(
            DECIMAL_FRACTION,
            DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
        )))?,
        tax_cap,
    ))
    */
    Ok(Uint128::zero())
}

pub fn deduct_tax(querier: &QuerierWrapper, coin: Coin) -> StdResult<Coin> {
    let tax_amount = compute_tax(querier, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: (coin.amount.checked_sub(tax_amount))?,
    })
}
