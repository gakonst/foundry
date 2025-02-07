contract DivideBeforeMultiply {
    function arithmetic() public {
        (1 / 2) * 3; // Unsafe
        (1 * 2) / 3; // Safe
        ((1 / 2) * 3) * 4; // Unsafe
        ((1 * 2) / 3) * 4; // Unsafe
        (1 / 2 / 3) * 4; // Unsafe
        (1 / (2 + 3)) * 4; // Unsafe
        (1 / 2 + 3) * 4; // Safe
        (1 / 2 - 3) * 4; // Safe
        (1 + 2 / 3) * 4; // Safe
        (1 / 2 - 3) * 4; // Safe
        ((1 / 2) % 3) * 4; // Safe
        1 / (2 * 3 + 3); // Safe
        1 / ((2 / 3) * 3); // Unsafe
        1 / ((2 * 3) + 3); // Safe
    }
}
