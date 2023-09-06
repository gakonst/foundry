pub use hevm::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod hevm {
    #[allow(deprecated)]
    fn __abi() -> ::ethers_core::abi::Abi {
        ::ethers_core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("accesses"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("accesses"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("activeFork"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("activeFork"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("addr"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("addr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("allowCheatcodes"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("allowCheatcodes"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("assume"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("assume"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("breakpoint"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("breakpoint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("breakpoint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("broadcast"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("broadcast"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("broadcast"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("broadcast"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("chainId"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("chainId"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("clearMockedCalls"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("clearMockedCalls"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("closeFile"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("closeFile"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("coinbase"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("coinbase"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("copyFile"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("copyFile"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createDir"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createDir"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createFork"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createSelectFork"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createSelectFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createSelectFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createSelectFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createWallet"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createWallet"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createWallet"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createWallet"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("deal"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("deal"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("deriveKey"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("deriveKey"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("deriveKey"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("deriveKey"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("deriveKey"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("difficulty"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("difficulty"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envAddress"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envAddress"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envAddress"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envBool"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envBool"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envBool"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envBytes"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envBytes"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envBytes"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envBytes32"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envBytes32"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envBytes32"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envInt"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envInt"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envInt"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envOr"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envOr"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envString"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("envUint"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envUint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("envUint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("etch"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("etch"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("eth_getLogs"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("eth_getLogs"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers_core::abi::ethabi::ParamType::Address,
                                                    ::ethers_core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                        ),
                                                    ),
                                                    ::ethers_core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                    ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                    ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers_core::abi::ethabi::ParamType::Bool,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("exists"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("exists"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("expectCall"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("expectCallMinGas"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCallMinGas"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectCallMinGas"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("expectEmit"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectEmit"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectEmit"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectEmit"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectEmit"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("expectRevert"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectRevert"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectRevert"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectRevert"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("expectSafeMemory"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("expectSafeMemory"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("expectSafeMemoryCall"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "expectSafeMemoryCall",
                            ),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("fee"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("fee"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ffi"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("ffi"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("fsMetadata"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("fsMetadata"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCode"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getCode"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getDeployedCode"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getDeployedCode"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getDeployedCode"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getLabel"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getLabel"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getMappingKeyAndParentOf"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getMappingKeyAndParentOf",
                            ),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getMappingLength"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getMappingLength"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getMappingSlotAt"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getMappingSlotAt"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getNonce"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getNonce"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getNonce"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getRecordedLogs"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getRecordedLogs"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers_core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                        ),
                                                    ),
                                                    ::ethers_core::abi::ethabi::ParamType::Bytes,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("isDir"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("isDir"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("isFile"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("isFile"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("isPersistent"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("isPersistent"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("keyExists"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("keyExists"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("label"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("label"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("load"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("load"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("makePersistent"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("makePersistent"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("makePersistent"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("makePersistent"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("makePersistent"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("mockCall"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("mockCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("mockCall"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("mockCallRevert"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("mockCallRevert"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("mockCallRevert"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("openFile"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("openFile"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseAddress"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseAddress"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseBool"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseBool"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseBytes"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseBytes"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseBytes32"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseBytes32"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseInt"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseInt"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJson"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJson"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJson"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonAddress"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonAddress"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonAddressArray"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "parseJsonAddressArray",
                            ),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonBool"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonBool"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonBoolArray"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonBoolArray"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonBytes"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonBytes"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonBytes32"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonBytes32"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonBytes32Array"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "parseJsonBytes32Array",
                            ),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonBytesArray"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "parseJsonBytesArray",
                            ),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonInt"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonInt"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonIntArray"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonIntArray"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonKeys"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonKeys"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonString"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonStringArray"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "parseJsonStringArray",
                            ),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonUint"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonUint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseJsonUintArray"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseJsonUintArray"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("parseUint"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("parseUint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("pauseGasMetering"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("pauseGasMetering"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("prank"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("prank"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("prank"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("prevrandao"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("prevrandao"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("projectRoot"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("projectRoot"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("readCallers"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readCallers"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("readDir"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readDir"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                    ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers_core::abi::ethabi::ParamType::Bool,
                                                    ::ethers_core::abi::ethabi::ParamType::Bool,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readDir"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                    ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers_core::abi::ethabi::ParamType::Bool,
                                                    ::ethers_core::abi::ethabi::ParamType::Bool,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readDir"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                    ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers_core::abi::ethabi::ParamType::Bool,
                                                    ::ethers_core::abi::ethabi::ParamType::Bool,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("readFile"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readFile"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("readFileBinary"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readFileBinary"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("readLine"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readLine"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("readLink"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("readLink"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("record"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("record"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("recordLogs"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("recordLogs"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rememberKey"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rememberKey"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("removeDir"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("removeDir"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("removeFile"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("removeFile"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("resetNonce"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("resetNonce"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("resumeGasMetering"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("resumeGasMetering"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("revertTo"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("revertTo"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("revokePersistent"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("revokePersistent"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("revokePersistent"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("roll"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("roll"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rollFork"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rpc"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rpc"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rpcUrl"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rpcUrl"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rpcUrlStructs"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rpcUrlStructs"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rpcUrls"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rpcUrls"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedArray(
                                                ::std::boxed::Box::new(
                                                    ::ethers_core::abi::ethabi::ParamType::String,
                                                ),
                                                2usize,
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("selectFork"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("selectFork"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("serializeAddress"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeAddress"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeAddress"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("serializeBool"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeBool"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeBool"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bool,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("serializeBytes"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeBytes"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeBytes"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("serializeBytes32"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeBytes32"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeBytes32"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("serializeInt"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeInt"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeInt"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("serializeString"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("serializeUint"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeUint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("serializeUint"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setEnv"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("setEnv"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setNonce"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("setNonce"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setNonceUnsafe"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("setNonceUnsafe"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("sign"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("sign"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("sign"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers_core::abi::ethabi::ParamType::Address,
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("skip"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("skip"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("sleep"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("sleep"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("snapshot"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("snapshot"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("startBroadcast"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("startBroadcast"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("startBroadcast"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("startBroadcast"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("startMappingRecording"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "startMappingRecording",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("startPrank"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("startPrank"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("startPrank"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("stopBroadcast"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("stopBroadcast"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("stopMappingRecording"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "stopMappingRecording",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("stopPrank"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("stopPrank"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("store"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("store"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("toString"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("toString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("toString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("toString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("toString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("toString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("toString"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("transact"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("transact"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("transact"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("tryFfi"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("tryFfi"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers_core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers_core::abi::ethabi::ParamType::Int(32usize),
                                            ::ethers_core::abi::ethabi::ParamType::Bytes,
                                            ::ethers_core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("txGasPrice"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("txGasPrice"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("warp"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("warp"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Uint(256usize),
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("writeFile"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("writeFile"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("writeFileBinary"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("writeFileBinary"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("writeJson"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("writeJson"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("writeJson"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("writeLine"),
                    ::std::vec![
                        ::ethers_core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("writeLine"),
                            inputs: ::std::vec![
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                                ::ethers_core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers_core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::None,
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers_core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
            ]),
            events: ::std::collections::BTreeMap::new(),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed human-readable ABI of the contract.
    pub static HEVM_ABI: ::ethers_contract::Lazy<::ethers_core::abi::Abi> = ::ethers_contract::Lazy::new(
        __abi,
    );
    pub struct HEVM<M>(::ethers_contract::Contract<M>);
    impl<M> ::core::clone::Clone for HEVM<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HEVM<M> {
        type Target = ::ethers_contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HEVM<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HEVM<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(HEVM)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers_providers::Middleware> HEVM<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers_core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers_contract::Contract::new(
                    address.into(),
                    HEVM_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `accesses` (0x65bc9481) function
        pub fn accesses(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            (::std::vec::Vec<[u8; 32]>, ::std::vec::Vec<[u8; 32]>),
        > {
            self.0
                .method_hash([101, 188, 148, 129], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `activeFork` (0x2f103f22) function
        pub fn active_fork(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([47, 16, 63, 34], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `addr` (0xffa18649) function
        pub fn addr(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::ethers_core::types::Address,
        > {
            self.0
                .method_hash([255, 161, 134, 73], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `allowCheatcodes` (0xea060291) function
        pub fn allow_cheatcodes(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([234, 6, 2, 145], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `assume` (0x4c63e562) function
        pub fn assume(
            &self,
            p0: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([76, 99, 229, 98], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `breakpoint` (0xf0259e92) function
        pub fn breakpoint_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([240, 37, 158, 146], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `breakpoint` (0xf7d39a8d) function
        pub fn breakpoint_1(
            &self,
            p0: ::std::string::String,
            p1: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([247, 211, 154, 141], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `broadcast` (0xafc98040) function
        pub fn broadcast_0(&self) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([175, 201, 128, 64], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `broadcast` (0xe6962cdb) function
        pub fn broadcast_1(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([230, 150, 44, 219], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `broadcast` (0xf67a965b) function
        pub fn broadcast_2(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([246, 122, 150, 91], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `chainId` (0x4049ddd2) function
        pub fn chain_id(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([64, 73, 221, 210], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `clearMockedCalls` (0x3fdf4e15) function
        pub fn clear_mocked_calls(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([63, 223, 78, 21], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `closeFile` (0x48c3241f) function
        pub fn close_file(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([72, 195, 36, 31], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `coinbase` (0xff483c54) function
        pub fn coinbase(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([255, 72, 60, 84], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `copyFile` (0xa54a87d8) function
        pub fn copy_file(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([165, 74, 135, 216], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createDir` (0x168b64d3) function
        pub fn create_dir(
            &self,
            p0: ::std::string::String,
            p1: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([22, 139, 100, 211], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createFork` (0x6ba3ba2b) function
        pub fn create_fork_1(
            &self,
            p0: ::std::string::String,
            p1: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([107, 163, 186, 43], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createFork` (0x7ca29682) function
        pub fn create_fork_2(
            &self,
            p0: ::std::string::String,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([124, 162, 150, 130], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createFork` (0x31ba3498) function
        pub fn create_fork_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([49, 186, 52, 152], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createSelectFork` (0x71ee464d) function
        pub fn create_select_fork_1(
            &self,
            p0: ::std::string::String,
            p1: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([113, 238, 70, 77], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createSelectFork` (0x84d52b7a) function
        pub fn create_select_fork_2(
            &self,
            p0: ::std::string::String,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([132, 213, 43, 122], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createSelectFork` (0x98680034) function
        pub fn create_select_fork_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([152, 104, 0, 52], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createWallet` (0x7404f1d2) function
        pub fn create_wallet_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            (
                ::ethers_core::types::Address,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
            ),
        > {
            self.0
                .method_hash([116, 4, 241, 210], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createWallet` (0x7a675bb6) function
        pub fn create_wallet_1(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            (
                ::ethers_core::types::Address,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
            ),
        > {
            self.0
                .method_hash([122, 103, 91, 182], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createWallet` (0xed7c5462) function
        pub fn create_wallet_2(
            &self,
            p0: ::ethers_core::types::U256,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            (
                ::ethers_core::types::Address,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
            ),
        > {
            self.0
                .method_hash([237, 124, 84, 98], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `deal` (0xc88a5e6d) function
        pub fn deal(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([200, 138, 94, 109], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `deriveKey` (0x6229498b) function
        pub fn derive_key_0(
            &self,
            p0: ::std::string::String,
            p1: u32,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([98, 41, 73, 139], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `deriveKey` (0x6bcb2c1b) function
        pub fn derive_key_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: u32,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([107, 203, 44, 27], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `deriveKey` (0x32c8176d) function
        pub fn derive_key_2(
            &self,
            p0: ::std::string::String,
            p1: u32,
            p2: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([50, 200, 23, 109], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `deriveKey` (0x29233b1f) function
        pub fn derive_key_3(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: u32,
            p3: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([41, 35, 59, 31], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `difficulty` (0x46cc92d9) function
        pub fn difficulty(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([70, 204, 146, 217], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envAddress` (0x350d56bf) function
        pub fn env_address_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::ethers_core::types::Address,
        > {
            self.0
                .method_hash([53, 13, 86, 191], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envAddress` (0xad31b9fa) function
        pub fn env_address_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::Address>,
        > {
            self.0
                .method_hash([173, 49, 185, 250], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envBool` (0x7ed1ec7d) function
        pub fn env_bool_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([126, 209, 236, 125], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envBool` (0xaaaddeaf) function
        pub fn env_bool_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::vec::Vec<bool>> {
            self.0
                .method_hash([170, 173, 222, 175], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envBytes` (0x4d7baf06) function
        pub fn env_bytes_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([77, 123, 175, 6], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envBytes` (0xddc2651b) function
        pub fn env_bytes_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::Bytes>,
        > {
            self.0
                .method_hash([221, 194, 101, 27], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envBytes32` (0x97949042) function
        pub fn env_bytes_320(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([151, 148, 144, 66], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envBytes32` (0x5af231c1) function
        pub fn env_bytes_321(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 32]>> {
            self.0
                .method_hash([90, 242, 49, 193], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envInt` (0x892a0c61) function
        pub fn env_int_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::I256> {
            self.0
                .method_hash([137, 42, 12, 97], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envInt` (0x42181150) function
        pub fn env_int_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::I256>,
        > {
            self.0
                .method_hash([66, 24, 17, 80], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x4777f3cf) function
        pub fn env_or_0(
            &self,
            p0: ::std::string::String,
            p1: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([71, 119, 243, 207], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x5e97348f) function
        pub fn env_or_1(
            &self,
            p0: ::std::string::String,
            p1: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([94, 151, 52, 143], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0xbbcb713e) function
        pub fn env_or_2(
            &self,
            p0: ::std::string::String,
            p1: ::ethers_core::types::I256,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::I256> {
            self.0
                .method_hash([187, 203, 113, 62], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x561fe540) function
        pub fn env_or_3(
            &self,
            p0: ::std::string::String,
            p1: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::ethers_core::types::Address,
        > {
            self.0
                .method_hash([86, 31, 229, 64], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0xb4a85892) function
        pub fn env_or_4(
            &self,
            p0: ::std::string::String,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([180, 168, 88, 146], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0xd145736c) function
        pub fn env_or_5(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([209, 69, 115, 108], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0xb3e47705) function
        pub fn env_or_6(
            &self,
            p0: ::std::string::String,
            p1: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([179, 228, 119, 5], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0xeb85e83b) function
        pub fn env_or_7(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<bool>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::vec::Vec<bool>> {
            self.0
                .method_hash([235, 133, 232, 59], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x74318528) function
        pub fn env_or_8(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::U256>,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::U256>,
        > {
            self.0
                .method_hash([116, 49, 133, 40], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x4700d74b) function
        pub fn env_or_9(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::I256>,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::I256>,
        > {
            self.0
                .method_hash([71, 0, 215, 75], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0xc74e9deb) function
        pub fn env_or_10(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::Address>,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::Address>,
        > {
            self.0
                .method_hash([199, 78, 157, 235], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x2281f367) function
        pub fn env_or_11(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<[u8; 32]>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 32]>> {
            self.0
                .method_hash([34, 129, 243, 103], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x859216bc) function
        pub fn env_or_12(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::std::string::String>,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::std::string::String>,
        > {
            self.0
                .method_hash([133, 146, 22, 188], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envOr` (0x64bc3e64) function
        pub fn env_or_13(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::Bytes>,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::Bytes>,
        > {
            self.0
                .method_hash([100, 188, 62, 100], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envString` (0xf877cb19) function
        pub fn env_string_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([248, 119, 203, 25], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envString` (0x14b02bc9) function
        pub fn env_string_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::std::string::String>,
        > {
            self.0
                .method_hash([20, 176, 43, 201], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envUint` (0xc1978d1f) function
        pub fn env_uint_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([193, 151, 141, 31], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `envUint` (0xf3dec099) function
        pub fn env_uint_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::U256>,
        > {
            self.0
                .method_hash([243, 222, 192, 153], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `etch` (0xb4d6c782) function
        pub fn etch(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([180, 214, 199, 130], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `eth_getLogs` (0x35e1349b) function
        pub fn eth_get_logs(
            &self,
            p0: ::ethers_core::types::U256,
            p1: ::ethers_core::types::U256,
            p2: ::ethers_core::types::Address,
            p3: ::std::vec::Vec<[u8; 32]>,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<
                (
                    ::ethers_core::types::Address,
                    ::std::vec::Vec<[u8; 32]>,
                    ::ethers_core::types::Bytes,
                    ::ethers_core::types::U256,
                    [u8; 32],
                    ::ethers_core::types::U256,
                    [u8; 32],
                    ::ethers_core::types::U256,
                    bool,
                ),
            >,
        > {
            self.0
                .method_hash([53, 225, 52, 155], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `exists` (0x261a323e) function
        pub fn exists(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([38, 26, 50, 62], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCall` (0xbd6af434) function
        pub fn expect_call_0(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([189, 106, 244, 52], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCall` (0xc1adbbff) function
        pub fn expect_call_1(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Bytes,
            p2: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([193, 173, 187, 255], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCall` (0xf30c7ba3) function
        pub fn expect_call_2(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([243, 12, 123, 163], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCall` (0xa2b1a1ae) function
        pub fn expect_call_3(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: ::ethers_core::types::Bytes,
            p3: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([162, 177, 161, 174], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCall` (0x23361207) function
        pub fn expect_call_4(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: u64,
            p3: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([35, 54, 18, 7], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCall` (0x65b7b7cc) function
        pub fn expect_call_5(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: u64,
            p3: ::ethers_core::types::Bytes,
            p4: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([101, 183, 183, 204], (p0, p1, p2, p3, p4))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCallMinGas` (0x08e4e116) function
        pub fn expect_call_min_gas_0(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: u64,
            p3: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([8, 228, 225, 22], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectCallMinGas` (0xe13a1834) function
        pub fn expect_call_min_gas_1(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: u64,
            p3: ::ethers_core::types::Bytes,
            p4: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([225, 58, 24, 52], (p0, p1, p2, p3, p4))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectEmit` (0x440ed10d) function
        pub fn expect_emit_0(&self) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([68, 14, 209, 13], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectEmit` (0x86b9620d) function
        pub fn expect_emit_1(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([134, 185, 98, 13], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectEmit` (0x491cc7c2) function
        pub fn expect_emit_2(
            &self,
            p0: bool,
            p1: bool,
            p2: bool,
            p3: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([73, 28, 199, 194], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectEmit` (0x81bad6f3) function
        pub fn expect_emit_3(
            &self,
            p0: bool,
            p1: bool,
            p2: bool,
            p3: bool,
            p4: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([129, 186, 214, 243], (p0, p1, p2, p3, p4))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectRevert` (0xf4844814) function
        pub fn expect_revert_0(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([244, 132, 72, 20], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectRevert` (0xf28dceb3) function
        pub fn expect_revert_1(
            &self,
            p0: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 141, 206, 179], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectRevert` (0xc31eb0e0) function
        pub fn expect_revert_2(
            &self,
            p0: [u8; 4],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([195, 30, 176, 224], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectSafeMemory` (0x6d016688) function
        pub fn expect_safe_memory(
            &self,
            p0: u64,
            p1: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([109, 1, 102, 136], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `expectSafeMemoryCall` (0x05838bf4) function
        pub fn expect_safe_memory_call(
            &self,
            p0: u64,
            p1: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([5, 131, 139, 244], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `fee` (0x39b37ab0) function
        pub fn fee(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([57, 179, 122, 176], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ffi` (0x89160467) function
        pub fn ffi(
            &self,
            p0: ::std::vec::Vec<::std::string::String>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([137, 22, 4, 103], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `fsMetadata` (0xaf368a08) function
        pub fn fs_metadata(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            (
                bool,
                bool,
                ::ethers_core::types::U256,
                bool,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
                ::ethers_core::types::U256,
            ),
        > {
            self.0
                .method_hash([175, 54, 138, 8], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCode` (0x8d1cc925) function
        pub fn get_code(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([141, 28, 201, 37], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getDeployedCode` (0x3ebf73b4) function
        pub fn get_deployed_code_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([62, 191, 115, 180], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getDeployedCode` (0x4b848e77) function
        pub fn get_deployed_code_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::vec::Vec<[u8; 32]>,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([75, 132, 142, 119], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLabel` (0x28a249b0) function
        pub fn get_label(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([40, 162, 73, 176], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getMappingKeyAndParentOf` (0x876e24e6) function
        pub fn get_mapping_key_and_parent_of(
            &self,
            p0: ::ethers_core::types::Address,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([135, 110, 36, 230], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getMappingLength` (0x2f2fd63f) function
        pub fn get_mapping_length(
            &self,
            p0: ::ethers_core::types::Address,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([47, 47, 214, 63], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getMappingSlotAt` (0xebc73ab4) function
        pub fn get_mapping_slot_at(
            &self,
            p0: ::ethers_core::types::Address,
            p1: [u8; 32],
            p2: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([235, 199, 58, 180], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getNonce` (0xa5748aad) function
        pub fn get_nonce_0(
            &self,
            p0: Wallet,
        ) -> ::ethers_contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([165, 116, 138, 173], (p0,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getNonce` (0x2d0335ab) function
        pub fn get_nonce_1(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([45, 3, 53, 171], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRecordedLogs` (0x191553a4) function
        pub fn get_recorded_logs(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<(::std::vec::Vec<[u8; 32]>, ::ethers_core::types::Bytes)>,
        > {
            self.0
                .method_hash([25, 21, 83, 164], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `isDir` (0x7d15d019) function
        pub fn is_dir(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([125, 21, 208, 25], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `isFile` (0xe0eb04d4) function
        pub fn is_file(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([224, 235, 4, 212], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `isPersistent` (0xd92d8efd) function
        pub fn is_persistent(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([217, 45, 142, 253], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `keyExists` (0x528a683c) function
        pub fn key_exists(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([82, 138, 104, 60], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `label` (0xc657c718) function
        pub fn label(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([198, 87, 199, 24], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `load` (0x667f9d70) function
        pub fn load(
            &self,
            p0: ::ethers_core::types::Address,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([102, 127, 157, 112], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `makePersistent` (0x57e22dde) function
        pub fn make_persistent_0(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([87, 226, 45, 222], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `makePersistent` (0x4074e0a8) function
        pub fn make_persistent_2(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([64, 116, 224, 168], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `makePersistent` (0xefb77a75) function
        pub fn make_persistent_3(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Address,
            p2: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([239, 183, 122, 117], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `makePersistent` (0x1d9e269e) function
        pub fn make_persistent_1(
            &self,
            p0: ::std::vec::Vec<::ethers_core::types::Address>,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([29, 158, 38, 158], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `mockCall` (0xb96213e4) function
        pub fn mock_call_0(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Bytes,
            p2: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([185, 98, 19, 228], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `mockCall` (0x81409b91) function
        pub fn mock_call_1(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: ::ethers_core::types::Bytes,
            p3: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([129, 64, 155, 145], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `mockCallRevert` (0xdbaad147) function
        pub fn mock_call_revert_0(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Bytes,
            p2: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([219, 170, 209, 71], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `mockCallRevert` (0xd23cd037) function
        pub fn mock_call_revert_1(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::U256,
            p2: ::ethers_core::types::Bytes,
            p3: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([210, 60, 208, 55], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `openFile` (0x7e0394bc) function
        pub fn open_file(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([126, 3, 148, 188], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseAddress` (0xc6ce059d) function
        pub fn parse_address(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::ethers_core::types::Address,
        > {
            self.0
                .method_hash([198, 206, 5, 157], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseBool` (0x974ef924) function
        pub fn parse_bool(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([151, 78, 249, 36], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseBytes` (0x8f5d232d) function
        pub fn parse_bytes(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([143, 93, 35, 45], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseBytes32` (0x087e6e81) function
        pub fn parse_bytes_32(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([8, 126, 110, 129], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseInt` (0x42346c5e) function
        pub fn parse_int(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::I256> {
            self.0
                .method_hash([66, 52, 108, 94], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJson` (0x6a82600a) function
        pub fn parse_json_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([106, 130, 96, 10], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJson` (0x85940ef1) function
        pub fn parse_json_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([133, 148, 14, 241], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonAddress` (0x1e19e657) function
        pub fn parse_json_address(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::ethers_core::types::Address,
        > {
            self.0
                .method_hash([30, 25, 230, 87], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonAddressArray` (0x2fce7883) function
        pub fn parse_json_address_array(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::Address>,
        > {
            self.0
                .method_hash([47, 206, 120, 131], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonBool` (0x9f86dc91) function
        pub fn parse_json_bool(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([159, 134, 220, 145], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonBoolArray` (0x91f3b94f) function
        pub fn parse_json_bool_array(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::vec::Vec<bool>> {
            self.0
                .method_hash([145, 243, 185, 79], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonBytes` (0xfd921be8) function
        pub fn parse_json_bytes(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([253, 146, 27, 232], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonBytes32` (0x1777e59d) function
        pub fn parse_json_bytes_32(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([23, 119, 229, 157], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonBytes32Array` (0x91c75bc3) function
        pub fn parse_json_bytes_32_array(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 32]>> {
            self.0
                .method_hash([145, 199, 91, 195], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonBytesArray` (0x6631aa99) function
        pub fn parse_json_bytes_array(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::Bytes>,
        > {
            self.0
                .method_hash([102, 49, 170, 153], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonInt` (0x7b048ccd) function
        pub fn parse_json_int(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::I256> {
            self.0
                .method_hash([123, 4, 140, 205], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonIntArray` (0x9983c28a) function
        pub fn parse_json_int_array(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::I256>,
        > {
            self.0
                .method_hash([153, 131, 194, 138], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonKeys` (0x213e4198) function
        pub fn parse_json_keys(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::std::string::String>,
        > {
            self.0
                .method_hash([33, 62, 65, 152], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonString` (0x49c4fac8) function
        pub fn parse_json_string(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([73, 196, 250, 200], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonStringArray` (0x498fdcf4) function
        pub fn parse_json_string_array(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::std::string::String>,
        > {
            self.0
                .method_hash([73, 143, 220, 244], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonUint` (0xaddde2b6) function
        pub fn parse_json_uint(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([173, 221, 226, 182], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseJsonUintArray` (0x522074ab) function
        pub fn parse_json_uint_array(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers_core::types::U256>,
        > {
            self.0
                .method_hash([82, 32, 116, 171], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `parseUint` (0xfa91454d) function
        pub fn parse_uint(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([250, 145, 69, 77], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `pauseGasMetering` (0xd1a5b36f) function
        pub fn pause_gas_metering(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([209, 165, 179, 111], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `prank` (0xca669fa7) function
        pub fn prank_0(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([202, 102, 159, 167], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `prank` (0x47e50cce) function
        pub fn prank_1(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([71, 229, 12, 206], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `prevrandao` (0x3b925549) function
        pub fn prevrandao(
            &self,
            p0: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([59, 146, 85, 73], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `projectRoot` (0xd930a0e6) function
        pub fn project_root(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([217, 48, 160, 230], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readCallers` (0x4ad0bac9) function
        pub fn read_callers(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            (
                ::ethers_core::types::U256,
                ::ethers_core::types::Address,
                ::ethers_core::types::Address,
            ),
        > {
            self.0
                .method_hash([74, 208, 186, 201], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readDir` (0xc4bc59e0) function
        pub fn read_dir_0(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<
                (::std::string::String, ::std::string::String, u64, bool, bool),
            >,
        > {
            self.0
                .method_hash([196, 188, 89, 224], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readDir` (0x1497876c) function
        pub fn read_dir_1(
            &self,
            p0: ::std::string::String,
            p1: u64,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<
                (::std::string::String, ::std::string::String, u64, bool, bool),
            >,
        > {
            self.0
                .method_hash([20, 151, 135, 108], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readDir` (0x8102d70d) function
        pub fn read_dir_2(
            &self,
            p0: ::std::string::String,
            p1: u64,
            p2: bool,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<
                (::std::string::String, ::std::string::String, u64, bool, bool),
            >,
        > {
            self.0
                .method_hash([129, 2, 215, 13], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readFile` (0x60f9bb11) function
        pub fn read_file(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([96, 249, 187, 17], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readFileBinary` (0x16ed7bc4) function
        pub fn read_file_binary(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([22, 237, 123, 196], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readLine` (0x70f55728) function
        pub fn read_line(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([112, 245, 87, 40], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `readLink` (0x9f5684a2) function
        pub fn read_link(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([159, 86, 132, 162], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `record` (0x266cf109) function
        pub fn record(&self) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([38, 108, 241, 9], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `recordLogs` (0x41af2f52) function
        pub fn record_logs(&self) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([65, 175, 47, 82], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rememberKey` (0x22100064) function
        pub fn remember_key(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::ethers_core::types::Address,
        > {
            self.0
                .method_hash([34, 16, 0, 100], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `removeDir` (0x45c62011) function
        pub fn remove_dir(
            &self,
            p0: ::std::string::String,
            p1: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([69, 198, 32, 17], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `removeFile` (0xf1afe04d) function
        pub fn remove_file(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([241, 175, 224, 77], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `resetNonce` (0x1c72346d) function
        pub fn reset_nonce(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([28, 114, 52, 109], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `resumeGasMetering` (0x2bcd50e0) function
        pub fn resume_gas_metering(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([43, 205, 80, 224], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `revertTo` (0x44d7f0a4) function
        pub fn revert_to(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([68, 215, 240, 164], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `revokePersistent` (0x997a0222) function
        pub fn revoke_persistent_0(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([153, 122, 2, 34], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `revokePersistent` (0x3ce969e6) function
        pub fn revoke_persistent_1(
            &self,
            p0: ::std::vec::Vec<::ethers_core::types::Address>,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([60, 233, 105, 230], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `roll` (0x1f7b4f30) function
        pub fn roll(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([31, 123, 79, 48], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollFork` (0xd9bbf3a1) function
        pub fn roll_fork_0(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([217, 187, 243, 161], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollFork` (0x0f29772b) function
        pub fn roll_fork_1(
            &self,
            p0: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([15, 41, 119, 43], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollFork` (0xd74c83a4) function
        pub fn roll_fork_2(
            &self,
            p0: ::ethers_core::types::U256,
            p1: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([215, 76, 131, 164], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollFork` (0xf2830f7b) function
        pub fn roll_fork_3(
            &self,
            p0: ::ethers_core::types::U256,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 131, 15, 123], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rpc` (0x1206c8a8) function
        pub fn rpc(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::Bytes> {
            self.0
                .method_hash([18, 6, 200, 168], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rpcUrl` (0x975a6ce9) function
        pub fn rpc_url(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([151, 90, 108, 233], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rpcUrlStructs` (0x9d2ad72a) function
        pub fn rpc_url_structs(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        > {
            self.0
                .method_hash([157, 42, 215, 42], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rpcUrls` (0xa85a8418) function
        pub fn rpc_urls(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            ::std::vec::Vec<[::std::string::String; 2]>,
        > {
            self.0
                .method_hash([168, 90, 132, 24], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `selectFork` (0x9ebf6827) function
        pub fn select_fork(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([158, 191, 104, 39], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeAddress` (0x972c6062) function
        pub fn serialize_address_0(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([151, 44, 96, 98], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeAddress` (0x1e356e1a) function
        pub fn serialize_address_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::Address>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([30, 53, 110, 26], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeBool` (0xac22e971) function
        pub fn serialize_bool_0(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([172, 34, 233, 113], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeBool` (0x92925aa1) function
        pub fn serialize_bool_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<bool>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([146, 146, 90, 161], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeBytes` (0xf21d52c7) function
        pub fn serialize_bytes_0(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([242, 29, 82, 199], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeBytes` (0x9884b232) function
        pub fn serialize_bytes_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::Bytes>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([152, 132, 178, 50], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeBytes32` (0x2d812b44) function
        pub fn serialize_bytes_320(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([45, 129, 43, 68], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeBytes32` (0x201e43e2) function
        pub fn serialize_bytes_321(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<[u8; 32]>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([32, 30, 67, 226], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeInt` (0x3f33db60) function
        pub fn serialize_int_0(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::ethers_core::types::I256,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([63, 51, 219, 96], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeInt` (0x7676e127) function
        pub fn serialize_int_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::I256>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([118, 118, 225, 39], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeString` (0x88da6d35) function
        pub fn serialize_string_0(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([136, 218, 109, 53], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeString` (0x561cd6f3) function
        pub fn serialize_string_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::std::string::String>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([86, 28, 214, 243], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeUint` (0x129e9002) function
        pub fn serialize_uint_0(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([18, 158, 144, 2], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `serializeUint` (0xfee9a469) function
        pub fn serialize_uint_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::vec::Vec<::ethers_core::types::U256>,
        ) -> ::ethers_contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([254, 233, 164, 105], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setEnv` (0x3d5923ee) function
        pub fn set_env(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([61, 89, 35, 238], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setNonce` (0xf8e18b57) function
        pub fn set_nonce(
            &self,
            p0: ::ethers_core::types::Address,
            p1: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([248, 225, 139, 87], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setNonceUnsafe` (0x9b67b21c) function
        pub fn set_nonce_unsafe(
            &self,
            p0: ::ethers_core::types::Address,
            p1: u64,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([155, 103, 178, 28], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `sign` (0xe341eaa4) function
        pub fn sign_0(
            &self,
            p0: ::ethers_core::types::U256,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, (u8, [u8; 32], [u8; 32])> {
            self.0
                .method_hash([227, 65, 234, 164], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `sign` (0xb25c5a25) function
        pub fn sign_1(
            &self,
            p0: Wallet,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, (u8, [u8; 32], [u8; 32])> {
            self.0
                .method_hash([178, 92, 90, 37], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `skip` (0xdd82d13e) function
        pub fn skip(
            &self,
            p0: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([221, 130, 209, 62], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `sleep` (0xfa9d8713) function
        pub fn sleep(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([250, 157, 135, 19], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `snapshot` (0x9711715a) function
        pub fn snapshot(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ::ethers_core::types::U256> {
            self.0
                .method_hash([151, 17, 113, 90], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `startBroadcast` (0x7fb5297f) function
        pub fn start_broadcast_0(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([127, 181, 41, 127], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `startBroadcast` (0x7fec2a8d) function
        pub fn start_broadcast_1(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([127, 236, 42, 141], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `startBroadcast` (0xce817d47) function
        pub fn start_broadcast_2(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([206, 129, 125, 71], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `startMappingRecording` (0x3e9705c0) function
        pub fn start_mapping_recording(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([62, 151, 5, 192], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `startPrank` (0x06447d56) function
        pub fn start_prank_0(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([6, 68, 125, 86], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `startPrank` (0x45b56078) function
        pub fn start_prank_1(
            &self,
            p0: ::ethers_core::types::Address,
            p1: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([69, 181, 96, 120], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stopBroadcast` (0x76eadd36) function
        pub fn stop_broadcast(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([118, 234, 221, 54], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stopMappingRecording` (0x0d4aae9b) function
        pub fn stop_mapping_recording(
            &self,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([13, 74, 174, 155], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stopPrank` (0x90c5013b) function
        pub fn stop_prank(&self) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([144, 197, 1, 59], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `store` (0x70ca10bb) function
        pub fn store(
            &self,
            p0: ::ethers_core::types::Address,
            p1: [u8; 32],
            p2: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([112, 202, 16, 187], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `toString` (0x71aad10d) function
        pub fn to_string_0(
            &self,
            p0: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 170, 209, 13], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `toString` (0x56ca623e) function
        pub fn to_string_1(
            &self,
            p0: ::ethers_core::types::Address,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([86, 202, 98, 62], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `toString` (0x6900a3ae) function
        pub fn to_string_2(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([105, 0, 163, 174], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `toString` (0xa322c40e) function
        pub fn to_string_3(
            &self,
            p0: ::ethers_core::types::I256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([163, 34, 196, 14], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `toString` (0xb11a19e8) function
        pub fn to_string_4(
            &self,
            p0: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([177, 26, 25, 232], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `toString` (0x71dce7da) function
        pub fn to_string_5(
            &self,
            p0: bool,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 220, 231, 218], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `transact` (0xbe646da1) function
        pub fn transact_0(
            &self,
            p0: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([190, 100, 109, 161], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `transact` (0x4d8abc4b) function
        pub fn transact_1(
            &self,
            p0: ::ethers_core::types::U256,
            p1: [u8; 32],
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([77, 138, 188, 75], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `tryFfi` (0xf45c1ce7) function
        pub fn try_ffi(
            &self,
            p0: ::std::vec::Vec<::std::string::String>,
        ) -> ::ethers_contract::builders::ContractCall<
            M,
            (i32, ::ethers_core::types::Bytes, ::ethers_core::types::Bytes),
        > {
            self.0
                .method_hash([244, 92, 28, 231], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `txGasPrice` (0x48f50c0f) function
        pub fn tx_gas_price(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([72, 245, 12, 15], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `warp` (0xe5d6bf02) function
        pub fn warp(
            &self,
            p0: ::ethers_core::types::U256,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([229, 214, 191, 2], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `writeFile` (0x897e0a97) function
        pub fn write_file(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([137, 126, 10, 151], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `writeFileBinary` (0x1f21fc80) function
        pub fn write_file_binary(
            &self,
            p0: ::std::string::String,
            p1: ::ethers_core::types::Bytes,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([31, 33, 252, 128], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `writeJson` (0xe23cd19f) function
        pub fn write_json_0(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([226, 60, 209, 159], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `writeJson` (0x35d6ad46) function
        pub fn write_json_1(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([53, 214, 173, 70], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `writeLine` (0x619d897f) function
        pub fn write_line(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([97, 157, 137, 127], (p0, p1))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers_providers::Middleware> From<::ethers_contract::Contract<M>>
    for HEVM<M> {
        fn from(contract: ::ethers_contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `accesses` function with signature `accesses(address)` and selector `0x65bc9481`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "accesses", abi = "accesses(address)")]
    pub struct AccessesCall(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `activeFork` function with signature `activeFork()` and selector `0x2f103f22`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "activeFork", abi = "activeFork()")]
    pub struct ActiveForkCall;
    ///Container type for all input parameters for the `addr` function with signature `addr(uint256)` and selector `0xffa18649`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "addr", abi = "addr(uint256)")]
    pub struct AddrCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `allowCheatcodes` function with signature `allowCheatcodes(address)` and selector `0xea060291`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "allowCheatcodes", abi = "allowCheatcodes(address)")]
    pub struct AllowCheatcodesCall(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `assume` function with signature `assume(bool)` and selector `0x4c63e562`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "assume", abi = "assume(bool)")]
    pub struct AssumeCall(pub bool);
    ///Container type for all input parameters for the `breakpoint` function with signature `breakpoint(string)` and selector `0xf0259e92`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "breakpoint", abi = "breakpoint(string)")]
    pub struct Breakpoint0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `breakpoint` function with signature `breakpoint(string,bool)` and selector `0xf7d39a8d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "breakpoint", abi = "breakpoint(string,bool)")]
    pub struct Breakpoint1Call(pub ::std::string::String, pub bool);
    ///Container type for all input parameters for the `broadcast` function with signature `broadcast()` and selector `0xafc98040`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "broadcast", abi = "broadcast()")]
    pub struct Broadcast0Call;
    ///Container type for all input parameters for the `broadcast` function with signature `broadcast(address)` and selector `0xe6962cdb`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "broadcast", abi = "broadcast(address)")]
    pub struct Broadcast1Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `broadcast` function with signature `broadcast(uint256)` and selector `0xf67a965b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "broadcast", abi = "broadcast(uint256)")]
    pub struct Broadcast2Call(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `chainId` function with signature `chainId(uint256)` and selector `0x4049ddd2`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "chainId", abi = "chainId(uint256)")]
    pub struct ChainIdCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `clearMockedCalls` function with signature `clearMockedCalls()` and selector `0x3fdf4e15`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "clearMockedCalls", abi = "clearMockedCalls()")]
    pub struct ClearMockedCallsCall;
    ///Container type for all input parameters for the `closeFile` function with signature `closeFile(string)` and selector `0x48c3241f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "closeFile", abi = "closeFile(string)")]
    pub struct CloseFileCall(pub ::std::string::String);
    ///Container type for all input parameters for the `coinbase` function with signature `coinbase(address)` and selector `0xff483c54`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "coinbase", abi = "coinbase(address)")]
    pub struct CoinbaseCall(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `copyFile` function with signature `copyFile(string,string)` and selector `0xa54a87d8`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "copyFile", abi = "copyFile(string,string)")]
    pub struct CopyFileCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `createDir` function with signature `createDir(string,bool)` and selector `0x168b64d3`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createDir", abi = "createDir(string,bool)")]
    pub struct CreateDirCall(pub ::std::string::String, pub bool);
    ///Container type for all input parameters for the `createFork` function with signature `createFork(string,uint256)` and selector `0x6ba3ba2b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createFork", abi = "createFork(string,uint256)")]
    pub struct CreateFork1Call(
        pub ::std::string::String,
        pub ::ethers_core::types::U256,
    );
    ///Container type for all input parameters for the `createFork` function with signature `createFork(string,bytes32)` and selector `0x7ca29682`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createFork", abi = "createFork(string,bytes32)")]
    pub struct CreateFork2Call(pub ::std::string::String, pub [u8; 32]);
    ///Container type for all input parameters for the `createFork` function with signature `createFork(string)` and selector `0x31ba3498`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createFork", abi = "createFork(string)")]
    pub struct CreateFork0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `createSelectFork` function with signature `createSelectFork(string,uint256)` and selector `0x71ee464d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createSelectFork", abi = "createSelectFork(string,uint256)")]
    pub struct CreateSelectFork1Call(
        pub ::std::string::String,
        pub ::ethers_core::types::U256,
    );
    ///Container type for all input parameters for the `createSelectFork` function with signature `createSelectFork(string,bytes32)` and selector `0x84d52b7a`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createSelectFork", abi = "createSelectFork(string,bytes32)")]
    pub struct CreateSelectFork2Call(pub ::std::string::String, pub [u8; 32]);
    ///Container type for all input parameters for the `createSelectFork` function with signature `createSelectFork(string)` and selector `0x98680034`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createSelectFork", abi = "createSelectFork(string)")]
    pub struct CreateSelectFork0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `createWallet` function with signature `createWallet(string)` and selector `0x7404f1d2`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createWallet", abi = "createWallet(string)")]
    pub struct CreateWallet0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `createWallet` function with signature `createWallet(uint256)` and selector `0x7a675bb6`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createWallet", abi = "createWallet(uint256)")]
    pub struct CreateWallet1Call(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `createWallet` function with signature `createWallet(uint256,string)` and selector `0xed7c5462`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createWallet", abi = "createWallet(uint256,string)")]
    pub struct CreateWallet2Call(
        pub ::ethers_core::types::U256,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `deal` function with signature `deal(address,uint256)` and selector `0xc88a5e6d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "deal", abi = "deal(address,uint256)")]
    pub struct DealCall(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
    );
    ///Container type for all input parameters for the `deriveKey` function with signature `deriveKey(string,uint32)` and selector `0x6229498b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "deriveKey", abi = "deriveKey(string,uint32)")]
    pub struct DeriveKey0Call(pub ::std::string::String, pub u32);
    ///Container type for all input parameters for the `deriveKey` function with signature `deriveKey(string,string,uint32)` and selector `0x6bcb2c1b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "deriveKey", abi = "deriveKey(string,string,uint32)")]
    pub struct DeriveKey1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub u32,
    );
    ///Container type for all input parameters for the `deriveKey` function with signature `deriveKey(string,uint32,string)` and selector `0x32c8176d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "deriveKey", abi = "deriveKey(string,uint32,string)")]
    pub struct DeriveKey2Call(
        pub ::std::string::String,
        pub u32,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `deriveKey` function with signature `deriveKey(string,string,uint32,string)` and selector `0x29233b1f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "deriveKey", abi = "deriveKey(string,string,uint32,string)")]
    pub struct DeriveKey3Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub u32,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `difficulty` function with signature `difficulty(uint256)` and selector `0x46cc92d9`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "difficulty", abi = "difficulty(uint256)")]
    pub struct DifficultyCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `envAddress` function with signature `envAddress(string)` and selector `0x350d56bf`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envAddress", abi = "envAddress(string)")]
    pub struct EnvAddress0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `envAddress` function with signature `envAddress(string,string)` and selector `0xad31b9fa`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envAddress", abi = "envAddress(string,string)")]
    pub struct EnvAddress1Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `envBool` function with signature `envBool(string)` and selector `0x7ed1ec7d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envBool", abi = "envBool(string)")]
    pub struct EnvBool0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `envBool` function with signature `envBool(string,string)` and selector `0xaaaddeaf`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envBool", abi = "envBool(string,string)")]
    pub struct EnvBool1Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `envBytes` function with signature `envBytes(string)` and selector `0x4d7baf06`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envBytes", abi = "envBytes(string)")]
    pub struct EnvBytes0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `envBytes` function with signature `envBytes(string,string)` and selector `0xddc2651b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envBytes", abi = "envBytes(string,string)")]
    pub struct EnvBytes1Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `envBytes32` function with signature `envBytes32(string)` and selector `0x97949042`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envBytes32", abi = "envBytes32(string)")]
    pub struct EnvBytes320Call(pub ::std::string::String);
    ///Container type for all input parameters for the `envBytes32` function with signature `envBytes32(string,string)` and selector `0x5af231c1`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envBytes32", abi = "envBytes32(string,string)")]
    pub struct EnvBytes321Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `envInt` function with signature `envInt(string)` and selector `0x892a0c61`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envInt", abi = "envInt(string)")]
    pub struct EnvInt0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `envInt` function with signature `envInt(string,string)` and selector `0x42181150`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envInt", abi = "envInt(string,string)")]
    pub struct EnvInt1Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,bool)` and selector `0x4777f3cf`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,bool)")]
    pub struct EnvOr0Call(pub ::std::string::String, pub bool);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,uint256)` and selector `0x5e97348f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,uint256)")]
    pub struct EnvOr1Call(pub ::std::string::String, pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,int256)` and selector `0xbbcb713e`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,int256)")]
    pub struct EnvOr2Call(pub ::std::string::String, pub ::ethers_core::types::I256);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,address)` and selector `0x561fe540`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,address)")]
    pub struct EnvOr3Call(pub ::std::string::String, pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,bytes32)` and selector `0xb4a85892`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,bytes32)")]
    pub struct EnvOr4Call(pub ::std::string::String, pub [u8; 32]);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string)` and selector `0xd145736c`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string)")]
    pub struct EnvOr5Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,bytes)` and selector `0xb3e47705`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,bytes)")]
    pub struct EnvOr6Call(pub ::std::string::String, pub ::ethers_core::types::Bytes);
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string,bool[])` and selector `0xeb85e83b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string,bool[])")]
    pub struct EnvOr7Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<bool>,
    );
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string,uint256[])` and selector `0x74318528`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string,uint256[])")]
    pub struct EnvOr8Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::U256>,
    );
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string,int256[])` and selector `0x4700d74b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string,int256[])")]
    pub struct EnvOr9Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::I256>,
    );
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string,address[])` and selector `0xc74e9deb`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string,address[])")]
    pub struct EnvOr10Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::Address>,
    );
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string,bytes32[])` and selector `0x2281f367`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string,bytes32[])")]
    pub struct EnvOr11Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<[u8; 32]>,
    );
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string,string[])` and selector `0x859216bc`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string,string[])")]
    pub struct EnvOr12Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::std::string::String>,
    );
    ///Container type for all input parameters for the `envOr` function with signature `envOr(string,string,bytes[])` and selector `0x64bc3e64`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envOr", abi = "envOr(string,string,bytes[])")]
    pub struct EnvOr13Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::Bytes>,
    );
    ///Container type for all input parameters for the `envString` function with signature `envString(string)` and selector `0xf877cb19`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envString", abi = "envString(string)")]
    pub struct EnvString0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `envString` function with signature `envString(string,string)` and selector `0x14b02bc9`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envString", abi = "envString(string,string)")]
    pub struct EnvString1Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `envUint` function with signature `envUint(string)` and selector `0xc1978d1f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envUint", abi = "envUint(string)")]
    pub struct EnvUint0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `envUint` function with signature `envUint(string,string)` and selector `0xf3dec099`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "envUint", abi = "envUint(string,string)")]
    pub struct EnvUint1Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `etch` function with signature `etch(address,bytes)` and selector `0xb4d6c782`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "etch", abi = "etch(address,bytes)")]
    pub struct EtchCall(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `eth_getLogs` function with signature `eth_getLogs(uint256,uint256,address,bytes32[])` and selector `0x35e1349b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "eth_getLogs",
        abi = "eth_getLogs(uint256,uint256,address,bytes32[])"
    )]
    pub struct EthGetLogsCall(
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::Address,
        pub ::std::vec::Vec<[u8; 32]>,
    );
    ///Container type for all input parameters for the `exists` function with signature `exists(string)` and selector `0x261a323e`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "exists", abi = "exists(string)")]
    pub struct ExistsCall(pub ::std::string::String);
    ///Container type for all input parameters for the `expectCall` function with signature `expectCall(address,bytes)` and selector `0xbd6af434`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectCall", abi = "expectCall(address,bytes)")]
    pub struct ExpectCall0Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `expectCall` function with signature `expectCall(address,bytes,uint64)` and selector `0xc1adbbff`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectCall", abi = "expectCall(address,bytes,uint64)")]
    pub struct ExpectCall1Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Bytes,
        pub u64,
    );
    ///Container type for all input parameters for the `expectCall` function with signature `expectCall(address,uint256,bytes)` and selector `0xf30c7ba3`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectCall", abi = "expectCall(address,uint256,bytes)")]
    pub struct ExpectCall2Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `expectCall` function with signature `expectCall(address,uint256,bytes,uint64)` and selector `0xa2b1a1ae`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectCall", abi = "expectCall(address,uint256,bytes,uint64)")]
    pub struct ExpectCall3Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::Bytes,
        pub u64,
    );
    ///Container type for all input parameters for the `expectCall` function with signature `expectCall(address,uint256,uint64,bytes)` and selector `0x23361207`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectCall", abi = "expectCall(address,uint256,uint64,bytes)")]
    pub struct ExpectCall4Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub u64,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `expectCall` function with signature `expectCall(address,uint256,uint64,bytes,uint64)` and selector `0x65b7b7cc`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "expectCall",
        abi = "expectCall(address,uint256,uint64,bytes,uint64)"
    )]
    pub struct ExpectCall5Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub u64,
        pub ::ethers_core::types::Bytes,
        pub u64,
    );
    ///Container type for all input parameters for the `expectCallMinGas` function with signature `expectCallMinGas(address,uint256,uint64,bytes)` and selector `0x08e4e116`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "expectCallMinGas",
        abi = "expectCallMinGas(address,uint256,uint64,bytes)"
    )]
    pub struct ExpectCallMinGas0Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub u64,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `expectCallMinGas` function with signature `expectCallMinGas(address,uint256,uint64,bytes,uint64)` and selector `0xe13a1834`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "expectCallMinGas",
        abi = "expectCallMinGas(address,uint256,uint64,bytes,uint64)"
    )]
    pub struct ExpectCallMinGas1Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub u64,
        pub ::ethers_core::types::Bytes,
        pub u64,
    );
    ///Container type for all input parameters for the `expectEmit` function with signature `expectEmit()` and selector `0x440ed10d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectEmit", abi = "expectEmit()")]
    pub struct ExpectEmit0Call;
    ///Container type for all input parameters for the `expectEmit` function with signature `expectEmit(address)` and selector `0x86b9620d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectEmit", abi = "expectEmit(address)")]
    pub struct ExpectEmit1Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `expectEmit` function with signature `expectEmit(bool,bool,bool,bool)` and selector `0x491cc7c2`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectEmit", abi = "expectEmit(bool,bool,bool,bool)")]
    pub struct ExpectEmit2Call(pub bool, pub bool, pub bool, pub bool);
    ///Container type for all input parameters for the `expectEmit` function with signature `expectEmit(bool,bool,bool,bool,address)` and selector `0x81bad6f3`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectEmit", abi = "expectEmit(bool,bool,bool,bool,address)")]
    pub struct ExpectEmit3Call(
        pub bool,
        pub bool,
        pub bool,
        pub bool,
        pub ::ethers_core::types::Address,
    );
    ///Container type for all input parameters for the `expectRevert` function with signature `expectRevert()` and selector `0xf4844814`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectRevert", abi = "expectRevert()")]
    pub struct ExpectRevert0Call;
    ///Container type for all input parameters for the `expectRevert` function with signature `expectRevert(bytes)` and selector `0xf28dceb3`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectRevert", abi = "expectRevert(bytes)")]
    pub struct ExpectRevert1Call(pub ::ethers_core::types::Bytes);
    ///Container type for all input parameters for the `expectRevert` function with signature `expectRevert(bytes4)` and selector `0xc31eb0e0`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectRevert", abi = "expectRevert(bytes4)")]
    pub struct ExpectRevert2Call(pub [u8; 4]);
    ///Container type for all input parameters for the `expectSafeMemory` function with signature `expectSafeMemory(uint64,uint64)` and selector `0x6d016688`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectSafeMemory", abi = "expectSafeMemory(uint64,uint64)")]
    pub struct ExpectSafeMemoryCall(pub u64, pub u64);
    ///Container type for all input parameters for the `expectSafeMemoryCall` function with signature `expectSafeMemoryCall(uint64,uint64)` and selector `0x05838bf4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "expectSafeMemoryCall",
        abi = "expectSafeMemoryCall(uint64,uint64)"
    )]
    pub struct ExpectSafeMemoryCallCall(pub u64, pub u64);
    ///Container type for all input parameters for the `fee` function with signature `fee(uint256)` and selector `0x39b37ab0`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "fee", abi = "fee(uint256)")]
    pub struct FeeCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `ffi` function with signature `ffi(string[])` and selector `0x89160467`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "ffi", abi = "ffi(string[])")]
    pub struct FfiCall(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all input parameters for the `fsMetadata` function with signature `fsMetadata(string)` and selector `0xaf368a08`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "fsMetadata", abi = "fsMetadata(string)")]
    pub struct FsMetadataCall(pub ::std::string::String);
    ///Container type for all input parameters for the `getCode` function with signature `getCode(string)` and selector `0x8d1cc925`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getCode", abi = "getCode(string)")]
    pub struct GetCodeCall(pub ::std::string::String);
    ///Container type for all input parameters for the `getDeployedCode` function with signature `getDeployedCode(string)` and selector `0x3ebf73b4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getDeployedCode", abi = "getDeployedCode(string)")]
    pub struct GetDeployedCode0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `getDeployedCode` function with signature `getDeployedCode(string,bytes32[])` and selector `0x4b848e77`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getDeployedCode", abi = "getDeployedCode(string,bytes32[])")]
    pub struct GetDeployedCode1Call(
        pub ::std::string::String,
        pub ::std::vec::Vec<[u8; 32]>,
    );
    ///Container type for all input parameters for the `getLabel` function with signature `getLabel(address)` and selector `0x28a249b0`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getLabel", abi = "getLabel(address)")]
    pub struct GetLabelCall(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `getMappingKeyAndParentOf` function with signature `getMappingKeyAndParentOf(address,bytes32)` and selector `0x876e24e6`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getMappingKeyAndParentOf",
        abi = "getMappingKeyAndParentOf(address,bytes32)"
    )]
    pub struct GetMappingKeyAndParentOfCall(
        pub ::ethers_core::types::Address,
        pub [u8; 32],
    );
    ///Container type for all input parameters for the `getMappingLength` function with signature `getMappingLength(address,bytes32)` and selector `0x2f2fd63f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getMappingLength", abi = "getMappingLength(address,bytes32)")]
    pub struct GetMappingLengthCall(pub ::ethers_core::types::Address, pub [u8; 32]);
    ///Container type for all input parameters for the `getMappingSlotAt` function with signature `getMappingSlotAt(address,bytes32,uint256)` and selector `0xebc73ab4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getMappingSlotAt",
        abi = "getMappingSlotAt(address,bytes32,uint256)"
    )]
    pub struct GetMappingSlotAtCall(
        pub ::ethers_core::types::Address,
        pub [u8; 32],
        pub ::ethers_core::types::U256,
    );
    ///Container type for all input parameters for the `getNonce` function with signature `getNonce((address,uint256,uint256,uint256))` and selector `0xa5748aad`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getNonce", abi = "getNonce((address,uint256,uint256,uint256))")]
    pub struct GetNonce0Call(pub Wallet);
    ///Container type for all input parameters for the `getNonce` function with signature `getNonce(address)` and selector `0x2d0335ab`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getNonce", abi = "getNonce(address)")]
    pub struct GetNonce1Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `getRecordedLogs` function with signature `getRecordedLogs()` and selector `0x191553a4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getRecordedLogs", abi = "getRecordedLogs()")]
    pub struct GetRecordedLogsCall;
    ///Container type for all input parameters for the `isDir` function with signature `isDir(string)` and selector `0x7d15d019`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "isDir", abi = "isDir(string)")]
    pub struct IsDirCall(pub ::std::string::String);
    ///Container type for all input parameters for the `isFile` function with signature `isFile(string)` and selector `0xe0eb04d4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "isFile", abi = "isFile(string)")]
    pub struct IsFileCall(pub ::std::string::String);
    ///Container type for all input parameters for the `isPersistent` function with signature `isPersistent(address)` and selector `0xd92d8efd`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "isPersistent", abi = "isPersistent(address)")]
    pub struct IsPersistentCall(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `keyExists` function with signature `keyExists(string,string)` and selector `0x528a683c`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "keyExists", abi = "keyExists(string,string)")]
    pub struct KeyExistsCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `label` function with signature `label(address,string)` and selector `0xc657c718`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "label", abi = "label(address,string)")]
    pub struct LabelCall(pub ::ethers_core::types::Address, pub ::std::string::String);
    ///Container type for all input parameters for the `load` function with signature `load(address,bytes32)` and selector `0x667f9d70`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "load", abi = "load(address,bytes32)")]
    pub struct LoadCall(pub ::ethers_core::types::Address, pub [u8; 32]);
    ///Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address)` and selector `0x57e22dde`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "makePersistent", abi = "makePersistent(address)")]
    pub struct MakePersistent0Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address,address)` and selector `0x4074e0a8`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "makePersistent", abi = "makePersistent(address,address)")]
    pub struct MakePersistent2Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Address,
    );
    ///Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address,address,address)` and selector `0xefb77a75`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "makePersistent", abi = "makePersistent(address,address,address)")]
    pub struct MakePersistent3Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Address,
    );
    ///Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address[])` and selector `0x1d9e269e`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "makePersistent", abi = "makePersistent(address[])")]
    pub struct MakePersistent1Call(pub ::std::vec::Vec<::ethers_core::types::Address>);
    ///Container type for all input parameters for the `mockCall` function with signature `mockCall(address,bytes,bytes)` and selector `0xb96213e4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "mockCall", abi = "mockCall(address,bytes,bytes)")]
    pub struct MockCall0Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Bytes,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `mockCall` function with signature `mockCall(address,uint256,bytes,bytes)` and selector `0x81409b91`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "mockCall", abi = "mockCall(address,uint256,bytes,bytes)")]
    pub struct MockCall1Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::Bytes,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `mockCallRevert` function with signature `mockCallRevert(address,bytes,bytes)` and selector `0xdbaad147`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "mockCallRevert", abi = "mockCallRevert(address,bytes,bytes)")]
    pub struct MockCallRevert0Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Bytes,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `mockCallRevert` function with signature `mockCallRevert(address,uint256,bytes,bytes)` and selector `0xd23cd037`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "mockCallRevert",
        abi = "mockCallRevert(address,uint256,bytes,bytes)"
    )]
    pub struct MockCallRevert1Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::Bytes,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `openFile` function with signature `openFile(string)` and selector `0x7e0394bc`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "openFile", abi = "openFile(string)")]
    pub struct OpenFileCall(pub ::std::string::String);
    ///Container type for all input parameters for the `parseAddress` function with signature `parseAddress(string)` and selector `0xc6ce059d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseAddress", abi = "parseAddress(string)")]
    pub struct ParseAddressCall(pub ::std::string::String);
    ///Container type for all input parameters for the `parseBool` function with signature `parseBool(string)` and selector `0x974ef924`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseBool", abi = "parseBool(string)")]
    pub struct ParseBoolCall(pub ::std::string::String);
    ///Container type for all input parameters for the `parseBytes` function with signature `parseBytes(string)` and selector `0x8f5d232d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseBytes", abi = "parseBytes(string)")]
    pub struct ParseBytesCall(pub ::std::string::String);
    ///Container type for all input parameters for the `parseBytes32` function with signature `parseBytes32(string)` and selector `0x087e6e81`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseBytes32", abi = "parseBytes32(string)")]
    pub struct ParseBytes32Call(pub ::std::string::String);
    ///Container type for all input parameters for the `parseInt` function with signature `parseInt(string)` and selector `0x42346c5e`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseInt", abi = "parseInt(string)")]
    pub struct ParseIntCall(pub ::std::string::String);
    ///Container type for all input parameters for the `parseJson` function with signature `parseJson(string)` and selector `0x6a82600a`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJson", abi = "parseJson(string)")]
    pub struct ParseJson0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `parseJson` function with signature `parseJson(string,string)` and selector `0x85940ef1`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJson", abi = "parseJson(string,string)")]
    pub struct ParseJson1Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `parseJsonAddress` function with signature `parseJsonAddress(string,string)` and selector `0x1e19e657`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonAddress", abi = "parseJsonAddress(string,string)")]
    pub struct ParseJsonAddressCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonAddressArray` function with signature `parseJsonAddressArray(string,string)` and selector `0x2fce7883`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "parseJsonAddressArray",
        abi = "parseJsonAddressArray(string,string)"
    )]
    pub struct ParseJsonAddressArrayCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonBool` function with signature `parseJsonBool(string,string)` and selector `0x9f86dc91`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonBool", abi = "parseJsonBool(string,string)")]
    pub struct ParseJsonBoolCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `parseJsonBoolArray` function with signature `parseJsonBoolArray(string,string)` and selector `0x91f3b94f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonBoolArray", abi = "parseJsonBoolArray(string,string)")]
    pub struct ParseJsonBoolArrayCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonBytes` function with signature `parseJsonBytes(string,string)` and selector `0xfd921be8`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonBytes", abi = "parseJsonBytes(string,string)")]
    pub struct ParseJsonBytesCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `parseJsonBytes32` function with signature `parseJsonBytes32(string,string)` and selector `0x1777e59d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonBytes32", abi = "parseJsonBytes32(string,string)")]
    pub struct ParseJsonBytes32Call(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonBytes32Array` function with signature `parseJsonBytes32Array(string,string)` and selector `0x91c75bc3`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "parseJsonBytes32Array",
        abi = "parseJsonBytes32Array(string,string)"
    )]
    pub struct ParseJsonBytes32ArrayCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonBytesArray` function with signature `parseJsonBytesArray(string,string)` and selector `0x6631aa99`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonBytesArray", abi = "parseJsonBytesArray(string,string)")]
    pub struct ParseJsonBytesArrayCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonInt` function with signature `parseJsonInt(string,string)` and selector `0x7b048ccd`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonInt", abi = "parseJsonInt(string,string)")]
    pub struct ParseJsonIntCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `parseJsonIntArray` function with signature `parseJsonIntArray(string,string)` and selector `0x9983c28a`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonIntArray", abi = "parseJsonIntArray(string,string)")]
    pub struct ParseJsonIntArrayCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonKeys` function with signature `parseJsonKeys(string,string)` and selector `0x213e4198`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonKeys", abi = "parseJsonKeys(string,string)")]
    pub struct ParseJsonKeysCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `parseJsonString` function with signature `parseJsonString(string,string)` and selector `0x49c4fac8`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonString", abi = "parseJsonString(string,string)")]
    pub struct ParseJsonStringCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `parseJsonStringArray` function with signature `parseJsonStringArray(string,string)` and selector `0x498fdcf4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "parseJsonStringArray",
        abi = "parseJsonStringArray(string,string)"
    )]
    pub struct ParseJsonStringArrayCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseJsonUint` function with signature `parseJsonUint(string,string)` and selector `0xaddde2b6`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonUint", abi = "parseJsonUint(string,string)")]
    pub struct ParseJsonUintCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `parseJsonUintArray` function with signature `parseJsonUintArray(string,string)` and selector `0x522074ab`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseJsonUintArray", abi = "parseJsonUintArray(string,string)")]
    pub struct ParseJsonUintArrayCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `parseUint` function with signature `parseUint(string)` and selector `0xfa91454d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "parseUint", abi = "parseUint(string)")]
    pub struct ParseUintCall(pub ::std::string::String);
    ///Container type for all input parameters for the `pauseGasMetering` function with signature `pauseGasMetering()` and selector `0xd1a5b36f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "pauseGasMetering", abi = "pauseGasMetering()")]
    pub struct PauseGasMeteringCall;
    ///Container type for all input parameters for the `prank` function with signature `prank(address)` and selector `0xca669fa7`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "prank", abi = "prank(address)")]
    pub struct Prank0Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `prank` function with signature `prank(address,address)` and selector `0x47e50cce`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "prank", abi = "prank(address,address)")]
    pub struct Prank1Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Address,
    );
    ///Container type for all input parameters for the `prevrandao` function with signature `prevrandao(bytes32)` and selector `0x3b925549`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "prevrandao", abi = "prevrandao(bytes32)")]
    pub struct PrevrandaoCall(pub [u8; 32]);
    ///Container type for all input parameters for the `projectRoot` function with signature `projectRoot()` and selector `0xd930a0e6`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "projectRoot", abi = "projectRoot()")]
    pub struct ProjectRootCall;
    ///Container type for all input parameters for the `readCallers` function with signature `readCallers()` and selector `0x4ad0bac9`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readCallers", abi = "readCallers()")]
    pub struct ReadCallersCall;
    ///Container type for all input parameters for the `readDir` function with signature `readDir(string)` and selector `0xc4bc59e0`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readDir", abi = "readDir(string)")]
    pub struct ReadDir0Call(pub ::std::string::String);
    ///Container type for all input parameters for the `readDir` function with signature `readDir(string,uint64)` and selector `0x1497876c`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readDir", abi = "readDir(string,uint64)")]
    pub struct ReadDir1Call(pub ::std::string::String, pub u64);
    ///Container type for all input parameters for the `readDir` function with signature `readDir(string,uint64,bool)` and selector `0x8102d70d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readDir", abi = "readDir(string,uint64,bool)")]
    pub struct ReadDir2Call(pub ::std::string::String, pub u64, pub bool);
    ///Container type for all input parameters for the `readFile` function with signature `readFile(string)` and selector `0x60f9bb11`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readFile", abi = "readFile(string)")]
    pub struct ReadFileCall(pub ::std::string::String);
    ///Container type for all input parameters for the `readFileBinary` function with signature `readFileBinary(string)` and selector `0x16ed7bc4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readFileBinary", abi = "readFileBinary(string)")]
    pub struct ReadFileBinaryCall(pub ::std::string::String);
    ///Container type for all input parameters for the `readLine` function with signature `readLine(string)` and selector `0x70f55728`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readLine", abi = "readLine(string)")]
    pub struct ReadLineCall(pub ::std::string::String);
    ///Container type for all input parameters for the `readLink` function with signature `readLink(string)` and selector `0x9f5684a2`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "readLink", abi = "readLink(string)")]
    pub struct ReadLinkCall(pub ::std::string::String);
    ///Container type for all input parameters for the `record` function with signature `record()` and selector `0x266cf109`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "record", abi = "record()")]
    pub struct RecordCall;
    ///Container type for all input parameters for the `recordLogs` function with signature `recordLogs()` and selector `0x41af2f52`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "recordLogs", abi = "recordLogs()")]
    pub struct RecordLogsCall;
    ///Container type for all input parameters for the `rememberKey` function with signature `rememberKey(uint256)` and selector `0x22100064`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rememberKey", abi = "rememberKey(uint256)")]
    pub struct RememberKeyCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `removeDir` function with signature `removeDir(string,bool)` and selector `0x45c62011`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "removeDir", abi = "removeDir(string,bool)")]
    pub struct RemoveDirCall(pub ::std::string::String, pub bool);
    ///Container type for all input parameters for the `removeFile` function with signature `removeFile(string)` and selector `0xf1afe04d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "removeFile", abi = "removeFile(string)")]
    pub struct RemoveFileCall(pub ::std::string::String);
    ///Container type for all input parameters for the `resetNonce` function with signature `resetNonce(address)` and selector `0x1c72346d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "resetNonce", abi = "resetNonce(address)")]
    pub struct ResetNonceCall(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `resumeGasMetering` function with signature `resumeGasMetering()` and selector `0x2bcd50e0`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "resumeGasMetering", abi = "resumeGasMetering()")]
    pub struct ResumeGasMeteringCall;
    ///Container type for all input parameters for the `revertTo` function with signature `revertTo(uint256)` and selector `0x44d7f0a4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "revertTo", abi = "revertTo(uint256)")]
    pub struct RevertToCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `revokePersistent` function with signature `revokePersistent(address)` and selector `0x997a0222`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "revokePersistent", abi = "revokePersistent(address)")]
    pub struct RevokePersistent0Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `revokePersistent` function with signature `revokePersistent(address[])` and selector `0x3ce969e6`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "revokePersistent", abi = "revokePersistent(address[])")]
    pub struct RevokePersistent1Call(pub ::std::vec::Vec<::ethers_core::types::Address>);
    ///Container type for all input parameters for the `roll` function with signature `roll(uint256)` and selector `0x1f7b4f30`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "roll", abi = "roll(uint256)")]
    pub struct RollCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `rollFork` function with signature `rollFork(uint256)` and selector `0xd9bbf3a1`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(uint256)")]
    pub struct RollFork0Call(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `rollFork` function with signature `rollFork(bytes32)` and selector `0x0f29772b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(bytes32)")]
    pub struct RollFork1Call(pub [u8; 32]);
    ///Container type for all input parameters for the `rollFork` function with signature `rollFork(uint256,uint256)` and selector `0xd74c83a4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(uint256,uint256)")]
    pub struct RollFork2Call(
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::U256,
    );
    ///Container type for all input parameters for the `rollFork` function with signature `rollFork(uint256,bytes32)` and selector `0xf2830f7b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(uint256,bytes32)")]
    pub struct RollFork3Call(pub ::ethers_core::types::U256, pub [u8; 32]);
    ///Container type for all input parameters for the `rpc` function with signature `rpc(string,string)` and selector `0x1206c8a8`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rpc", abi = "rpc(string,string)")]
    pub struct RpcCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `rpcUrl` function with signature `rpcUrl(string)` and selector `0x975a6ce9`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rpcUrl", abi = "rpcUrl(string)")]
    pub struct RpcUrlCall(pub ::std::string::String);
    ///Container type for all input parameters for the `rpcUrlStructs` function with signature `rpcUrlStructs()` and selector `0x9d2ad72a`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rpcUrlStructs", abi = "rpcUrlStructs()")]
    pub struct RpcUrlStructsCall;
    ///Container type for all input parameters for the `rpcUrls` function with signature `rpcUrls()` and selector `0xa85a8418`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rpcUrls", abi = "rpcUrls()")]
    pub struct RpcUrlsCall;
    ///Container type for all input parameters for the `selectFork` function with signature `selectFork(uint256)` and selector `0x9ebf6827`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "selectFork", abi = "selectFork(uint256)")]
    pub struct SelectForkCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `serializeAddress` function with signature `serializeAddress(string,string,address)` and selector `0x972c6062`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "serializeAddress",
        abi = "serializeAddress(string,string,address)"
    )]
    pub struct SerializeAddress0Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::ethers_core::types::Address,
    );
    ///Container type for all input parameters for the `serializeAddress` function with signature `serializeAddress(string,string,address[])` and selector `0x1e356e1a`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "serializeAddress",
        abi = "serializeAddress(string,string,address[])"
    )]
    pub struct SerializeAddress1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::Address>,
    );
    ///Container type for all input parameters for the `serializeBool` function with signature `serializeBool(string,string,bool)` and selector `0xac22e971`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeBool", abi = "serializeBool(string,string,bool)")]
    pub struct SerializeBool0Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub bool,
    );
    ///Container type for all input parameters for the `serializeBool` function with signature `serializeBool(string,string,bool[])` and selector `0x92925aa1`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeBool", abi = "serializeBool(string,string,bool[])")]
    pub struct SerializeBool1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<bool>,
    );
    ///Container type for all input parameters for the `serializeBytes` function with signature `serializeBytes(string,string,bytes)` and selector `0xf21d52c7`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeBytes", abi = "serializeBytes(string,string,bytes)")]
    pub struct SerializeBytes0Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `serializeBytes` function with signature `serializeBytes(string,string,bytes[])` and selector `0x9884b232`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeBytes", abi = "serializeBytes(string,string,bytes[])")]
    pub struct SerializeBytes1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::Bytes>,
    );
    ///Container type for all input parameters for the `serializeBytes32` function with signature `serializeBytes32(string,string,bytes32)` and selector `0x2d812b44`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "serializeBytes32",
        abi = "serializeBytes32(string,string,bytes32)"
    )]
    pub struct SerializeBytes320Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub [u8; 32],
    );
    ///Container type for all input parameters for the `serializeBytes32` function with signature `serializeBytes32(string,string,bytes32[])` and selector `0x201e43e2`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "serializeBytes32",
        abi = "serializeBytes32(string,string,bytes32[])"
    )]
    pub struct SerializeBytes321Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<[u8; 32]>,
    );
    ///Container type for all input parameters for the `serializeInt` function with signature `serializeInt(string,string,int256)` and selector `0x3f33db60`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeInt", abi = "serializeInt(string,string,int256)")]
    pub struct SerializeInt0Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::ethers_core::types::I256,
    );
    ///Container type for all input parameters for the `serializeInt` function with signature `serializeInt(string,string,int256[])` and selector `0x7676e127`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeInt", abi = "serializeInt(string,string,int256[])")]
    pub struct SerializeInt1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::I256>,
    );
    ///Container type for all input parameters for the `serializeString` function with signature `serializeString(string,string,string)` and selector `0x88da6d35`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeString", abi = "serializeString(string,string,string)")]
    pub struct SerializeString0Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `serializeString` function with signature `serializeString(string,string,string[])` and selector `0x561cd6f3`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeString", abi = "serializeString(string,string,string[])")]
    pub struct SerializeString1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::std::string::String>,
    );
    ///Container type for all input parameters for the `serializeUint` function with signature `serializeUint(string,string,uint256)` and selector `0x129e9002`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeUint", abi = "serializeUint(string,string,uint256)")]
    pub struct SerializeUint0Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::ethers_core::types::U256,
    );
    ///Container type for all input parameters for the `serializeUint` function with signature `serializeUint(string,string,uint256[])` and selector `0xfee9a469`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "serializeUint", abi = "serializeUint(string,string,uint256[])")]
    pub struct SerializeUint1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::vec::Vec<::ethers_core::types::U256>,
    );
    ///Container type for all input parameters for the `setEnv` function with signature `setEnv(string,string)` and selector `0x3d5923ee`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setEnv", abi = "setEnv(string,string)")]
    pub struct SetEnvCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `setNonce` function with signature `setNonce(address,uint64)` and selector `0xf8e18b57`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setNonce", abi = "setNonce(address,uint64)")]
    pub struct SetNonceCall(pub ::ethers_core::types::Address, pub u64);
    ///Container type for all input parameters for the `setNonceUnsafe` function with signature `setNonceUnsafe(address,uint64)` and selector `0x9b67b21c`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setNonceUnsafe", abi = "setNonceUnsafe(address,uint64)")]
    pub struct SetNonceUnsafeCall(pub ::ethers_core::types::Address, pub u64);
    ///Container type for all input parameters for the `sign` function with signature `sign(uint256,bytes32)` and selector `0xe341eaa4`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "sign", abi = "sign(uint256,bytes32)")]
    pub struct Sign0Call(pub ::ethers_core::types::U256, pub [u8; 32]);
    ///Container type for all input parameters for the `sign` function with signature `sign((address,uint256,uint256,uint256),bytes32)` and selector `0xb25c5a25`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "sign", abi = "sign((address,uint256,uint256,uint256),bytes32)")]
    pub struct Sign1Call(pub Wallet, pub [u8; 32]);
    ///Container type for all input parameters for the `skip` function with signature `skip(bool)` and selector `0xdd82d13e`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "skip", abi = "skip(bool)")]
    pub struct SkipCall(pub bool);
    ///Container type for all input parameters for the `sleep` function with signature `sleep(uint256)` and selector `0xfa9d8713`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "sleep", abi = "sleep(uint256)")]
    pub struct SleepCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `snapshot` function with signature `snapshot()` and selector `0x9711715a`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "snapshot", abi = "snapshot()")]
    pub struct SnapshotCall;
    ///Container type for all input parameters for the `startBroadcast` function with signature `startBroadcast()` and selector `0x7fb5297f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "startBroadcast", abi = "startBroadcast()")]
    pub struct StartBroadcast0Call;
    ///Container type for all input parameters for the `startBroadcast` function with signature `startBroadcast(address)` and selector `0x7fec2a8d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "startBroadcast", abi = "startBroadcast(address)")]
    pub struct StartBroadcast1Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `startBroadcast` function with signature `startBroadcast(uint256)` and selector `0xce817d47`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "startBroadcast", abi = "startBroadcast(uint256)")]
    pub struct StartBroadcast2Call(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `startMappingRecording` function with signature `startMappingRecording()` and selector `0x3e9705c0`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "startMappingRecording", abi = "startMappingRecording()")]
    pub struct StartMappingRecordingCall;
    ///Container type for all input parameters for the `startPrank` function with signature `startPrank(address)` and selector `0x06447d56`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "startPrank", abi = "startPrank(address)")]
    pub struct StartPrank0Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `startPrank` function with signature `startPrank(address,address)` and selector `0x45b56078`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "startPrank", abi = "startPrank(address,address)")]
    pub struct StartPrank1Call(
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Address,
    );
    ///Container type for all input parameters for the `stopBroadcast` function with signature `stopBroadcast()` and selector `0x76eadd36`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "stopBroadcast", abi = "stopBroadcast()")]
    pub struct StopBroadcastCall;
    ///Container type for all input parameters for the `stopMappingRecording` function with signature `stopMappingRecording()` and selector `0x0d4aae9b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "stopMappingRecording", abi = "stopMappingRecording()")]
    pub struct StopMappingRecordingCall;
    ///Container type for all input parameters for the `stopPrank` function with signature `stopPrank()` and selector `0x90c5013b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "stopPrank", abi = "stopPrank()")]
    pub struct StopPrankCall;
    ///Container type for all input parameters for the `store` function with signature `store(address,bytes32,bytes32)` and selector `0x70ca10bb`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "store", abi = "store(address,bytes32,bytes32)")]
    pub struct StoreCall(pub ::ethers_core::types::Address, pub [u8; 32], pub [u8; 32]);
    ///Container type for all input parameters for the `toString` function with signature `toString(bytes)` and selector `0x71aad10d`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "toString", abi = "toString(bytes)")]
    pub struct ToString0Call(pub ::ethers_core::types::Bytes);
    ///Container type for all input parameters for the `toString` function with signature `toString(address)` and selector `0x56ca623e`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "toString", abi = "toString(address)")]
    pub struct ToString1Call(pub ::ethers_core::types::Address);
    ///Container type for all input parameters for the `toString` function with signature `toString(uint256)` and selector `0x6900a3ae`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "toString", abi = "toString(uint256)")]
    pub struct ToString2Call(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `toString` function with signature `toString(int256)` and selector `0xa322c40e`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "toString", abi = "toString(int256)")]
    pub struct ToString3Call(pub ::ethers_core::types::I256);
    ///Container type for all input parameters for the `toString` function with signature `toString(bytes32)` and selector `0xb11a19e8`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "toString", abi = "toString(bytes32)")]
    pub struct ToString4Call(pub [u8; 32]);
    ///Container type for all input parameters for the `toString` function with signature `toString(bool)` and selector `0x71dce7da`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "toString", abi = "toString(bool)")]
    pub struct ToString5Call(pub bool);
    ///Container type for all input parameters for the `transact` function with signature `transact(bytes32)` and selector `0xbe646da1`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "transact", abi = "transact(bytes32)")]
    pub struct Transact0Call(pub [u8; 32]);
    ///Container type for all input parameters for the `transact` function with signature `transact(uint256,bytes32)` and selector `0x4d8abc4b`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "transact", abi = "transact(uint256,bytes32)")]
    pub struct Transact1Call(pub ::ethers_core::types::U256, pub [u8; 32]);
    ///Container type for all input parameters for the `tryFfi` function with signature `tryFfi(string[])` and selector `0xf45c1ce7`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "tryFfi", abi = "tryFfi(string[])")]
    pub struct TryFfiCall(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all input parameters for the `txGasPrice` function with signature `txGasPrice(uint256)` and selector `0x48f50c0f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "txGasPrice", abi = "txGasPrice(uint256)")]
    pub struct TxGasPriceCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `warp` function with signature `warp(uint256)` and selector `0xe5d6bf02`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "warp", abi = "warp(uint256)")]
    pub struct WarpCall(pub ::ethers_core::types::U256);
    ///Container type for all input parameters for the `writeFile` function with signature `writeFile(string,string)` and selector `0x897e0a97`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "writeFile", abi = "writeFile(string,string)")]
    pub struct WriteFileCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `writeFileBinary` function with signature `writeFileBinary(string,bytes)` and selector `0x1f21fc80`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "writeFileBinary", abi = "writeFileBinary(string,bytes)")]
    pub struct WriteFileBinaryCall(
        pub ::std::string::String,
        pub ::ethers_core::types::Bytes,
    );
    ///Container type for all input parameters for the `writeJson` function with signature `writeJson(string,string)` and selector `0xe23cd19f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "writeJson", abi = "writeJson(string,string)")]
    pub struct WriteJson0Call(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `writeJson` function with signature `writeJson(string,string,string)` and selector `0x35d6ad46`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "writeJson", abi = "writeJson(string,string,string)")]
    pub struct WriteJson1Call(
        pub ::std::string::String,
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `writeLine` function with signature `writeLine(string,string)` and selector `0x619d897f`
    #[derive(
        Clone,
        ::ethers_contract::EthCall,
        ::ethers_contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "writeLine", abi = "writeLine(string,string)")]
    pub struct WriteLineCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers_contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HEVMCalls {
        Accesses(AccessesCall),
        ActiveFork(ActiveForkCall),
        Addr(AddrCall),
        AllowCheatcodes(AllowCheatcodesCall),
        Assume(AssumeCall),
        Breakpoint0(Breakpoint0Call),
        Breakpoint1(Breakpoint1Call),
        Broadcast0(Broadcast0Call),
        Broadcast1(Broadcast1Call),
        Broadcast2(Broadcast2Call),
        ChainId(ChainIdCall),
        ClearMockedCalls(ClearMockedCallsCall),
        CloseFile(CloseFileCall),
        Coinbase(CoinbaseCall),
        CopyFile(CopyFileCall),
        CreateDir(CreateDirCall),
        CreateFork1(CreateFork1Call),
        CreateFork2(CreateFork2Call),
        CreateFork0(CreateFork0Call),
        CreateSelectFork1(CreateSelectFork1Call),
        CreateSelectFork2(CreateSelectFork2Call),
        CreateSelectFork0(CreateSelectFork0Call),
        CreateWallet0(CreateWallet0Call),
        CreateWallet1(CreateWallet1Call),
        CreateWallet2(CreateWallet2Call),
        Deal(DealCall),
        DeriveKey0(DeriveKey0Call),
        DeriveKey1(DeriveKey1Call),
        DeriveKey2(DeriveKey2Call),
        DeriveKey3(DeriveKey3Call),
        Difficulty(DifficultyCall),
        EnvAddress0(EnvAddress0Call),
        EnvAddress1(EnvAddress1Call),
        EnvBool0(EnvBool0Call),
        EnvBool1(EnvBool1Call),
        EnvBytes0(EnvBytes0Call),
        EnvBytes1(EnvBytes1Call),
        EnvBytes320(EnvBytes320Call),
        EnvBytes321(EnvBytes321Call),
        EnvInt0(EnvInt0Call),
        EnvInt1(EnvInt1Call),
        EnvOr0(EnvOr0Call),
        EnvOr1(EnvOr1Call),
        EnvOr2(EnvOr2Call),
        EnvOr3(EnvOr3Call),
        EnvOr4(EnvOr4Call),
        EnvOr5(EnvOr5Call),
        EnvOr6(EnvOr6Call),
        EnvOr7(EnvOr7Call),
        EnvOr8(EnvOr8Call),
        EnvOr9(EnvOr9Call),
        EnvOr10(EnvOr10Call),
        EnvOr11(EnvOr11Call),
        EnvOr12(EnvOr12Call),
        EnvOr13(EnvOr13Call),
        EnvString0(EnvString0Call),
        EnvString1(EnvString1Call),
        EnvUint0(EnvUint0Call),
        EnvUint1(EnvUint1Call),
        Etch(EtchCall),
        EthGetLogs(EthGetLogsCall),
        Exists(ExistsCall),
        ExpectCall0(ExpectCall0Call),
        ExpectCall1(ExpectCall1Call),
        ExpectCall2(ExpectCall2Call),
        ExpectCall3(ExpectCall3Call),
        ExpectCall4(ExpectCall4Call),
        ExpectCall5(ExpectCall5Call),
        ExpectCallMinGas0(ExpectCallMinGas0Call),
        ExpectCallMinGas1(ExpectCallMinGas1Call),
        ExpectEmit0(ExpectEmit0Call),
        ExpectEmit1(ExpectEmit1Call),
        ExpectEmit2(ExpectEmit2Call),
        ExpectEmit3(ExpectEmit3Call),
        ExpectRevert0(ExpectRevert0Call),
        ExpectRevert1(ExpectRevert1Call),
        ExpectRevert2(ExpectRevert2Call),
        ExpectSafeMemory(ExpectSafeMemoryCall),
        ExpectSafeMemoryCall(ExpectSafeMemoryCallCall),
        Fee(FeeCall),
        Ffi(FfiCall),
        FsMetadata(FsMetadataCall),
        GetCode(GetCodeCall),
        GetDeployedCode0(GetDeployedCode0Call),
        GetDeployedCode1(GetDeployedCode1Call),
        GetLabel(GetLabelCall),
        GetMappingKeyAndParentOf(GetMappingKeyAndParentOfCall),
        GetMappingLength(GetMappingLengthCall),
        GetMappingSlotAt(GetMappingSlotAtCall),
        GetNonce0(GetNonce0Call),
        GetNonce1(GetNonce1Call),
        GetRecordedLogs(GetRecordedLogsCall),
        IsDir(IsDirCall),
        IsFile(IsFileCall),
        IsPersistent(IsPersistentCall),
        KeyExists(KeyExistsCall),
        Label(LabelCall),
        Load(LoadCall),
        MakePersistent0(MakePersistent0Call),
        MakePersistent2(MakePersistent2Call),
        MakePersistent3(MakePersistent3Call),
        MakePersistent1(MakePersistent1Call),
        MockCall0(MockCall0Call),
        MockCall1(MockCall1Call),
        MockCallRevert0(MockCallRevert0Call),
        MockCallRevert1(MockCallRevert1Call),
        OpenFile(OpenFileCall),
        ParseAddress(ParseAddressCall),
        ParseBool(ParseBoolCall),
        ParseBytes(ParseBytesCall),
        ParseBytes32(ParseBytes32Call),
        ParseInt(ParseIntCall),
        ParseJson0(ParseJson0Call),
        ParseJson1(ParseJson1Call),
        ParseJsonAddress(ParseJsonAddressCall),
        ParseJsonAddressArray(ParseJsonAddressArrayCall),
        ParseJsonBool(ParseJsonBoolCall),
        ParseJsonBoolArray(ParseJsonBoolArrayCall),
        ParseJsonBytes(ParseJsonBytesCall),
        ParseJsonBytes32(ParseJsonBytes32Call),
        ParseJsonBytes32Array(ParseJsonBytes32ArrayCall),
        ParseJsonBytesArray(ParseJsonBytesArrayCall),
        ParseJsonInt(ParseJsonIntCall),
        ParseJsonIntArray(ParseJsonIntArrayCall),
        ParseJsonKeys(ParseJsonKeysCall),
        ParseJsonString(ParseJsonStringCall),
        ParseJsonStringArray(ParseJsonStringArrayCall),
        ParseJsonUint(ParseJsonUintCall),
        ParseJsonUintArray(ParseJsonUintArrayCall),
        ParseUint(ParseUintCall),
        PauseGasMetering(PauseGasMeteringCall),
        Prank0(Prank0Call),
        Prank1(Prank1Call),
        Prevrandao(PrevrandaoCall),
        ProjectRoot(ProjectRootCall),
        ReadCallers(ReadCallersCall),
        ReadDir0(ReadDir0Call),
        ReadDir1(ReadDir1Call),
        ReadDir2(ReadDir2Call),
        ReadFile(ReadFileCall),
        ReadFileBinary(ReadFileBinaryCall),
        ReadLine(ReadLineCall),
        ReadLink(ReadLinkCall),
        Record(RecordCall),
        RecordLogs(RecordLogsCall),
        RememberKey(RememberKeyCall),
        RemoveDir(RemoveDirCall),
        RemoveFile(RemoveFileCall),
        ResetNonce(ResetNonceCall),
        ResumeGasMetering(ResumeGasMeteringCall),
        RevertTo(RevertToCall),
        RevokePersistent0(RevokePersistent0Call),
        RevokePersistent1(RevokePersistent1Call),
        Roll(RollCall),
        RollFork0(RollFork0Call),
        RollFork1(RollFork1Call),
        RollFork2(RollFork2Call),
        RollFork3(RollFork3Call),
        Rpc(RpcCall),
        RpcUrl(RpcUrlCall),
        RpcUrlStructs(RpcUrlStructsCall),
        RpcUrls(RpcUrlsCall),
        SelectFork(SelectForkCall),
        SerializeAddress0(SerializeAddress0Call),
        SerializeAddress1(SerializeAddress1Call),
        SerializeBool0(SerializeBool0Call),
        SerializeBool1(SerializeBool1Call),
        SerializeBytes0(SerializeBytes0Call),
        SerializeBytes1(SerializeBytes1Call),
        SerializeBytes320(SerializeBytes320Call),
        SerializeBytes321(SerializeBytes321Call),
        SerializeInt0(SerializeInt0Call),
        SerializeInt1(SerializeInt1Call),
        SerializeString0(SerializeString0Call),
        SerializeString1(SerializeString1Call),
        SerializeUint0(SerializeUint0Call),
        SerializeUint1(SerializeUint1Call),
        SetEnv(SetEnvCall),
        SetNonce(SetNonceCall),
        SetNonceUnsafe(SetNonceUnsafeCall),
        Sign0(Sign0Call),
        Sign1(Sign1Call),
        Skip(SkipCall),
        Sleep(SleepCall),
        Snapshot(SnapshotCall),
        StartBroadcast0(StartBroadcast0Call),
        StartBroadcast1(StartBroadcast1Call),
        StartBroadcast2(StartBroadcast2Call),
        StartMappingRecording(StartMappingRecordingCall),
        StartPrank0(StartPrank0Call),
        StartPrank1(StartPrank1Call),
        StopBroadcast(StopBroadcastCall),
        StopMappingRecording(StopMappingRecordingCall),
        StopPrank(StopPrankCall),
        Store(StoreCall),
        ToString0(ToString0Call),
        ToString1(ToString1Call),
        ToString2(ToString2Call),
        ToString3(ToString3Call),
        ToString4(ToString4Call),
        ToString5(ToString5Call),
        Transact0(Transact0Call),
        Transact1(Transact1Call),
        TryFfi(TryFfiCall),
        TxGasPrice(TxGasPriceCall),
        Warp(WarpCall),
        WriteFile(WriteFileCall),
        WriteFileBinary(WriteFileBinaryCall),
        WriteJson0(WriteJson0Call),
        WriteJson1(WriteJson1Call),
        WriteLine(WriteLineCall),
    }
    impl ::ethers_core::abi::AbiDecode for HEVMCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers_core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <AccessesCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Accesses(decoded));
            }
            if let Ok(decoded)
                = <ActiveForkCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ActiveFork(decoded));
            }
            if let Ok(decoded)
                = <AddrCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Addr(decoded));
            }
            if let Ok(decoded)
                = <AllowCheatcodesCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AllowCheatcodes(decoded));
            }
            if let Ok(decoded)
                = <AssumeCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Assume(decoded));
            }
            if let Ok(decoded)
                = <Breakpoint0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Breakpoint0(decoded));
            }
            if let Ok(decoded)
                = <Breakpoint1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Breakpoint1(decoded));
            }
            if let Ok(decoded)
                = <Broadcast0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Broadcast0(decoded));
            }
            if let Ok(decoded)
                = <Broadcast1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Broadcast1(decoded));
            }
            if let Ok(decoded)
                = <Broadcast2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Broadcast2(decoded));
            }
            if let Ok(decoded)
                = <ChainIdCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ChainId(decoded));
            }
            if let Ok(decoded)
                = <ClearMockedCallsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ClearMockedCalls(decoded));
            }
            if let Ok(decoded)
                = <CloseFileCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CloseFile(decoded));
            }
            if let Ok(decoded)
                = <CoinbaseCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Coinbase(decoded));
            }
            if let Ok(decoded)
                = <CopyFileCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CopyFile(decoded));
            }
            if let Ok(decoded)
                = <CreateDirCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreateDir(decoded));
            }
            if let Ok(decoded)
                = <CreateFork1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreateFork1(decoded));
            }
            if let Ok(decoded)
                = <CreateFork2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreateFork2(decoded));
            }
            if let Ok(decoded)
                = <CreateFork0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreateFork0(decoded));
            }
            if let Ok(decoded)
                = <CreateSelectFork1Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CreateSelectFork1(decoded));
            }
            if let Ok(decoded)
                = <CreateSelectFork2Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CreateSelectFork2(decoded));
            }
            if let Ok(decoded)
                = <CreateSelectFork0Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CreateSelectFork0(decoded));
            }
            if let Ok(decoded)
                = <CreateWallet0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreateWallet0(decoded));
            }
            if let Ok(decoded)
                = <CreateWallet1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreateWallet1(decoded));
            }
            if let Ok(decoded)
                = <CreateWallet2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreateWallet2(decoded));
            }
            if let Ok(decoded)
                = <DealCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Deal(decoded));
            }
            if let Ok(decoded)
                = <DeriveKey0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::DeriveKey0(decoded));
            }
            if let Ok(decoded)
                = <DeriveKey1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::DeriveKey1(decoded));
            }
            if let Ok(decoded)
                = <DeriveKey2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::DeriveKey2(decoded));
            }
            if let Ok(decoded)
                = <DeriveKey3Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::DeriveKey3(decoded));
            }
            if let Ok(decoded)
                = <DifficultyCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Difficulty(decoded));
            }
            if let Ok(decoded)
                = <EnvAddress0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvAddress0(decoded));
            }
            if let Ok(decoded)
                = <EnvAddress1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvAddress1(decoded));
            }
            if let Ok(decoded)
                = <EnvBool0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvBool0(decoded));
            }
            if let Ok(decoded)
                = <EnvBool1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvBool1(decoded));
            }
            if let Ok(decoded)
                = <EnvBytes0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvBytes0(decoded));
            }
            if let Ok(decoded)
                = <EnvBytes1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvBytes1(decoded));
            }
            if let Ok(decoded)
                = <EnvBytes320Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvBytes320(decoded));
            }
            if let Ok(decoded)
                = <EnvBytes321Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvBytes321(decoded));
            }
            if let Ok(decoded)
                = <EnvInt0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvInt0(decoded));
            }
            if let Ok(decoded)
                = <EnvInt1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvInt1(decoded));
            }
            if let Ok(decoded)
                = <EnvOr0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr0(decoded));
            }
            if let Ok(decoded)
                = <EnvOr1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr1(decoded));
            }
            if let Ok(decoded)
                = <EnvOr2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr2(decoded));
            }
            if let Ok(decoded)
                = <EnvOr3Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr3(decoded));
            }
            if let Ok(decoded)
                = <EnvOr4Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr4(decoded));
            }
            if let Ok(decoded)
                = <EnvOr5Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr5(decoded));
            }
            if let Ok(decoded)
                = <EnvOr6Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr6(decoded));
            }
            if let Ok(decoded)
                = <EnvOr7Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr7(decoded));
            }
            if let Ok(decoded)
                = <EnvOr8Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr8(decoded));
            }
            if let Ok(decoded)
                = <EnvOr9Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr9(decoded));
            }
            if let Ok(decoded)
                = <EnvOr10Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr10(decoded));
            }
            if let Ok(decoded)
                = <EnvOr11Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr11(decoded));
            }
            if let Ok(decoded)
                = <EnvOr12Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr12(decoded));
            }
            if let Ok(decoded)
                = <EnvOr13Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvOr13(decoded));
            }
            if let Ok(decoded)
                = <EnvString0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvString0(decoded));
            }
            if let Ok(decoded)
                = <EnvString1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvString1(decoded));
            }
            if let Ok(decoded)
                = <EnvUint0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvUint0(decoded));
            }
            if let Ok(decoded)
                = <EnvUint1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnvUint1(decoded));
            }
            if let Ok(decoded)
                = <EtchCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Etch(decoded));
            }
            if let Ok(decoded)
                = <EthGetLogsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EthGetLogs(decoded));
            }
            if let Ok(decoded)
                = <ExistsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Exists(decoded));
            }
            if let Ok(decoded)
                = <ExpectCall0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectCall0(decoded));
            }
            if let Ok(decoded)
                = <ExpectCall1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectCall1(decoded));
            }
            if let Ok(decoded)
                = <ExpectCall2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectCall2(decoded));
            }
            if let Ok(decoded)
                = <ExpectCall3Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectCall3(decoded));
            }
            if let Ok(decoded)
                = <ExpectCall4Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectCall4(decoded));
            }
            if let Ok(decoded)
                = <ExpectCall5Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectCall5(decoded));
            }
            if let Ok(decoded)
                = <ExpectCallMinGas0Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ExpectCallMinGas0(decoded));
            }
            if let Ok(decoded)
                = <ExpectCallMinGas1Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ExpectCallMinGas1(decoded));
            }
            if let Ok(decoded)
                = <ExpectEmit0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectEmit0(decoded));
            }
            if let Ok(decoded)
                = <ExpectEmit1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectEmit1(decoded));
            }
            if let Ok(decoded)
                = <ExpectEmit2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectEmit2(decoded));
            }
            if let Ok(decoded)
                = <ExpectEmit3Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectEmit3(decoded));
            }
            if let Ok(decoded)
                = <ExpectRevert0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectRevert0(decoded));
            }
            if let Ok(decoded)
                = <ExpectRevert1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectRevert1(decoded));
            }
            if let Ok(decoded)
                = <ExpectRevert2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectRevert2(decoded));
            }
            if let Ok(decoded)
                = <ExpectSafeMemoryCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectSafeMemory(decoded));
            }
            if let Ok(decoded)
                = <ExpectSafeMemoryCallCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ExpectSafeMemoryCall(decoded));
            }
            if let Ok(decoded)
                = <FeeCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Fee(decoded));
            }
            if let Ok(decoded)
                = <FfiCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Ffi(decoded));
            }
            if let Ok(decoded)
                = <FsMetadataCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::FsMetadata(decoded));
            }
            if let Ok(decoded)
                = <GetCodeCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetCode(decoded));
            }
            if let Ok(decoded)
                = <GetDeployedCode0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetDeployedCode0(decoded));
            }
            if let Ok(decoded)
                = <GetDeployedCode1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetDeployedCode1(decoded));
            }
            if let Ok(decoded)
                = <GetLabelCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetLabel(decoded));
            }
            if let Ok(decoded)
                = <GetMappingKeyAndParentOfCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::GetMappingKeyAndParentOf(decoded));
            }
            if let Ok(decoded)
                = <GetMappingLengthCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetMappingLength(decoded));
            }
            if let Ok(decoded)
                = <GetMappingSlotAtCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetMappingSlotAt(decoded));
            }
            if let Ok(decoded)
                = <GetNonce0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetNonce0(decoded));
            }
            if let Ok(decoded)
                = <GetNonce1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetNonce1(decoded));
            }
            if let Ok(decoded)
                = <GetRecordedLogsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetRecordedLogs(decoded));
            }
            if let Ok(decoded)
                = <IsDirCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::IsDir(decoded));
            }
            if let Ok(decoded)
                = <IsFileCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::IsFile(decoded));
            }
            if let Ok(decoded)
                = <IsPersistentCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::IsPersistent(decoded));
            }
            if let Ok(decoded)
                = <KeyExistsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::KeyExists(decoded));
            }
            if let Ok(decoded)
                = <LabelCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Label(decoded));
            }
            if let Ok(decoded)
                = <LoadCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Load(decoded));
            }
            if let Ok(decoded)
                = <MakePersistent0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MakePersistent0(decoded));
            }
            if let Ok(decoded)
                = <MakePersistent2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MakePersistent2(decoded));
            }
            if let Ok(decoded)
                = <MakePersistent3Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MakePersistent3(decoded));
            }
            if let Ok(decoded)
                = <MakePersistent1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MakePersistent1(decoded));
            }
            if let Ok(decoded)
                = <MockCall0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MockCall0(decoded));
            }
            if let Ok(decoded)
                = <MockCall1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MockCall1(decoded));
            }
            if let Ok(decoded)
                = <MockCallRevert0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MockCallRevert0(decoded));
            }
            if let Ok(decoded)
                = <MockCallRevert1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MockCallRevert1(decoded));
            }
            if let Ok(decoded)
                = <OpenFileCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::OpenFile(decoded));
            }
            if let Ok(decoded)
                = <ParseAddressCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseAddress(decoded));
            }
            if let Ok(decoded)
                = <ParseBoolCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseBool(decoded));
            }
            if let Ok(decoded)
                = <ParseBytesCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseBytes(decoded));
            }
            if let Ok(decoded)
                = <ParseBytes32Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseBytes32(decoded));
            }
            if let Ok(decoded)
                = <ParseIntCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseInt(decoded));
            }
            if let Ok(decoded)
                = <ParseJson0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJson0(decoded));
            }
            if let Ok(decoded)
                = <ParseJson1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJson1(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonAddressCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonAddress(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonAddressArrayCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ParseJsonAddressArray(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonBoolCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonBool(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonBoolArrayCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ParseJsonBoolArray(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonBytesCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonBytes(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonBytes32Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonBytes32(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonBytes32ArrayCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ParseJsonBytes32Array(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonBytesArrayCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ParseJsonBytesArray(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonIntCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonInt(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonIntArrayCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ParseJsonIntArray(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonKeysCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonKeys(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonStringCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonString(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonStringArrayCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ParseJsonStringArray(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonUintCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseJsonUint(decoded));
            }
            if let Ok(decoded)
                = <ParseJsonUintArrayCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ParseJsonUintArray(decoded));
            }
            if let Ok(decoded)
                = <ParseUintCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParseUint(decoded));
            }
            if let Ok(decoded)
                = <PauseGasMeteringCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::PauseGasMetering(decoded));
            }
            if let Ok(decoded)
                = <Prank0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Prank0(decoded));
            }
            if let Ok(decoded)
                = <Prank1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Prank1(decoded));
            }
            if let Ok(decoded)
                = <PrevrandaoCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Prevrandao(decoded));
            }
            if let Ok(decoded)
                = <ProjectRootCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ProjectRoot(decoded));
            }
            if let Ok(decoded)
                = <ReadCallersCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadCallers(decoded));
            }
            if let Ok(decoded)
                = <ReadDir0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadDir0(decoded));
            }
            if let Ok(decoded)
                = <ReadDir1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadDir1(decoded));
            }
            if let Ok(decoded)
                = <ReadDir2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadDir2(decoded));
            }
            if let Ok(decoded)
                = <ReadFileCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadFile(decoded));
            }
            if let Ok(decoded)
                = <ReadFileBinaryCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadFileBinary(decoded));
            }
            if let Ok(decoded)
                = <ReadLineCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadLine(decoded));
            }
            if let Ok(decoded)
                = <ReadLinkCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReadLink(decoded));
            }
            if let Ok(decoded)
                = <RecordCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Record(decoded));
            }
            if let Ok(decoded)
                = <RecordLogsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RecordLogs(decoded));
            }
            if let Ok(decoded)
                = <RememberKeyCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RememberKey(decoded));
            }
            if let Ok(decoded)
                = <RemoveDirCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RemoveDir(decoded));
            }
            if let Ok(decoded)
                = <RemoveFileCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RemoveFile(decoded));
            }
            if let Ok(decoded)
                = <ResetNonceCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ResetNonce(decoded));
            }
            if let Ok(decoded)
                = <ResumeGasMeteringCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ResumeGasMetering(decoded));
            }
            if let Ok(decoded)
                = <RevertToCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RevertTo(decoded));
            }
            if let Ok(decoded)
                = <RevokePersistent0Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RevokePersistent0(decoded));
            }
            if let Ok(decoded)
                = <RevokePersistent1Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RevokePersistent1(decoded));
            }
            if let Ok(decoded)
                = <RollCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Roll(decoded));
            }
            if let Ok(decoded)
                = <RollFork0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RollFork0(decoded));
            }
            if let Ok(decoded)
                = <RollFork1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RollFork1(decoded));
            }
            if let Ok(decoded)
                = <RollFork2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RollFork2(decoded));
            }
            if let Ok(decoded)
                = <RollFork3Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RollFork3(decoded));
            }
            if let Ok(decoded)
                = <RpcCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Rpc(decoded));
            }
            if let Ok(decoded)
                = <RpcUrlCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RpcUrl(decoded));
            }
            if let Ok(decoded)
                = <RpcUrlStructsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RpcUrlStructs(decoded));
            }
            if let Ok(decoded)
                = <RpcUrlsCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RpcUrls(decoded));
            }
            if let Ok(decoded)
                = <SelectForkCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SelectFork(decoded));
            }
            if let Ok(decoded)
                = <SerializeAddress0Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SerializeAddress0(decoded));
            }
            if let Ok(decoded)
                = <SerializeAddress1Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SerializeAddress1(decoded));
            }
            if let Ok(decoded)
                = <SerializeBool0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeBool0(decoded));
            }
            if let Ok(decoded)
                = <SerializeBool1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeBool1(decoded));
            }
            if let Ok(decoded)
                = <SerializeBytes0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeBytes0(decoded));
            }
            if let Ok(decoded)
                = <SerializeBytes1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeBytes1(decoded));
            }
            if let Ok(decoded)
                = <SerializeBytes320Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SerializeBytes320(decoded));
            }
            if let Ok(decoded)
                = <SerializeBytes321Call as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SerializeBytes321(decoded));
            }
            if let Ok(decoded)
                = <SerializeInt0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeInt0(decoded));
            }
            if let Ok(decoded)
                = <SerializeInt1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeInt1(decoded));
            }
            if let Ok(decoded)
                = <SerializeString0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeString0(decoded));
            }
            if let Ok(decoded)
                = <SerializeString1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeString1(decoded));
            }
            if let Ok(decoded)
                = <SerializeUint0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeUint0(decoded));
            }
            if let Ok(decoded)
                = <SerializeUint1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SerializeUint1(decoded));
            }
            if let Ok(decoded)
                = <SetEnvCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SetEnv(decoded));
            }
            if let Ok(decoded)
                = <SetNonceCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SetNonce(decoded));
            }
            if let Ok(decoded)
                = <SetNonceUnsafeCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SetNonceUnsafe(decoded));
            }
            if let Ok(decoded)
                = <Sign0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Sign0(decoded));
            }
            if let Ok(decoded)
                = <Sign1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Sign1(decoded));
            }
            if let Ok(decoded)
                = <SkipCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Skip(decoded));
            }
            if let Ok(decoded)
                = <SleepCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Sleep(decoded));
            }
            if let Ok(decoded)
                = <SnapshotCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Snapshot(decoded));
            }
            if let Ok(decoded)
                = <StartBroadcast0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StartBroadcast0(decoded));
            }
            if let Ok(decoded)
                = <StartBroadcast1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StartBroadcast1(decoded));
            }
            if let Ok(decoded)
                = <StartBroadcast2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StartBroadcast2(decoded));
            }
            if let Ok(decoded)
                = <StartMappingRecordingCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::StartMappingRecording(decoded));
            }
            if let Ok(decoded)
                = <StartPrank0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StartPrank0(decoded));
            }
            if let Ok(decoded)
                = <StartPrank1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StartPrank1(decoded));
            }
            if let Ok(decoded)
                = <StopBroadcastCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StopBroadcast(decoded));
            }
            if let Ok(decoded)
                = <StopMappingRecordingCall as ::ethers_core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::StopMappingRecording(decoded));
            }
            if let Ok(decoded)
                = <StopPrankCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StopPrank(decoded));
            }
            if let Ok(decoded)
                = <StoreCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Store(decoded));
            }
            if let Ok(decoded)
                = <ToString0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ToString0(decoded));
            }
            if let Ok(decoded)
                = <ToString1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ToString1(decoded));
            }
            if let Ok(decoded)
                = <ToString2Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ToString2(decoded));
            }
            if let Ok(decoded)
                = <ToString3Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ToString3(decoded));
            }
            if let Ok(decoded)
                = <ToString4Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ToString4(decoded));
            }
            if let Ok(decoded)
                = <ToString5Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ToString5(decoded));
            }
            if let Ok(decoded)
                = <Transact0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Transact0(decoded));
            }
            if let Ok(decoded)
                = <Transact1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Transact1(decoded));
            }
            if let Ok(decoded)
                = <TryFfiCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TryFfi(decoded));
            }
            if let Ok(decoded)
                = <TxGasPriceCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TxGasPrice(decoded));
            }
            if let Ok(decoded)
                = <WarpCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Warp(decoded));
            }
            if let Ok(decoded)
                = <WriteFileCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::WriteFile(decoded));
            }
            if let Ok(decoded)
                = <WriteFileBinaryCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::WriteFileBinary(decoded));
            }
            if let Ok(decoded)
                = <WriteJson0Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::WriteJson0(decoded));
            }
            if let Ok(decoded)
                = <WriteJson1Call as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::WriteJson1(decoded));
            }
            if let Ok(decoded)
                = <WriteLineCall as ::ethers_core::abi::AbiDecode>::decode(data) {
                return Ok(Self::WriteLine(decoded));
            }
            Err(::ethers_core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers_core::abi::AbiEncode for HEVMCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Accesses(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ActiveFork(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Addr(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::AllowCheatcodes(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Assume(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Breakpoint0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Breakpoint1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Broadcast0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Broadcast1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Broadcast2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ChainId(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ClearMockedCalls(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CloseFile(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Coinbase(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::CopyFile(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::CreateDir(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateFork1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateFork2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateFork0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateSelectFork1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateSelectFork2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateSelectFork0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateWallet0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateWallet1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::CreateWallet2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Deal(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::DeriveKey0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::DeriveKey1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::DeriveKey2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::DeriveKey3(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Difficulty(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvAddress0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvAddress1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvBool0(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvBool1(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvBytes0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvBytes1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvBytes320(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvBytes321(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvInt0(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvInt1(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr0(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr1(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr2(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr3(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr4(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr5(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr6(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr7(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr8(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr9(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr10(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr11(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr12(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvOr13(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvString0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvString1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::EnvUint0(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EnvUint1(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Etch(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::EthGetLogs(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Exists(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ExpectCall0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectCall1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectCall2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectCall3(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectCall4(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectCall5(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectCallMinGas0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectCallMinGas1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectEmit0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectEmit1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectEmit2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectEmit3(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectRevert0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectRevert1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectRevert2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectSafeMemory(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ExpectSafeMemoryCall(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Fee(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Ffi(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::FsMetadata(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetCode(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::GetDeployedCode0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetDeployedCode1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetLabel(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::GetMappingKeyAndParentOf(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetMappingLength(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetMappingSlotAt(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetNonce0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetNonce1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::GetRecordedLogs(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::IsDir(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::IsFile(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::IsPersistent(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::KeyExists(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Label(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Load(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::MakePersistent0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::MakePersistent2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::MakePersistent3(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::MakePersistent1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::MockCall0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::MockCall1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::MockCallRevert0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::MockCallRevert1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::OpenFile(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ParseAddress(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseBool(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseBytes(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseBytes32(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseInt(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ParseJson0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJson1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonAddress(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonAddressArray(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonBool(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonBoolArray(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonBytes(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonBytes32(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonBytes32Array(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonBytesArray(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonInt(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonIntArray(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonKeys(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonString(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonStringArray(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonUint(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseJsonUintArray(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ParseUint(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::PauseGasMetering(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Prank0(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Prank1(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Prevrandao(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ProjectRoot(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ReadCallers(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ReadDir0(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ReadDir1(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ReadDir2(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ReadFile(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ReadFileBinary(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ReadLine(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ReadLink(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Record(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::RecordLogs(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RememberKey(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RemoveDir(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RemoveFile(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ResetNonce(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ResumeGasMetering(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RevertTo(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::RevokePersistent0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RevokePersistent1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Roll(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::RollFork0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RollFork1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RollFork2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RollFork3(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Rpc(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::RpcUrl(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::RpcUrlStructs(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::RpcUrls(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::SelectFork(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeAddress0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeAddress1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeBool0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeBool1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeBytes0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeBytes1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeBytes320(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeBytes321(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeInt0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeInt1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeString0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeString1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeUint0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SerializeUint1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::SetEnv(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::SetNonce(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::SetNonceUnsafe(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Sign0(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Sign1(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Skip(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Sleep(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::Snapshot(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::StartBroadcast0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StartBroadcast1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StartBroadcast2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StartMappingRecording(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StartPrank0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StartPrank1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StopBroadcast(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StopMappingRecording(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::StopPrank(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Store(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::ToString0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ToString1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ToString2(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ToString3(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ToString4(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::ToString5(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Transact0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Transact1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::TryFfi(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::TxGasPrice(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::Warp(element) => ::ethers_core::abi::AbiEncode::encode(element),
                Self::WriteFile(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::WriteFileBinary(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::WriteJson0(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::WriteJson1(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
                Self::WriteLine(element) => {
                    ::ethers_core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for HEVMCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Accesses(element) => ::core::fmt::Display::fmt(element, f),
                Self::ActiveFork(element) => ::core::fmt::Display::fmt(element, f),
                Self::Addr(element) => ::core::fmt::Display::fmt(element, f),
                Self::AllowCheatcodes(element) => ::core::fmt::Display::fmt(element, f),
                Self::Assume(element) => ::core::fmt::Display::fmt(element, f),
                Self::Breakpoint0(element) => ::core::fmt::Display::fmt(element, f),
                Self::Breakpoint1(element) => ::core::fmt::Display::fmt(element, f),
                Self::Broadcast0(element) => ::core::fmt::Display::fmt(element, f),
                Self::Broadcast1(element) => ::core::fmt::Display::fmt(element, f),
                Self::Broadcast2(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChainId(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClearMockedCalls(element) => ::core::fmt::Display::fmt(element, f),
                Self::CloseFile(element) => ::core::fmt::Display::fmt(element, f),
                Self::Coinbase(element) => ::core::fmt::Display::fmt(element, f),
                Self::CopyFile(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateDir(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateFork1(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateFork2(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateFork0(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateSelectFork1(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateSelectFork2(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateSelectFork0(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateWallet0(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateWallet1(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateWallet2(element) => ::core::fmt::Display::fmt(element, f),
                Self::Deal(element) => ::core::fmt::Display::fmt(element, f),
                Self::DeriveKey0(element) => ::core::fmt::Display::fmt(element, f),
                Self::DeriveKey1(element) => ::core::fmt::Display::fmt(element, f),
                Self::DeriveKey2(element) => ::core::fmt::Display::fmt(element, f),
                Self::DeriveKey3(element) => ::core::fmt::Display::fmt(element, f),
                Self::Difficulty(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvAddress0(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvAddress1(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvBool0(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvBool1(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvBytes0(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvBytes1(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvBytes320(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvBytes321(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvInt0(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvInt1(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr0(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr1(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr2(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr3(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr4(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr5(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr6(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr7(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr8(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr9(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr10(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr11(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr12(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvOr13(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvString0(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvString1(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvUint0(element) => ::core::fmt::Display::fmt(element, f),
                Self::EnvUint1(element) => ::core::fmt::Display::fmt(element, f),
                Self::Etch(element) => ::core::fmt::Display::fmt(element, f),
                Self::EthGetLogs(element) => ::core::fmt::Display::fmt(element, f),
                Self::Exists(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCall0(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCall1(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCall2(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCall3(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCall4(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCall5(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCallMinGas0(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectCallMinGas1(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectEmit0(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectEmit1(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectEmit2(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectEmit3(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectRevert0(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectRevert1(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectRevert2(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectSafeMemory(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectSafeMemoryCall(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Fee(element) => ::core::fmt::Display::fmt(element, f),
                Self::Ffi(element) => ::core::fmt::Display::fmt(element, f),
                Self::FsMetadata(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCode(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetDeployedCode0(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetDeployedCode1(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLabel(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetMappingKeyAndParentOf(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetMappingLength(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetMappingSlotAt(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetNonce0(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetNonce1(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetRecordedLogs(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsDir(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsFile(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsPersistent(element) => ::core::fmt::Display::fmt(element, f),
                Self::KeyExists(element) => ::core::fmt::Display::fmt(element, f),
                Self::Label(element) => ::core::fmt::Display::fmt(element, f),
                Self::Load(element) => ::core::fmt::Display::fmt(element, f),
                Self::MakePersistent0(element) => ::core::fmt::Display::fmt(element, f),
                Self::MakePersistent2(element) => ::core::fmt::Display::fmt(element, f),
                Self::MakePersistent3(element) => ::core::fmt::Display::fmt(element, f),
                Self::MakePersistent1(element) => ::core::fmt::Display::fmt(element, f),
                Self::MockCall0(element) => ::core::fmt::Display::fmt(element, f),
                Self::MockCall1(element) => ::core::fmt::Display::fmt(element, f),
                Self::MockCallRevert0(element) => ::core::fmt::Display::fmt(element, f),
                Self::MockCallRevert1(element) => ::core::fmt::Display::fmt(element, f),
                Self::OpenFile(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseBool(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseBytes(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseBytes32(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseInt(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJson0(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJson1(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonAddressArray(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ParseJsonBool(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonBoolArray(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ParseJsonBytes(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonBytes32(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonBytes32Array(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ParseJsonBytesArray(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ParseJsonInt(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonIntArray(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonKeys(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonString(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonStringArray(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ParseJsonUint(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParseJsonUintArray(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ParseUint(element) => ::core::fmt::Display::fmt(element, f),
                Self::PauseGasMetering(element) => ::core::fmt::Display::fmt(element, f),
                Self::Prank0(element) => ::core::fmt::Display::fmt(element, f),
                Self::Prank1(element) => ::core::fmt::Display::fmt(element, f),
                Self::Prevrandao(element) => ::core::fmt::Display::fmt(element, f),
                Self::ProjectRoot(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadCallers(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadDir0(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadDir1(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadDir2(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadFile(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadFileBinary(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadLine(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReadLink(element) => ::core::fmt::Display::fmt(element, f),
                Self::Record(element) => ::core::fmt::Display::fmt(element, f),
                Self::RecordLogs(element) => ::core::fmt::Display::fmt(element, f),
                Self::RememberKey(element) => ::core::fmt::Display::fmt(element, f),
                Self::RemoveDir(element) => ::core::fmt::Display::fmt(element, f),
                Self::RemoveFile(element) => ::core::fmt::Display::fmt(element, f),
                Self::ResetNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::ResumeGasMetering(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertTo(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevokePersistent0(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevokePersistent1(element) => ::core::fmt::Display::fmt(element, f),
                Self::Roll(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollFork0(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollFork1(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollFork2(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollFork3(element) => ::core::fmt::Display::fmt(element, f),
                Self::Rpc(element) => ::core::fmt::Display::fmt(element, f),
                Self::RpcUrl(element) => ::core::fmt::Display::fmt(element, f),
                Self::RpcUrlStructs(element) => ::core::fmt::Display::fmt(element, f),
                Self::RpcUrls(element) => ::core::fmt::Display::fmt(element, f),
                Self::SelectFork(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeAddress0(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeAddress1(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeBool0(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeBool1(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeBytes0(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeBytes1(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeBytes320(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeBytes321(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeInt0(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeInt1(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeString0(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeString1(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeUint0(element) => ::core::fmt::Display::fmt(element, f),
                Self::SerializeUint1(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetEnv(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetNonceUnsafe(element) => ::core::fmt::Display::fmt(element, f),
                Self::Sign0(element) => ::core::fmt::Display::fmt(element, f),
                Self::Sign1(element) => ::core::fmt::Display::fmt(element, f),
                Self::Skip(element) => ::core::fmt::Display::fmt(element, f),
                Self::Sleep(element) => ::core::fmt::Display::fmt(element, f),
                Self::Snapshot(element) => ::core::fmt::Display::fmt(element, f),
                Self::StartBroadcast0(element) => ::core::fmt::Display::fmt(element, f),
                Self::StartBroadcast1(element) => ::core::fmt::Display::fmt(element, f),
                Self::StartBroadcast2(element) => ::core::fmt::Display::fmt(element, f),
                Self::StartMappingRecording(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StartPrank0(element) => ::core::fmt::Display::fmt(element, f),
                Self::StartPrank1(element) => ::core::fmt::Display::fmt(element, f),
                Self::StopBroadcast(element) => ::core::fmt::Display::fmt(element, f),
                Self::StopMappingRecording(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StopPrank(element) => ::core::fmt::Display::fmt(element, f),
                Self::Store(element) => ::core::fmt::Display::fmt(element, f),
                Self::ToString0(element) => ::core::fmt::Display::fmt(element, f),
                Self::ToString1(element) => ::core::fmt::Display::fmt(element, f),
                Self::ToString2(element) => ::core::fmt::Display::fmt(element, f),
                Self::ToString3(element) => ::core::fmt::Display::fmt(element, f),
                Self::ToString4(element) => ::core::fmt::Display::fmt(element, f),
                Self::ToString5(element) => ::core::fmt::Display::fmt(element, f),
                Self::Transact0(element) => ::core::fmt::Display::fmt(element, f),
                Self::Transact1(element) => ::core::fmt::Display::fmt(element, f),
                Self::TryFfi(element) => ::core::fmt::Display::fmt(element, f),
                Self::TxGasPrice(element) => ::core::fmt::Display::fmt(element, f),
                Self::Warp(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteFile(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteFileBinary(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteJson0(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteJson1(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteLine(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AccessesCall> for HEVMCalls {
        fn from(value: AccessesCall) -> Self {
            Self::Accesses(value)
        }
    }
    impl ::core::convert::From<ActiveForkCall> for HEVMCalls {
        fn from(value: ActiveForkCall) -> Self {
            Self::ActiveFork(value)
        }
    }
    impl ::core::convert::From<AddrCall> for HEVMCalls {
        fn from(value: AddrCall) -> Self {
            Self::Addr(value)
        }
    }
    impl ::core::convert::From<AllowCheatcodesCall> for HEVMCalls {
        fn from(value: AllowCheatcodesCall) -> Self {
            Self::AllowCheatcodes(value)
        }
    }
    impl ::core::convert::From<AssumeCall> for HEVMCalls {
        fn from(value: AssumeCall) -> Self {
            Self::Assume(value)
        }
    }
    impl ::core::convert::From<Breakpoint0Call> for HEVMCalls {
        fn from(value: Breakpoint0Call) -> Self {
            Self::Breakpoint0(value)
        }
    }
    impl ::core::convert::From<Breakpoint1Call> for HEVMCalls {
        fn from(value: Breakpoint1Call) -> Self {
            Self::Breakpoint1(value)
        }
    }
    impl ::core::convert::From<Broadcast0Call> for HEVMCalls {
        fn from(value: Broadcast0Call) -> Self {
            Self::Broadcast0(value)
        }
    }
    impl ::core::convert::From<Broadcast1Call> for HEVMCalls {
        fn from(value: Broadcast1Call) -> Self {
            Self::Broadcast1(value)
        }
    }
    impl ::core::convert::From<Broadcast2Call> for HEVMCalls {
        fn from(value: Broadcast2Call) -> Self {
            Self::Broadcast2(value)
        }
    }
    impl ::core::convert::From<ChainIdCall> for HEVMCalls {
        fn from(value: ChainIdCall) -> Self {
            Self::ChainId(value)
        }
    }
    impl ::core::convert::From<ClearMockedCallsCall> for HEVMCalls {
        fn from(value: ClearMockedCallsCall) -> Self {
            Self::ClearMockedCalls(value)
        }
    }
    impl ::core::convert::From<CloseFileCall> for HEVMCalls {
        fn from(value: CloseFileCall) -> Self {
            Self::CloseFile(value)
        }
    }
    impl ::core::convert::From<CoinbaseCall> for HEVMCalls {
        fn from(value: CoinbaseCall) -> Self {
            Self::Coinbase(value)
        }
    }
    impl ::core::convert::From<CopyFileCall> for HEVMCalls {
        fn from(value: CopyFileCall) -> Self {
            Self::CopyFile(value)
        }
    }
    impl ::core::convert::From<CreateDirCall> for HEVMCalls {
        fn from(value: CreateDirCall) -> Self {
            Self::CreateDir(value)
        }
    }
    impl ::core::convert::From<CreateFork1Call> for HEVMCalls {
        fn from(value: CreateFork1Call) -> Self {
            Self::CreateFork1(value)
        }
    }
    impl ::core::convert::From<CreateFork2Call> for HEVMCalls {
        fn from(value: CreateFork2Call) -> Self {
            Self::CreateFork2(value)
        }
    }
    impl ::core::convert::From<CreateFork0Call> for HEVMCalls {
        fn from(value: CreateFork0Call) -> Self {
            Self::CreateFork0(value)
        }
    }
    impl ::core::convert::From<CreateSelectFork1Call> for HEVMCalls {
        fn from(value: CreateSelectFork1Call) -> Self {
            Self::CreateSelectFork1(value)
        }
    }
    impl ::core::convert::From<CreateSelectFork2Call> for HEVMCalls {
        fn from(value: CreateSelectFork2Call) -> Self {
            Self::CreateSelectFork2(value)
        }
    }
    impl ::core::convert::From<CreateSelectFork0Call> for HEVMCalls {
        fn from(value: CreateSelectFork0Call) -> Self {
            Self::CreateSelectFork0(value)
        }
    }
    impl ::core::convert::From<CreateWallet0Call> for HEVMCalls {
        fn from(value: CreateWallet0Call) -> Self {
            Self::CreateWallet0(value)
        }
    }
    impl ::core::convert::From<CreateWallet1Call> for HEVMCalls {
        fn from(value: CreateWallet1Call) -> Self {
            Self::CreateWallet1(value)
        }
    }
    impl ::core::convert::From<CreateWallet2Call> for HEVMCalls {
        fn from(value: CreateWallet2Call) -> Self {
            Self::CreateWallet2(value)
        }
    }
    impl ::core::convert::From<DealCall> for HEVMCalls {
        fn from(value: DealCall) -> Self {
            Self::Deal(value)
        }
    }
    impl ::core::convert::From<DeriveKey0Call> for HEVMCalls {
        fn from(value: DeriveKey0Call) -> Self {
            Self::DeriveKey0(value)
        }
    }
    impl ::core::convert::From<DeriveKey1Call> for HEVMCalls {
        fn from(value: DeriveKey1Call) -> Self {
            Self::DeriveKey1(value)
        }
    }
    impl ::core::convert::From<DeriveKey2Call> for HEVMCalls {
        fn from(value: DeriveKey2Call) -> Self {
            Self::DeriveKey2(value)
        }
    }
    impl ::core::convert::From<DeriveKey3Call> for HEVMCalls {
        fn from(value: DeriveKey3Call) -> Self {
            Self::DeriveKey3(value)
        }
    }
    impl ::core::convert::From<DifficultyCall> for HEVMCalls {
        fn from(value: DifficultyCall) -> Self {
            Self::Difficulty(value)
        }
    }
    impl ::core::convert::From<EnvAddress0Call> for HEVMCalls {
        fn from(value: EnvAddress0Call) -> Self {
            Self::EnvAddress0(value)
        }
    }
    impl ::core::convert::From<EnvAddress1Call> for HEVMCalls {
        fn from(value: EnvAddress1Call) -> Self {
            Self::EnvAddress1(value)
        }
    }
    impl ::core::convert::From<EnvBool0Call> for HEVMCalls {
        fn from(value: EnvBool0Call) -> Self {
            Self::EnvBool0(value)
        }
    }
    impl ::core::convert::From<EnvBool1Call> for HEVMCalls {
        fn from(value: EnvBool1Call) -> Self {
            Self::EnvBool1(value)
        }
    }
    impl ::core::convert::From<EnvBytes0Call> for HEVMCalls {
        fn from(value: EnvBytes0Call) -> Self {
            Self::EnvBytes0(value)
        }
    }
    impl ::core::convert::From<EnvBytes1Call> for HEVMCalls {
        fn from(value: EnvBytes1Call) -> Self {
            Self::EnvBytes1(value)
        }
    }
    impl ::core::convert::From<EnvBytes320Call> for HEVMCalls {
        fn from(value: EnvBytes320Call) -> Self {
            Self::EnvBytes320(value)
        }
    }
    impl ::core::convert::From<EnvBytes321Call> for HEVMCalls {
        fn from(value: EnvBytes321Call) -> Self {
            Self::EnvBytes321(value)
        }
    }
    impl ::core::convert::From<EnvInt0Call> for HEVMCalls {
        fn from(value: EnvInt0Call) -> Self {
            Self::EnvInt0(value)
        }
    }
    impl ::core::convert::From<EnvInt1Call> for HEVMCalls {
        fn from(value: EnvInt1Call) -> Self {
            Self::EnvInt1(value)
        }
    }
    impl ::core::convert::From<EnvOr0Call> for HEVMCalls {
        fn from(value: EnvOr0Call) -> Self {
            Self::EnvOr0(value)
        }
    }
    impl ::core::convert::From<EnvOr1Call> for HEVMCalls {
        fn from(value: EnvOr1Call) -> Self {
            Self::EnvOr1(value)
        }
    }
    impl ::core::convert::From<EnvOr2Call> for HEVMCalls {
        fn from(value: EnvOr2Call) -> Self {
            Self::EnvOr2(value)
        }
    }
    impl ::core::convert::From<EnvOr3Call> for HEVMCalls {
        fn from(value: EnvOr3Call) -> Self {
            Self::EnvOr3(value)
        }
    }
    impl ::core::convert::From<EnvOr4Call> for HEVMCalls {
        fn from(value: EnvOr4Call) -> Self {
            Self::EnvOr4(value)
        }
    }
    impl ::core::convert::From<EnvOr5Call> for HEVMCalls {
        fn from(value: EnvOr5Call) -> Self {
            Self::EnvOr5(value)
        }
    }
    impl ::core::convert::From<EnvOr6Call> for HEVMCalls {
        fn from(value: EnvOr6Call) -> Self {
            Self::EnvOr6(value)
        }
    }
    impl ::core::convert::From<EnvOr7Call> for HEVMCalls {
        fn from(value: EnvOr7Call) -> Self {
            Self::EnvOr7(value)
        }
    }
    impl ::core::convert::From<EnvOr8Call> for HEVMCalls {
        fn from(value: EnvOr8Call) -> Self {
            Self::EnvOr8(value)
        }
    }
    impl ::core::convert::From<EnvOr9Call> for HEVMCalls {
        fn from(value: EnvOr9Call) -> Self {
            Self::EnvOr9(value)
        }
    }
    impl ::core::convert::From<EnvOr10Call> for HEVMCalls {
        fn from(value: EnvOr10Call) -> Self {
            Self::EnvOr10(value)
        }
    }
    impl ::core::convert::From<EnvOr11Call> for HEVMCalls {
        fn from(value: EnvOr11Call) -> Self {
            Self::EnvOr11(value)
        }
    }
    impl ::core::convert::From<EnvOr12Call> for HEVMCalls {
        fn from(value: EnvOr12Call) -> Self {
            Self::EnvOr12(value)
        }
    }
    impl ::core::convert::From<EnvOr13Call> for HEVMCalls {
        fn from(value: EnvOr13Call) -> Self {
            Self::EnvOr13(value)
        }
    }
    impl ::core::convert::From<EnvString0Call> for HEVMCalls {
        fn from(value: EnvString0Call) -> Self {
            Self::EnvString0(value)
        }
    }
    impl ::core::convert::From<EnvString1Call> for HEVMCalls {
        fn from(value: EnvString1Call) -> Self {
            Self::EnvString1(value)
        }
    }
    impl ::core::convert::From<EnvUint0Call> for HEVMCalls {
        fn from(value: EnvUint0Call) -> Self {
            Self::EnvUint0(value)
        }
    }
    impl ::core::convert::From<EnvUint1Call> for HEVMCalls {
        fn from(value: EnvUint1Call) -> Self {
            Self::EnvUint1(value)
        }
    }
    impl ::core::convert::From<EtchCall> for HEVMCalls {
        fn from(value: EtchCall) -> Self {
            Self::Etch(value)
        }
    }
    impl ::core::convert::From<EthGetLogsCall> for HEVMCalls {
        fn from(value: EthGetLogsCall) -> Self {
            Self::EthGetLogs(value)
        }
    }
    impl ::core::convert::From<ExistsCall> for HEVMCalls {
        fn from(value: ExistsCall) -> Self {
            Self::Exists(value)
        }
    }
    impl ::core::convert::From<ExpectCall0Call> for HEVMCalls {
        fn from(value: ExpectCall0Call) -> Self {
            Self::ExpectCall0(value)
        }
    }
    impl ::core::convert::From<ExpectCall1Call> for HEVMCalls {
        fn from(value: ExpectCall1Call) -> Self {
            Self::ExpectCall1(value)
        }
    }
    impl ::core::convert::From<ExpectCall2Call> for HEVMCalls {
        fn from(value: ExpectCall2Call) -> Self {
            Self::ExpectCall2(value)
        }
    }
    impl ::core::convert::From<ExpectCall3Call> for HEVMCalls {
        fn from(value: ExpectCall3Call) -> Self {
            Self::ExpectCall3(value)
        }
    }
    impl ::core::convert::From<ExpectCall4Call> for HEVMCalls {
        fn from(value: ExpectCall4Call) -> Self {
            Self::ExpectCall4(value)
        }
    }
    impl ::core::convert::From<ExpectCall5Call> for HEVMCalls {
        fn from(value: ExpectCall5Call) -> Self {
            Self::ExpectCall5(value)
        }
    }
    impl ::core::convert::From<ExpectCallMinGas0Call> for HEVMCalls {
        fn from(value: ExpectCallMinGas0Call) -> Self {
            Self::ExpectCallMinGas0(value)
        }
    }
    impl ::core::convert::From<ExpectCallMinGas1Call> for HEVMCalls {
        fn from(value: ExpectCallMinGas1Call) -> Self {
            Self::ExpectCallMinGas1(value)
        }
    }
    impl ::core::convert::From<ExpectEmit0Call> for HEVMCalls {
        fn from(value: ExpectEmit0Call) -> Self {
            Self::ExpectEmit0(value)
        }
    }
    impl ::core::convert::From<ExpectEmit1Call> for HEVMCalls {
        fn from(value: ExpectEmit1Call) -> Self {
            Self::ExpectEmit1(value)
        }
    }
    impl ::core::convert::From<ExpectEmit2Call> for HEVMCalls {
        fn from(value: ExpectEmit2Call) -> Self {
            Self::ExpectEmit2(value)
        }
    }
    impl ::core::convert::From<ExpectEmit3Call> for HEVMCalls {
        fn from(value: ExpectEmit3Call) -> Self {
            Self::ExpectEmit3(value)
        }
    }
    impl ::core::convert::From<ExpectRevert0Call> for HEVMCalls {
        fn from(value: ExpectRevert0Call) -> Self {
            Self::ExpectRevert0(value)
        }
    }
    impl ::core::convert::From<ExpectRevert1Call> for HEVMCalls {
        fn from(value: ExpectRevert1Call) -> Self {
            Self::ExpectRevert1(value)
        }
    }
    impl ::core::convert::From<ExpectRevert2Call> for HEVMCalls {
        fn from(value: ExpectRevert2Call) -> Self {
            Self::ExpectRevert2(value)
        }
    }
    impl ::core::convert::From<ExpectSafeMemoryCall> for HEVMCalls {
        fn from(value: ExpectSafeMemoryCall) -> Self {
            Self::ExpectSafeMemory(value)
        }
    }
    impl ::core::convert::From<ExpectSafeMemoryCallCall> for HEVMCalls {
        fn from(value: ExpectSafeMemoryCallCall) -> Self {
            Self::ExpectSafeMemoryCall(value)
        }
    }
    impl ::core::convert::From<FeeCall> for HEVMCalls {
        fn from(value: FeeCall) -> Self {
            Self::Fee(value)
        }
    }
    impl ::core::convert::From<FfiCall> for HEVMCalls {
        fn from(value: FfiCall) -> Self {
            Self::Ffi(value)
        }
    }
    impl ::core::convert::From<FsMetadataCall> for HEVMCalls {
        fn from(value: FsMetadataCall) -> Self {
            Self::FsMetadata(value)
        }
    }
    impl ::core::convert::From<GetCodeCall> for HEVMCalls {
        fn from(value: GetCodeCall) -> Self {
            Self::GetCode(value)
        }
    }
    impl ::core::convert::From<GetDeployedCode0Call> for HEVMCalls {
        fn from(value: GetDeployedCode0Call) -> Self {
            Self::GetDeployedCode0(value)
        }
    }
    impl ::core::convert::From<GetDeployedCode1Call> for HEVMCalls {
        fn from(value: GetDeployedCode1Call) -> Self {
            Self::GetDeployedCode1(value)
        }
    }
    impl ::core::convert::From<GetLabelCall> for HEVMCalls {
        fn from(value: GetLabelCall) -> Self {
            Self::GetLabel(value)
        }
    }
    impl ::core::convert::From<GetMappingKeyAndParentOfCall> for HEVMCalls {
        fn from(value: GetMappingKeyAndParentOfCall) -> Self {
            Self::GetMappingKeyAndParentOf(value)
        }
    }
    impl ::core::convert::From<GetMappingLengthCall> for HEVMCalls {
        fn from(value: GetMappingLengthCall) -> Self {
            Self::GetMappingLength(value)
        }
    }
    impl ::core::convert::From<GetMappingSlotAtCall> for HEVMCalls {
        fn from(value: GetMappingSlotAtCall) -> Self {
            Self::GetMappingSlotAt(value)
        }
    }
    impl ::core::convert::From<GetNonce0Call> for HEVMCalls {
        fn from(value: GetNonce0Call) -> Self {
            Self::GetNonce0(value)
        }
    }
    impl ::core::convert::From<GetNonce1Call> for HEVMCalls {
        fn from(value: GetNonce1Call) -> Self {
            Self::GetNonce1(value)
        }
    }
    impl ::core::convert::From<GetRecordedLogsCall> for HEVMCalls {
        fn from(value: GetRecordedLogsCall) -> Self {
            Self::GetRecordedLogs(value)
        }
    }
    impl ::core::convert::From<IsDirCall> for HEVMCalls {
        fn from(value: IsDirCall) -> Self {
            Self::IsDir(value)
        }
    }
    impl ::core::convert::From<IsFileCall> for HEVMCalls {
        fn from(value: IsFileCall) -> Self {
            Self::IsFile(value)
        }
    }
    impl ::core::convert::From<IsPersistentCall> for HEVMCalls {
        fn from(value: IsPersistentCall) -> Self {
            Self::IsPersistent(value)
        }
    }
    impl ::core::convert::From<KeyExistsCall> for HEVMCalls {
        fn from(value: KeyExistsCall) -> Self {
            Self::KeyExists(value)
        }
    }
    impl ::core::convert::From<LabelCall> for HEVMCalls {
        fn from(value: LabelCall) -> Self {
            Self::Label(value)
        }
    }
    impl ::core::convert::From<LoadCall> for HEVMCalls {
        fn from(value: LoadCall) -> Self {
            Self::Load(value)
        }
    }
    impl ::core::convert::From<MakePersistent0Call> for HEVMCalls {
        fn from(value: MakePersistent0Call) -> Self {
            Self::MakePersistent0(value)
        }
    }
    impl ::core::convert::From<MakePersistent2Call> for HEVMCalls {
        fn from(value: MakePersistent2Call) -> Self {
            Self::MakePersistent2(value)
        }
    }
    impl ::core::convert::From<MakePersistent3Call> for HEVMCalls {
        fn from(value: MakePersistent3Call) -> Self {
            Self::MakePersistent3(value)
        }
    }
    impl ::core::convert::From<MakePersistent1Call> for HEVMCalls {
        fn from(value: MakePersistent1Call) -> Self {
            Self::MakePersistent1(value)
        }
    }
    impl ::core::convert::From<MockCall0Call> for HEVMCalls {
        fn from(value: MockCall0Call) -> Self {
            Self::MockCall0(value)
        }
    }
    impl ::core::convert::From<MockCall1Call> for HEVMCalls {
        fn from(value: MockCall1Call) -> Self {
            Self::MockCall1(value)
        }
    }
    impl ::core::convert::From<MockCallRevert0Call> for HEVMCalls {
        fn from(value: MockCallRevert0Call) -> Self {
            Self::MockCallRevert0(value)
        }
    }
    impl ::core::convert::From<MockCallRevert1Call> for HEVMCalls {
        fn from(value: MockCallRevert1Call) -> Self {
            Self::MockCallRevert1(value)
        }
    }
    impl ::core::convert::From<OpenFileCall> for HEVMCalls {
        fn from(value: OpenFileCall) -> Self {
            Self::OpenFile(value)
        }
    }
    impl ::core::convert::From<ParseAddressCall> for HEVMCalls {
        fn from(value: ParseAddressCall) -> Self {
            Self::ParseAddress(value)
        }
    }
    impl ::core::convert::From<ParseBoolCall> for HEVMCalls {
        fn from(value: ParseBoolCall) -> Self {
            Self::ParseBool(value)
        }
    }
    impl ::core::convert::From<ParseBytesCall> for HEVMCalls {
        fn from(value: ParseBytesCall) -> Self {
            Self::ParseBytes(value)
        }
    }
    impl ::core::convert::From<ParseBytes32Call> for HEVMCalls {
        fn from(value: ParseBytes32Call) -> Self {
            Self::ParseBytes32(value)
        }
    }
    impl ::core::convert::From<ParseIntCall> for HEVMCalls {
        fn from(value: ParseIntCall) -> Self {
            Self::ParseInt(value)
        }
    }
    impl ::core::convert::From<ParseJson0Call> for HEVMCalls {
        fn from(value: ParseJson0Call) -> Self {
            Self::ParseJson0(value)
        }
    }
    impl ::core::convert::From<ParseJson1Call> for HEVMCalls {
        fn from(value: ParseJson1Call) -> Self {
            Self::ParseJson1(value)
        }
    }
    impl ::core::convert::From<ParseJsonAddressCall> for HEVMCalls {
        fn from(value: ParseJsonAddressCall) -> Self {
            Self::ParseJsonAddress(value)
        }
    }
    impl ::core::convert::From<ParseJsonAddressArrayCall> for HEVMCalls {
        fn from(value: ParseJsonAddressArrayCall) -> Self {
            Self::ParseJsonAddressArray(value)
        }
    }
    impl ::core::convert::From<ParseJsonBoolCall> for HEVMCalls {
        fn from(value: ParseJsonBoolCall) -> Self {
            Self::ParseJsonBool(value)
        }
    }
    impl ::core::convert::From<ParseJsonBoolArrayCall> for HEVMCalls {
        fn from(value: ParseJsonBoolArrayCall) -> Self {
            Self::ParseJsonBoolArray(value)
        }
    }
    impl ::core::convert::From<ParseJsonBytesCall> for HEVMCalls {
        fn from(value: ParseJsonBytesCall) -> Self {
            Self::ParseJsonBytes(value)
        }
    }
    impl ::core::convert::From<ParseJsonBytes32Call> for HEVMCalls {
        fn from(value: ParseJsonBytes32Call) -> Self {
            Self::ParseJsonBytes32(value)
        }
    }
    impl ::core::convert::From<ParseJsonBytes32ArrayCall> for HEVMCalls {
        fn from(value: ParseJsonBytes32ArrayCall) -> Self {
            Self::ParseJsonBytes32Array(value)
        }
    }
    impl ::core::convert::From<ParseJsonBytesArrayCall> for HEVMCalls {
        fn from(value: ParseJsonBytesArrayCall) -> Self {
            Self::ParseJsonBytesArray(value)
        }
    }
    impl ::core::convert::From<ParseJsonIntCall> for HEVMCalls {
        fn from(value: ParseJsonIntCall) -> Self {
            Self::ParseJsonInt(value)
        }
    }
    impl ::core::convert::From<ParseJsonIntArrayCall> for HEVMCalls {
        fn from(value: ParseJsonIntArrayCall) -> Self {
            Self::ParseJsonIntArray(value)
        }
    }
    impl ::core::convert::From<ParseJsonKeysCall> for HEVMCalls {
        fn from(value: ParseJsonKeysCall) -> Self {
            Self::ParseJsonKeys(value)
        }
    }
    impl ::core::convert::From<ParseJsonStringCall> for HEVMCalls {
        fn from(value: ParseJsonStringCall) -> Self {
            Self::ParseJsonString(value)
        }
    }
    impl ::core::convert::From<ParseJsonStringArrayCall> for HEVMCalls {
        fn from(value: ParseJsonStringArrayCall) -> Self {
            Self::ParseJsonStringArray(value)
        }
    }
    impl ::core::convert::From<ParseJsonUintCall> for HEVMCalls {
        fn from(value: ParseJsonUintCall) -> Self {
            Self::ParseJsonUint(value)
        }
    }
    impl ::core::convert::From<ParseJsonUintArrayCall> for HEVMCalls {
        fn from(value: ParseJsonUintArrayCall) -> Self {
            Self::ParseJsonUintArray(value)
        }
    }
    impl ::core::convert::From<ParseUintCall> for HEVMCalls {
        fn from(value: ParseUintCall) -> Self {
            Self::ParseUint(value)
        }
    }
    impl ::core::convert::From<PauseGasMeteringCall> for HEVMCalls {
        fn from(value: PauseGasMeteringCall) -> Self {
            Self::PauseGasMetering(value)
        }
    }
    impl ::core::convert::From<Prank0Call> for HEVMCalls {
        fn from(value: Prank0Call) -> Self {
            Self::Prank0(value)
        }
    }
    impl ::core::convert::From<Prank1Call> for HEVMCalls {
        fn from(value: Prank1Call) -> Self {
            Self::Prank1(value)
        }
    }
    impl ::core::convert::From<PrevrandaoCall> for HEVMCalls {
        fn from(value: PrevrandaoCall) -> Self {
            Self::Prevrandao(value)
        }
    }
    impl ::core::convert::From<ProjectRootCall> for HEVMCalls {
        fn from(value: ProjectRootCall) -> Self {
            Self::ProjectRoot(value)
        }
    }
    impl ::core::convert::From<ReadCallersCall> for HEVMCalls {
        fn from(value: ReadCallersCall) -> Self {
            Self::ReadCallers(value)
        }
    }
    impl ::core::convert::From<ReadDir0Call> for HEVMCalls {
        fn from(value: ReadDir0Call) -> Self {
            Self::ReadDir0(value)
        }
    }
    impl ::core::convert::From<ReadDir1Call> for HEVMCalls {
        fn from(value: ReadDir1Call) -> Self {
            Self::ReadDir1(value)
        }
    }
    impl ::core::convert::From<ReadDir2Call> for HEVMCalls {
        fn from(value: ReadDir2Call) -> Self {
            Self::ReadDir2(value)
        }
    }
    impl ::core::convert::From<ReadFileCall> for HEVMCalls {
        fn from(value: ReadFileCall) -> Self {
            Self::ReadFile(value)
        }
    }
    impl ::core::convert::From<ReadFileBinaryCall> for HEVMCalls {
        fn from(value: ReadFileBinaryCall) -> Self {
            Self::ReadFileBinary(value)
        }
    }
    impl ::core::convert::From<ReadLineCall> for HEVMCalls {
        fn from(value: ReadLineCall) -> Self {
            Self::ReadLine(value)
        }
    }
    impl ::core::convert::From<ReadLinkCall> for HEVMCalls {
        fn from(value: ReadLinkCall) -> Self {
            Self::ReadLink(value)
        }
    }
    impl ::core::convert::From<RecordCall> for HEVMCalls {
        fn from(value: RecordCall) -> Self {
            Self::Record(value)
        }
    }
    impl ::core::convert::From<RecordLogsCall> for HEVMCalls {
        fn from(value: RecordLogsCall) -> Self {
            Self::RecordLogs(value)
        }
    }
    impl ::core::convert::From<RememberKeyCall> for HEVMCalls {
        fn from(value: RememberKeyCall) -> Self {
            Self::RememberKey(value)
        }
    }
    impl ::core::convert::From<RemoveDirCall> for HEVMCalls {
        fn from(value: RemoveDirCall) -> Self {
            Self::RemoveDir(value)
        }
    }
    impl ::core::convert::From<RemoveFileCall> for HEVMCalls {
        fn from(value: RemoveFileCall) -> Self {
            Self::RemoveFile(value)
        }
    }
    impl ::core::convert::From<ResetNonceCall> for HEVMCalls {
        fn from(value: ResetNonceCall) -> Self {
            Self::ResetNonce(value)
        }
    }
    impl ::core::convert::From<ResumeGasMeteringCall> for HEVMCalls {
        fn from(value: ResumeGasMeteringCall) -> Self {
            Self::ResumeGasMetering(value)
        }
    }
    impl ::core::convert::From<RevertToCall> for HEVMCalls {
        fn from(value: RevertToCall) -> Self {
            Self::RevertTo(value)
        }
    }
    impl ::core::convert::From<RevokePersistent0Call> for HEVMCalls {
        fn from(value: RevokePersistent0Call) -> Self {
            Self::RevokePersistent0(value)
        }
    }
    impl ::core::convert::From<RevokePersistent1Call> for HEVMCalls {
        fn from(value: RevokePersistent1Call) -> Self {
            Self::RevokePersistent1(value)
        }
    }
    impl ::core::convert::From<RollCall> for HEVMCalls {
        fn from(value: RollCall) -> Self {
            Self::Roll(value)
        }
    }
    impl ::core::convert::From<RollFork0Call> for HEVMCalls {
        fn from(value: RollFork0Call) -> Self {
            Self::RollFork0(value)
        }
    }
    impl ::core::convert::From<RollFork1Call> for HEVMCalls {
        fn from(value: RollFork1Call) -> Self {
            Self::RollFork1(value)
        }
    }
    impl ::core::convert::From<RollFork2Call> for HEVMCalls {
        fn from(value: RollFork2Call) -> Self {
            Self::RollFork2(value)
        }
    }
    impl ::core::convert::From<RollFork3Call> for HEVMCalls {
        fn from(value: RollFork3Call) -> Self {
            Self::RollFork3(value)
        }
    }
    impl ::core::convert::From<RpcCall> for HEVMCalls {
        fn from(value: RpcCall) -> Self {
            Self::Rpc(value)
        }
    }
    impl ::core::convert::From<RpcUrlCall> for HEVMCalls {
        fn from(value: RpcUrlCall) -> Self {
            Self::RpcUrl(value)
        }
    }
    impl ::core::convert::From<RpcUrlStructsCall> for HEVMCalls {
        fn from(value: RpcUrlStructsCall) -> Self {
            Self::RpcUrlStructs(value)
        }
    }
    impl ::core::convert::From<RpcUrlsCall> for HEVMCalls {
        fn from(value: RpcUrlsCall) -> Self {
            Self::RpcUrls(value)
        }
    }
    impl ::core::convert::From<SelectForkCall> for HEVMCalls {
        fn from(value: SelectForkCall) -> Self {
            Self::SelectFork(value)
        }
    }
    impl ::core::convert::From<SerializeAddress0Call> for HEVMCalls {
        fn from(value: SerializeAddress0Call) -> Self {
            Self::SerializeAddress0(value)
        }
    }
    impl ::core::convert::From<SerializeAddress1Call> for HEVMCalls {
        fn from(value: SerializeAddress1Call) -> Self {
            Self::SerializeAddress1(value)
        }
    }
    impl ::core::convert::From<SerializeBool0Call> for HEVMCalls {
        fn from(value: SerializeBool0Call) -> Self {
            Self::SerializeBool0(value)
        }
    }
    impl ::core::convert::From<SerializeBool1Call> for HEVMCalls {
        fn from(value: SerializeBool1Call) -> Self {
            Self::SerializeBool1(value)
        }
    }
    impl ::core::convert::From<SerializeBytes0Call> for HEVMCalls {
        fn from(value: SerializeBytes0Call) -> Self {
            Self::SerializeBytes0(value)
        }
    }
    impl ::core::convert::From<SerializeBytes1Call> for HEVMCalls {
        fn from(value: SerializeBytes1Call) -> Self {
            Self::SerializeBytes1(value)
        }
    }
    impl ::core::convert::From<SerializeBytes320Call> for HEVMCalls {
        fn from(value: SerializeBytes320Call) -> Self {
            Self::SerializeBytes320(value)
        }
    }
    impl ::core::convert::From<SerializeBytes321Call> for HEVMCalls {
        fn from(value: SerializeBytes321Call) -> Self {
            Self::SerializeBytes321(value)
        }
    }
    impl ::core::convert::From<SerializeInt0Call> for HEVMCalls {
        fn from(value: SerializeInt0Call) -> Self {
            Self::SerializeInt0(value)
        }
    }
    impl ::core::convert::From<SerializeInt1Call> for HEVMCalls {
        fn from(value: SerializeInt1Call) -> Self {
            Self::SerializeInt1(value)
        }
    }
    impl ::core::convert::From<SerializeString0Call> for HEVMCalls {
        fn from(value: SerializeString0Call) -> Self {
            Self::SerializeString0(value)
        }
    }
    impl ::core::convert::From<SerializeString1Call> for HEVMCalls {
        fn from(value: SerializeString1Call) -> Self {
            Self::SerializeString1(value)
        }
    }
    impl ::core::convert::From<SerializeUint0Call> for HEVMCalls {
        fn from(value: SerializeUint0Call) -> Self {
            Self::SerializeUint0(value)
        }
    }
    impl ::core::convert::From<SerializeUint1Call> for HEVMCalls {
        fn from(value: SerializeUint1Call) -> Self {
            Self::SerializeUint1(value)
        }
    }
    impl ::core::convert::From<SetEnvCall> for HEVMCalls {
        fn from(value: SetEnvCall) -> Self {
            Self::SetEnv(value)
        }
    }
    impl ::core::convert::From<SetNonceCall> for HEVMCalls {
        fn from(value: SetNonceCall) -> Self {
            Self::SetNonce(value)
        }
    }
    impl ::core::convert::From<SetNonceUnsafeCall> for HEVMCalls {
        fn from(value: SetNonceUnsafeCall) -> Self {
            Self::SetNonceUnsafe(value)
        }
    }
    impl ::core::convert::From<Sign0Call> for HEVMCalls {
        fn from(value: Sign0Call) -> Self {
            Self::Sign0(value)
        }
    }
    impl ::core::convert::From<Sign1Call> for HEVMCalls {
        fn from(value: Sign1Call) -> Self {
            Self::Sign1(value)
        }
    }
    impl ::core::convert::From<SkipCall> for HEVMCalls {
        fn from(value: SkipCall) -> Self {
            Self::Skip(value)
        }
    }
    impl ::core::convert::From<SleepCall> for HEVMCalls {
        fn from(value: SleepCall) -> Self {
            Self::Sleep(value)
        }
    }
    impl ::core::convert::From<SnapshotCall> for HEVMCalls {
        fn from(value: SnapshotCall) -> Self {
            Self::Snapshot(value)
        }
    }
    impl ::core::convert::From<StartBroadcast0Call> for HEVMCalls {
        fn from(value: StartBroadcast0Call) -> Self {
            Self::StartBroadcast0(value)
        }
    }
    impl ::core::convert::From<StartBroadcast1Call> for HEVMCalls {
        fn from(value: StartBroadcast1Call) -> Self {
            Self::StartBroadcast1(value)
        }
    }
    impl ::core::convert::From<StartBroadcast2Call> for HEVMCalls {
        fn from(value: StartBroadcast2Call) -> Self {
            Self::StartBroadcast2(value)
        }
    }
    impl ::core::convert::From<StartMappingRecordingCall> for HEVMCalls {
        fn from(value: StartMappingRecordingCall) -> Self {
            Self::StartMappingRecording(value)
        }
    }
    impl ::core::convert::From<StartPrank0Call> for HEVMCalls {
        fn from(value: StartPrank0Call) -> Self {
            Self::StartPrank0(value)
        }
    }
    impl ::core::convert::From<StartPrank1Call> for HEVMCalls {
        fn from(value: StartPrank1Call) -> Self {
            Self::StartPrank1(value)
        }
    }
    impl ::core::convert::From<StopBroadcastCall> for HEVMCalls {
        fn from(value: StopBroadcastCall) -> Self {
            Self::StopBroadcast(value)
        }
    }
    impl ::core::convert::From<StopMappingRecordingCall> for HEVMCalls {
        fn from(value: StopMappingRecordingCall) -> Self {
            Self::StopMappingRecording(value)
        }
    }
    impl ::core::convert::From<StopPrankCall> for HEVMCalls {
        fn from(value: StopPrankCall) -> Self {
            Self::StopPrank(value)
        }
    }
    impl ::core::convert::From<StoreCall> for HEVMCalls {
        fn from(value: StoreCall) -> Self {
            Self::Store(value)
        }
    }
    impl ::core::convert::From<ToString0Call> for HEVMCalls {
        fn from(value: ToString0Call) -> Self {
            Self::ToString0(value)
        }
    }
    impl ::core::convert::From<ToString1Call> for HEVMCalls {
        fn from(value: ToString1Call) -> Self {
            Self::ToString1(value)
        }
    }
    impl ::core::convert::From<ToString2Call> for HEVMCalls {
        fn from(value: ToString2Call) -> Self {
            Self::ToString2(value)
        }
    }
    impl ::core::convert::From<ToString3Call> for HEVMCalls {
        fn from(value: ToString3Call) -> Self {
            Self::ToString3(value)
        }
    }
    impl ::core::convert::From<ToString4Call> for HEVMCalls {
        fn from(value: ToString4Call) -> Self {
            Self::ToString4(value)
        }
    }
    impl ::core::convert::From<ToString5Call> for HEVMCalls {
        fn from(value: ToString5Call) -> Self {
            Self::ToString5(value)
        }
    }
    impl ::core::convert::From<Transact0Call> for HEVMCalls {
        fn from(value: Transact0Call) -> Self {
            Self::Transact0(value)
        }
    }
    impl ::core::convert::From<Transact1Call> for HEVMCalls {
        fn from(value: Transact1Call) -> Self {
            Self::Transact1(value)
        }
    }
    impl ::core::convert::From<TryFfiCall> for HEVMCalls {
        fn from(value: TryFfiCall) -> Self {
            Self::TryFfi(value)
        }
    }
    impl ::core::convert::From<TxGasPriceCall> for HEVMCalls {
        fn from(value: TxGasPriceCall) -> Self {
            Self::TxGasPrice(value)
        }
    }
    impl ::core::convert::From<WarpCall> for HEVMCalls {
        fn from(value: WarpCall) -> Self {
            Self::Warp(value)
        }
    }
    impl ::core::convert::From<WriteFileCall> for HEVMCalls {
        fn from(value: WriteFileCall) -> Self {
            Self::WriteFile(value)
        }
    }
    impl ::core::convert::From<WriteFileBinaryCall> for HEVMCalls {
        fn from(value: WriteFileBinaryCall) -> Self {
            Self::WriteFileBinary(value)
        }
    }
    impl ::core::convert::From<WriteJson0Call> for HEVMCalls {
        fn from(value: WriteJson0Call) -> Self {
            Self::WriteJson0(value)
        }
    }
    impl ::core::convert::From<WriteJson1Call> for HEVMCalls {
        fn from(value: WriteJson1Call) -> Self {
            Self::WriteJson1(value)
        }
    }
    impl ::core::convert::From<WriteLineCall> for HEVMCalls {
        fn from(value: WriteLineCall) -> Self {
            Self::WriteLine(value)
        }
    }
    ///Container type for all return fields from the `accesses` function with signature `accesses(address)` and selector `0x65bc9481`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct AccessesReturn(
        pub ::std::vec::Vec<[u8; 32]>,
        pub ::std::vec::Vec<[u8; 32]>,
    );
    ///Container type for all return fields from the `activeFork` function with signature `activeFork()` and selector `0x2f103f22`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ActiveForkReturn(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `addr` function with signature `addr(uint256)` and selector `0xffa18649`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct AddrReturn(pub ::ethers_core::types::Address);
    ///Container type for all return fields from the `createFork` function with signature `createFork(string,uint256)` and selector `0x6ba3ba2b`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateFork1Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `createFork` function with signature `createFork(string,bytes32)` and selector `0x7ca29682`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateFork2Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `createFork` function with signature `createFork(string)` and selector `0x31ba3498`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateFork0Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `createSelectFork` function with signature `createSelectFork(string,uint256)` and selector `0x71ee464d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateSelectFork1Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `createSelectFork` function with signature `createSelectFork(string,bytes32)` and selector `0x84d52b7a`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateSelectFork2Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `createSelectFork` function with signature `createSelectFork(string)` and selector `0x98680034`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateSelectFork0Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `createWallet` function with signature `createWallet(string)` and selector `0x7404f1d2`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateWallet0Return(
        pub (
            ::ethers_core::types::Address,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
        ),
    );
    ///Container type for all return fields from the `createWallet` function with signature `createWallet(uint256)` and selector `0x7a675bb6`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateWallet1Return(
        pub (
            ::ethers_core::types::Address,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
        ),
    );
    ///Container type for all return fields from the `createWallet` function with signature `createWallet(uint256,string)` and selector `0xed7c5462`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateWallet2Return(
        pub (
            ::ethers_core::types::Address,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
        ),
    );
    ///Container type for all return fields from the `deriveKey` function with signature `deriveKey(string,uint32)` and selector `0x6229498b`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct DeriveKey0Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `deriveKey` function with signature `deriveKey(string,string,uint32)` and selector `0x6bcb2c1b`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct DeriveKey1Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `deriveKey` function with signature `deriveKey(string,uint32,string)` and selector `0x32c8176d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct DeriveKey2Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `deriveKey` function with signature `deriveKey(string,string,uint32,string)` and selector `0x29233b1f`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct DeriveKey3Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `envAddress` function with signature `envAddress(string)` and selector `0x350d56bf`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvAddress0Return(pub ::ethers_core::types::Address);
    ///Container type for all return fields from the `envAddress` function with signature `envAddress(string,string)` and selector `0xad31b9fa`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvAddress1Return(pub ::std::vec::Vec<::ethers_core::types::Address>);
    ///Container type for all return fields from the `envBool` function with signature `envBool(string)` and selector `0x7ed1ec7d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvBool0Return(pub bool);
    ///Container type for all return fields from the `envBool` function with signature `envBool(string,string)` and selector `0xaaaddeaf`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvBool1Return(pub ::std::vec::Vec<bool>);
    ///Container type for all return fields from the `envBytes` function with signature `envBytes(string)` and selector `0x4d7baf06`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvBytes0Return(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `envBytes` function with signature `envBytes(string,string)` and selector `0xddc2651b`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvBytes1Return(pub ::std::vec::Vec<::ethers_core::types::Bytes>);
    ///Container type for all return fields from the `envBytes32` function with signature `envBytes32(string)` and selector `0x97949042`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvBytes320Return(pub [u8; 32]);
    ///Container type for all return fields from the `envBytes32` function with signature `envBytes32(string,string)` and selector `0x5af231c1`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvBytes321Return(pub ::std::vec::Vec<[u8; 32]>);
    ///Container type for all return fields from the `envInt` function with signature `envInt(string)` and selector `0x892a0c61`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvInt0Return(pub ::ethers_core::types::I256);
    ///Container type for all return fields from the `envInt` function with signature `envInt(string,string)` and selector `0x42181150`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvInt1Return(pub ::std::vec::Vec<::ethers_core::types::I256>);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,bool)` and selector `0x4777f3cf`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr0Return(pub bool);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,uint256)` and selector `0x5e97348f`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr1Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,int256)` and selector `0xbbcb713e`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr2Return(pub ::ethers_core::types::I256);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,address)` and selector `0x561fe540`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr3Return(pub ::ethers_core::types::Address);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,bytes32)` and selector `0xb4a85892`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr4Return(pub [u8; 32]);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string)` and selector `0xd145736c`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr5Return(pub ::std::string::String);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,bytes)` and selector `0xb3e47705`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr6Return(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string,bool[])` and selector `0xeb85e83b`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr7Return(pub ::std::vec::Vec<bool>);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string,uint256[])` and selector `0x74318528`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr8Return(pub ::std::vec::Vec<::ethers_core::types::U256>);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string,int256[])` and selector `0x4700d74b`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr9Return(pub ::std::vec::Vec<::ethers_core::types::I256>);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string,address[])` and selector `0xc74e9deb`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr10Return(pub ::std::vec::Vec<::ethers_core::types::Address>);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string,bytes32[])` and selector `0x2281f367`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr11Return(pub ::std::vec::Vec<[u8; 32]>);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string,string[])` and selector `0x859216bc`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr12Return(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all return fields from the `envOr` function with signature `envOr(string,string,bytes[])` and selector `0x64bc3e64`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvOr13Return(pub ::std::vec::Vec<::ethers_core::types::Bytes>);
    ///Container type for all return fields from the `envString` function with signature `envString(string)` and selector `0xf877cb19`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvString0Return(pub ::std::string::String);
    ///Container type for all return fields from the `envString` function with signature `envString(string,string)` and selector `0x14b02bc9`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvString1Return(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all return fields from the `envUint` function with signature `envUint(string)` and selector `0xc1978d1f`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvUint0Return(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `envUint` function with signature `envUint(string,string)` and selector `0xf3dec099`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EnvUint1Return(pub ::std::vec::Vec<::ethers_core::types::U256>);
    ///Container type for all return fields from the `eth_getLogs` function with signature `eth_getLogs(uint256,uint256,address,bytes32[])` and selector `0x35e1349b`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EthGetLogsReturn(
        pub ::std::vec::Vec<
            (
                ::ethers_core::types::Address,
                ::std::vec::Vec<[u8; 32]>,
                ::ethers_core::types::Bytes,
                ::ethers_core::types::U256,
                [u8; 32],
                ::ethers_core::types::U256,
                [u8; 32],
                ::ethers_core::types::U256,
                bool,
            ),
        >,
    );
    ///Container type for all return fields from the `exists` function with signature `exists(string)` and selector `0x261a323e`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ExistsReturn(pub bool);
    ///Container type for all return fields from the `ffi` function with signature `ffi(string[])` and selector `0x89160467`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct FfiReturn(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `fsMetadata` function with signature `fsMetadata(string)` and selector `0xaf368a08`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct FsMetadataReturn(
        pub (
            bool,
            bool,
            ::ethers_core::types::U256,
            bool,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
            ::ethers_core::types::U256,
        ),
    );
    ///Container type for all return fields from the `getLabel` function with signature `getLabel(address)` and selector `0x28a249b0`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetLabelReturn(pub ::std::string::String);
    ///Container type for all return fields from the `getNonce` function with signature `getNonce((address,uint256,uint256,uint256))` and selector `0xa5748aad`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetNonce0Return(pub u64);
    ///Container type for all return fields from the `getRecordedLogs` function with signature `getRecordedLogs()` and selector `0x191553a4`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetRecordedLogsReturn(
        pub ::std::vec::Vec<(::std::vec::Vec<[u8; 32]>, ::ethers_core::types::Bytes)>,
    );
    ///Container type for all return fields from the `isDir` function with signature `isDir(string)` and selector `0x7d15d019`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct IsDirReturn(pub bool);
    ///Container type for all return fields from the `isFile` function with signature `isFile(string)` and selector `0xe0eb04d4`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct IsFileReturn(pub bool);
    ///Container type for all return fields from the `isPersistent` function with signature `isPersistent(address)` and selector `0xd92d8efd`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct IsPersistentReturn(pub bool);
    ///Container type for all return fields from the `keyExists` function with signature `keyExists(string,string)` and selector `0x528a683c`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct KeyExistsReturn(pub bool);
    ///Container type for all return fields from the `load` function with signature `load(address,bytes32)` and selector `0x667f9d70`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LoadReturn(pub [u8; 32]);
    ///Container type for all return fields from the `parseAddress` function with signature `parseAddress(string)` and selector `0xc6ce059d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseAddressReturn(pub ::ethers_core::types::Address);
    ///Container type for all return fields from the `parseBool` function with signature `parseBool(string)` and selector `0x974ef924`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseBoolReturn(pub bool);
    ///Container type for all return fields from the `parseBytes` function with signature `parseBytes(string)` and selector `0x8f5d232d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseBytesReturn(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `parseBytes32` function with signature `parseBytes32(string)` and selector `0x087e6e81`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseBytes32Return(pub [u8; 32]);
    ///Container type for all return fields from the `parseInt` function with signature `parseInt(string)` and selector `0x42346c5e`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseIntReturn(pub ::ethers_core::types::I256);
    ///Container type for all return fields from the `parseJson` function with signature `parseJson(string)` and selector `0x6a82600a`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJson0Return(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `parseJson` function with signature `parseJson(string,string)` and selector `0x85940ef1`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJson1Return(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `parseJsonAddress` function with signature `parseJsonAddress(string,string)` and selector `0x1e19e657`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonAddressReturn(pub ::ethers_core::types::Address);
    ///Container type for all return fields from the `parseJsonAddressArray` function with signature `parseJsonAddressArray(string,string)` and selector `0x2fce7883`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonAddressArrayReturn(
        pub ::std::vec::Vec<::ethers_core::types::Address>,
    );
    ///Container type for all return fields from the `parseJsonBool` function with signature `parseJsonBool(string,string)` and selector `0x9f86dc91`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonBoolReturn(pub bool);
    ///Container type for all return fields from the `parseJsonBoolArray` function with signature `parseJsonBoolArray(string,string)` and selector `0x91f3b94f`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonBoolArrayReturn(pub ::std::vec::Vec<bool>);
    ///Container type for all return fields from the `parseJsonBytes` function with signature `parseJsonBytes(string,string)` and selector `0xfd921be8`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonBytesReturn(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `parseJsonBytes32` function with signature `parseJsonBytes32(string,string)` and selector `0x1777e59d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonBytes32Return(pub [u8; 32]);
    ///Container type for all return fields from the `parseJsonBytes32Array` function with signature `parseJsonBytes32Array(string,string)` and selector `0x91c75bc3`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonBytes32ArrayReturn(pub ::std::vec::Vec<[u8; 32]>);
    ///Container type for all return fields from the `parseJsonBytesArray` function with signature `parseJsonBytesArray(string,string)` and selector `0x6631aa99`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonBytesArrayReturn(
        pub ::std::vec::Vec<::ethers_core::types::Bytes>,
    );
    ///Container type for all return fields from the `parseJsonInt` function with signature `parseJsonInt(string,string)` and selector `0x7b048ccd`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonIntReturn(pub ::ethers_core::types::I256);
    ///Container type for all return fields from the `parseJsonIntArray` function with signature `parseJsonIntArray(string,string)` and selector `0x9983c28a`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonIntArrayReturn(pub ::std::vec::Vec<::ethers_core::types::I256>);
    ///Container type for all return fields from the `parseJsonKeys` function with signature `parseJsonKeys(string,string)` and selector `0x213e4198`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonKeysReturn(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all return fields from the `parseJsonString` function with signature `parseJsonString(string,string)` and selector `0x49c4fac8`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonStringReturn(pub ::std::string::String);
    ///Container type for all return fields from the `parseJsonStringArray` function with signature `parseJsonStringArray(string,string)` and selector `0x498fdcf4`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonStringArrayReturn(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all return fields from the `parseJsonUint` function with signature `parseJsonUint(string,string)` and selector `0xaddde2b6`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonUintReturn(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `parseJsonUintArray` function with signature `parseJsonUintArray(string,string)` and selector `0x522074ab`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseJsonUintArrayReturn(pub ::std::vec::Vec<::ethers_core::types::U256>);
    ///Container type for all return fields from the `parseUint` function with signature `parseUint(string)` and selector `0xfa91454d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParseUintReturn(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `projectRoot` function with signature `projectRoot()` and selector `0xd930a0e6`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ProjectRootReturn(pub ::std::string::String);
    ///Container type for all return fields from the `readCallers` function with signature `readCallers()` and selector `0x4ad0bac9`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadCallersReturn(
        pub ::ethers_core::types::U256,
        pub ::ethers_core::types::Address,
        pub ::ethers_core::types::Address,
    );
    ///Container type for all return fields from the `readDir` function with signature `readDir(string)` and selector `0xc4bc59e0`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadDir0Return(
        pub ::std::vec::Vec<
            (::std::string::String, ::std::string::String, u64, bool, bool),
        >,
    );
    ///Container type for all return fields from the `readDir` function with signature `readDir(string,uint64)` and selector `0x1497876c`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadDir1Return(
        pub ::std::vec::Vec<
            (::std::string::String, ::std::string::String, u64, bool, bool),
        >,
    );
    ///Container type for all return fields from the `readDir` function with signature `readDir(string,uint64,bool)` and selector `0x8102d70d`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadDir2Return(
        pub ::std::vec::Vec<
            (::std::string::String, ::std::string::String, u64, bool, bool),
        >,
    );
    ///Container type for all return fields from the `readFile` function with signature `readFile(string)` and selector `0x60f9bb11`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadFileReturn(pub ::std::string::String);
    ///Container type for all return fields from the `readFileBinary` function with signature `readFileBinary(string)` and selector `0x16ed7bc4`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadFileBinaryReturn(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `readLine` function with signature `readLine(string)` and selector `0x70f55728`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadLineReturn(pub ::std::string::String);
    ///Container type for all return fields from the `readLink` function with signature `readLink(string)` and selector `0x9f5684a2`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ReadLinkReturn(pub ::std::string::String);
    ///Container type for all return fields from the `rememberKey` function with signature `rememberKey(uint256)` and selector `0x22100064`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RememberKeyReturn(pub ::ethers_core::types::Address);
    ///Container type for all return fields from the `revertTo` function with signature `revertTo(uint256)` and selector `0x44d7f0a4`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RevertToReturn(pub bool);
    ///Container type for all return fields from the `rpc` function with signature `rpc(string,string)` and selector `0x1206c8a8`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RpcReturn(pub ::ethers_core::types::Bytes);
    ///Container type for all return fields from the `rpcUrl` function with signature `rpcUrl(string)` and selector `0x975a6ce9`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RpcUrlReturn(pub ::std::string::String);
    ///Container type for all return fields from the `rpcUrlStructs` function with signature `rpcUrlStructs()` and selector `0x9d2ad72a`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RpcUrlStructsReturn(
        pub ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
    );
    ///Container type for all return fields from the `rpcUrls` function with signature `rpcUrls()` and selector `0xa85a8418`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RpcUrlsReturn(pub ::std::vec::Vec<[::std::string::String; 2]>);
    ///Container type for all return fields from the `serializeAddress` function with signature `serializeAddress(string,string,address)` and selector `0x972c6062`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeAddress0Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeAddress` function with signature `serializeAddress(string,string,address[])` and selector `0x1e356e1a`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeAddress1Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeBool` function with signature `serializeBool(string,string,bool)` and selector `0xac22e971`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeBool0Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeBool` function with signature `serializeBool(string,string,bool[])` and selector `0x92925aa1`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeBool1Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeBytes` function with signature `serializeBytes(string,string,bytes)` and selector `0xf21d52c7`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeBytes0Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeBytes` function with signature `serializeBytes(string,string,bytes[])` and selector `0x9884b232`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeBytes1Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeBytes32` function with signature `serializeBytes32(string,string,bytes32)` and selector `0x2d812b44`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeBytes320Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeBytes32` function with signature `serializeBytes32(string,string,bytes32[])` and selector `0x201e43e2`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeBytes321Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeInt` function with signature `serializeInt(string,string,int256)` and selector `0x3f33db60`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeInt0Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeInt` function with signature `serializeInt(string,string,int256[])` and selector `0x7676e127`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeInt1Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeString` function with signature `serializeString(string,string,string)` and selector `0x88da6d35`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeString0Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeString` function with signature `serializeString(string,string,string[])` and selector `0x561cd6f3`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeString1Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeUint` function with signature `serializeUint(string,string,uint256)` and selector `0x129e9002`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeUint0Return(pub ::std::string::String);
    ///Container type for all return fields from the `serializeUint` function with signature `serializeUint(string,string,uint256[])` and selector `0xfee9a469`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SerializeUint1Return(pub ::std::string::String);
    ///Container type for all return fields from the `sign` function with signature `sign(uint256,bytes32)` and selector `0xe341eaa4`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Sign0Return(pub u8, pub [u8; 32], pub [u8; 32]);
    ///Container type for all return fields from the `sign` function with signature `sign((address,uint256,uint256,uint256),bytes32)` and selector `0xb25c5a25`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Sign1Return(pub u8, pub [u8; 32], pub [u8; 32]);
    ///Container type for all return fields from the `snapshot` function with signature `snapshot()` and selector `0x9711715a`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SnapshotReturn(pub ::ethers_core::types::U256);
    ///Container type for all return fields from the `tryFfi` function with signature `tryFfi(string[])` and selector `0xf45c1ce7`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct TryFfiReturn(
        pub (i32, ::ethers_core::types::Bytes, ::ethers_core::types::Bytes),
    );
    ///`DirEntry(string,string,uint64,bool,bool)`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct DirEntry {
        pub error_message: ::std::string::String,
        pub path: ::std::string::String,
        pub depth: u64,
        pub is_dir: bool,
        pub is_symlink: bool,
    }
    ///`EthGetLogs(address,bytes32[],bytes,uint256,bytes32,uint256,bytes32,uint256,bool)`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EthGetLogs {
        pub emitter: ::ethers_core::types::Address,
        pub topics: ::std::vec::Vec<[u8; 32]>,
        pub data: ::ethers_core::types::Bytes,
        pub block_number: ::ethers_core::types::U256,
        pub transaction_hash: [u8; 32],
        pub transaction_index: ::ethers_core::types::U256,
        pub block_hash: [u8; 32],
        pub log_index: ::ethers_core::types::U256,
        pub removed: bool,
    }
    ///`FfiResult(int32,bytes,bytes)`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct FfiResult {
        pub exit_code: i32,
        pub stdout: ::ethers_core::types::Bytes,
        pub stderr: ::ethers_core::types::Bytes,
    }
    ///`FsMetadata(bool,bool,uint256,bool,uint256,uint256,uint256)`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct FsMetadata {
        pub is_dir: bool,
        pub is_symlink: bool,
        pub length: ::ethers_core::types::U256,
        pub read_only: bool,
        pub modified: ::ethers_core::types::U256,
        pub accessed: ::ethers_core::types::U256,
        pub created: ::ethers_core::types::U256,
    }
    ///`Log(bytes32[],bytes)`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Log {
        pub topics: ::std::vec::Vec<[u8; 32]>,
        pub data: ::ethers_core::types::Bytes,
    }
    ///`Rpc(string,string)`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Rpc {
        pub name: ::std::string::String,
        pub url: ::std::string::String,
    }
    ///`Wallet(address,uint256,uint256,uint256)`
    #[derive(
        Clone,
        ::ethers_contract::EthAbiType,
        ::ethers_contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Wallet {
        pub addr: ::ethers_core::types::Address,
        pub public_key_x: ::ethers_core::types::U256,
        pub public_key_y: ::ethers_core::types::U256,
        pub private_key: ::ethers_core::types::U256,
    }
}
