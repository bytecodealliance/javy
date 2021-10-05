module.exports = {
  roots: ['<rootDir>/src'],
  testMatch: [
    "**/__tests__/**/*.+(ts|tsx|js)",
    "**/?(*.)+(spec|test).+(ts|tsx|js)"
  ],
  transform: {
    "^.+\\.(ts|tsx|js)$": "babel-jest"
  },
  transformIgnorePatterns: [
    "node_modules/(?!@shopify\/scripts-checkout-apis-ts/.+\\.js$)"
  ],
}

