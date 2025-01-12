use pyth_sdk::PriceIdentifier;

use crate::price_info::PriceInfoObject;

impl PriceInfoObject {
    /// Get the off chain price identifier from the Pyth sdk
    pub fn pyth_price_id(&self) -> PriceIdentifier {
        PriceIdentifier::new(
            self.price_info
                .price_feed
                .price_identifier
                .bytes
                .to_vec()
                .try_into()
                .expect("Validated lenght onchain"),
        )
    }
}
