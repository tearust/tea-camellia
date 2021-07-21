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