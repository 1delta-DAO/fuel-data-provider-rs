# FUEL DATA PROVIDER


### Endpoints


GET: http://localhost:8080/status/

GET: http://localhost:8080/tokens/

GET: http://localhost:8080/tokens/by-time/?start=2024-02-26T12:00:00&end=2024-02-26T14:00:00

POST: 
```shell
curl -X POST "http://localhost:8080/tokens/by-address" \
     -H "Content-Type: application/json" \
     -d '{
           "addresses": [
             "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
             "33a6d90877f12c7954cca6d65587c25e9214c7bed2231c188981c7114c1bdb78"
           ]
         }'
```

### SeaORM

For now, we have only manual execution. To populate schema to the DB you have to:

1. Install SeaORM CLI
```shell
cargo install sea-orm-cli
```
2. Execute db migration

```shell
sea-orm-cli migrate up --database-url "postgres://admin:admin@localhost:5432/fuel_data"
```
If you need revert migration you can always execute this:

```shell
sea-orm-cli migrate down --database-url "postgres://admin:admin@localhost:5432/fuel_data"
```

Regenerate entities (if you have to)
```shell
sea-orm-cli generate entity -u postgresql://admin:admin@localhost:5432/fuel_data -o src/ports/db/model

```