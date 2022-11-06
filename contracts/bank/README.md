# cw-bank

The `bank` contract handles minting, burning, and transfers of fungible tokens.

## Namespaces

The contract allows creation of token **namespaces**, and appointing third party accounts as admins to namespaces. The admin has the power to mint, burn, and force-transfer tokens under the namespace, as well as configuring an "after send hook" which is invoked every time a token under the namespace is transferred.

See the comments in the [`denom`](https://github.com/steak-enjoyers/cw-sdk/blob/main/contracts/bank/src/denom/mod.rs#L1-L20) module for further explanations on the namespace semantics.

See the [`token-factory`](https://github.com/steak-enjoyers/cw-sdk/blob/main/contracts/token-factory) contract for an example implementation of namespace admin contracts.

## License

Contents of this crate are open source under [GNU Affero General Public License](../../LICENSE) v3 or later.
