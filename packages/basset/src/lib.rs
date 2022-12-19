mod tax_querier;

pub use tax_querier::deduct_tax;
pub mod contract_error;
pub mod dex_router;
pub mod hub;
pub mod reward;

#[cfg(test)]
mod testing;
