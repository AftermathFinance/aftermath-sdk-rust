use af_ptbuilder::ProgrammableTransactionBuilder;
use clap::Parser;
use serde_with::base64::Base64;
use serde_with::{TryFromInto, serde_as};
use sui_sdk_types::ProgrammableTransaction;
use sui_sdk_types::bcs::{FromBcs, ToBcs};

const DEFAULT_PTB_BASE64: &str = r#"{
"builder": "IwEAbvkm9unJ4JcOxhlrOw08M/xKb7DO+30cR25/gkfz/SraOfkTAAAAACDbg3ct3L8dmGhGzB1wwugYjNTEsOQAdKM0WMJ++j/QpAEBSb1AzHiAvTWEZRFhV/AnHCXSM2G5Tqzpol3CAZtEm/yNnOcPAAAAAAEBAS4mgWYWJE/pUu+SRFPTRo7Xat3qr1hzyvCXC6mysycimk/bDwAAAAAAAQHRCzhcgsuh9nOnRW68l+CUnrTKpxtKGpv+TVk4TnmieZxP2w8AAAAAAAEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYBAAAAAAAAAAAAEDD9FAAAAAAAdn65xP////8AECn9FAAAAAAAmpm5xP////8AEA/9FAAAAAAANTG7xP////8AEB/9FAAAAAAATHxTOwAAAAAAEBn9FAAAAAAAekJUOwAAAAAAEB39FAAAAAAAzlBVOwAAAAAAEAz9FAAAAAAA465TOwAAAAAAEBv9FAAAAAAAMrlTOwAAAAAAEBj9FAAAAAAAMdtSOwAAAAAAAQEAAQAACAAAAAAAAAAAAAg/QUg7AAAAAAAIgoYIAAAAAAAACPYnRjsAAAAAAAgRxA4AAAAAAAAIANNPOwAAAAAACAqUCQAAAAAAAAgWz0w7AAAAAAAIj7MJAAAAAAAACAlDSTsAAAAAAAhxEA8AAAAAAAAIDxROOwAAAAAACAdeDAAAAAAAAAjS00E7AAAAAAAIPNYMAAAAAAAACPeUPTsAAAAAAAjY0gwAAAAAAAAIgYdAOwAAAAAACCw1CwAAAAAADgUBAwkBBQABBgABBwABCAABCQABCgABCwABDAABDQAAlyUVWnDPLSJBuMwvqDdoCWiTEsq7Ssqlylukfq9NYR8JaW50ZXJmYWNlDWNhbmNlbF9vcmRlcnMBB0VwSTcfW13CvahXu4BMpuk8WjyuFjbQzRe7a2Bw0ZRYBHVzZGMEVVNEQwADAQEAAQAAAgAAAJclFVpwzy0iQbjML6g3aAlokxLKu0rKpcpbpH6vTWEfCWludGVyZmFjZQ1zdGFydF9zZXNzaW9uAQdFcEk3H1tdwr2oV7uATKbpPFo8rhY20M0Xu2tgcNGUWAR1c2RjBFVTREMABQEBAAEAAAECAAEDAAEEAACXJRVacM8tIkG4zC+oN2gJaJMSyrtKyqXKW6R+r01hHwlpbnRlcmZhY2URcGxhY2VfbGltaXRfb3JkZXIBB0VwSTcfW13CvahXu4BMpuk8WjyuFjbQzRe7a2Bw0ZRYBHVzZGMEVVNEQwAFAgIAAQ4AARIAAREAARAAAJclFVpwzy0iQbjML6g3aAlokxLKu0rKpcpbpH6vTWEfCWludGVyZmFjZRFwbGFjZV9saW1pdF9vcmRlcgEHRXBJNx9bXcK9qFe7gEym6TxaPK4WNtDNF7trYHDRlFgEdXNkYwRVU0RDAAUCAgABDgABFAABEwABEAAAlyUVWnDPLSJBuMwvqDdoCWiTEsq7Ssqlylukfq9NYR8JaW50ZXJmYWNlEXBsYWNlX2xpbWl0X29yZGVyAQdFcEk3H1tdwr2oV7uATKbpPFo8rhY20M0Xu2tgcNGUWAR1c2RjBFVTREMABQICAAEOAAEWAAEVAAEQAACXJRVacM8tIkG4zC+oN2gJaJMSyrtKyqXKW6R+r01hHwlpbnRlcmZhY2URcGxhY2VfbGltaXRfb3JkZXIBB0VwSTcfW13CvahXu4BMpuk8WjyuFjbQzRe7a2Bw0ZRYBHVzZGMEVVNEQwAFAgIAAQ4AARgAARcAARAAAJclFVpwzy0iQbjML6g3aAlokxLKu0rKpcpbpH6vTWEfCWludGVyZmFjZRFwbGFjZV9saW1pdF9vcmRlcgEHRXBJNx9bXcK9qFe7gEym6TxaPK4WNtDNF7trYHDRlFgEdXNkYwRVU0RDAAUCAgABDgABGgABGQABEAAAlyUVWnDPLSJBuMwvqDdoCWiTEsq7Ssqlylukfq9NYR8JaW50ZXJmYWNlEXBsYWNlX2xpbWl0X29yZGVyAQdFcEk3H1tdwr2oV7uATKbpPFo8rhY20M0Xu2tgcNGUWAR1c2RjBFVTREMABQICAAEOAAEcAAEbAAEQAACXJRVacM8tIkG4zC+oN2gJaJMSyrtKyqXKW6R+r01hHwlpbnRlcmZhY2URcGxhY2VfbGltaXRfb3JkZXIBB0VwSTcfW13CvahXu4BMpuk8WjyuFjbQzRe7a2Bw0ZRYBHVzZGMEVVNEQwAFAgIAAQ8AAR4AAR0AARAAAJclFVpwzy0iQbjML6g3aAlokxLKu0rKpcpbpH6vTWEfCWludGVyZmFjZRFwbGFjZV9saW1pdF9vcmRlcgEHRXBJNx9bXcK9qFe7gEym6TxaPK4WNtDNF7trYHDRlFgEdXNkYwRVU0RDAAUCAgABDwABIAABHwABEAAAlyUVWnDPLSJBuMwvqDdoCWiTEsq7Ssqlylukfq9NYR8JaW50ZXJmYWNlEXBsYWNlX2xpbWl0X29yZGVyAQdFcEk3H1tdwr2oV7uATKbpPFo8rhY20M0Xu2tgcNGUWAR1c2RjBFVTREMABQICAAEPAAEiAAEhAAEQAACXJRVacM8tIkG4zC+oN2gJaJMSyrtKyqXKW6R+r01hHwlpbnRlcmZhY2ULZW5kX3Nlc3Npb24BB0VwSTcfW13CvahXu4BMpuk8WjyuFjbQzRe7a2Bw0ZRYBHVzZGMEVVNEQwABAgIAAJclFVpwzy0iQbjML6g3aAlokxLKu0rKpcpbpH6vTWEfCWludGVyZmFjZRRzaGFyZV9jbGVhcmluZ19ob3VzZQEHRXBJNx9bXcK9qFe7gEym6TxaPK4WNtDNF7trYHDRlFgEdXNkYwRVU0RDAAEDDAAAAA=="
}"#;

/// Showcases how one can leverage the conversions to/from `ProgrammableTransaction` to
/// de/serialize the builder.
#[derive(Parser)]
struct Cli {
    #[arg(long, value_parser = parse_payload, default_value = DEFAULT_PTB_BASE64)]
    payload: Payload,
}

/// Example of a JSON payload we might wanna transmit.
#[serde_as]
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Payload {
    #[serde_as(as = "TryFromInto<Base64BcsPtb>")]
    pub builder: ProgrammableTransactionBuilder,
}

/// Intermediate layer to leverage [`ProgrammableTransaction`]'s serialization to base64.
#[serde_as]
#[derive(serde::Deserialize, serde::Serialize)]
struct Base64BcsPtb(#[serde_as(as = "Base64")] Vec<u8>);

/// For [`TryFromInto`], which requires an infallible `From` in this direction; BCS
/// serialization of a valid [`ProgrammableTransaction`] cannot fail.
#[allow(clippy::fallible_impl_from)]
impl From<ProgrammableTransactionBuilder> for Base64BcsPtb {
    fn from(value: ProgrammableTransactionBuilder) -> Self {
        let pt = ProgrammableTransaction::from(value);
        Self(
            pt.to_bcs()
                .expect("BCS serialization of a programmable transaction cannot fail"),
        )
    }
}

/// For [`TryFromInto`]
impl TryFrom<Base64BcsPtb> for ProgrammableTransactionBuilder {
    type Error = <Self as TryFrom<ProgrammableTransaction>>::Error;

    fn try_from(value: Base64BcsPtb) -> Result<Self, Self::Error> {
        ProgrammableTransaction::from_bcs(&value.0)?.try_into()
    }
}

/// For [`clap`] only
fn parse_payload(s: &str) -> eyre::Result<Payload> {
    Ok(serde_json::from_str(s)?)
}

fn main() -> eyre::Result<()> {
    let Cli { payload } = Cli::parse();

    println!("{:#?}", payload.builder);

    println!("{}", serde_json::to_string_pretty(&payload)?);

    Ok(())
}
