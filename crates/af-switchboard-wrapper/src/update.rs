use af_ptbuilder::{Argument, ProgrammableTransactionBuilder, ptbuilder};
use af_sui_pkg_sdk::ObjectId;
use af_sui_pkg_sdk::af_sui_types::ObjectArg;

/// Groups the [ProgrammableTransactionBuilder] arguments for updating AfOracle `PriceFeed`s.
pub struct UpdateAfOracleArguments {
    /// `wrapper::SwitchboardWrapper` object.
    pub switchboard_wrapper: Argument,
    /// Mapping from AfOracle `PriceFeedStorage` -> Switchboard `Aggregator`
    pub pfs_to_source: Vec<(Argument, Argument)>,
}

#[extension_traits::extension(pub trait ProgrammableTransactionBuilderExt)]
impl ProgrammableTransactionBuilder {
    /// Construct the PTB arguments to be used in [`update_af_oracle_switchboard_feed`].
    ///
    /// This is separate from [`update_af_oracle_switchboard_feed`] since the caller may want to use some of the
    /// arguments created here in subsequent PTB calls.
    ///
    /// [`update_af_oracle_switchboard_feed`]: ProgrammableTransactionBuilderExt::update_af_oracle_switchboard_feed
    fn update_af_oracle_switchboard_feed_args(
        &mut self,
        switchboard_wrapper: ObjectArg,
        pfs_to_source: Vec<(ObjectArg, ObjectArg)>,
    ) -> Result<UpdateAfOracleArguments, af_ptbuilder::Error> {
        ptbuilder!(self {
            input obj switchboard_wrapper;
        });
        let mut vars = UpdateAfOracleArguments {
            pfs_to_source: vec![],
            switchboard_wrapper,
        };

        for (pfs, sba) in pfs_to_source {
            ptbuilder!(self {
                input obj pfs;
                input obj sba;
            });
            vars.pfs_to_source.push((pfs, sba));
        }

        Ok(vars)
    }

    /// Add a SwitchboardWrapper update to the PTB being built.
    fn update_af_oracle_switchboard_feed(
        &mut self,
        switchboard_wrapper_pkg: ObjectId,
        arguments: UpdateAfOracleArguments,
    ) -> Result<(), af_ptbuilder::Error> {
        let UpdateAfOracleArguments {
            pfs_to_source,
            switchboard_wrapper,
        } = arguments;
        ptbuilder!(self {
            package switchboard_wrapper_pkg;
            input obj clock: ObjectArg::CLOCK_IMM;
        });
        for (price_feed_storage, switchboard_agg) in pfs_to_source {
            ptbuilder!(self {
                switchboard_wrapper_pkg::wrapper::update_price_feed(
                    clock,
                    price_feed_storage,
                    switchboard_wrapper,
                    switchboard_agg,
                );
            });
        }
        Ok(())
    }
}
