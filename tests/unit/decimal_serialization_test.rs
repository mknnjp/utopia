use proptest::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use utopia::core::compatibility::decimal_amount::DecimalAmount;

proptest! {
    #[test]
    fn decimal_round_trip(value in "-?[0-9]{1,9}(\\.[0-9]{1,6})?") {
        let decimal = Decimal::from_str(&value).unwrap();
        let wrapped = DecimalAmount(decimal);

        let json = serde_json::to_string(&wrapped).unwrap();
        let parsed: DecimalAmount = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(wrapped.0, parsed.0);
    }
}
