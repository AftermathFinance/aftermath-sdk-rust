use af_move_type::MoveInstance;
use af_sui_types::{Address, ObjectId, Version};
use futures::Stream;
use sui_gql_client::GraphQlClient;
pub use sui_gql_client::queries::Error;

use crate::Vault;
use crate::orderbook::Order;
use crate::position::Position;

mod ch_orders;
mod ch_positions;
mod ch_vault;
mod map_orders;
mod order_maps;
mod registry;

pub use self::ch_vault::Error as ChVaultError;
pub use self::order_maps::OrderMaps;

type StdResult<T, E> = ::std::result::Result<T, E>;
type Result<T, C> = StdResult<T, Error<<C as GraphQlClient>::Error>>;

/// Extension trait to [`GraphQlClient`] collecting all defined queries in one place.
pub trait GraphQlClientExt: GraphQlClient + Sized {
    /// Snapshot of the orders on one side of the orderbook, rooted at the [`ClearingHouse`] id
    /// and version.
    ///
    /// If you already know the object ID of the orders [`Map`], then [`map_orders`] is more
    /// efficient.
    ///
    /// [`ClearingHouse`]: crate::ClearingHouse
    /// [`Map`]: crate::ordered_map::Map
    /// [`map_orders`]: GraphQlClientExt::map_orders
    fn clearing_house_orders(
        &self,
        package: Address,
        ch: ObjectId,
        version: Option<Version>,
        asks: bool,
    ) -> impl Stream<Item = Result<(u128, Order), Self>> + '_ {
        ch_orders::query(self, package, ch, version, asks)
    }

    /// Snapshot of the orders on one side of the orderbook, rooted at the [`Map`] id and
    /// [`ClearingHouse`] version.
    ///
    /// To find the [`Map`] id, you can use [`order_maps`].
    ///
    /// [`Map`]: crate::ordered_map::Map
    /// [`ClearingHouse`]: crate::ClearingHouse
    /// [`order_maps`]: GraphQlClientExt::order_maps
    fn map_orders(
        &self,
        map: ObjectId,
        ch_version: Option<Version>,
    ) -> impl Stream<Item = Result<(u128, Order), Self>> + '_ {
        map_orders::query(self, map, ch_version)
    }

    /// Object IDs of the orderbook and asks/bids maps for a market.
    ///
    /// These never change, so you can query them once and save them.
    fn order_maps(
        &self,
        package: Address,
        ch: ObjectId,
    ) -> impl Future<Output = Result<OrderMaps, Self>> + Send + '_ {
        order_maps::query(self, package, ch)
    }

    /// The unparsed clearing house's collateral [`Vault`].
    ///
    /// [`Vault`]: crate::Vault
    fn clearing_house_vault(
        &self,
        package: Address,
        ch: ObjectId,
    ) -> impl Future<Output = StdResult<MoveInstance<Vault>, ChVaultError<Self::Error>>> + Send + '_
    {
        ch_vault::query(self, package, ch)
    }

    /// Snapshot of positions under the [`ClearingHouse`].
    ///
    /// [`ClearingHouse`]: crate::ClearingHouse
    fn clearing_house_positions(
        &self,
        ch: ObjectId,
        version: Option<Version>,
    ) -> impl Stream<Item = Result<(u64, MoveInstance<Position>), Self>> + '_ {
        ch_positions::query(self, ch, version)
    }

    /// List of registered [`ClearingHouse`](crate::ClearingHouse) object IDs.
    fn registered_clearing_houses(
        &self,
        registry_address: ObjectId,
        version: Option<Version>,
    ) -> impl Stream<Item = Result<ObjectId, Self>> + '_ {
        self::registry::query(self, registry_address, version)
    }
}

impl<T: GraphQlClient> GraphQlClientExt for T {}
