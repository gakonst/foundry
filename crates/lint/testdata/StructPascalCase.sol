// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract StructPascalCaseTest {
    // Passes
    struct PascalCase {
        uint256 a;
    }

    // Fails
    struct _PascalCase {
        uint256 a;
    }

    struct pascalCase {
        uint256 a;
    }

    struct pascalcase {
        uint256 a;
    }

    struct pascal_case {
        uint256 a;
    }

    struct PASCAL_CASE {
        uint256 a;
    }

    struct PASCALCASE {
        uint256 a;
    }

    struct PascalCAse {
        uint256 a;
    }
}
