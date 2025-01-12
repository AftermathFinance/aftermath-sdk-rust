#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for the core `sui` Sui package located at "0x2" onchain.

pub use af_move_type;
use af_move_type::{MoveInstance, MoveType};
pub use af_sui_types::ObjectId;
use bag::*;
use balance::*;
use move_stdlib_sdk::type_name::TypeName;
use object::*;
use table::*;
use url::*;
use vec_map::*;
use vec_set::*;
use versioned::*;

af_sui_pkg_sdk::sui_pkg_sdk!(sui @ "0x2" {
    module bag {
        struct Bag has key, store {
            /// the ID of this bag
            id: UID,
            /// the number of key-value pairs in the bag
            size: u64,
        }
    }

    module balance {
        /// A Supply of T. Used for minting and burning.
        /// Wrapped into a `TreasuryCap` in the `Coin` module.
        struct Supply<!phantom T> has store {
            value: u64
        }

        /// Storable balance - an inner struct of a Coin type.
        /// Can be used to store coins which don't need the key ability.
        struct Balance<!phantom T> has store {
            value: u64
        }
    }

    module bcs {
        /// A helper struct that saves resources on operations. For better
        /// vector performance, it stores reversed bytes of the BCS and
        /// enables use of `vector::pop_back`.
        struct BCS has store, copy, drop {
            bytes: vector<u8>
        }
    }

    module borrow {
        /// An object wrapping a `T` and providing the borrow API.
        struct Referent<T: key + store> has store {
            id: address,
            value: move_stdlib_sdk::option::Option<T>
        }

        /// A hot potato making sure the object is put back once borrowed.
        struct Borrow { #[serde(rename = "ref")] ref_: address, obj: ID }
    }

    module clock {
        /// Singleton shared object that exposes time to Move calls.  This
        /// object is found at address 0x6, and can only be read (accessed
        /// via an immutable reference) by entry functions.
        ///
        /// Entry Functions that attempt to accept `Clock` by mutable
        /// reference or value will fail to verify, and honest validators
        /// will not sign or execute transactions that use `Clock` as an
        /// input parameter, unless it is passed by immutable reference.
        struct Clock has key {
            id: UID,
            /// The clock's timestamp, which is set automatically by a
            /// system transaction every time consensus commits a
            /// schedule, or by `sui::clock::increment_for_testing` during
            /// testing.
            timestamp_ms: u64,
        }
    }

    module coin {
        /// A coin of type `T` worth `value`. Transferable and storable
        struct Coin<!phantom T> has key, store {
            id: UID,
            balance: Balance<T>
        }

        /// Each Coin type T created through `create_currency` function will have a
        /// unique instance of `CoinMetadata<T>` that stores the metadata for this coin type.
        struct CoinMetadata<!phantom T> has key, store {
            id: UID,
            /// Number of decimal places the coin uses.
            /// A coin with `value ` N and `decimals` D should be shown as N / 10^D
            /// E.g., a coin with `value` 7002 and decimals 3 should be displayed as 7.002
            /// This is metadata for display usage only.
            decimals: u8,
            /// Name for the token
            name: String, // from std::string::String
            /// Symbol for the token
            symbol: String, // from std::ascii::String
            /// Description of the token
            description: String, // from std::string::String
            /// URL for the token logo
            icon_url: move_stdlib_sdk::option::Option<Url>
        }

        /// Similar to CoinMetadata, but created only for regulated coins that use the DenyList.
        /// This object is always immutable.
        struct RegulatedCoinMetadata<!phantom T> has key {
            id: UID,
            /// The ID of the coin's CoinMetadata object.
            coin_metadata_object: ID,
            /// The ID of the coin's DenyCap object.
            deny_cap_object: ID,
        }

        /// Capability allowing the bearer to mint and burn
        /// coins of type `T`. Transferable
        struct TreasuryCap<!phantom T> has key, store {
            id: UID,
            total_supply: Supply<T>
        }

        /// Capability allowing the bearer to freeze addresses, preventing those addresses from
        /// interacting with the coin as an input to a transaction.
        struct DenyCap<!phantom T> has key, store {
            id: UID,
        }
    }

    module deny_list {
        /// A shared object that stores the addresses that are blocked for a given core type.
        struct DenyList has key {
            id: UID,
            /// The individual deny lists.
            lists: Bag,
        }

        /// Stores the addresses that are denied for a given core type.
        struct PerTypeList has key, store {
            id: UID,
            /// Number of object types that have been banned for a given address.
            /// Used to quickly skip checks for most addresses.
            denied_count: Table<address, u64>,
            /// Set of addresses that are banned for a given type.
            /// For example with `sui::coin::Coin`: If addresses A and B are banned from using
            /// "0...0123::my_coin::MY_COIN", this will be "0...0123::my_coin::MY_COIN" -> {A, B}.
            denied_addresses: Table<vector<u8>, VecSet<address>>,
        }
    }

    module display {
        /// The `Display<T>` object. Defines the way a T instance should be
        /// displayed. Display object can only be created and modified with
        /// a PublisherCap, making sure that the rules are set by the owner
        /// of the type.
        ///
        /// Each of the display properties should support patterns outside
        /// of the system, making it simpler to customize Display based
        /// on the property values of an Object.
        /// ```move
        /// // Example of a display object
        /// Display<0x...::capy::Capy> {
        ///  fields:
        ///    <name, "Capy { genes }">
        ///    <link, "https://capy.art/capy/{ id }">
        ///    <image, "https://api.capy.art/capy/{ id }/svg">
        ///    <description, "Lovely Capy, one of many">
        /// }
        /// ```
        ///
        /// Uses only String type due to external-facing nature of the object,
        /// the property names have a priority over their types.
        struct Display<!phantom T: key> has key, store {
            id: UID,
            /// Contains fields for display. Currently supported
            /// fields are: name, link, image and description.
            fields: VecMap<String, String>,
            /// Version that can only be updated manually by the Publisher.
            version: u16
        }

        /// Event: emitted when a new Display object has been created for type T.
        /// Type signature of the event corresponds to the type while id serves for
        /// the discovery.
        ///
        /// Since Sui RPC supports querying events by type, finding a Display for the T
        /// would be as simple as looking for the first event with `Display<T>`.
        struct DisplayCreated<!phantom T: key> has copy, drop {
            id: ID
        }

        /// Version of Display got updated -
        struct VersionUpdated<!phantom T: key> has copy, drop {
            id: ID,
            version: u16,
            fields: VecMap<String, String>,
        }
    }

    module dynamic_field {
        /// Internal object used for storing the field and value
        struct Field<Name: copy + drop + store, Value: store> has key {
            /// Determined by the hash of the object ID, the field name value and it's type,
            /// i.e. hash(parent.id || name || Name)
            id: UID,
            /// The value for the name of this field
            name: Name,
            /// The value bound to this field
            value: Value,
        }
    }

    module dynamic_object_field {
        // Internal object used for storing the field and the name associated with the value
        // The separate type is necessary to prevent key collision with direct usage of dynamic_field
        struct Wrapper<Name> has copy, drop, store {
            name: Name,
        }
    }

    module linked_table {
        struct LinkedTable<K: copy + drop + store, !phantom V: store> has key, store {
            /// the ID of this table
            id: UID,
            /// the number of key-value pairs in the table
            size: u64,
            /// the front of the table, i.e. the key of the first entry
            head: move_stdlib_sdk::option::Option<K>,
            /// the back of the table, i.e. the key of the last entry
            tail: move_stdlib_sdk::option::Option<K>,
        }

        struct Node<K: copy + drop + store, V: store> has store {
            /// the previous key
            prev: move_stdlib_sdk::option::Option<K>,
            /// the next key
            next: move_stdlib_sdk::option::Option<K>,
            /// the value being stored
            value: V
        }
    }

    module object_bag {
        struct ObjectBag has key, store {
            /// the ID of this bag
            id: UID,
            /// the number of key-value pairs in the bag
            size: u64,
        }
    }

    module object_table {
        struct ObjectTable<!phantom K: copy + drop + store, !phantom V: key + store> has key, store {
            /// the ID of this table
            id: UID,
            /// the number of key-value pairs in the table
            size: u64,
        }
    }

    module package {
        /// This type can only be created in the transaction that
        /// generates a module, by consuming its one-time witness, so it
        /// can be used to identify the address that published the package
        /// a type originated from.
        struct Publisher has key, store {
            id: UID,
            package: String,
            module_name: String,
        }

        /// Capability controlling the ability to upgrade a package.
        struct UpgradeCap has key, store {
            id: UID,
            /// (Mutable) ID of the package that can be upgraded.
            package: ID,
            /// (Mutable) The number of upgrades that have been applied
            /// successively to the original package.  Initially 0.
            version: u64,
            /// What kind of upgrades are allowed.
            policy: u8,
        }

        /// Permission to perform a particular upgrade (for a fixed version of
        /// the package, bytecode to upgrade with and transitive dependencies to
        /// depend against).
        ///
        /// An `UpgradeCap` can only issue one ticket at a time, to prevent races
        /// between concurrent updates or a change in its upgrade policy after
        /// issuing a ticket, so the ticket is a "Hot Potato" to preserve forward
        /// progress.
        struct UpgradeTicket {
            /// (Immutable) ID of the `UpgradeCap` this originated from.
            cap: ID,
            /// (Immutable) ID of the package that can be upgraded.
            package: ID,
            /// (Immutable) The policy regarding what kind of upgrade this ticket
            /// permits.
            policy: u8,
            /// (Immutable) SHA256 digest of the bytecode and transitive
            /// dependencies that will be used in the upgrade.
            digest: vector<u8>,
        }

        /// Issued as a result of a successful upgrade, containing the
        /// information to be used to update the `UpgradeCap`.  This is a "Hot
        /// Potato" to ensure that it is used to update its `UpgradeCap` before
        /// the end of the transaction that performed the upgrade.
        struct UpgradeReceipt {
            /// (Immutable) ID of the `UpgradeCap` this originated from.
            cap: ID,
            /// (Immutable) ID of the package after it was upgraded.
            package: ID,
        }
    }

    module priority_queue {
        /// Struct representing a priority queue. The `entries` vector represents a max
        /// heap structure, where entries\[0\] is the root, entries\[1\] and entries\[2\] are the
        /// left child and right child of the root, etc. More generally, the children of
        /// entries\[i\] are at at i * 2 + 1 and i * 2 + 2. The max heap should have the invariant
        /// that the parent node's priority is always higher than its child nodes' priorities.
        struct PriorityQueue<T: drop> has store, drop {
            entries: vector<Entry<T>>,
        }

        struct Entry<T: drop> has store, drop {
            priority: u64, // higher value means higher priority and will be popped first
            value: T,
        }
    }

    module random {
        /// Singleton shared object which stores the global randomness state.
        /// The actual state is stored in a versioned inner field.
        struct Random has key {
            id: UID,
            inner: Versioned,
        }

        struct RandomInner has store {
            version: u64,

            epoch: u64,
            randomness_round: u64,
            random_bytes: vector<u8>,
        }
    }

    module sui {
        /// Name of the coin
        struct SUI has drop {}
    }

    module table {
        struct Table<!phantom K: copy + drop + store, !phantom V: store> has key, store {
            /// the ID of this table
            id: UID,
            /// the number of key-value pairs in the table
            size: u64,
        }
    }

    module table_vec {
        struct TableVec<!phantom Element: store> has store {
            /// The contents of the table vector.
            contents: Table<u64, Element>,
        }
    }

    module token {
        /// A single `Token` with `Balance` inside. Can only be owned by an address,
        /// and actions performed on it must be confirmed in a matching `TokenPolicy`.
        struct Token<!phantom T> has key {
            id: UID,
            /// The Balance of the `Token`.
            balance: Balance<T>,
        }

        /// A Capability that manages a single `TokenPolicy` specified in the `for`
        /// field. Created together with `TokenPolicy` in the `new` function.
        struct TokenPolicyCap<!phantom T> has key, store {
            id: UID,
            #[serde(rename = "for")]
            for_: ID
        }

        /// `TokenPolicy` represents a set of rules that define what actions can be
        /// performed on a `Token` and which `Rules` must be satisfied for the
        /// action to succeed.
        ///
        /// - For the sake of availability, `TokenPolicy` is a `key`-only object.
        /// - Each `TokenPolicy` is managed by a matching `TokenPolicyCap`.
        /// - For an action to become available, there needs to be a record in the
        /// `rules` VecMap. To allow an action to be performed freely, there's an
        /// `allow` function that can be called by the `TokenPolicyCap` owner.
        struct TokenPolicy<!phantom T> has key {
            id: UID,
            /// The balance that is effectively spent by the user on the "spend"
            /// action. However, actual decrease of the supply can only be done by
            /// the `TreasuryCap` owner when `flush` is called.
            ///
            /// This balance is effectively spent and cannot be accessed by anyone
            /// but the `TreasuryCap` owner.
            spent_balance: Balance<T>,
            /// The set of rules that define what actions can be performed on the
            /// token. For each "action" there's a set of Rules that must be
            /// satisfied for the `ActionRequest` to be confirmed.
            rules: VecMap<String, VecSet<TypeName>>
        }

        /// A request to perform an "Action" on a token. Stores the information
        /// about the action to be performed and must be consumed by the `confirm_request`
        /// or `confirm_request_mut` functions when the Rules are satisfied.
        struct ActionRequest<!phantom T> {
            /// Name of the Action to look up in the Policy. Name can be one of the
            /// default actions: `transfer`, `spend`, `to_coin`, `from_coin` or a
            /// custom action.
            name: String,
            /// Amount is present in all of the txs
            amount: u64,
            /// Sender is a permanent field always
            sender: address,
            /// Recipient is only available in `transfer` action.
            recipient: move_stdlib_sdk::option::Option<address>,
            /// The balance to be "spent" in the `TokenPolicy`, only available
            /// in the `spend` action.
            spent_balance: move_stdlib_sdk::option::Option<Balance<T>>,
            /// Collected approvals (stamps) from completed `Rules`. They're matched
            /// against `TokenPolicy.rules` to determine if the request can be
            /// confirmed.
            approvals: VecSet<TypeName>,
        }

        /// Dynamic field key for the `TokenPolicy` to store the `Config` for a
        /// specific action `Rule`. There can be only one configuration per
        /// `Rule` per `TokenPolicy`.
        struct RuleKey<!phantom T> has store, copy, drop { is_protected: bool }

        /// An event emitted when a `TokenPolicy` is created and shared. Because
        /// `TokenPolicy` can only be shared (and potentially frozen in the future),
        /// we emit this event in the `share_policy` function and mark it as mutable.
        struct TokenPolicyCreated<!phantom T> has copy, drop {
            /// ID of the `TokenPolicy` that was created.
            id: ID,
            /// Whether the `TokenPolicy` is "shared" (mutable) or "frozen"
            /// (immutable) - TBD.
            is_mutable: bool,
        }
    }

    module transfer {
        /// This represents the ability to `receive` an object of type `T`.
        /// This type is ephemeral per-transaction and cannot be stored on-chain.
        /// This does not represent the obligation to receive the object that it
        /// references, but simply the ability to receive the object with object ID
        /// `id` at version `version` if you can prove mutable access to the parent
        /// object during the transaction.
        /// Internals of this struct are opaque outside this module.
        struct Receiving<!phantom T: key> has drop {
            id: ID,
            version: u64,
        }
    }

    module tx_context {
        /// Information about the transaction currently being executed.
        /// This cannot be constructed by a transaction--it is a privileged object created by
        /// the VM and passed in to the entrypoint of the transaction as `&mut TxContext`.
        struct TxContext has drop {
            /// The address of the user that signed the current transaction
            sender: address,
            /// Hash of the current transaction
            tx_hash: vector<u8>,
            /// The current epoch number
            epoch: u64,
            /// Timestamp that the epoch started at
            epoch_timestamp_ms: u64,
            /// Counter recording the number of fresh id's created while executing
            /// this transaction. Always 0 at the start of a transaction
            ids_created: u64
        }
    }

    module url {
        /// Standard Uniform Resource Locator (URL) string.
        struct Url has store, copy, drop {
            // TODO: validate URL format
            url: String,
        }
    }

    module vec_map {
        /// A map data structure backed by a vector. The map is guaranteed not to contain duplicate keys, but entries
        /// are *not* sorted by key--entries are included in insertion order.
        /// All operations are O(N) in the size of the map--the intention of this data structure is only to provide
        /// the convenience of programming against a map API.
        /// Large maps should use handwritten parent/child relationships instead.
        /// Maps that need sorted iteration rather than insertion order iteration should also be handwritten.
        struct VecMap<K: copy, V> has copy, drop, store {
            contents: vector<Entry<K, V>>,
        }

        /// An entry in the map
        struct Entry<K: copy, V> has copy, drop, store {
            key: K,
            value: V,
        }
    }

    module vec_set {
        /// A set data structure backed by a vector. The set is guaranteed not to
        /// contain duplicate keys. All operations are O(N) in the size of the set
        /// - the intention of this data structure is only to provide the convenience
        /// of programming against a set API. Sets that need sorted iteration rather
        /// than insertion order iteration should be handwritten.
        struct VecSet<K: copy + drop> has copy, drop, store {
            contents: vector<K>,
        }
    }

    module versioned {
        /// A wrapper type that supports versioning of the inner type.
        /// The inner type is a dynamic field of the Versioned object, and is keyed using version.
        /// User of this type could load the inner object using corresponding type based on the version.
        /// You can also upgrade the inner object to a new type version.
        /// If you want to support lazy upgrade of the inner type, one caveat is that all APIs would have
        /// to use mutable reference even if it's a read-only API.
        struct Versioned has key, store {
            id: UID,
            version: u64,
        }

        /// Represents a hot potato object generated when we take out the dynamic field.
        /// This is to make sure that we always put a new value back.
        struct VersionChangeCap {
            versioned_id: ID,
            old_version: u64,
        }
    }
});

/// Custom `ID` and `UID` impls with better [`Display`](std::fmt::Display).
pub mod object {
    #![expect(
        clippy::too_long_first_doc_paragraph,
        reason = "Docs for the sui-framework have long first paragraphs."
    )]
    use super::ObjectId;

    /// An object ID. This is used to reference Sui Objects.
    /// This is *not* guaranteed to be globally unique--anyone can create an `ID` from a `UID` or
    /// from an object, and ID's can be freely copied and dropped.
    /// Here, the values are not globally unique because there can be multiple values of type `ID`
    /// with the same underlying bytes. For example, `object::id(&obj)` can be called as many times
    /// as you want for a given `obj`, and each `ID` value will be identical.
    #[derive(
        af_sui_pkg_sdk::MoveStruct,
        af_sui_pkg_sdk::serde::Deserialize,
        af_sui_pkg_sdk::serde::Serialize,
        af_sui_pkg_sdk::Tabled,
        derive_more::From,
        Clone,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[move_(crate = af_sui_pkg_sdk::af_move_type)]
    #[serde(crate = "af_sui_pkg_sdk::serde", transparent)]
    #[tabled(crate = "af_sui_pkg_sdk::tabled")]
    pub struct ID {
        pub bytes: ObjectId,
    }

    impl std::fmt::Display for ID {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.bytes)
        }
    }

    /// Globally unique IDs that define an object's ID in storage. Any Sui Object, that is a struct
    /// with the `key` ability, must have `id: UID` as its first field.
    /// These are globally unique in the sense that no two values of type `UID` are ever equal, in
    /// other words for any two values `id1: UID` and `id2: UID`, `id1` != `id2`.
    /// This is a privileged type that can only be derived from a `TxContext`.
    /// `UID` doesn't have the `drop` ability, so deleting a `UID` requires a call to `delete`.
    #[derive(
        af_sui_pkg_sdk::MoveStruct,
        af_sui_pkg_sdk::serde::Deserialize,
        af_sui_pkg_sdk::serde::Serialize,
        af_sui_pkg_sdk::Tabled,
        Clone,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[move_(crate = af_sui_pkg_sdk::af_move_type)]
    #[serde(crate = "af_sui_pkg_sdk::serde")]
    #[tabled(crate = "af_sui_pkg_sdk::tabled")]
    pub struct UID {
        pub id: ID,
    }

    impl std::fmt::Display for UID {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.id)
        }
    }
}

// =============================================================================
// Convenience functions
// =============================================================================
use dynamic_field::{Field, FieldTypeTag};

/// Unpack an instance of a dynamic field into its name and value instances.
pub fn unpack_field_instance<K: MoveType, V: MoveType>(
    field: MoveInstance<Field<K, V>>,
) -> (MoveInstance<K>, MoveInstance<V>) {
    let MoveInstance {
        type_: FieldTypeTag {
            name: name_type,
            value: value_type,
        },
        value: Field { name, value, .. },
    } = field;
    (
        MoveInstance {
            type_: name_type,
            value: name,
        },
        MoveInstance {
            type_: value_type,
            value,
        },
    )
}
