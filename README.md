# StarkNet Database Interface

This repository provides a robust interface for managing and querying blockchain data about contract, specifically tailored for StarkNet's `StarkFelt` data types. It includes a backend database module powered by RocksDB for efficient data storage and retrieval, and a command-line interface module for interactive data manipulation and querying.

## Features

- **Database Management**: Utilize RocksDB for high-performance storage and retrieval of blockchain-related data.
- **Data Versioning**: Track historical changes in data with rollback capabilities.
- **Serialization/Deserialization**: Leverage `bincode` for efficient serialization of blockchain data.
- **Interactive CLI**: A user-friendly command-line interface to interact with the database using `dialoguer` for input handling.
- **Error Handling**: Comprehensive error management to ensure robust operation and ease of debugging.

## Modules

### Database Module (`db`)

This module includes the `Database` class that provides methods for:

- Inserting, retrieving, and deleting data in a RocksDB instance.
- Managing data related to contracts and transaction nonces.
- Handling versioned histories of blockchain states with rollback support.

### Command-Line Interface Module (`cli`)

The CLI module offers interactive prompts to:

- Fetch and display specific blockchain-related data like class hashes, nonces, and storage keys.
- Revert the state of the database to a previous block.
- Flush pending database writes to disk.
- Quit the application.

## Contributing

Contributions are welcome! Please fork the repository and submit pull requests with your features or fixes.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

StarkNet API for providing the `StarkFelt` data structures.
The Rust community for the invaluable libraries like `rocksdb` and `dialoguer`.
