# Tokens

There are multiple sources of token data:

1. Keplr Tokens: [registry](https://github.com/chainapsis/keplr-contract-registry.git)
2. Secret Foundation Tokens: [registry](https://github.com/SecretFoundation/AssetRegistry)
3. Unofficial Secret Tokens: _TBD_
4. Unknown Tokens: Require 2 separate queries for code_hash and token_info

## Registries

### Keplr Contract Registry

These are the tokens that are searchable inside of Keplr. Secret tokens live [here](https://github.com/chainapsis/keplr-contract-registry/tree/main/cosmos/secret/tokens).
Each token is a separate file. The file names are contract addresses.

```json
{
  "contractAddress": "secret1097nagcaavlkchl87xkqptww2qkwuvhdnsqs2v",
  "imageUrl": "https://raw.githubusercontent.com/chainapsis/keplr-contract-registry/main/images/secret/sstjuno.svg",
  "metadata": {
    "name": "Secret stJUNO",
    "symbol": "sstJUNO",
    "decimals": 6
  }
}
```

### Secret Foundation

All tokens are combined in a single [`assets.json`](https://github.com/SecretFoundation/AssetRegistry/blob/main/assets.json) file.

```json
{
  "Native": [
    {
      "contract_name": "Secret Secret",
      "visible_asset_name": "sSCRT",
      "symbol": "sSCRT",
      "decimals": 6,
      "denom": "uscrt",
      "contract_address": "secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek",
      "contract_hash": "af74387e276be8874f07bec3a87023ee49b0e7ebe08178c49d0a49c3c98ed60e",
      "version": "SNIP-20"
    },
    {
      ...
    }
  ],
  "Axelar Bridged Assets": [
    {
      "contract_name": "Secret Axelar",
      "visible_asset_name": "AXL",
      "symbol": "sAXL",
      "decimals": 6,
      "denom": "uaxl",
      "contract_address": "secret1vcau4rkn7mvfwl8hf0dqa9p0jr59983e3qqe3z",
      "contract_hash": "638a3e1d50175fbcb8373cf801565283e3eb23d88a9b7b7f99fcc5eb1e6b561e",
      "version": "SNIP-25"
    },
    {
      ...
    }
  ]
}
```

## My preferred data structure

Idea 1

```json
{
  "contract_info": {
    "contract_address": "secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek",
    "code_hash": "af74387e276be8874f07bec3a87023ee49b0e7ebe08178c49d0a49c3c98ed60e",
  }
  "metadata": {
    "name": "Secret Secret",
    "symbol": "sSCRT",
    "decimals": 6
  },
  "image_url": "https://raw.githubusercontent.com/chainapsis/keplr-contract-registry/main/images/secret/sscrt.svg"
}
```

Idea 2

```json
{
  "contract_address": "secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek",
  "code_hash": "af74387e276be8874f07bec3a87023ee49b0e7ebe08178c49d0a49c3c98ed60e",
  "decimals": 6,
  "name": "Secret Secret",
  "symbol": "sSCRT",
  "display_name": "sSCRT",
  "image_url": "https://raw.githubusercontent.com/chainapsis/keplr-contract-registry/main/images/secret/sscrt.svg"
}
```

Maps? A map from symbol to address is convenient for well-known tokens.

```rust
static SYMBOL_TO_ADDR: LazyLock<HashMap<String, Addr>> = LazyLock::new(|| todo!());
static ADDR_TO_TOKEN: LazyLock<HashMap<Addr, Token>> = LazyLock::new(|| todo!());
```
