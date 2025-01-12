//! Simple example showing how to instantiate a primitive [`MoveInstance`].
use af_move_type::vector::MoveVec;
use af_move_type::MoveInstance;
use af_sui_types::Address;

fn main() {
    let instance: MoveInstance<u64> = 1.into();
    println!("{instance}");
    let address: Address = "0x6".parse().expect("Valid address");
    let instance: MoveInstance<_> = address.into();
    println!("{instance}");
    let vector: MoveVec<u32> = vec![1, 2, 3].into();
    let instance: MoveInstance<_> = vector.into();
    println!("{instance}");
}
