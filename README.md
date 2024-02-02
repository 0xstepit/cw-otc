# CW-OTC

`cw-otc` is a smart contracts workspace that implements an over-the-counter market to securely allow two parties
to exchange values without a middleman and in a trust-minimized way.

## Contracts

The workspace is composed by two contracts:

- **Factory**: the factory has the purposes of orchestrating multiple markets. It keeps track of the
 available markets and to avoid creation of duplicates.

- **Market**: this is the main contract that implements the logic for otc exchange for a given pair.
Every market has the possibility to deifine a fee that is deducted from both the partiesduring an
exchange.

Below the draft of the MVP that highlights the APIs of the contracts and their interactions:

![image](./assets/mvp.png)

## Getting Started

These instructions will help you get a copy of the smart contract up and running on your local machine for development and testing purposes.

### Prerequisites

- [CosmWasm](https://github.com/CosmWasm/cosmwasm)
- Rust: [Installation Guide](https://www.rust-lang.org/tools/install)
- Command runner: [just](https://github.com/casey/just)

### Installation

1. Clone the repository and move into project directory:

    ```shell
    git clone <repository_url>
    cd <project_directory>
    ```

2. Build the smart contract:

    ```shell
    just optimize
    ```

## Usage

### Test

```shell
just test
```

### Lint

```shell
just clippy && just fmt 
```

### JSON Schema

```shell
just schema
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
