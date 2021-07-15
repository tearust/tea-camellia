# Developing

## Benchmark

### Prepare

To let layer1 support benchmark, please compile as follows:

```
./benchmark.sh build
```

And run `./benchmark.sh test` anytime you make a change about benchmark.

### Generate `WeightInfo`
For custom pallet, run the following command to generate `WeightInfo` trait and related implementation:
```
./benchmark.sh template [pallet name] [sub-folder name of the pallet]
```
About the command params:
- [pallet name]: is the pallet name defined in runtime
- [sub-folder name of the pallet]: all custom pallets are under the `pallets` folder, specify sub-folder name here

Take `pallet_tea` for example, generate command is:
```
./benchmark.sh template pallet_tea tea
```

### Generate Weight files

Run the following command to generate all weight files:
```
./benchmark.sh batch_weight
```

Or run the following command if you want to update single weight file:
```
./benchmark.sh weight [pallet name]
```

About the command params:
- [pallet name]: is the pallet name defined in runtime

# Running
## Running on native host
### Running with `dev` mode 

```bash
./tea-camellia --tmp --alice --dev
```
### Running with `dev` and open ports for network access (no safe)
```
./target/debug/tea-camellia --dev --alice --port 30334 --ws-port 9944 --rpc-port 9933 --unsafe-rpc-external --unsafe-ws-external --tmp
```

## Running multiple nodes with docker compose
First, you should prepare the docker image `tearust/tea-camellia:latest`, if you want to build by yourself please run `docker/build-image.sh` or `docker/build-image.sh fast` if you want to test with fast mode.

Then, cd into the `tests/local-network` directory and run `docker-compose up` (or `docker compose up` if you have new version of docker client) to start nodes.

## Running on canary network
Follow the guide [here](https://github.com/tearust/tea-camellia/tree/main/tests/canary-network)

## Setup the testing account
We have export all test accounts to the file `tests/batch_exported_account_asdfasdf.json` in the root folder of this git repo.

You need to install a Polkadot browser extension. Click Restore accounts from backup JSON file. Select this JSON file, input the restore password `asdfasdf`. You can see all acounts are restored.

the password `asdfasdf` are also used to sign transactions.

This dummy password will not be used in production or testnet. 




## Genesis seeds coupon distribution CSV file
Use the following parameters to start

`--genesis-coupons-path <genesis-coupons-path>`


./tea-camellia --alice --dev --tmp --genesis-coupons-path test.csv
If this parameter is missing, system will use https://github.com/tearust/tea-camellia/blob/main/node/src/dev.csv instead.
