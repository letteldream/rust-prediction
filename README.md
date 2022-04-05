# pm-smart-contract

## Deploy on the devnet

Before build and test, you must install the node modules

```
npm i
```

### Build

```
anchor build
```

### Set to devent environment
```
solana config set --url devnet
```
Change the provider information on Anchor.toml
```
[provider]
cluster = "devnet"
wallet = "/home/john/.config/solana/id.json"
```

You must have at least 3 SOL in your wallet
Check the balance of wallet and airdrop 3 SOL
```
solana balance

solana airdrop 3
```

### Deploy to devnet
```
anchor deploy
```

Then you will get the program id after deploy successfully

You must change the program_id and declar_id on Anchor.toml and lib.rs

### Test

```
anchor test --skip-deploy
```

You must modify the [provider] -> wallet on Anchor.toml
