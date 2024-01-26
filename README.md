# up_ynab

Imports and synchronizes Up transactions into YNAB.

## Setup

Install Nix:
```
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Enter Nix development shell:
```
nix develop
```

Open project:
```
code .
```

## Help

```
Usage: up_ynab [OPTIONS] <COMMAND>

Commands:
  sync-transactions      Syncs transactions from Up to YNAB
  get-up-accounts        Fetches Up accounts
  get-up-transactions    Fetches Up transactions
  get-ynab-accounts      Fetches YNAB accounts
  get-ynab-budgets       Fetches YNAB budgets
  get-ynab-transactions  Fetches YNAB transactions
  load-run               Load an past run
  help                   Print this message or the help of the given subcommand(s)

Options:
      --config <FILE>  Config file path
  -h, --help           Print help
  -V, --version        Print version
```
