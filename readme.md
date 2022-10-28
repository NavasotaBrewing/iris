# Iris

This exposes an RTU through an HTTP interface. It starts a web server that accepts RTU configurations, to either update or enact. See the [architecture page](https://github.com/NavasotaBrewing/documentation/blob/master/architecture.md) for more information on RTUs.

## Setup

You can install the latest version with `cargo`:

```
$ cargo install nbc_iris
```

## Usage

The primary function of this crate is to run the Iris web server. Do that by calling the installed executable

```
$ nbc_iris
```

### Configuration Validation
This crate can also read your configuration file and tell you if there's any errors in it. Write you configuration file [according to this guide](https://github.com/NavasotaBrewing/documentation/blob/master/RTU_Configuration/configuration.md) and run this crate with

```
$ nbc_iris validate-config
```

Your config will be automatically validated when the server starts, but this way you can know ahead of time.
