use num_traits::ToPrimitive;
use separator::Separatable;

use broker_statement::ib::IbStatementParser;
use config::Config;
use core::EmptyResult;
use currency::converter::CurrencyConverter;
use db;
use util;

use self::performance::PortfolioPerformanceAnalyser;

mod deposit_emulator;
mod performance;

pub fn analyse(config: &Config, broker_statement_path: &str) -> EmptyResult {
    let database = db::connect(&config.db_path)?;
    let statement = IbStatementParser::parse(&config, broker_statement_path, false)?;
    let converter = CurrencyConverter::new(database, false);

    println!("Portfolio performance:");

    for currency in ["USD", "RUB"].iter() {
        let (deposits, current_assets, interest) = PortfolioPerformanceAnalyser::analyse(
            &statement, *currency, &converter)?;

        let deposits = util::round_to(deposits, 0).to_i64().unwrap();
        let current_assets = util::round_to(current_assets, 0).to_i64().unwrap();
        let profit = current_assets - deposits;
        let profit_sign = if profit < 0 {
            '-'
        } else {
            '+'
        };

        println!(
            "* {currency}: {deposits} {profit_sign} {profit} = {current_assets} ({interest}%)",
            currency=currency, deposits=deposits.separated_string(), profit_sign=profit_sign,
            profit=profit.abs().separated_string(), current_assets=current_assets.separated_string(),
            interest=interest);
    }

    Ok(())
}