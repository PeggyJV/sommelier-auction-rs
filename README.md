# Sommelier Auction Rust Crates

> DISCLAIMER: Users consume these crates at their own risk, understanding that they do not come with any implied expectation of profit and do come with the potential for realizing losses.

This is a quickly thrown together repo that contains a simple bot designed to participate in Sommelier fee auctions.

By "simple" I mean *very simple*. Once it decides a bid can be submitted it basically fires and forgets, submitting and then deleting the order from it's state without confirmation. It does not check your wallet balance. It's up to you to make sure the orders in your config file add up to a value less than or equal to your current usomm balance. Once there are no more orders to submit, it shuts down.

The `sommelier-auction-protos` crate contains proto bindings for the Sommelier chain's `x/auction` and `x/cellarfees` Cosmos SDK modules.

The `sommelier-auction` crate is a generalized auction library containing a client for querying auction and bid data and submitting bids.

The `sommelier-auction-order-engine` crate is a *very* simple order engine. It *does not* check that the uSOMM it will bid has a USD value that results in a favorable transaction. It only check that the uSOMM-denominated cost meets the user's specified parameters.

# Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as below, without any additional terms or conditions.

# Installation

Right now you'll have to build the binary yourself. This requires `cargo`, which is installed by `rustup`, a Rust toolchain management program.

To install `rustup`, go [here](https://www.rust-lang.org/tools/install). They provide a link for installing on Windows if you can't run the curl commands.

If everything worked, you should be able to run

```bash
cargo --version
# output will be something like: cargo 1.74.0 (ecb9851af 2023-10-18)
```

Next, clone this repo and build the bot. If you don't have `git` installed, you can also go [here](https://github.com/peggyjv/sommelier-auction-bot), click the green button that says "Code", and click "Download Zip". You'll then need to unzip the folder.

```bash
# don't worry about the git command if you downloaded a .zip and unzipped the repo
git clone https://github.com/peggyjv/sommelier-auction-bot.git
cd sommelier-auction-bot
cargo install --bin auction-bot --path ./bin/auction-bot
```

If the installation worked, you should be able to run 

```bash
auction-bot
```

You should see some kind of error about needing a config file.

# Usage

There is an example config TOML `example-config.toml`. You'll need to set the `rpc_endpoint` and `grpc_endpoint` if you don't want to use Polkachu (default).

The bidder wallet is set by either setting `key_path` in the config file to a path to a .pem key file, or by setting the `SOMMELIER_AUCTION_MNEMONIC` environment variable to a 24-word phrase. It cannot be 12. Obviously, the wallet must have enough uSOMM in it to cover your orders.

Simply run

```bash
auction-bot --config <PATH TO CONFIG TOML>
```

If you want more verbose logs run

```bash
RUST_LOG=debug,h2=info,hyper=info,tower=info,rustls=info auction-bot --config <PATH TO CONFIG TOML>
```

*PLEASE NOTE*: If you see an error when a bid is submitted it is very possible the transaction was successful. You'll need to confirm on-chain by querying bids for the auction and checking for any with your sender address as the bidder, or by checking your wallet for gravity-denominated balances with the `sommelier` CLI.

## Orders

The `orders` section of the config file is a list of orders to submit. Orders are denom, amount and price in USD. The bot will take care of converting auctioned denoms and SOMM to USD.

User will find a following queries useful.

```bash

# Query active auctions and get all denoms
sommelier query auction active-auctions --node "url"

# Query the underlying erc20 contract address for a given denom
sommelier query gravity denom-to-erc20 gravityxxxx --node  https://sommelier-rpc.polkachu.com:443

```

Running the app will show delta between the current price of the auction.

# License

© 2024 Peggy J.V.

This project is licensed under either of

Apache License, Version 2.0 (LICENSE-APACHE)
MIT license (LICENSE-MIT)
at your option.

The SPDX license identifier for this project is MIT OR Apache-2.0.
