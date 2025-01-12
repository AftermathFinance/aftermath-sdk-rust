use af_sui_pkg_sdk::sui_pkg_sdk;

sui_pkg_sdk!(coin_package {
    module main {
        struct Key {
            dummy_field: bool,
        }

        struct Wallet<T> {
            balances: vector<Balance<T>>,
        }

        struct Coin<T> {
            balance: Balance<T>,
        }

        struct Balance<!phantom T> {
            balance: u64,
            other: bool,
        }
    }
});

fn main() -> anyhow::Result<()> {
    use crate::main::*;

    let balance = Balance::new(0, false);
    println!("{balance}");
    let coin = Coin::<Key>::new(balance);
    println!("{coin}");

    let Coin { balance } = coin;
    let wallet = Wallet::new(vec![balance, Balance::new(1, false)].into());
    println!("{wallet}");

    let instance = wallet.move_instance(
        "0x1".parse()?,
        KeyTypeTag {
            address: "0x1".parse()?,
        },
    );
    println!("{instance}");
    Ok(())
}
