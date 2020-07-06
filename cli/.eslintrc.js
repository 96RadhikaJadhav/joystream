module.exports = {
  "extends": [
    // The oclif rules have some code-style/formatting rules which may conflict with
    // our prettier global settings. Disabling for now
    // I suggest to only add essential rules absolutely required to make the cli work with oclif
    // at the end of this file, to override @joystream/eslint-config
    // "oclif",
    // "oclif-typescript",

    // not strictly necessary becase we have this in the root and will be used
    // through cascading rules
    "@joystream/eslint-config",
  ]
}
