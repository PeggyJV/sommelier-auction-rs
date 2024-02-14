# Sommelier Auction Rust Crates

> DISCLAIMER: Users consume these crates at their own risk, understanding that they do not come with any implied expectation of profit and do come with the potential for realizing losses.

This is a quickly thrown together repo that contains a simple bot designed to participate in Sommelier fee auctions. 

The `sommelier-auction-protos` crate contains proto bindings for the Sommelier chain's `x/auction` and `x/cellarfees` Cosmos SDK modules.

The `sommelier-auction` crate is a generalized auction library containing a client for querying auction and bid data and submitting bids.

The `sommelier-auction-order-engine` crate is a *very* simple order engine. It *does not* check that the uSOMM it will bid has a USD value that results in a favorable transaction. It only check that the uSOMM-denominated cost meets the user's specified parameters.

#Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as below, without any additional terms or conditions.

# License

© 2024 Peggy J.V.

This project is licensed under either of

Apache License, Version 2.0 (LICENSE-APACHE)
MIT license (LICENSE-MIT)
at your option.

The SPDX license identifier for this project is MIT OR Apache-2.0. 
