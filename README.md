# FUEL DATA PROVIDER



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