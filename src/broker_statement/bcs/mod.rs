mod assets;
mod cash_flow;
mod common;
mod period;
mod trades;

use crate::brokers::{Broker, BrokerInfo};
use crate::config::Config;
use crate::core::GenericResult;
#[cfg(test)] use crate::taxes::TaxRemapping;

#[cfg(test)] use super::{BrokerStatement};
use super::{BrokerStatementReader, PartialBrokerStatement};
use super::xls::{XlsStatementParser, Section};

use assets::AssetsParser;
use cash_flow::CashFlowParser;
use period::PeriodParser;
use trades::TradesParser;

pub struct StatementReader {
    broker_info: BrokerInfo,
}

impl StatementReader {
    pub fn new(config: &Config) -> GenericResult<Box<dyn BrokerStatementReader>> {
        Ok(Box::new(StatementReader {
            broker_info: Broker::Bcs.get_info(config)?,
        }))
    }
}

impl BrokerStatementReader for StatementReader {
    fn is_statement(&self, path: &str) -> GenericResult<bool> {
        Ok(path.ends_with(".xls"))
    }

    fn read(&mut self, path: &str) -> GenericResult<PartialBrokerStatement> {
        XlsStatementParser::read(self.broker_info.clone(), path, "TDSheet", vec![
            Section::new("Период:").parser(Box::new(PeriodParser{})).required(),

            Section::new("1. Движение денежных средств").required(),
            Section::new("1.1. Движение денежных средств по совершенным сделкам:").required(),
            Section::new(concat!(
                "1.1.1. Движение денежных средств по совершенным сделкам (иным операциям) с ",
                "ценными бумагами, по срочным сделкам, а также сделкам с иностранной валютой:",
            )).required(),
            Section::new("Остаток денежных средств на начало периода (Рубль):").required(),
            Section::new("Остаток денежных средств на конец периода (Рубль):").required(),
            Section::new("Рубль").parser(Box::new(CashFlowParser{})).required(),

            Section::new("2.1. Сделки:"),
            Section::new("Пай").parser(Box::new(TradesParser{})),
            Section::new("2.3. Незавершенные сделки"),

            Section::new("3. Активы:").required(),
            Section::new("Вид актива").parser(Box::new(AssetsParser{})).required(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_real() {
        let statement = BrokerStatement::read(
            &Config::mock(), Broker::Bcs, "testdata/bcs", TaxRemapping::new(), true).unwrap();

        assert!(!statement.cash_flows.is_empty());
        assert!(!statement.cash_assets.is_empty());

        assert!(statement.fees.is_empty());
        assert!(statement.idle_cash_interest.is_empty());

        assert!(statement.forex_trades.is_empty());
        assert!(!statement.stock_buys.is_empty());
        assert!(statement.stock_sells.is_empty());
        assert!(statement.dividends.is_empty());

        assert!(!statement.open_positions.is_empty());
        assert!(statement.instrument_names.is_empty());
    }
}