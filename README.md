# FUEL MARKET DATA PROVIDER

A lightweight solution providing aggregated trading data from the FUEL network at minimal cost. This service collects, processes, and serves market data through a simple REST API, making it accessible for various applications and analysis.

## Current State
The codebase contains multiple data fetching strategies and retry mechanisms, which might appear overly complex and inconsistent. This complexity stems from historical instability of FUEL network endpoints, which required implementing various fallback mechanisms and alternative data sources. While the current implementation could be simplified, we've chosen to maintain these redundancies as a precaution against potential future endpoint issues.

## Installation and Setup

### Prerequisites
- Docker and Docker Compose installed on your system
- Git

### Installation Steps

1. Clone the repository:
```shell
git clone https://github.com/your-username/fuel-data-provider-rs.git
cd fuel-data-provider-rs
```

2. Start the database and Adminer using Docker Compose:
```shell
docker-compose up -d db adminer
```

3. Wait for the database to be ready (you can check the status in Adminer at http://localhost:8082)

4. Install SeaORM CLI:
```shell
cargo install sea-orm-cli
```

5. Run database migrations:
```shell
sea-orm-cli migrate up --database-url "postgres://admin:admin@localhost:5432/fuel_data"
```

6. Build and run the application:
```shell
cargo build --release
cargo run --release
```

### Accessing the Services
- API: http://localhost:8080
- Database Admin (Adminer): http://localhost:8082
  - System: PostgreSQL
  - Server: db
  - Username: admin
  - Password: admin
  - Database: fuel_data

### Environment Variables
The application uses the following environment variables (with default values):
- `SERVER_PORT_HTTP`: 8080
- `RUST_LOG`: "info"
- `DB_URL`: "db:5432/fuel_data"
- `API_QUERY_SLEEP_TIME`: 35

### Configuration
The application uses a TOML-based configuration system with the following hierarchy (from highest to lowest priority):

1. Environment Variables
2. Runtime-specific config files
3. Default config file (`resources/config.toml`)

#### Default Configuration
The default configuration is stored in `resources/config.toml` and includes settings for:
- Server configuration (ports, API keys)
- Database connection details
- RPC endpoints
- Contract addresses
- Calculation parameters
- External service configurations

#### Overriding Configuration
You can override the default configuration in several ways:

1. Using Environment Variables:
```shell
export SERVER_PORT_HTTP=9090
export DB_URL="custom-db:5432/fuel_data"
cargo run
```

2. Using Runtime-specific Config Files:
Create a new TOML file (e.g., `config.dev.toml`) and run:
```shell
RUST_ENV=dev cargo run
```

3. Using Command Line Arguments:
```shell
cargo run -- --config-path /path/to/custom/config.toml
```

4. Using Docker Compose:
You can override configuration values in your `docker-compose.yml` file:

```yaml
services:
  fuel-dp:
    environment:
      - SERVER_PORT_HTTP=${SERVER_PORT_HTTP}
      - DB_URL=${DB_URL}
      - API_KEY=${API_KEY}
      # Add other environment variables as needed
```

Or use a `.env` file:
```env
SERVER_PORT_HTTP=8080
DB_URL=db:5432/fuel_data
API_KEY=your_secret_key
# Add other environment variables as needed
```

### Security Warning
The service uses a simple API key authentication mechanism (`x-api-key` header) which provides minimal security. This is NOT suitable for direct internet exposure. The service should:

1. Always be deployed behind a proper API Gateway or Reverse Proxy
2. Use HTTPS/TLS encryption
3. Implement proper rate limiting
4. Be placed in a private network with controlled access

Example of API key usage:
```shell
curl -H "x-api-key: your_secret_api_key" http://localhost:8080/tokens/
```

### Endpoints

#### Assets

GET: http://localhost:8080/status/

GET: http://localhost:8080/tokens/

GET: http://localhost:8080/tokens/by-time/?start=2024-02-26T12:00:00&end=2025-03-26T14:00:00

GET http://localhost:8080/tokens/prices?address=0x0000000000000000000000000000000000000000000000000000000000000000

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

#### Trending

GET: http://localhost:8080/tokens/top-gainers?count=5

GET: http://localhost:8080/tokens/top-losers?count=5

GET: http://localhost:8080/tokens/top-volume?count=5


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

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.