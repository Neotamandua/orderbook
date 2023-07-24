# Work in Progress (WIP)

**Please note that this project is currently under active development and not all features are fully implemented or released. The codebase and architecture are subject to change.**

---


# Orderbook & Matching Engine (Explanatory Project)
![Powered by Rust][badge_rust]
![License: MPL 2.0][badge_license]
![Github CI][github_ci]

[badge_rust]: https://badgers.space/badge/Powered%20by/Rust/orange
[badge_license]: https://badgers.space/github/license/Neotamandua/orderbook
[github_ci]: https://badgers.space/github/checks/Neotamandua/orderbook/development/Test%20Build?label=build

> Price/time priority i.e. FIFO Design

This Orderbook & Matching Engine project is designed to serve as an educational tool to introduce and explain the concept of orderbooks through live interaction. Please note that this project is not intended for production or professional use, but rather to facilitate a better understanding of how markets work.

## Introduction 

The Orderbook & Matching Engine is a simplified implementation that demonstrates the fundamental principles of price-time priority order matching in financial markets. It showcases the mechanics of maintaining an orderbook, processing new orders, and executing trades.

I developed this implementation primarily to gain a deeper understanding of matching algorithms and orderbook dynamics, especially detecting specific edge cases.

## Features

- **Orderbook Management**: The project provides a basic infrastructure for managing buy and sell orders in an orderbook structure.
- **gRPC Integration**: The project includes gRPC as a feature to interact with the orderbook and matching engine, allowing seamless communication with external systems.
- **Order Matching**: The matching engine algorithm matches buy and sell orders based on predefined rules and executes trades accordingly.
- **Price-Time Priority**: The order matching algorithm follows a price-time priority, where the best available price takes precedence, and orders with the same price are prioritized based on the time they were received.
- **Basic Order Types**: The project supports various order types, including:

| Order Type                | Description                                                                                              |
| ------------------------- | -------------------------------------------------------------------------------------------------------- |
| Limit Order               | An order to buy or sell at a specific price or better.                                                   |
| Market Order              | An order to buy or sell at the best available price in the market.                                       |
| Limit or Cancel Order     | A limit order to buy or sell at a specific price, but cancel if any part of it would be market executed. |
| Immediate or Cancel Order | An order to buy or sell immediately, and any unfilled portion is canceled.                               |
| Fill or Kill Order        | An order to buy or sell, which must be executed in its entirety immediately or canceled.                 |

## 

## Usage

To use this project, follow these steps:
WIP

## Contributions

Contributions to this explanatory project are not actively sought, as it primarily serves as a demonstration tool. However, if you discover any bugs or have suggestions for improvements that enhance the project's clarity or at least maintain its current level of complexity, please feel free to create an issue in the project repository.

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE). Feel free to use it for educational purposes, but be aware that it comes with no warranties or guarantees.
