# Arthouse

[![crates.io](https://img.shields.io/crates/v/arthouse)](https://crates.io/crates/arthouse)
[![Build](https://github.com/fwcd/arthouse/actions/workflows/build.yml/badge.svg)](https://github.com/fwcd/arthouse/actions/workflows/build.yml)

Art-Net (DMX over UDP/IP) adapter for Project Lighthouse.

Using this adapter, the Project Lighthouse display can be controlled like a standard DMX fixture from lighting controllers such as [QLC+](https://www.qlcplus.org/).

## Getting Started

Make sure that `LIGHTHOUSE_USER` and `LIGHTHOUSE_TOKEN` contain valid Project Lighthouse credentials, then run

```sh
cargo run
```

Alternatively you can also pass the corresponding command-line options, see `--help` for a detailed overview. By default, the adapter will listen on `0.0.0.0:6454`, i.e. the default Art-Net port, for UDP packets.

The following guide explains how to configure QLC+, though any other Art-Net client can be used too. First, make sure that the first three universes output to Art-Net on localhost:

![Input/Output](screenshots/getting-started/01-input-output.png)

Then add an RGB panel fixture:

![Fixtures](screenshots/getting-started/02-fixtures.png)

![Add RGB Panel](screenshots/getting-started/03-add-rgb-panel.png)

![Fixtures](screenshots/getting-started/04-fixtures.png)

Finally, create an RGB matrix and test one of the animations:

![Functions](screenshots/getting-started/05-functions.png)

![Functions](screenshots/getting-started/06-functions.png)

If everything went well, the output should be mirrored to the lighthouse:

![Lighthouse](screenshots/getting-started/07-lighthouse.png)
