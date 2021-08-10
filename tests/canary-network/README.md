## How to connect 

Use `docker-compose up` to start layer1 node in docker, the "docker-compose.yaml" in the current directory can connect to the canary network automatically.

## Knowing nodes

| IP             | Role  | Address                                              | Comment |
| -------------- | ----- | ---------------------------------------------------- | ------- |
| 81.70.93.215   | Alice | 12D3KooWMYhaBN5Kq2DyAmk7Ao4FSspdP1bC9JdypBUPSQ5JXi9m |         |
| 139.198.14.230 | Bob   | 12D3KooWKoeWURkrHgvBcqwa79Smm1TBDK12AnHUXXbgQaJuK4R6 |         |
| 139.198.187.91 | Dave  | 12D3KooWDw6jQjs21k13yKWdQxKkuzrNdVEe9ZcQM5bxh9c5iuHp | jacky_qinyun        |
| 68.183.182.174 | Eve | ?? | Digital ocean node in Singapore |

## Maintenance
We can use the "canary-tools.sh" to do the usual maintenance.

### Start
Use the following command to start canary node by docker-compose:
```bash
./canary-tools.sh start
```
### Stop
Use the following command to stop canary node, this will clean the docker containers but leave the tea-camellia node related data:
```bash
./canary-tools.sh stop
```
### Refresh
Use the following command to clean tea-camellia node database, and restart docker-compose with current (the newer) "docker-compose.yaml":
```bash
./canary-tools.sh refresh
```
## Run as validator
To run as a validator, you should have corresponding validator key stored in layer1 local `keystore`, and modify command of "layer1" service in "docker-compose.yaml" as following:
```
bash -c "tea-camellia --validator --chain canary --genesis-seed tearust-fast-0.1.13 --rpc-cors all --boot    nodes /ip4/81.70.93.215/tcp/30333/p2p/12D3KooWMYhaBN5Kq2DyAmk7Ao4FSspdP1bC9JdypBUPSQ5JXi9m --bootnodes /ip4/139.198.14.230/tcp/30333/p2    p/12D3KooWJyMCoHwmLjcC3XQBZBydsjahWorLw91xcxQsJqrGpzNK"
```
You may notice that we add the `--validator` paramter and removed `--ws-external --rpc-external` paramters here.
