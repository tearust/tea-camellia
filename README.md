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
