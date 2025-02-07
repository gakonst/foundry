// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract ScreamingSnakeCaseTest {
    // Passes
    uint256 constant _SCREAMING_SNAKE_CASE = 0;
    uint256 constant SCREAMING_SNAKE_CASE = 0;
    uint256 immutable _SCREAMING_SNAKE_CASE_1 = 0;
    uint256 immutable SCREAMING_SNAKE_CASE_1 = 0;

    // Fails
    uint256 constant SCREAMINGSNAKECASE = 0;
    uint256 constant screamingSnakeCase = 0;
    uint256 constant screaming_snake_case = 0;
    uint256 constant ScreamingSnakeCase = 0;
    uint256 constant SCREAMING_snake_case = 0;
    uint256 immutable SCREAMINGSNAKECASE0 = 0;
    uint256 immutable screamingSnakeCase0 = 0;
    uint256 immutable screaming_snake_case0 = 0;
    uint256 immutable ScreamingSnakeCase0 = 0;
    uint256 immutable SCREAMING_snake_case_0 = 0;
}
