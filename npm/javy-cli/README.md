# Javy npm package

This is the npm package for Javy. The package contains a small Node script
that downloads the appropriate Javy binary on demand and invokes it with the
parameters given. 

## Usage

```
# Install javy globally
$ npm install -g javy-cli

# Directly invoke it via npm
$ npx javy-cli@latest
```

## Updating javy

The npm package will automatically download the newest version of Javy if a
newer version is available.

## Using a specific version of javy

To use a specific version of Javy, set the environment variable
`FORCE_RELEASE` to the version you would like to use.

```
FORCE_RELEASE=v1.1.0 npx javy-cli@latest
```
