use af_ptbuilder::{ProgrammableTransactionBuilder, ptbuilder};
use af_sui_types::{Argument, ObjectArg, ObjectId, TypeTag};

/// Groups the [ProgrammableTransactionBuilder] variables for updating a Switchboard `Aggregator`.
#[derive(Clone, Debug)]
pub struct OraclePtbArguments {
    pub oracle: ObjectArg,
    pub value: u128,
    pub neg: bool,
    pub timestamp_seconds: u64,
    pub signature: Vec<u8>,
}

#[extension_traits::extension(pub trait ProgrammableTransactionBuilderExt)]
impl ProgrammableTransactionBuilder {
    /// Add all the necessary oracle results to update a switchboard `Aggregator`.
    /// It assumes the Aggregator's `min_sample_size` check is respected.
    /// This means that `oracle_args.len() >= aggregator.min_sample_size`
    fn update_switchboard_aggregator(
        &mut self,
        switchboard_pkg: ObjectId,
        switchboard_agg: ObjectArg,
        queue: ObjectArg,
        fee_coin: Argument,
        fee_coin_otw: TypeTag,
        oracle_args: Vec<OraclePtbArguments>,
    ) -> Result<(), af_ptbuilder::Error> {
        ptbuilder!(self {
            package switchboard_pkg;

            type T = fee_coin_otw;

            input obj switchboard_agg;
            input obj queue;
            input obj clock: ObjectArg::CLOCK_IMM;
        });

        ptbuilder!(self {
            let queue_fee = switchboard_pkg::queue::fee(queue);
        });

        let fee_coins = self.split_coins_into_vec(fee_coin, vec![queue_fee; oracle_args.len()]);

        for (args, fee) in oracle_args.into_iter().zip(fee_coins) {
            ptbuilder!(self {
                input obj oracle: args.oracle;
                input pure value: &args.value;
                input pure neg: &args.neg;
                input pure timestamp_seconds: &args.timestamp_seconds;
                input pure signature: &args.signature;

                switchboard_pkg::aggregator_submit_result_action::run<T>(
                    switchboard_agg,
                    queue,
                    value,
                    neg,
                    timestamp_seconds,
                    oracle,
                    signature,
                    clock,
                    fee,
                );
            });
        }

        Ok(())
    }
}
