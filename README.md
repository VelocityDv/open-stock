<p align="center">
   <img src="./docs/open-stock.svg" alt="Logo">
</p>

The open source inventory manager.

### Organise inventory, employees, customers and orders simply, reliably and fast. 

> OpenStock is currently in BETA, so features may not yet be considered reliable as features are still considered experimental with API formatting being subject to change.

With `OpenStock`, you can manage:
- Stock Control
- Purchase Orders
- Point of Sale Operations
- Transactions
- Orders
- Employees (with Authentication)

In an API format. 
> For a visual interface, see `open-stock-pos`. 

OpenStock is a rust project utilizing [`rocket`](https://rocket.rs/) and [`sea-orm`](https://github.com/SeaQL/sea-orm) to batch and execute queries to a stored DB. 

### Why rust?
Rust was the perfect choice for this project as it aims to produce a reliable and consistent output whilst taking various forms of input. Rust offers this in combination with high performance, albeit slower development times. However, for this project the trade-off is more than worth it. As rust has recently become a far more matured language, database ORM's like sea-orm (based on SQLx) and Diesel are well build and provide a high degree of type-safety in formatting, reading, writing and relaying information from the database - preventing poorly formatted entries and invalid column values. 

## Getting Started
OpenStock is available as a crate to be integrated in your project or project-space if you wish to utilize the type-system created by it. However, for a default setup the API can be hosted yourself by performing the following:

First pull the docker image from docker

```sh
docker pull ...
```

Next, Initialize your `docker-compose.yaml` file (template file [here](./docker-compose.yaml)). This should include a MySQL database and OpenStock. Notably, it is recommended to add a database viewer such as adminer for development to monitor changes and trace any issues of object structure you may encounter.