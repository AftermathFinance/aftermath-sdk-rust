use af_ptbuilder::{ptbuilder, Argument, ProgrammableTransactionBuilder};
use af_sui_pkg_sdk::af_sui_types::ObjectArg;
use af_sui_pkg_sdk::ObjectId;

/// Groups the [ProgrammableTransactionBuilder] arguments for updating AfOracle `PriceFeed`s.
pub struct UpdateAfOracleArguments {
    /// `state::State` object from Pyth.
    pub pyth_state: Argument,
    /// `wrapper::PythWrapper` object.
    pub pyth_wrapper: Argument,
    /// Mapping from AfOracle `PriceFeedStorage` -> Pyth `PriceInfoObject`
    pub pfs_to_source: Vec<(Argument, Argument)>,
}

#[extension_traits::extension(pub trait ProgrammableTransactionBuilderExt)]
impl ProgrammableTransactionBuilder {
    /// Construct the PTB arguments to be used in [`update_af_oracle_pyth_feed`].
    ///
    /// This is separate from [`update_af_oracle_pyth_feed`] since the caller may want to use some of the
    /// arguments created here in subsequent PTB calls.
    ///
    /// [`update_af_oracle_pyth_feed`]: ProgrammableTransactionBuilderExt::update_af_oracle_pyth_feed
    fn update_af_oracle_pyth_feed_args(
        &mut self,
        pyth_state: ObjectArg,
        pyth_wrapper: ObjectArg,
        pfs_to_source: Vec<(ObjectArg, ObjectArg)>,
    ) -> Result<UpdateAfOracleArguments, af_ptbuilder::Error> {
        ptbuilder!(self {
            input obj pyth_state;
            input obj pyth_wrapper;
        });
        let mut vars = UpdateAfOracleArguments {
            pfs_to_source: vec![],
            pyth_state,
            pyth_wrapper,
        };

        for (pfs, pio) in pfs_to_source {
            ptbuilder!(self {
                input obj pfs;
                input obj pio;
            });
            vars.pfs_to_source.push((pfs, pio));
        }

        Ok(vars)
    }

    /// Add a PythWrapper update to the PTB being built.
    fn update_af_oracle_pyth_feed(
        &mut self,
        pyth_wrapper_pkg: ObjectId,
        arguments: UpdateAfOracleArguments,
    ) -> Result<(), af_ptbuilder::Error> {
        let UpdateAfOracleArguments {
            pfs_to_source,
            pyth_wrapper,
            pyth_state,
        } = arguments;
        ptbuilder!(self {
            package pyth_wrapper_pkg;
            input obj clock: ObjectArg::CLOCK_IMM;
        });
        for (price_feed_storage, price_info_object) in pfs_to_source {
            ptbuilder!(self {
                pyth_wrapper_pkg::wrapper::update_price_feed(
                    price_feed_storage,
                    pyth_wrapper,
                    pyth_state,
                    price_info_object,
                    clock,
                );
            });
        }
        Ok(())
    }
}
