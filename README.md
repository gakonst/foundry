# <h1 align="center"> foundry </h1>

![Github Actions](https://github.com/gakonst/foundry/workflows/Tests/badge.svg)
[![Telegram Chat][tg-badge]][tg-url]
[![Crates.io][crates-badge]][crates-url]

[crates-badge]: https://img.shields.io/crates/v/foundry.svg
[crates-url]: https://crates.io/crates/foundry-rs
[tg-badge]: https://img.shields.io/endpoint?color=neon&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Ffoundry_rs
[tg-url]: https://t.me/foundry_rs

**Foundry is a blazing fast, portable and modular toolkit for Ethereum application development written in Rust.**

Foundry consists of:

- [**Forge**](./forge): Ethereum testing framework (like Truffle, Hardhat and Dapptools).
- [**Cast**](./cast): Swiss army knife for interacting with EVM smart contracts, sending transactions and getting chain
  data.

![demo](./assets/demo.svg)

## Installation

First run the command below to get `foundryup`, the Foundry toolchain installer:

```
curl -L https://foundry.paradigm.xyz | bash
```

Then in a new terminal session or after reloading your PATH, run it to get the latest `forge` and `cast` binaries:

```
foundryup
```

Advanced ways to use `foundryup` and other documentation can be found in the [foundryup package](./foundryup/README.md).
Happy forging!

### Libusb error when running forge/cast

If you are using the binaries as released, you may see the following error on MacOS:

```
dyld: Library not loaded: /usr/local/opt/libusb/lib/libusb-1.0.0.dylib
```

In order to fix this, you must install `libusb` like so:

```
brew install libusb 
```

## Forge

More documentation can be found in the [forge package](./forge/README.md) and in the [CLI README](./cli/README.md).

### Features

1. Fast & flexible compilation pipeline:
    1. Automatic Solidity compiler version detection & installation (under
       `~/.svm`)
    1. Incremental compilation & caching: Only changed files are re-compiled
    1. Parallel compilation
    1. Non-standard directory structures support (e.g. can build
       [Hardhat repos](https://twitter.com/gakonst/status/1461289225337421829))
1. Tests are written in Solidity (like in DappTools)
1. Fast fuzz Tests with shrinking of inputs & printing of counter-examples
1. Fast remote RPC forking mode leveraging Rust's async infrastructure like tokio
1. Flexible debug logging:
    1. Dapptools-style, using `DsTest`'s emitted logs
    1. Hardhat-style, using the popular `console.sol` contract
1. Portable (5-10MB) & easy to install statically linked binary without requiring Nix or any other package manager
1. Abstracted over EVM implementations (currently supported: Sputnik, EvmOdin)

### How Fast?

Forge is quite fast at both compiling (leveraging the
[ethers-solc](https://github.com/gakonst/ethers-rs/tree/master/ethers-solc/)
package) and testing.

Some benchmarks below:

| Project                                             | Forge | DappTools | Speedup |
| --------------------------------------------------- | ----- | --------- | ------- |
| [guni-lev](https://github.com/hexonaut/guni-lev/)   | 28.6s | 2m36s     | 5.45x   |
| [solmate](https://github.com/Rari-Capital/solmate/) | 6s    | 46s       | 7.66x   |
| [geb](https://github.com/reflexer-labs/geb)         | 11s   | 40s       | 3.63x   |
| [vaults](https://github.com/rari-capital/vaults)    | 1.4s  | 5.5s      | 3.9x    |

It also works with "non-standard" directory structures (i.e. contracts not in
`src/`, libraries not in `lib/`). When
[tested](https://twitter.com/gakonst/status/1461289225337421829) with
[`openzeppelin-contracts`](https://github.com/OpenZeppelin/openzeppelin-contracts), Hardhat compilation took 15.244s,
whereas Forge took 9.449 (~4s cached)

## Cast

Cast is a swiss army knife for interacting with Ethereum applications from the command line.

More documentation can be found in the [cast package](./cast/README.md).

## Setup

### Configuring foundry/forge

foundry is designed to be very configurable. You can create a TOML file called [`foundry.toml`](./config/README.md)
place it in the project or any other parent directory, and it will apply the options in that file. See [_Config
Readme_](./config/README.md#all-options) for all available options.

Configurations can be arbitrarily namespaced by profiles. Foundry's default configuration is also named `default`. The
selected profile is the value of the `FOUNDRY_PROFILE` environment variable, or if it is not set, "default".
`FOUNDRY_` or `DAPP_` prefixed environment variables, like `FOUNDRY_SRC` take precedence, [see _Default
Profile_](./config/README.md#default-profile)

`forge init` creates a basic, extendable `foundry.toml` file.

To set all `.dapprc` env vars run `source .dapprc` beforehand.

To see all currently set options run `forge config`, to only see the basic options (as set with `forge init`)
run `forge config --basic`, this can be used to create a new `foundry.toml` file
with `forge config --basic > foundry.toml`. By default `forge config` shows the currently selected foundry profile and
its values. It also accepts the same arguments as `forge build`.

### VSCode

[juanfranblanco/vscode-solidity](https://github.com/juanfranblanco/vscode-solidity) describes in detail how to configure
the `vscode` extension.

If you're using dependency libraries, then you'll need [_
Remappings_](https://github.com/juanfranblanco/vscode-solidity#remappings) so the extension can find the imports.

The easiest way to add remappings is creating a `remappings.txt` file in the root folder, which can be generated with
auto inferred remappings:

```shell
forge remappings > remappings.txt
```

See also [./cli/README.md](./cli/README.md#Remappings).

Alternatively you can use the extension's dir structure settings to configure your contracts and dependency directory.
If your contracts are stored in `./src` and libraries in `./lib`, you can add

```json
"solidity.packageDefaultDependenciesContractsDirectory": "src",
"solidity.packageDefaultDependenciesDirectory": "lib"
```

to your `.vscode` file

It's also recommended to specify a solc compiler version for the
extension, [read more](https://github.com/juanfranblanco/vscode-solidity#remote-download):

```json
"solidity.compileUsingRemoteVersion": "v0.8.10"
```

## Autocompletion

You can generate autocompletion shell scripts for bash, elvish, fish, powershell, and zsh.

Example (zsh / [oh-my-zsh](https://ohmyz.sh/))

```shell
mkdir -p ~/.oh-my-zsh/completions
forge completions zsh > ~/.oh-my-zsh/completions/_forge
cast completions zsh > ~/.oh-my-zsh/completions/_cast
source ~/.zshrc
```

## Contributing

See our [contributing guidelines](./CONTRIBUTING.md).

## Getting Help

First, see if the answer to your question can be found in the API documentation. If the answer is not there, try opening
an
[issue](https://github.com/gakonst/foundry/issues/new) with the question.

To join the Foundry community, you can use our [main telegram](https://t.me/foundry_rs) to chat with us!

To receive help with Foundry, you can use our [support telegram](https://t.me/+pqodMdZCoQQyZGI6) to asky any questions you may have.

## Acknowledgements

- Foundry is a clean-room rewrite of the testing framework
  [dapptools](https://github.com/dapphub/dapptools). None of this would have been possible without the DappHub team's
  work over the years.
- [Matthias Seitz](https://twitter.com/mattsse_): Created
  [ethers-solc](https://github.com/gakonst/ethers-rs/tree/master/ethers-solc/)
  which is the backbone of our compilation pipeline, as well as countless contributions to ethers, in particular
  the `abigen` macros.
- [Rohit Narurkar](https://twitter.com/rohitnarurkar): Created the Rust Solidity version
  manager [svm-rs](https://github.com/roynalnaruto/svm-rs) which we use to auto-detect and manage multiple Solidity
  versions.
- [Brock Elmore](https://twitter.com/brockjelmore): For extending the VM's cheatcodes and implementing
  [structured call tracing](https://github.com/gakonst/foundry/pull/192), a critical feature for debugging smart
  contract calls.
- All the other
  [contributors](https://github.com/gakonst/foundry/graphs/contributors) to the
  [ethers-rs](https://github.com/gakonst/ethers-rs) &
  [foundry](https://github.com/gakonst/foundry) repositories and chatrooms.
