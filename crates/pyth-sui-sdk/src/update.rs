use af_ptbuilder::{ptbuilder, ProgrammableTransactionBuilder};
use af_sui_types::{Argument, ObjectArg, ObjectId};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::price_info::PriceInfo;

const ACCUMULATOR_MAGIC: [u8; 4] = [0x50, 0x4e, 0x41, 0x55];

/// Data for updating price feeds on the Sui network.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum UpdatePayload {
    Accumulator { vaa: Bytes, message: Bytes },
    Normal(Vec<Bytes>),
}

impl UpdatePayload {
    /// Construct an update from the decoded offchain binary price update.
    pub fn new(binary_update: Vec<Vec<u8>>) -> Result<Self, MixedVaasError> {
        let mut bytes_vec: Vec<_> = binary_update.into_iter().map(Bytes::from).collect();

        let accumulator_msg = bytes_vec
            .iter()
            .position(is_accumulator_msg)
            .map(|index| bytes_vec.swap_remove(index));

        if accumulator_msg.is_some() && !bytes_vec.is_empty() {
            return Err(MixedVaasError);
        }

        let update = accumulator_msg.map_or(Self::Normal(bytes_vec), |bytes| Self::Accumulator {
            vaa: accumulator_payload(&bytes),
            message: bytes,
        });
        Ok(update)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Multiple accumulator messages or mixed accumulator and non-accumulator messages")]
pub struct MixedVaasError;

/// Payload for verification in the Wormhole package (vaa::parse_and_verify).
fn accumulator_payload(acc_message: &Bytes) -> Bytes {
    // the first 6 bytes in the accumulator message encode the header, major, and minor bytes
    // we ignore them, since we are only interested in the VAA bytes
    let trailing_payload_size = acc_message[6] as usize;
    let vaa_size_offset = 7 + // header bytes (header(4) + major(1) + minor(1) + trailing payload size(1))
      trailing_payload_size + // trailing payload (variable number of bytes)
      1; // proof_type (1 byte)
    let vaa_size = u16::from_be_bytes([
        acc_message[vaa_size_offset],
        acc_message[vaa_size_offset + 1],
    ]) as usize;
    let vaa_offset = vaa_size_offset + 2;
    acc_message.slice(vaa_offset..(vaa_offset + vaa_size))
}

fn is_accumulator_msg(bytes: &Bytes) -> bool {
    bytes[..4] == ACCUMULATOR_MAGIC
}

/// Groups the [ProgrammableTransactionBuilder] variables for updating Pyth `PriceInfoObject`s.
#[derive(Clone, Debug)]
pub struct PtbArguments {
    /// `state::State` object from Pyth.
    pub pyth_state: Argument,
    /// Wormhole state object.
    pub wormhole_state: Argument,
    /// `price_info::PriceInfoObject`s from Pyth to update.
    pub price_info_objects: Vec<Argument>,
    /// SUI coin to use for Pyth's fee. Can be [`Argument::Gas`].
    pub fee_coin: Argument,
}

#[extension_traits::extension(pub trait ProgrammableTransactionBuilderExt)]
impl ProgrammableTransactionBuilder {
    /// Construct the PTB arguments to be used in [`update_pyth_price_info`].
    ///
    /// This is separate from [`update_pyth_price_info`] since the caller may want to use some of the
    /// arguments created here in subsequent PTB calls.
    ///
    /// [`update_pyth_price_info`]: ProgrammableTransactionBuilderExt::update_pyth_price_info
    fn update_pyth_price_info_args(
        &mut self,
        pyth_state: ObjectArg,
        wormhole_state: ObjectArg,
        price_info_objects: Vec<ObjectArg>,
        fee_coin: Argument,
    ) -> Result<PtbArguments, af_ptbuilder::Error> {
        ptbuilder!(self {
            input obj pyth_state;
            input obj wormhole_state;
        });
        let mut vars = PtbArguments {
            pyth_state,
            wormhole_state,
            price_info_objects: vec![],
            fee_coin,
        };
        for pio in price_info_objects {
            ptbuilder!(self {
                input obj pio;
            });
            vars.price_info_objects.push(pio);
        }
        Ok(vars)
    }

    /// Add a Pyth price update(s) to PTB being built.
    ///
    /// Arguments:
    /// * `pyth_pkg`: Address of the Pyth package.
    /// * `wormhole_pkg`: Address of the Wormhole package.
    fn update_pyth_price_info(
        &mut self,
        pyth_pkg: ObjectId,
        wormhole_pkg: ObjectId,
        arguments: PtbArguments,
        update: UpdatePayload,
    ) -> Result<(), af_ptbuilder::Error> {
        let PtbArguments {
            pyth_state,
            wormhole_state,
            price_info_objects,
            fee_coin,
        } = arguments;

        // Declare packages for interaction once.
        ptbuilder!(self {
            package pyth: pyth_pkg;
            package wormhole: wormhole_pkg;

            input obj clock: ObjectArg::CLOCK_IMM;
        });

        let mut price_updates = match update {
            UpdatePayload::Accumulator { vaa, message } => {
                ptbuilder!(self {
                    input pure vaa: vaa.as_ref();
                    input pure accumulator_msg: message.as_ref();

                    let verified_vaa = wormhole::vaa::parse_and_verify(wormhole_state, vaa, clock);
                    let updates = pyth::pyth::create_authenticated_price_infos_using_accumulator(
                        pyth_state,
                        accumulator_msg,
                        verified_vaa,
                        clock
                    );
                });
                updates
            }
            UpdatePayload::Normal(bytes) => {
                let mut verified_vaas = Vec::new();
                for vaa in bytes {
                    ptbuilder!(self {
                        input pure vaa: vaa.as_ref();
                        let verified = wormhole::vaa::parse_and_verify(wormhole_state, vaa, clock);
                    });
                    verified_vaas.push(verified);
                }
                ptbuilder!(self {
                    let verified_vaas = command! MakeMoveVec(None, verified_vaas);
                    let updates = pyth::pyth::create_price_infos_hot_potato(
                        pyth_state,
                        verified_vaas,
                        clock
                    );
                });
                updates
            }
        };

        ptbuilder!(self {
            let base_update_fee = pyth::state::get_base_update_fee(pyth_state);
        });
        let fee_coins =
            self.split_coins_into_vec(fee_coin, vec![base_update_fee; price_info_objects.len()]);
        for (price_info_object, fee) in price_info_objects.into_iter().zip(fee_coins) {
            ptbuilder!(self {
                let price_updates_ = pyth::pyth::update_single_price_feed(
                    pyth_state,
                    price_updates,
                    price_info_object,
                    fee,
                    clock,
                );
            });
            price_updates = price_updates_;
        }
        ptbuilder!(self {
            type T = PriceInfo::type_(pyth_pkg.into()).into();
            pyth::hot_potato_vector::destroy<T>(price_updates);
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCUMULATOR_MAGIC_HEX: &str = "504e4155";

    #[test]
    fn magics_match() {
        assert_eq!(hex::encode(ACCUMULATOR_MAGIC), ACCUMULATOR_MAGIC_HEX);
    }
}
