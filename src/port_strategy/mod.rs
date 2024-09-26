//! Provides a means to hold configuration options specifically for port scanning.
mod range_iterator;
use crate::input::{PortRange, ScanOrder};
use rand::seq::SliceRandom;
use rand::thread_rng;
use range_iterator::RangeIterator;

/// Represents options of port scanning.
///
/// Right now all these options involve ranges, but in the future
/// it will also contain custom lists of ports.
#[derive(Debug)]
pub enum PortStrategy {
    Manual(Vec<u16>),
    Serial(SerialRange),
    Random(RandomRange),
}

impl PortStrategy {
    pub fn pick(range: &Option<PortRange>, ports: Option<Vec<u16>>, order: ScanOrder) -> Self {
        match order {
            ScanOrder::Serial if ports.is_none() => {
                let range = range.as_ref().unwrap();
                PortStrategy::Serial(SerialRange {
                    ranges: range.ranges.clone(),
                })
            }
            ScanOrder::Random if ports.is_none() => {
                let range = range.as_ref().unwrap();
                PortStrategy::Random(RandomRange {
                    ranges: range.ranges.clone(),
                })
            }
            ScanOrder::Serial => PortStrategy::Manual(ports.unwrap()),
            ScanOrder::Random => {
                let mut rng = thread_rng();
                let mut ports = ports.unwrap();
                ports.shuffle(&mut rng);
                PortStrategy::Manual(ports)
            }
        }
    }

    pub fn order(&self) -> Vec<u16> {
        match self {
            PortStrategy::Manual(ports) => ports.clone(),
            PortStrategy::Serial(range) => range.generate(),
            PortStrategy::Random(range) => range.generate(),
        }
    }
}

/// Trait associated with a port strategy. Each PortStrategy must be able
/// to generate an order for future port scanning.
trait RangeOrder {
    fn generate(&self) -> Vec<u16>;
}

/// As the name implies SerialRange will always generate a vector in
/// ascending order.
#[derive(Debug)]
pub struct SerialRange {
    ranges: Vec<(u16, u16)>,
}

impl RangeOrder for SerialRange {
    fn generate(&self) -> Vec<u16> {
        self.ranges
            .iter()
            .flat_map(|&(start, end)| (start..=end).collect::<Vec<u16>>())
            .collect()
    }
}

/// As the name implies RandomRange will always generate a vector with
/// a random order. This vector is built following the LCG algorithm.
#[derive(Debug)]
pub struct RandomRange {
    ranges: Vec<(u16, u16)>,
}

impl RangeOrder for RandomRange {
    // Right now using RangeIterator and generating a range + shuffling the
    // vector is pretty much the same. The advantages of it will come once
    // we have to generate different ranges for different IPs without storing
    // actual vectors.
    //
    // Another benefit of RangeIterator is that it always generate a range with
    // a certain distance between the items in the Array. The chances of having
    // port numbers close to each other are pretty slim due to the way the
    // algorithm works.
    fn generate(&self) -> Vec<u16> {
        // 通过 RangeIterator 收集每个范围内的端口
        let mut all_ports: Vec<u16> = self
            .ranges
            .iter()
            .flat_map(|&(start, end)| {
                // 使用 RangeIterator 来生成每个范围内的随机顺序端口
                RangeIterator::new(start.into(), end.into()).collect::<Vec<u16>>()
            })
            .collect();

        // 将所有端口打乱顺序
        all_ports.shuffle(&mut thread_rng());

        all_ports
    }
}

#[cfg(test)]
mod tests {
    use super::PortStrategy;
    use crate::input::{PortRange, ScanOrder};

    #[test]
    fn serial_strategy_with_range() {
        let range = PortRange {
            ranges: vec![(1, 100)],
        };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Serial);
        let result = strategy.order();
        let expected_range = (1..=100).into_iter().collect::<Vec<u16>>();
        assert_eq!(expected_range, result);
    }
    #[test]
    fn random_strategy_with_range() {
        let range = PortRange {
            ranges: vec![(1, 100)],
        };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let mut result = strategy.order();
        let expected_range = (1..=100).into_iter().collect::<Vec<u16>>();
        assert_ne!(expected_range, result);

        result.sort_unstable();
        assert_eq!(expected_range, result);
    }

    #[test]
    fn serial_strategy_with_ports() {
        let strategy = PortStrategy::pick(&None, Some(vec![80, 443]), ScanOrder::Serial);
        let result = strategy.order();
        assert_eq!(vec![80, 443], result);
    }

    #[test]
    fn random_strategy_with_ports() {
        let strategy = PortStrategy::pick(&None, Some((1..10).collect()), ScanOrder::Random);
        let mut result = strategy.order();
        let expected_range = (1..10).into_iter().collect::<Vec<u16>>();
        assert_ne!(expected_range, result);

        result.sort_unstable();
        assert_eq!(expected_range, result);
    }
}
