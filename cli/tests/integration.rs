use foundry_cli_test_utils::forgetest_external;

forgetest_external!(solmate, "Rari-Capital/solmate");
forgetest_external!(geb, "reflexer-labs/geb");
forgetest_external!(stringutils, "Arachnid/solidity-stringutils");
forgetest_external!(vaults, "Rari-Capital/vaults");
forgetest_external!(multicall, "makerdao/multicall");
forgetest_external!(lootloose, "gakonst/lootloose");

// Forking tests
forgetest_external!(drai, "mds1/drai", 13633752);
forgetest_external!(gunilev, "hexonaut/guni-lev", 13633752);
